//! TaskApplicationService — 任务应用服务
//!
//! 职责：
//! 1. 创建任务（Rust 拥有 DB 写入）
//! 2. 调用 Python 适配器（爬取/下载/解析）
//! 3. 写入 DB 事务（task + items + download_record + video_info + user_info）
//! 4. 发射类型化事件（TaskEvent → 前端）
//! 5. 返回类型化响应
//!
//! Phase 2.1: 骨架
//! Phase 2.2: 事务性 DB API
//! Phase 2.3: Python 适配器（本文件）
//! Follow-up F1.1: 所有关键 DB 写入使用显式错误处理，不再静默忽略

use log::{error, info, warn};
use serde_json::Value;
use uuid::Uuid;

use crate::db::{Database, NewDownloadRecord, NewTaskItem, UserInfo, VideoInfo};
use crate::python;

use super::{
    DownloadMode, DownloadRequest, PythonBatchDownloadResult, PythonDownloadResult,
    PythonMusicBatchResult, TaskPatch, TaskStatus,
};
use super::events;

fn json_str(value: Option<&Value>, key: &str) -> Option<String> {
    value.and_then(|v| v.get(key)).and_then(|v| v.as_str()).map(|s| s.to_string())
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

/// 任务应用服务
///
/// 持有 Database 引用，负责任务生命周期管理。
/// 所有任务的创建、状态更新、DB 写入都通过此服务。
pub struct TaskApplicationService<'a> {
    db: &'a Database,
}

impl<'a> TaskApplicationService<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// 统一下载入口（对齐 task_manager.start_download）
    ///
    /// Phase 3 将此方法接入 Tauri command，替代当前的 py_start_download。
    pub fn start_download(&self, request: DownloadRequest) -> Result<String, String> {
        let task_id = Uuid::new_v4().to_string();
        let task_id = task_id[..8].to_string();

        info!(
            "[TaskService] start_download: task_id={}, mode={}, url={}",
            task_id, request.mode, &request.url[..request.url.len().min(80)]
        );

        // 1. 创建任务记录（Rust 拥有 DB 写入）
        let new_task = crate::db::NewDownloadTask {
            id: task_id.clone(),
            mode: request.mode.as_str().to_string(),
            url: request.url.clone(),
            title: None,
            author_nickname: None,
        };
        if let Err(e) = self.db.create_task(&new_task) {
            error!("[TaskService] 创建任务失败: {}", e);
            return Err(format!("创建任务失败: {}", e));
        }

        // 2. 发射任务启动事件
        events::emit_started(&task_id, request.mode, &request.url);

        // 3. 调用 Python 适配器
        let result = self.call_python_download(&request.url);

        // 4. 处理结果
        match result {
            Ok(py_result) => {
                if py_result.success {
                    info!("[TaskService] 下载成功: task_id={}", task_id);

                    // F2.1: 提取 detail，构建事务性写入所需的结构体
                    let detail = py_result.detail.as_ref();
                    let aweme_id = detail
                        .and_then(|d| d.get("aweme_id"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");

                    let file_path = py_result.path.as_deref()
                        .or_else(|| py_result.paths.as_ref().and_then(|p| p.first().map(|s| s.as_str())));
                    let file_size: i64 = file_path
                        .and_then(|p| std::fs::metadata(p).ok())
                        .map(|m| m.len() as i64)
                        .unwrap_or(0);

                    // 构建 task_item
                    let new_item = NewTaskItem {
                        task_id: task_id.clone(),
                        aweme_id: Some(aweme_id.to_string()),
                        title: detail.and_then(|d| d.get("desc")).and_then(|v| v.as_str()).map(|s| s.to_string()),
                        author_nickname: detail.and_then(|d| d.get("author_nickname")).and_then(|v| v.as_str()).map(|s| s.to_string()),
                        cover_url: detail.and_then(|d| d.get("cover_url")).and_then(|v| v.as_str()).map(|s| s.to_string()),
                    };

                    // 构建 download_record
                    let download_record = NewDownloadRecord {
                        aweme_id: Some(aweme_id.to_string()),
                        download_type: "video".to_string(),
                        title: detail.and_then(|d| d.get("desc")).and_then(|v| v.as_str()).map(|s| s.to_string()),
                        author_nickname: detail.and_then(|d| d.get("author_nickname")).and_then(|v| v.as_str()).map(|s| s.to_string()),
                        author_sec_uid: detail.and_then(|d| d.get("author_sec_uid")).and_then(|v| v.as_str()).map(|s| s.to_string()),
                        file_path: file_path.map(|s| s.to_string()),
                        file_size,
                        cover_url: detail.and_then(|d| d.get("cover_url")).and_then(|v| v.as_str()).map(|s| s.to_string()),
                        status: "completed".to_string(),
                        error_msg: None,
                    };

                    // 反序列化 video_info 和 user_info
                    let video_info: Option<VideoInfo> = detail
                        .and_then(|d| serde_json::from_value(cleaned_json(d)).ok());
                    let user_info: Option<UserInfo> = py_result.user_profile.as_ref()
                        .or(detail)
                        .and_then(parse_user_info);

                    // F2.1: 事务性写入 — 任何一步失败则整体回滚，不发射完成事件
                    if let Err(e) = self.db.complete_single_download(
                        &task_id, &new_item, file_path, file_size,
                        &download_record, video_info.as_ref(), user_info.as_ref(),
                    ) {
                        error!("[TaskService] 事务性写入失败: task_id={}, error={}", task_id, e);
                        return Err(format!("下载成功但持久化失败: {}", e));
                    }

                    // 发射完成事件（仅在事务成功后）
                    let patch = TaskPatch::new(&task_id)
                        .with_status(TaskStatus::Completed)
                        .with_counts(1, 1, 0, 0);
                    events::emit_finished(patch);
                } else {
                    let err = py_result.error.unwrap_or_else(|| "未知错误".to_string());
                    warn!("[TaskService] 下载失败: task_id={}, error={}", task_id, err);

                    // 更新任务状态为 error
                    if let Err(db_err) = self.db.update_task_status(&task_id, "error", Some(&err)) {
                        error!("[TaskService] 更新错误状态也失败: task_id={}, db_error={}", task_id, db_err);
                    }

                    events::emit_error(&task_id, &err);
                }
                Ok(task_id)
            }
            Err(e) => {
                error!("[TaskService] Python 调用异常: task_id={}, error={}", task_id, e);

                // 更新任务状态为 error
                if let Err(db_err) = self.db.update_task_status(&task_id, "error", Some(&e)) {
                    error!("[TaskService] 更新错误状态也失败: task_id={}, db_error={}", task_id, db_err);
                }

                events::emit_error(&task_id, &e);
                Err(e)
            }
        }
    }

    /// 音乐批量下载（Phase 5.1: music mode 迁移）
    ///
    /// 1. 创建任务
    /// 2. 调用 Python download_music_batch（不写 task DB 表）
    /// 3. 为每首音乐创建 task_item + 保存 download_record
    /// 4. 更新任务状态
    pub fn start_music_download(&self, url: &str) -> Result<String, String> {
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
        if let Err(e) = self.db.create_task(&new_task) {
            error!("[TaskService] 创建音乐任务失败: {}", e);
            return Err(format!("创建任务失败: {}", e));
        }

        // 2. 发射启动事件
        events::emit_started(&task_id, DownloadMode::Music, url);

        // 3. 调用 Python 适配器
        let result = self.call_python_music_batch(url);

        match result {
            Ok(py_result) => {
                if py_result.success {
                    let items = py_result.results.unwrap_or_default();
                    let total = items.len() as i64;
                    let mut completed: i64 = 0;
                    let mut failed: i64 = 0;

                    // 4. 为每首音乐创建 task_item + 保存 download_record
                    for item in &items {
                        let aweme_id = &item.music_id;

                        // 创建 task_item — 失败则返回错误
                        let new_item = NewTaskItem {
                            task_id: task_id.clone(),
                            aweme_id: Some(aweme_id.clone()),
                            title: item.title.clone(),
                            author_nickname: item.author.clone(),
                            cover_url: None,
                        };
                        if let Err(e) = self.db.create_task_item(&new_item) {
                            error!("[TaskService] 创建音乐任务子项失败: task_id={}, aweme_id={}, error={}", task_id, aweme_id, e);
                            return Err(format!("创建音乐任务子项失败: {}", e));
                        }

                        if item.success {
                            // 更新 task_item 状态 — 失败则返回错误
                            let path = item.path.as_deref();
                            let file_size = item.file_size.unwrap_or(0);
                            if let Err(e) = self.db.update_task_item_status(
                                &task_id, aweme_id, "completed", path, file_size, None,
                            ) {
                                error!("[TaskService] 更新音乐子项状态失败: task_id={}, error={}", task_id, e);
                                return Err(format!("更新音乐子项状态失败: {}", e));
                            }

                            // 保存 download_record — 失败则返回错误
                            let record = NewDownloadRecord {
                                aweme_id: Some(aweme_id.clone()),
                                download_type: "music".to_string(),
                                title: item.title.clone(),
                                author_nickname: item.author.clone(),
                                author_sec_uid: None,
                                file_path: path.map(|s| s.to_string()),
                                file_size,
                                cover_url: None,
                                status: "completed".to_string(),
                                error_msg: None,
                            };
                            if let Err(e) = self.db.save_batch_results(&[record], &[], &[]) {
                                error!("[TaskService] 保存音乐下载记录失败: task_id={}, error={}", task_id, e);
                                return Err(format!("保存音乐下载记录失败: {}", e));
                            }
                            completed += 1;
                        } else {
                            let err = item.error.as_deref().unwrap_or("下载失败");
                            if let Err(e) = self.db.update_task_item_status(
                                &task_id, aweme_id, "failed", None, 0, Some(err),
                            ) {
                                error!("[TaskService] 更新音乐失败子项状态失败: task_id={}, error={}", task_id, e);
                                return Err(format!("更新音乐失败子项状态失败: {}", e));
                            }
                            failed += 1;
                        }
                    }

                    // 5. 更新任务计数和状态 — 失败则返回错误
                    if let Err(e) = self.db.update_task_counts(&task_id) {
                        error!("[TaskService] 更新音乐任务计数失败: task_id={}, error={}", task_id, e);
                        return Err(format!("更新音乐任务计数失败: {}", e));
                    }

                    // 更新任务状态为 completed — 失败则不发射完成事件
                    if let Err(e) = self.db.update_task_status(&task_id, "completed", None) {
                        error!("[TaskService] 更新音乐任务完成状态失败: task_id={}, error={}", task_id, e);
                        return Err(format!("音乐任务完成但状态更新失败: {}", e));
                    }

                    // 6. 发射完成事件（仅在 DB 写入成功后）
                    let patch = TaskPatch::new(&task_id)
                        .with_status(TaskStatus::Completed)
                        .with_counts(total, completed, failed, 0);
                    events::emit_finished(patch);

                    info!(
                        "[TaskService] 音乐下载完成: task_id={}, total={}, completed={}, failed={}",
                        task_id, total, completed, failed
                    );
                } else {
                    let err = py_result.error.unwrap_or_else(|| "未知错误".to_string());
                    warn!("[TaskService] 音乐下载失败: task_id={}, error={}", task_id, err);
                    if let Err(db_err) = self.db.update_task_status(&task_id, "error", Some(&err)) {
                        error!("[TaskService] 更新音乐错误状态也失败: task_id={}, db_error={}", task_id, db_err);
                    }
                    events::emit_error(&task_id, &err);
                }
                Ok(task_id)
            }
            Err(e) => {
                error!("[TaskService] 音乐下载异常: task_id={}, error={}", task_id, e);
                if let Err(db_err) = self.db.update_task_status(&task_id, "error", Some(&e)) {
                    error!("[TaskService] 更新音乐错误状态也失败: task_id={}, db_error={}", task_id, db_err);
                }
                events::emit_error(&task_id, &e);
                Err(e)
            }
        }
    }

    /// 批量下载（Phase 5.2: post/like/mix/collects 迁移）
    ///
    /// 1. 创建任务
    /// 2. 调用 Python download_batch（不写 task DB 表）
    /// 3. 为每个结果创建 task_item + 保存 download_record/video_info/user_info
    /// 4. 更新任务状态
    pub fn start_batch_download_mode(&self, mode: DownloadMode, url: &str) -> Result<String, String> {
        let task_id = Uuid::new_v4().to_string();
        let task_id = task_id[..8].to_string();

        info!("[TaskService] start_batch_download_mode: task_id={}, mode={}", task_id, mode);

        // 1. 创建任务记录
        let new_task = crate::db::NewDownloadTask {
            id: task_id.clone(),
            mode: mode.as_str().to_string(),
            url: url.to_string(),
            title: None,
            author_nickname: None,
        };
        if let Err(e) = self.db.create_task(&new_task) {
            error!("[TaskService] 创建批量任务失败: {}", e);
            return Err(format!("创建任务失败: {}", e));
        }

        // 2. 发射启动事件
        events::emit_started(&task_id, mode, url);

        // 3. 调用 Python 适配器
        let result = self.call_python_batch_download(mode.as_str(), url);

        match result {
            Ok(py_result) => {
                if py_result.success {
                    let items = py_result.results.unwrap_or_default();
                    let total = items.len() as i64;
                    let mut completed: i64 = 0;
                    let mut skipped: i64 = 0;
                    let failed: i64 = 0; // batch 下载不追踪逐项失败

                    let mut records = Vec::new();
                    let mut videos = Vec::new();
                    let mut users = Vec::new();
                    if matches!(mode, DownloadMode::Post | DownloadMode::Mix) {
                        if let Some(profile) = py_result.user_profile.as_ref() {
                            if let Some(user_info) = parse_user_info(profile) {
                                if self.db.get_user_by_sec_uid(&user_info.sec_user_id).ok().flatten().is_none() {
                                    users.push(user_info);
                                }
                            }
                        }
                    }

                    // 4. 为每个结果创建 task_item + 收集 download_record/video_info/user_info
                    let task_start_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();

                    for item in &items {
                        let detail = item.detail.as_ref();
                        let aweme_id = detail
                            .and_then(|d| d.get("aweme_id"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        let file_path = item.path.as_deref();

                        // 创建 task_item — 失败则返回错误
                        let new_item = NewTaskItem {
                            task_id: task_id.clone(),
                            aweme_id: Some(aweme_id.to_string()),
                            title: json_str(detail, "desc"),
                            author_nickname: json_str(detail, "author_nickname"),
                            cover_url: json_str(detail, "cover_url"),
                        };
                        if let Err(e) = self.db.create_task_item(&new_item) {
                            error!("[TaskService] 创建批量任务子项失败: task_id={}, error={}", task_id, e);
                            return Err(format!("创建批量任务子项失败: {}", e));
                        }

                        // 获取文件大小
                        let file_size: i64 = file_path
                            .and_then(|p| std::fs::metadata(p).ok())
                            .map(|m| m.len() as i64)
                            .unwrap_or(0);

                        // 判断是否跳过（文件已存在且修改时间早于任务开始时间）
                        let is_skipped = file_path
                            .and_then(|p| std::fs::metadata(p).ok())
                            .and_then(|m| m.modified().ok())
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs() < task_start_time)
                            .unwrap_or(false);

                        if is_skipped {
                            if let Err(e) = self.db.update_task_item_status(
                                &task_id, aweme_id, "skipped", file_path, file_size, None,
                            ) {
                                error!("[TaskService] 更新批量跳过子项状态失败: task_id={}, error={}", task_id, e);
                                return Err(format!("更新批量跳过子项状态失败: {}", e));
                            }
                            skipped += 1;
                        } else {
                            if let Err(e) = self.db.update_task_item_status(
                                &task_id, aweme_id, "completed", file_path, file_size, None,
                            ) {
                                error!("[TaskService] 更新批量完成子项状态失败: task_id={}, error={}", task_id, e);
                                return Err(format!("更新批量完成子项状态失败: {}", e));
                            }
                            completed += 1;
                        }

                        // 收集 download_record
                        let download_type = match mode {
                            DownloadMode::Post => "user_post",
                            DownloadMode::Like => "user_like",
                            DownloadMode::Mix => "mix",
                            DownloadMode::Collects => "collects",
                            _ => "batch",
                        };
                        records.push(NewDownloadRecord {
                            aweme_id: json_str(detail, "aweme_id"),
                            download_type: download_type.to_string(),
                            title: json_str(detail, "desc"),
                            author_nickname: json_str(detail, "author_nickname"),
                            author_sec_uid: json_str(detail, "author_sec_uid"),
                            file_path: file_path.map(|s| s.to_string()),
                            file_size,
                            cover_url: json_str(detail, "cover_url"),
                            status: "completed".to_string(),
                            error_msg: None,
                        });

                        // 收集完整 video_info/user_info，保持旧 Python db_bridge 的 JSON 清洗语义。
                        if let Some(d) = detail {
                            if d.get("aweme_id").and_then(|v| v.as_str()).is_some() {
                                if let Ok(video_info) = serde_json::from_value::<crate::db::VideoInfo>(cleaned_json(d)) {
                                    videos.push(video_info);
                                }
                            }
                        }
                    }

                    // 5. 批量保存 download_record + video_info + user_info — 失败则返回错误
                    if let Err(e) = self.db.save_batch_results(&records, &videos, &users) {
                        error!("[TaskService] 保存批量下载记录失败: task_id={}, error={}", task_id, e);
                        return Err(format!("保存批量下载记录失败: {}", e));
                    }

                    // 6. 更新任务计数和状态 — 失败则返回错误
                    if let Err(e) = self.db.update_task_counts(&task_id) {
                        error!("[TaskService] 更新批量任务计数失败: task_id={}, error={}", task_id, e);
                        return Err(format!("更新批量任务计数失败: {}", e));
                    }

                    // 更新任务状态为 completed — 失败则不发射完成事件
                    if let Err(e) = self.db.update_task_status(&task_id, "completed", None) {
                        error!("[TaskService] 更新批量任务完成状态失败: task_id={}, error={}", task_id, e);
                        return Err(format!("批量任务完成但状态更新失败: {}", e));
                    }

                    // 7. 发射完成事件（仅在 DB 写入成功后）
                    let patch = TaskPatch::new(&task_id)
                        .with_status(TaskStatus::Completed)
                        .with_counts(total, completed, failed, skipped);
                    events::emit_finished(patch);

                    info!(
                        "[TaskService] 批量下载完成: task_id={}, total={}, completed={}, skipped={}, failed={}",
                        task_id, total, completed, skipped, failed
                    );
                } else {
                    let err = py_result.error.unwrap_or_else(|| "未知错误".to_string());
                    warn!("[TaskService] 批量下载失败: task_id={}, error={}", task_id, err);
                    if let Err(db_err) = self.db.update_task_status(&task_id, "error", Some(&err)) {
                        error!("[TaskService] 更新批量错误状态也失败: task_id={}, db_error={}", task_id, db_err);
                    }
                    events::emit_error(&task_id, &err);
                }
                Ok(task_id)
            }
            Err(e) => {
                error!("[TaskService] 批量下载异常: task_id={}, error={}", task_id, e);
                if let Err(db_err) = self.db.update_task_status(&task_id, "error", Some(&e)) {
                    error!("[TaskService] 更新批量错误状态也失败: task_id={}, db_error={}", task_id, db_err);
                }
                events::emit_error(&task_id, &e);
                Err(e)
            }
        }
    }

    /// 调用 Python 批量下载
    fn call_python_batch_download(&self, mode: &str, url: &str) -> Result<PythonBatchDownloadResult, String> {
        let json_value = python::handler::download_batch(mode, url)
            .map_err(|e| format!("Python 调用失败: {}", e))?;

        serde_json::from_value::<PythonBatchDownloadResult>(json_value)
            .map_err(|e| format!("Python 返回值解析失败: {}", e))
    }

    /// 调用 Python 音乐批量下载
    fn call_python_music_batch(&self, url: &str) -> Result<PythonMusicBatchResult, String> {
        let json_value = python::handler::download_music_batch(url)
            .map_err(|e| format!("Python 调用失败: {}", e))?;

        serde_json::from_value::<PythonMusicBatchResult>(json_value)
            .map_err(|e| format!("Python 返回值解析失败: {}", e))
    }

    /// 调用 Python 单视频下载并标准化结果
    /// 注意：此函数调用 download_video 进行实际下载，返回文件路径和元数据
    fn call_python_download(&self, url: &str) -> Result<PythonDownloadResult, String> {
        let json_value = python::handler::download_video(url)
            .map_err(|e| format!("Python 调用失败: {}", e))?;

        serde_json::from_value::<PythonDownloadResult>(json_value)
            .map_err(|e| format!("Python 返回值解析失败: {}", e))
    }

}
