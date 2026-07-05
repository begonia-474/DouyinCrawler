//! TaskApplicationService — 任务应用服务 (Async Version)
//!
//! 职责：
//! 1. 创建任务（Rust 拥有 DB 写入）
//! 2. 通过 resolve_urls 获取下载 URL（Python 解析）
//! 3. 通过 DownloadEngine 执行实际下载（Rust 原生 HTTP）
//! 4. 写入 DB 事务（task + items + download_record + video_info + user_info）
//! 5. 发射类型化事件（TaskEvent → 前端）
//! 6. 返回类型化响应
//!
//! Phase 3 变更：
//! - start_download / start_batch_download_mode / start_music_download 改为 async
//! - 下载逻辑移到 tokio::spawn 后台任务
//! - 集成 DownloadEngine 替代 Python 下载
//! - 集成 resolve_urls 调用
//! - 集成取消信号（通过 AppState）
//! - 使用 emit_progress 发射进度事件
//! - 任务完成后清理取消信号

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use parking_lot::Mutex;

use log::{error, info, warn};
use serde_json::Value;
use uuid::Uuid;

use crate::db::{Database, NewDownloadRecord, NewTaskItem, UserInfo, VideoInfo};
use crate::state::AppState;

use super::{
    DownloadMode, DownloadRequest, ResolvedAccessory, ResolvedItem, ResolvedUrls,
    TaskPatch, TaskStatus,
};
use super::engine::{DownloadEngine, DownloadItem, DownloadUrl, EngineConfig};
use super::events;

// ============================================================
// 辅助函数
// ============================================================

fn json_str(value: Option<&Value>, key: &str) -> Option<String> {
    value
        .and_then(|v| v.get(key))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn cleaned_json(value: &Value) -> Value {
    let mut value = value.clone();
    crate::python::db_bridge::bool_to_int(&mut value);
    value
}

fn parse_user_info(value: &Value) -> Option<UserInfo> {
    serde_json::from_value::<UserInfo>(cleaned_json(value))
        .ok()
        .filter(|user| !user.sec_user_id.trim().is_empty())
}

/// 从 serde_json::Value 构建 DownloadUrl（支持字符串或数组）
fn build_download_url(url_value: &Value) -> DownloadUrl {
    match url_value {
        Value::String(s) => DownloadUrl::Single(s.clone()),
        Value::Array(arr) => {
            let urls: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            match urls.len() {
                0 => DownloadUrl::Single(String::new()),
                1 => DownloadUrl::Single(urls.into_iter().next().unwrap()),
                _ => DownloadUrl::Multiple(urls),
            }
        }
        _ => DownloadUrl::Single(String::new()),
    }
}

/// 从 ResolvedItem 构建 DownloadItem
fn build_download_item(item: &ResolvedItem, save_dir: &str, task_id: &str) -> DownloadItem {
    let save_path = PathBuf::from(save_dir).join(format!("{}{}", item.filename, item.suffix));
    let ext = item.suffix.trim_start_matches('.');
    let temp_path = save_path.with_extension(format!("{}.tmp", ext));

    let url = build_download_url(&item.download_url);

    DownloadItem {
        url,
        save_path,
        temp_path,
        headers: item.headers.clone(),
        task_id: task_id.to_string(),
        file_size: None,
    }
}

/// 从 AppConfig 构建 EngineConfig
fn build_engine_config(config: &crate::config::AppConfig) -> EngineConfig {
    EngineConfig {
        max_concurrent: config.max_connections as usize,
        max_retries: config.max_retries,
        timeout: config.timeout as u64,
        max_connections: config.max_connections as usize,
        cookie: config.cookie.clone(),
        ..EngineConfig::default()
    }
}

// ============================================================
// 进度发射节流器
// ============================================================

/// 进度事件发射节流器
///
/// 避免每个下载 chunk 都发射事件（可能每秒数百次），
/// 限制为每 500ms 发射一次，或者在下载完成时强制发射。
struct ProgressTracker {
    task_id: String,
    last_emit_ms: AtomicU64,
    interval_ms: u64,
}

impl ProgressTracker {
    fn new(task_id: String) -> Self {
        Self {
            task_id,
            last_emit_ms: AtomicU64::new(0),
            interval_ms: 500,
        }
    }

    fn update(&self, downloaded: u64, total: u64) {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let last = self.last_emit_ms.load(Ordering::Relaxed);

        // 完成时或超过间隔时发射
        if downloaded >= total || now_ms.saturating_sub(last) >= self.interval_ms {
            self.last_emit_ms.store(now_ms, Ordering::Relaxed);
            events::emit_progress(&self.task_id, downloaded, total);
        }
    }
}

// ============================================================
// 任务应用服务
// ============================================================

/// 任务应用服务
///
/// 持有 AppState 引用，负责任务生命周期管理。
/// 所有任务的创建、状态更新、DB 写入都通过此服务。
pub struct TaskApplicationService<'a> {
    state: &'a AppState,
}

impl<'a> TaskApplicationService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    /// 获取数据库引用
    fn db(&self) -> &Database {
        self.state.db.as_ref()
    }

    // ============================================================
    // 内部辅助方法
    // ============================================================

    /// 通过 Python resolve_urls 解析下载 URL（异步，使用 spawn_blocking）
    async fn resolve_download_urls(mode: &str, url: &str) -> Result<ResolvedUrls, String> {
        let mode = mode.to_string();
        let url = url.to_string();

        let json_value = tokio::task::spawn_blocking(move || {
            crate::python::handler::resolve_urls(&mode, &url)
                .map_err(|e| format!("resolve_urls 调用失败: {}", e))
        })
        .await
        .map_err(|e| format!("spawn_blocking 失败: {}", e))??;

        serde_json::from_value::<ResolvedUrls>(json_value)
            .map_err(|e| format!("resolve_urls 返回值解析失败: {}", e))
    }

    /// 创建下载引擎并绑定取消信号
    fn create_engine(app_config: &crate::config::AppConfig, cancel_signal: Arc<AtomicBool>) -> DownloadEngine {
        let config = build_engine_config(app_config);
        DownloadEngine::new(config).with_cancel_signal(cancel_signal)
    }

    /// 处理单个下载项的附属文件（音乐、封面、文案）
    ///
    /// 返回成功下载的附属文件路径列表
    async fn download_accessories(
        engine: &DownloadEngine,
        accessories: &[ResolvedAccessory],
        save_dir: &str,
        task_id: &str,
    ) -> Vec<String> {
        let mut downloaded_paths = Vec::new();

        for acc in accessories {
            let acc_path = PathBuf::from(save_dir).join(format!("{}{}", acc.filename, acc.suffix));

            match acc.content_type.as_str() {
                "desc" => {
                    // 文案：直接写入文件
                    if let Some(content) = &acc.content {
                        match tokio::fs::write(&acc_path, content).await {
                            Ok(()) => {
                                info!(
                                    "[TaskService] 文案已保存: {}",
                                    acc_path.display()
                                );
                                downloaded_paths.push(acc_path.to_string_lossy().to_string());
                            }
                            Err(e) => {
                                warn!(
                                    "[TaskService] 文案保存失败: {}, error={}",
                                    acc_path.display(),
                                    e
                                );
                            }
                        }
                    }
                }
                "music" | "cover" => {
                    // 音乐/封面：需要下载
                    let url = match &acc.url {
                        Some(u) => u.clone(),
                        None => continue,
                    };

                    let ext = acc.suffix.trim_start_matches('.');
                    let temp_path = acc_path.with_extension(format!("{}.tmp", ext));

                    let item = DownloadItem {
                        url: DownloadUrl::Single(url),
                        save_path: acc_path.clone(),
                        temp_path,
                        headers: HashMap::new(), // 附属文件使用引擎默认 headers
                        task_id: task_id.to_string(),
                        file_size: None,
                    };

                    match engine.download(&item, |_, _| {}).await {
                        Ok(result) => {
                            info!(
                                "[TaskService] 附属文件已下载: {} ({} bytes)",
                                result.path.display(),
                                result.file_size
                            );
                            downloaded_paths.push(result.path.to_string_lossy().to_string());
                        }
                        Err(e) => {
                            warn!(
                                "[TaskService] 附属文件下载失败: {}, error={}",
                                acc.filename,
                                e
                            );
                        }
                    }
                }
                _ => {
                    warn!(
                        "[TaskService] 未知附属文件类型: {}",
                        acc.content_type
                    );
                }
            }
        }

        downloaded_paths
    }

    /// 写入任务结果到数据库（单个下载项）
    ///
    /// 创建 task_item + download_record + video_info
    /// 保存下载元数据（download_history + video_info + user_info）
    ///
    /// 不创建 task_item，仅写入元数据表。
    /// 用于跳过和正常下载两种场景。
    fn save_download_metadata(
        db: &Database,
        item: &ResolvedItem,
        file_path: &str,
        file_size: i64,
        accessory_paths: &[String],
    ) -> Result<(), String> {
        let detail = item.detail.as_ref();
        let aweme_id = &item.aweme_id;

        // 构建 download_record
        let mut records = vec![NewDownloadRecord {
            aweme_id: Some(aweme_id.clone()),
            download_type: "video".to_string(),
            title: json_str(detail, "desc"),
            author_nickname: json_str(detail, "author_nickname"),
            author_sec_uid: json_str(detail, "author_sec_uid"),
            file_path: Some(file_path.to_string()),
            file_size,
            cover_url: json_str(detail, "cover_url"),
            status: "completed".to_string(),
            error_msg: None,
        }];

        // 为附属文件添加 download_record
        for acc_path in accessory_paths {
            let acc_type = if acc_path.ends_with(".mp3") {
                "music"
            } else if acc_path.ends_with(".jpg") || acc_path.ends_with(".png") {
                "cover"
            } else {
                "accessory"
            };
            let acc_size = std::fs::metadata(acc_path)
                .map(|m| m.len() as i64)
                .unwrap_or(0);

            records.push(NewDownloadRecord {
                aweme_id: Some(aweme_id.clone()),
                download_type: acc_type.to_string(),
                title: json_str(detail, "desc"),
                author_nickname: json_str(detail, "author_nickname"),
                author_sec_uid: json_str(detail, "author_sec_uid"),
                file_path: Some(acc_path.clone()),
                file_size: acc_size,
                cover_url: None,
                status: "completed".to_string(),
                error_msg: None,
            });
        }

        // 收集 video_info（user_info 由 execute_download 的 user_profile 路径独立保存，
        // 不从 detail 提取，避免不完整数据覆盖完整的 user_profile）
        let mut videos = Vec::new();
        if let Some(d) = detail {
            if d.get("aweme_id").and_then(|v| v.as_str()).is_some() {
                if let Ok(video_info) =
                    serde_json::from_value::<VideoInfo>(cleaned_json(d))
                {
                    videos.push(video_info);
                }
            }
        }

        // 保存到数据库（只写 download_history + video_info，不写 user_info）
        if let Err(e) = db.save_batch_results(&records, &videos, &[]) {
            error!("[TaskService] 保存下载记录失败: aweme_id={}, error={}", item.aweme_id, e);
            return Err(format!("保存下载记录失败: {}", e));
        }

        Ok(())
    }

    /// 清理取消信号
    fn cleanup_cancel_signal(cancel_signals: &Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>, task_id: &str) {
        let mut signals = cancel_signals.lock();
        signals.remove(task_id);
    }

    // ============================================================
    // 统一下载入口（mode=one）
    // ============================================================

    /// 保存单个下载项的完整结果（task_item + download_history + video_info + user_info）
    ///
    /// 正常下载完成后调用。跳过场景请直接调用 save_download_metadata。
    fn save_single_result(
        db: &Database,
        task_id: &str,
        item: &ResolvedItem,
        file_path: &str,
        file_size: i64,
        accessory_paths: &[String],
    ) -> Result<(), String> {
        let detail = item.detail.as_ref();
        let aweme_id = &item.aweme_id;

        // 创建 task_item
        let new_item = NewTaskItem {
            task_id: task_id.to_string(),
            aweme_id: Some(aweme_id.clone()),
            title: json_str(detail, "desc"),
            author_nickname: json_str(detail, "author_nickname"),
            cover_url: json_str(detail, "cover_url"),
        };
        if let Err(e) = db.create_task_item(&new_item) {
            error!("[TaskService] 创建任务子项失败: task_id={}, error={}", task_id, e);
            return Err(format!("创建任务子项失败: {}", e));
        }

        // 更新 task_item 状态
        if let Err(e) = db.update_task_item_status(task_id, aweme_id, "completed", Some(file_path), file_size, None) {
            error!("[TaskService] 更新任务子项状态失败: task_id={}, error={}", task_id, e);
            return Err(format!("更新任务子项状态失败: {}", e));
        }

        // 保存元数据
        Self::save_download_metadata(db, item, file_path, file_size, accessory_paths)
    }

    // ============================================================

    /// 统一下载入口（对齐 task_manager.start_download）
    ///
    /// 立即返回 task_id，下载在后台 tokio 任务中执行。
    pub async fn start_download(&self, request: DownloadRequest) -> Result<String, String> {
        let task_id = Uuid::new_v4().to_string();
        let task_id = task_id[..8].to_string();

        info!(
            "[TaskService] start_download: task_id={}, mode={}, url={}",
            task_id,
            request.mode,
            &request.url[..request.url.len().min(80)]
        );

        // 1. 创建任务记录
        let new_task = crate::db::NewDownloadTask {
            id: task_id.clone(),
            mode: request.mode.as_str().to_string(),
            url: request.url.clone(),
            title: None,
            author_nickname: None,
        };
        if let Err(e) = self.db().create_task(&new_task) {
            error!("[TaskService] 创建任务失败: {}", e);
            return Err(format!("创建任务失败: {}", e));
        }

        // 2. 发射任务启动事件
        events::emit_started(&task_id, request.mode, &request.url);

        // 3. 注册取消信号
        let cancel_signal = self.state.register_cancel_signal(&task_id);

        // 4. 克隆数据用于后台任务
        let db = self.state.db.clone();
        let cancel_signals = self.state.cancel_signals.clone();
        let app_config = self.state.config.lock().get_douyin_config();
        let task_id_clone = task_id.clone();
        let mode = request.mode;
        let url = request.url;

        // 5. 启动后台下载任务
        tokio::spawn(async move {
            let result = Self::execute_download(
                &db,
                &task_id_clone,
                mode,
                &url,
                &cancel_signal,
                &app_config,
            )
            .await;

            match result {
                Ok(()) => {
                    // 更新任务状态为 completed
                    if let Err(e) = db.update_task_status(&task_id_clone, "completed", None) {
                        error!(
                            "[TaskService] 更新任务完成状态失败: task_id={}, error={}",
                            task_id_clone, e
                        );
                    }
                    events::emit_finished(
                        TaskPatch::new(&task_id_clone).with_status(TaskStatus::Completed),
                    );
                    info!("[TaskService] 下载任务完成: task_id={}", task_id_clone);
                }
                Err(e) => {
                    // 检查是否是取消
                    if cancel_signal.load(Ordering::Relaxed) {
                        if let Err(db_err) =
                            db.update_task_status(&task_id_clone, "cancelled", None)
                        {
                            error!(
                                "[TaskService] 更新取消状态失败: task_id={}, error={}",
                                task_id_clone, db_err
                            );
                        }
                        events::emit_cancelled(&task_id_clone);
                        info!("[TaskService] 下载任务已取消: task_id={}", task_id_clone);
                    } else {
                        error!(
                            "[TaskService] 下载任务失败: task_id={}, error={}",
                            task_id_clone, e
                        );
                        if let Err(db_err) =
                            db.update_task_status(&task_id_clone, "error", Some(&e))
                        {
                            error!(
                                "[TaskService] 更新错误状态失败: task_id={}, error={}",
                                task_id_clone, db_err
                            );
                        }
                        events::emit_error(&task_id_clone, &e);
                    }
                }
            }

            // 清理取消信号
            Self::cleanup_cancel_signal(&cancel_signals, &task_id_clone);
        });

        Ok(task_id)
    }

    /// 执行单个/小批量下载（内部方法）
    ///
    /// 1. 调用 resolve_urls 获取下载 URL
    /// 2. 使用 DownloadEngine 下载
    /// 3. 保存结果到数据库
    async fn execute_download(
        db: &Database,
        task_id: &str,
        mode: DownloadMode,
        url: &str,
        cancel_signal: &Arc<AtomicBool>,
        app_config: &crate::config::AppConfig,
    ) -> Result<(), String> {
        // 1. 解析下载 URL
        let resolved = Self::resolve_download_urls(mode.as_str(), url).await?;
        if !resolved.success {
            let err = resolved.error.unwrap_or_else(|| "解析失败".to_string());
            return Err(err);
        }

        let items = resolved.items;
        let save_dir = resolved.save_dir.unwrap_or_else(|| {
            format!("./Download/{}", mode.as_str())
        });

        if items.is_empty() {
            return Err("没有可下载的内容".to_string());
        }

        // 2. 创建下载引擎
        let engine = Self::create_engine(app_config, cancel_signal.clone());

        // 3. 处理用户资料（batch 模式）
        if let Some(profile) = resolved.user_profile.as_ref() {
            if matches!(mode, DownloadMode::Post | DownloadMode::Mix) {
                if let Some(user_info) = parse_user_info(profile) {
                    if db
                        .get_user_by_sec_uid(&user_info.sec_user_id)
                        .ok()
                        .flatten()
                        .is_none()
                    {
                        let _ = db.save_user(&user_info);
                    }
                }
            }
        }

        // 4. 创建保存目录
        if let Err(e) = tokio::fs::create_dir_all(&save_dir).await {
            warn!("[TaskService] 创建保存目录失败: {}, error={}", save_dir, e);
        }

        // 5. 下载每个项目
        let total = items.len() as i64;
        let mut completed: i64 = 0;
        let mut failed: i64 = 0;
        let progress = ProgressTracker::new(task_id.to_string());

        for (index, item) in items.iter().enumerate() {
            // 检查取消信号
            if cancel_signal.load(Ordering::Relaxed) {
                info!("[TaskService] 下载被取消: task_id={}, 已完成 {}/{}", task_id, index, total);
                return Err("下载已取消".to_string());
            }

            let download_item = build_download_item(item, &save_dir, task_id);

            // 进度回调（节流）
            let progress_ref = &progress;
            let result = engine
                .download(&download_item, |downloaded, total_size| {
                    progress_ref.update(downloaded, total_size);
                })
                .await;

            match result {
                Ok(download_result) => {
                    let file_path = download_result.path.to_string_lossy().to_string();
                    let file_size = download_result.file_size as i64;

                    // 下载附属文件
                    let accessory_paths =
                        Self::download_accessories(&engine, &item.accessories, &save_dir, task_id)
                            .await;

                    // 保存到数据库
                    if download_result.skipped {
                        // 文件已存在，跳过下载但仍写入 download_history + video_info + user_info
                        let new_item = NewTaskItem {
                            task_id: task_id.to_string(),
                            aweme_id: Some(item.aweme_id.clone()),
                            title: json_str(item.detail.as_ref(), "desc"),
                            author_nickname: json_str(item.detail.as_ref(), "author_nickname"),
                            cover_url: json_str(item.detail.as_ref(), "cover_url"),
                        };
                        let _ = db.create_task_item(&new_item);
                        let _ = db.update_task_item_status(
                            task_id,
                            &item.aweme_id,
                            "skipped",
                            Some(&file_path),
                            file_size,
                            None,
                        );
                        // 跳过也需要写入 download_history + video_info + user_info
                        if let Err(e) = Self::save_download_metadata(
                            db,
                            item,
                            &file_path,
                            file_size,
                            &accessory_paths,
                        ) {
                            warn!("[TaskService] 跳过项保存元数据失败: {}", e);
                        }
                        info!(
                            "[TaskService] 文件已存在，跳过下载但已入库: {}",
                            download_result.path.display()
                        );
                    } else {
                        if let Err(e) = Self::save_single_result(
                            db,
                            task_id,
                            item,
                            &file_path,
                            file_size,
                            &accessory_paths,
                        ) {
                            warn!("[TaskService] 保存结果失败: {}", e);
                            failed += 1;
                            continue;
                        }
                    }

                    completed += 1;
                    // 发射最终进度
                    progress.update(
                        (completed + failed) as u64,
                        total as u64,
                    );
                }
                Err(e) => {
                    warn!(
                        "[TaskService] 下载失败: aweme_id={}, error={}",
                        item.aweme_id, e
                    );

                    // 记录失败
                    let new_item = NewTaskItem {
                        task_id: task_id.to_string(),
                        aweme_id: Some(item.aweme_id.clone()),
                        title: json_str(item.detail.as_ref(), "desc"),
                        author_nickname: json_str(item.detail.as_ref(), "author_nickname"),
                        cover_url: json_str(item.detail.as_ref(), "cover_url"),
                    };
                    let _ = db.create_task_item(&new_item);
                    let err_msg = e.to_string();
                    let _ = db.update_task_item_status(
                        task_id,
                        &item.aweme_id,
                        "failed",
                        None,
                        0,
                        Some(&err_msg),
                    );

                    failed += 1;
                }
            }
        }

        // 6. 更新任务计数
        if let Err(e) = db.update_task_counts(task_id) {
            warn!("[TaskService] 更新任务计数失败: {}", e);
        }

        if failed > 0 && completed == 0 {
            return Err(format!("所有下载均失败: {}/{} failed", failed, total));
        }

        info!(
            "[TaskService] 下载完成: task_id={}, total={}, completed={}, failed={}",
            task_id, total, completed, failed
        );

        Ok(())
    }

    // ============================================================
    // 批量下载入口（post/like/mix/collects）
    // ============================================================

    /// 批量下载（Phase 5.2: post/like/mix/collects 迁移）
    ///
    /// 立即返回 task_id，下载在后台 tokio 任务中执行。
    pub async fn start_batch_download_mode(
        &self,
        mode: DownloadMode,
        url: &str,
    ) -> Result<String, String> {
        let task_id = Uuid::new_v4().to_string();
        let task_id = task_id[..8].to_string();

        info!(
            "[TaskService] start_batch_download_mode: task_id={}, mode={}",
            task_id, mode
        );

        // 1. 创建任务记录
        let new_task = crate::db::NewDownloadTask {
            id: task_id.clone(),
            mode: mode.as_str().to_string(),
            url: url.to_string(),
            title: None,
            author_nickname: None,
        };
        if let Err(e) = self.db().create_task(&new_task) {
            error!("[TaskService] 创建批量任务失败: {}", e);
            return Err(format!("创建任务失败: {}", e));
        }

        // 2. 发射启动事件
        events::emit_started(&task_id, mode, url);

        // 3. 注册取消信号
        let cancel_signal = self.state.register_cancel_signal(&task_id);

        // 4. 克隆数据用于后台任务
        let db = self.state.db.clone();
        let cancel_signals = self.state.cancel_signals.clone();
        let app_config = self.state.config.lock().get_douyin_config();
        let task_id_clone = task_id.clone();
        let mode_val = mode;
        let url_val = url.to_string();

        // 5. 启动后台下载任务
        tokio::spawn(async move {
            let result = Self::execute_download(
                &db,
                &task_id_clone,
                mode_val,
                &url_val,
                &cancel_signal,
                &app_config,
            )
            .await;

            match result {
                Ok(()) => {
                    if let Err(e) = db.update_task_status(&task_id_clone, "completed", None) {
                        error!(
                            "[TaskService] 更新批量任务完成状态失败: task_id={}, error={}",
                            task_id_clone, e
                        );
                    }
                    events::emit_finished(
                        TaskPatch::new(&task_id_clone).with_status(TaskStatus::Completed),
                    );
                    info!(
                        "[TaskService] 批量下载完成: task_id={}",
                        task_id_clone
                    );
                }
                Err(e) => {
                    if cancel_signal.load(Ordering::Relaxed) {
                        if let Err(db_err) =
                            db.update_task_status(&task_id_clone, "cancelled", None)
                        {
                            error!(
                                "[TaskService] 更新取消状态失败: task_id={}, error={}",
                                task_id_clone, db_err
                            );
                        }
                        events::emit_cancelled(&task_id_clone);
                        info!(
                            "[TaskService] 批量下载已取消: task_id={}",
                            task_id_clone
                        );
                    } else {
                        error!(
                            "[TaskService] 批量下载失败: task_id={}, error={}",
                            task_id_clone, e
                        );
                        if let Err(db_err) =
                            db.update_task_status(&task_id_clone, "error", Some(&e))
                        {
                            error!(
                                "[TaskService] 更新批量错误状态失败: task_id={}, error={}",
                                task_id_clone, db_err
                            );
                        }
                        events::emit_error(&task_id_clone, &e);
                    }
                }
            }

            // 清理取消信号
            Self::cleanup_cancel_signal(&cancel_signals, &task_id_clone);
        });

        Ok(task_id)
    }

    // ============================================================
    // 音乐下载入口
    // ============================================================

    /// 音乐批量下载
    ///
    /// 立即返回 task_id，下载在后台 tokio 任务中执行。
    pub async fn start_music_download(&self, url: &str) -> Result<String, String> {
        let task_id = Uuid::new_v4().to_string();
        let task_id = task_id[..8].to_string();

        info!("[TaskService] start_music_download: task_id={}", task_id);

        // 1. 创建任务记录
        let new_task = crate::db::NewDownloadTask {
            id: task_id.clone(),
            mode: "music".to_string(),
            url: url.to_string(),
            title: None,
            author_nickname: None,
        };
        if let Err(e) = self.db().create_task(&new_task) {
            error!("[TaskService] 创建音乐任务失败: {}", e);
            return Err(format!("创建任务失败: {}", e));
        }

        // 2. 发射启动事件
        events::emit_started(&task_id, DownloadMode::Music, url);

        // 3. 注册取消信号
        let cancel_signal = self.state.register_cancel_signal(&task_id);

        // 4. 克隆数据用于后台任务
        let db = self.state.db.clone();
        let cancel_signals = self.state.cancel_signals.clone();
        let app_config = self.state.config.lock().get_douyin_config();
        let task_id_clone = task_id.clone();
        let url_val = url.to_string();

        // 5. 启动后台下载任务
        tokio::spawn(async move {
            let result = Self::execute_music_download(
                &db,
                &task_id_clone,
                &url_val,
                &cancel_signal,
                &app_config,
            )
            .await;

            match result {
                Ok(()) => {
                    if let Err(e) =
                        db.update_task_status(&task_id_clone, "completed", None)
                    {
                        error!(
                            "[TaskService] 更新音乐任务完成状态失败: task_id={}, error={}",
                            task_id_clone, e
                        );
                    }
                    events::emit_finished(
                        TaskPatch::new(&task_id_clone).with_status(TaskStatus::Completed),
                    );
                    info!(
                        "[TaskService] 音乐下载完成: task_id={}",
                        task_id_clone
                    );
                }
                Err(e) => {
                    if cancel_signal.load(Ordering::Relaxed) {
                        if let Err(db_err) =
                            db.update_task_status(&task_id_clone, "cancelled", None)
                        {
                            error!(
                                "[TaskService] 更新取消状态失败: task_id={}, error={}",
                                task_id_clone, db_err
                            );
                        }
                        events::emit_cancelled(&task_id_clone);
                        info!(
                            "[TaskService] 音乐下载已取消: task_id={}",
                            task_id_clone
                        );
                    } else {
                        error!(
                            "[TaskService] 音乐下载失败: task_id={}, error={}",
                            task_id_clone, e
                        );
                        if let Err(db_err) =
                            db.update_task_status(&task_id_clone, "error", Some(&e))
                        {
                            error!(
                                "[TaskService] 更新音乐错误状态失败: task_id={}, error={}",
                                task_id_clone, db_err
                            );
                        }
                        events::emit_error(&task_id_clone, &e);
                    }
                }
            }

            // 清理取消信号
            Self::cleanup_cancel_signal(&cancel_signals, &task_id_clone);
        });

        Ok(task_id)
    }

    /// 执行音乐下载（内部方法）
    ///
    /// 音乐模式的特殊处理：
    /// - detail 中的字段是音乐元数据（music_id, title, author）
    /// - download_type 为 "music"
    async fn execute_music_download(
        db: &Database,
        task_id: &str,
        url: &str,
        cancel_signal: &Arc<AtomicBool>,
        app_config: &crate::config::AppConfig,
    ) -> Result<(), String> {
        // 1. 解析下载 URL
        let resolved = Self::resolve_download_urls("music", url).await?;
        if !resolved.success {
            let err = resolved.error.unwrap_or_else(|| "解析失败".to_string());
            return Err(err);
        }

        let items = resolved.items;
        let save_dir = resolved.save_dir.unwrap_or_else(|| "./Download/music".to_string());

        if items.is_empty() {
            return Err("没有可下载的音乐".to_string());
        }

        // 2. 创建下载引擎
        let engine = Self::create_engine(app_config, cancel_signal.clone());

        // 3. 创建保存目录
        if let Err(e) = tokio::fs::create_dir_all(&save_dir).await {
            warn!("[TaskService] 创建保存目录失败: {}, error={}", save_dir, e);
        }

        // 4. 下载每首音乐
        let total = items.len() as i64;
        let mut completed: i64 = 0;
        let mut failed: i64 = 0;
        let progress = ProgressTracker::new(task_id.to_string());

        for (index, item) in items.iter().enumerate() {
            // 检查取消信号
            if cancel_signal.load(Ordering::Relaxed) {
                info!(
                    "[TaskService] 音乐下载被取消: task_id={}, 已完成 {}/{}",
                    task_id, index, total
                );
                return Err("下载已取消".to_string());
            }

            let download_item = build_download_item(item, &save_dir, task_id);

            let progress_ref = &progress;
            let result = engine
                .download(&download_item, |downloaded, total_size| {
                    progress_ref.update(downloaded, total_size);
                })
                .await;

            match result {
                Ok(download_result) => {
                    let file_path = download_result.path.to_string_lossy().to_string();
                    let file_size = download_result.file_size as i64;

                    // 音乐元数据在 detail 中
                    let detail = item.detail.as_ref();
                    let music_id = &item.aweme_id;
                    let title = detail
                        .and_then(|d| d.get("title"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let author = detail
                        .and_then(|d| d.get("author"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    if download_result.skipped {
                        // 标记为跳过
                        let new_item = NewTaskItem {
                            task_id: task_id.to_string(),
                            aweme_id: Some(music_id.clone()),
                            title: title.clone(),
                            author_nickname: author.clone(),
                            cover_url: None,
                        };
                        let _ = db.create_task_item(&new_item);
                        let _ = db.update_task_item_status(
                            task_id,
                            music_id,
                            "skipped",
                            Some(&file_path),
                            file_size,
                            None,
                        );
                        info!("[TaskService] 音乐文件已存在，跳过: {}", file_path);
                    } else {
                        // 创建 task_item
                        let new_item = NewTaskItem {
                            task_id: task_id.to_string(),
                            aweme_id: Some(music_id.clone()),
                            title: title.clone(),
                            author_nickname: author.clone(),
                            cover_url: None,
                        };
                        if let Err(e) = db.create_task_item(&new_item) {
                            warn!("[TaskService] 创建音乐任务子项失败: {}", e);
                            failed += 1;
                            continue;
                        }

                        // 更新 task_item 状态
                        if let Err(e) = db.update_task_item_status(
                            task_id,
                            music_id,
                            "completed",
                            Some(&file_path),
                            file_size,
                            None,
                        ) {
                            warn!("[TaskService] 更新音乐子项状态失败: {}", e);
                        }

                        // 保存 download_record
                        let record = NewDownloadRecord {
                            aweme_id: Some(music_id.clone()),
                            download_type: "music".to_string(),
                            title: title.clone(),
                            author_nickname: author.clone(),
                            author_sec_uid: None,
                            file_path: Some(file_path),
                            file_size,
                            cover_url: None,
                            status: "completed".to_string(),
                            error_msg: None,
                        };
                        if let Err(e) = db.save_batch_results(&[record], &[], &[]) {
                            warn!("[TaskService] 保存音乐下载记录失败: {}", e);
                        }
                    }

                    completed += 1;
                    progress.update(
                        (completed + failed) as u64,
                        total as u64,
                    );
                }
                Err(e) => {
                    warn!(
                        "[TaskService] 音乐下载失败: music_id={}, error={}",
                        item.aweme_id, e
                    );

                    let new_item = NewTaskItem {
                        task_id: task_id.to_string(),
                        aweme_id: Some(item.aweme_id.clone()),
                        title: None,
                        author_nickname: None,
                        cover_url: None,
                    };
                    let _ = db.create_task_item(&new_item);
                    let err_msg = e.to_string();
                    let _ = db.update_task_item_status(
                        task_id,
                        &item.aweme_id,
                        "failed",
                        None,
                        0,
                        Some(&err_msg),
                    );

                    failed += 1;
                }
            }
        }

        // 5. 更新任务计数
        if let Err(e) = db.update_task_counts(task_id) {
            warn!("[TaskService] 更新音乐任务计数失败: {}", e);
        }

        if failed > 0 && completed == 0 {
            return Err(format!("所有音乐下载均失败: {}/{} failed", failed, total));
        }

        info!(
            "[TaskService] 音乐下载完成: task_id={}, total={}, completed={}, failed={}",
            task_id, total, completed, failed
        );

        Ok(())
    }
}
