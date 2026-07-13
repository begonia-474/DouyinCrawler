//! TaskApplicationService — 任务应用服务 (Async Version)
//!
//! 职责：
//! 1. 创建任务（Rust 拥有 DB 写入）
//! 2. 通过 typed resolver 获取媒体计划（Python 解析）
//! 3. 通过 DownloadEngine 执行实际下载（Rust 原生 HTTP）
//! 4. 写入 DB 事务（task + items + video_info + user_info）
//! 5. 发射类型化事件（TaskEvent → 前端）
//! 6. 返回类型化响应
//!
//! Phase 3 变更：
//! - start_download / start_batch_download_mode / start_music_download 改为 async
//! - 下载逻辑移到 tokio::spawn 后台任务
//! - 集成 DownloadEngine 替代 Python 下载
//! - 集成 typed single/paged/live resolver 与 music-only resolver
//! - 集成取消信号（通过 AppState）
//! - 使用 emit_progress 发射进度事件
//! - 任务完成后清理取消信号

use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use log::{error, info, warn};
use serde_json::Value;
use uuid::Uuid;

use crate::db::{
    Database, LiveTerminalCommit, LiveTerminalStatus, MediaItemOutcome, MediaItemResult,
    NewTaskItem, RecordingLiveRecord, VideoInfo,
};
use crate::state::AppState;

use super::contract::{
    PagedDownloadPlanV1, SingleAccessory, SingleAccessoryKind, SingleDownloadItem,
    SingleDownloadPlanV1, SingleMediaKind,
};
use super::engine::{DownloadEngine, DownloadItem, DownloadUrl, EngineConfig};
use super::events;
use super::live::{
    LiveFailureKind, LiveOutputFacts, LivePlanV1, LiveRecorder, LiveRecorderOutcome,
};
use super::selection::SelectionTracker;
use super::{
    DownloadMode, DownloadRequest, MusicResolvedItem, MusicResolvedUrls, TaskEvent,
    TaskEventType, TaskPatch, TaskStatus,
};

// ============================================================
// 辅助函数
// ============================================================

fn unix_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
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

/// 从 music-only resolver item 构建 DownloadItem
///
/// 当 item.folder_name 存在时（folderize=True），在 save_dir 下创建子目录。
fn build_music_download_item(
    item: &MusicResolvedItem,
    save_dir: &str,
    task_id: &str,
) -> DownloadItem {
    let base_dir = match &item.folder_name {
        Some(folder) if !folder.is_empty() => PathBuf::from(save_dir).join(folder),
        _ => PathBuf::from(save_dir),
    };
    let save_path = base_dir.join(format!("{}{}", item.filename, item.suffix));
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

/// Build a download-engine item directly from the versioned mode=one contract.
fn build_single_download_item(
    item: &SingleDownloadItem,
    save_dir: &str,
    task_id: &str,
) -> DownloadItem {
    let base_dir = match &item.output.folder_name {
        Some(folder) => PathBuf::from(save_dir).join(folder),
        None => PathBuf::from(save_dir),
    };
    let save_path = base_dir.join(format!("{}{}", item.output.filename, item.output.suffix));
    let ext = item.output.suffix.trim_start_matches('.');
    let temp_path = save_path.with_extension(format!("{}.tmp", ext));
    let url = match item.urls.as_slice() {
        [url] => DownloadUrl::Single(url.clone()),
        urls => DownloadUrl::Multiple(urls.to_vec()),
    };

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
        max_concurrent: config.max_tasks as usize, // 并发下载任务数 = max_tasks
        max_retries: config.max_retries,
        timeout: config.timeout as u64,
        max_connections: config.max_connections as usize, // 单 URL 并发连接数
        cookie: config.cookie.clone(),
        proxy: config.proxy.clone(),
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
    event_sink: Arc<dyn TaskEventSink>,
}

#[derive(Debug, PartialEq, Eq)]
enum TaskTerminal {
    Completed,
    Cancelled,
    Error(String),
}

pub(super) type SinglePlanFuture =
    Pin<Box<dyn Future<Output = Result<SingleDownloadPlanV1, String>> + Send>>;

pub(super) trait SingleDownloadResolver: Send + Sync {
    fn resolve(&self, url: String) -> SinglePlanFuture;
}

pub(super) type PagedPlanFuture =
    Pin<Box<dyn Future<Output = Result<PagedDownloadPlanV1, String>> + Send>>;

pub(super) trait PagedDownloadResolver: Send + Sync {
    fn resolve_page(&self, mode: String, url: String, cursor: i64, count: i64) -> PagedPlanFuture;
}

pub(super) type LivePlanFuture = Pin<Box<dyn Future<Output = Result<LivePlanV1, String>> + Send>>;

pub(super) trait LiveResolver: Send + Sync {
    fn resolve(&self, url: String) -> LivePlanFuture;
}

pub(super) type LiveRecorderFuture = Pin<Box<dyn Future<Output = LiveRecorderOutcome> + Send>>;
pub(super) type LiveProgressCallback = Arc<dyn Fn(u64) + Send + Sync>;

pub(super) trait LiveRecorderRunner: Send + Sync {
    fn record(
        &self,
        config: crate::config::AppConfig,
        cancel_signal: Arc<AtomicBool>,
        plan: LivePlanV1,
        progress: LiveProgressCallback,
    ) -> LiveRecorderFuture;
}

pub(super) trait TaskEventSink: Send + Sync {
    fn emit(&self, event: TaskEvent);
}

struct PythonSingleDownloadResolver;

impl SingleDownloadResolver for PythonSingleDownloadResolver {
    fn resolve(&self, url: String) -> SinglePlanFuture {
        Box::pin(async move { TaskApplicationService::resolve_single_download(&url).await })
    }
}

struct PythonPagedDownloadResolver;

impl PagedDownloadResolver for PythonPagedDownloadResolver {
    fn resolve_page(&self, mode: String, url: String, cursor: i64, count: i64) -> PagedPlanFuture {
        Box::pin(async move {
            TaskApplicationService::resolve_paged_download_plan(&mode, &url, cursor, count).await
        })
    }
}

struct PythonLiveResolver;

impl LiveResolver for PythonLiveResolver {
    fn resolve(&self, url: String) -> LivePlanFuture {
        Box::pin(async move { TaskApplicationService::resolve_live(&url).await })
    }
}

struct RustLiveRecorderRunner;

impl LiveRecorderRunner for RustLiveRecorderRunner {
    fn record(
        &self,
        config: crate::config::AppConfig,
        cancel_signal: Arc<AtomicBool>,
        plan: LivePlanV1,
        progress: LiveProgressCallback,
    ) -> LiveRecorderFuture {
        Box::pin(async move {
            match LiveRecorder::new(&config, cancel_signal) {
                Ok(recorder) => recorder.record(&plan, move |bytes| progress(bytes)).await,
                Err(error) => {
                    let now = unix_timestamp();
                    LiveRecorderOutcome::Failed {
                        kind: LiveFailureKind::InvalidPlan,
                        error,
                        output: LiveOutputFacts {
                            path: plan.full_path(),
                            file_size: 0,
                            started_at: now,
                            ended_at: now,
                        },
                    }
                }
            }
        })
    }
}

struct TauriTaskEventSink;

impl TaskEventSink for TauriTaskEventSink {
    fn emit(&self, event: TaskEvent) {
        events::emit_task_event(&event);
    }
}

#[derive(Clone)]
struct TaskRuntimeAdapters {
    single_resolver: Arc<dyn SingleDownloadResolver>,
    paged_resolver: Arc<dyn PagedDownloadResolver>,
    live_resolver: Arc<dyn LiveResolver>,
    live_recorder: Arc<dyn LiveRecorderRunner>,
    event_sink: Arc<dyn TaskEventSink>,
}

struct PagedRunRequest<'a> {
    mode: DownloadMode,
    url: &'a str,
    aweme_ids: &'a [String],
}

impl ProgressTracker {
    fn new(task_id: String) -> Self {
        Self::with_sink(task_id, Arc::new(TauriTaskEventSink))
    }

    fn with_sink(task_id: String, event_sink: Arc<dyn TaskEventSink>) -> Self {
        Self {
            task_id,
            last_emit_ms: AtomicU64::new(0),
            interval_ms: 500,
            event_sink,
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
            self.event_sink.emit(TaskEvent::progress(
                TaskPatch::new(&self.task_id).with_counts(total as i64, downloaded as i64, 0, 0),
            ));
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
    adapters: TaskRuntimeAdapters,
}

impl<'a> TaskApplicationService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self {
            state,
            adapters: TaskRuntimeAdapters {
                single_resolver: Arc::new(PythonSingleDownloadResolver),
                paged_resolver: Arc::new(PythonPagedDownloadResolver),
                live_resolver: Arc::new(PythonLiveResolver),
                live_recorder: Arc::new(RustLiveRecorderRunner),
                event_sink: Arc::new(TauriTaskEventSink),
            },
        }
    }

    #[cfg(test)]
    pub(super) fn with_test_adapters(
        state: &'a AppState,
        single_resolver: Arc<dyn SingleDownloadResolver>,
        event_sink: Arc<dyn TaskEventSink>,
    ) -> Self {
        Self {
            state,
            adapters: TaskRuntimeAdapters {
                single_resolver,
                paged_resolver: Arc::new(PythonPagedDownloadResolver),
                live_resolver: Arc::new(PythonLiveResolver),
                live_recorder: Arc::new(RustLiveRecorderRunner),
                event_sink,
            },
        }
    }

    #[cfg(test)]
    pub(super) fn with_paged_test_adapters(
        state: &'a AppState,
        paged_resolver: Arc<dyn PagedDownloadResolver>,
        event_sink: Arc<dyn TaskEventSink>,
    ) -> Self {
        Self {
            state,
            adapters: TaskRuntimeAdapters {
                single_resolver: Arc::new(PythonSingleDownloadResolver),
                paged_resolver,
                live_resolver: Arc::new(PythonLiveResolver),
                live_recorder: Arc::new(RustLiveRecorderRunner),
                event_sink,
            },
        }
    }

    #[cfg(test)]
    pub(super) fn with_live_test_adapters(
        state: &'a AppState,
        live_resolver: Arc<dyn LiveResolver>,
        live_recorder: Arc<dyn LiveRecorderRunner>,
        event_sink: Arc<dyn TaskEventSink>,
    ) -> Self {
        Self {
            state,
            adapters: TaskRuntimeAdapters {
                single_resolver: Arc::new(PythonSingleDownloadResolver),
                paged_resolver: Arc::new(PythonPagedDownloadResolver),
                live_resolver,
                live_recorder,
                event_sink,
            },
        }
    }

    /// 获取数据库引用
    fn db(&self) -> &Database {
        self.state.db.as_ref()
    }

    // ============================================================
    // 内部辅助方法
    // ============================================================

    /// Resolve and validate the versioned mode=one contract before downloading.
    async fn resolve_single_download(url: &str) -> Result<SingleDownloadPlanV1, String> {
        let url = url.to_string();
        let json_value = tokio::task::spawn_blocking(move || {
            crate::python::handler::resolve_single(&url)
                .map_err(|e| format!("resolve_single 调用失败: {}", e))
        })
        .await
        .map_err(|e| format!("spawn_blocking 失败: {}", e))??;

        if json_value.get("success").and_then(Value::as_bool) == Some(false) {
            return Err(json_value
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("单视频解析失败")
                .to_string());
        }

        SingleDownloadPlanV1::from_value(json_value)
    }

    /// Resolve music download URLs (typed, mode=music-only)
    async fn resolve_music_urls(url: &str) -> Result<MusicResolvedUrls, String> {
        let url = url.to_string();
        let json_value = tokio::task::spawn_blocking(move || {
            crate::python::handler::resolve_music_urls(&url)
                .map_err(|e| format!("resolve_music_urls 调用失败: {}", e))
        })
        .await
        .map_err(|e| format!("spawn_blocking 失败: {}", e))??;

        serde_json::from_value::<MusicResolvedUrls>(json_value)
            .map_err(|e| format!("resolve_music_urls 返回值解析失败: {}", e))
    }

    /// Resolve a single paged page and validate the typed contract.
    ///
    /// Used by the typed paged runner for post mode; like/mix/collects
    /// still use the legacy `resolve_download_page` until issue 08.
    async fn resolve_paged_download_plan(
        mode: &str,
        url: &str,
        cursor: i64,
        count: i64,
    ) -> Result<PagedDownloadPlanV1, String> {
        let expected_mode = mode.to_string();
        let resolver_mode = expected_mode.clone();
        let url = url.to_string();
        let json_value = tokio::task::spawn_blocking(move || {
            crate::python::handler::resolve_page(&resolver_mode, &url, cursor, count)
                .map_err(|e| format!("resolve_page 调用失败: {}", e))
        })
        .await
        .map_err(|e| format!("spawn_blocking 失败: {}", e))??;

        if json_value.get("success").and_then(Value::as_bool) == Some(false) {
            return Err(json_value
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("分页解析失败")
                .to_string());
        }

        PagedDownloadPlanV1::from_value_for_mode(json_value, &expected_mode)
    }

    /// 通过 Python 解析直播元数据和 f2 FULL_HD1 录制地址。
    async fn resolve_live(url: &str) -> Result<LivePlanV1, String> {
        let url = url.to_string();
        let json_value = tokio::task::spawn_blocking(move || {
            crate::python::handler::resolve_live(&url)
                .map_err(|e| format!("resolve_live 调用失败: {}", e))
        })
        .await
        .map_err(|e| format!("spawn_blocking 失败: {}", e))??;

        if json_value.get("success").and_then(Value::as_bool) == Some(false) {
            return Err(json_value
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("直播解析失败")
                .to_string());
        }
        LivePlanV1::from_value(json_value)
    }

    /// 创建下载引擎并绑定取消信号
    fn create_engine(
        app_config: &crate::config::AppConfig,
        cancel_signal: Arc<AtomicBool>,
    ) -> DownloadEngine {
        let config = build_engine_config(app_config);
        DownloadEngine::new(config).with_cancel_signal(cancel_signal)
    }

    /// Process mode=one accessories without converting them to the legacy resolver shape.
    async fn download_single_accessories(
        engine: &DownloadEngine,
        accessories: &[SingleAccessory],
        save_dir: &str,
        item_folder_name: &Option<String>,
        task_id: &str,
        app_config: &crate::config::AppConfig,
    ) -> Vec<String> {
        let mut downloaded_paths = Vec::new();

        for accessory in accessories {
            let should_download = match accessory.kind {
                SingleAccessoryKind::Music => app_config.music,
                SingleAccessoryKind::Cover => app_config.cover,
                SingleAccessoryKind::Description => app_config.desc,
            };
            if !should_download {
                continue;
            }

            let folder_name = accessory
                .output
                .folder_name
                .as_ref()
                .or(item_folder_name.as_ref());
            let base_dir = match folder_name {
                Some(folder) => PathBuf::from(save_dir).join(folder),
                None => PathBuf::from(save_dir),
            };
            let accessory_path = base_dir.join(format!(
                "{}{}",
                accessory.output.filename, accessory.output.suffix
            ));

            match accessory.kind {
                SingleAccessoryKind::Description => {
                    if let Some(content) = &accessory.content {
                        match tokio::fs::write(&accessory_path, content).await {
                            Ok(()) => {
                                info!("[TaskService] 文案已保存: {}", accessory_path.display());
                                downloaded_paths.push(accessory_path.to_string_lossy().to_string());
                            }
                            Err(error) => warn!(
                                "[TaskService] 文案保存失败: {}, error={}",
                                accessory_path.display(),
                                error
                            ),
                        }
                    }
                }
                SingleAccessoryKind::Music | SingleAccessoryKind::Cover => {
                    let Some(url) = accessory.url.clone() else {
                        continue;
                    };
                    let ext = accessory.output.suffix.trim_start_matches('.');
                    let temp_path = accessory_path.with_extension(format!("{}.tmp", ext));
                    let download_item = DownloadItem {
                        url: DownloadUrl::Single(url),
                        save_path: accessory_path,
                        temp_path,
                        headers: HashMap::new(),
                        task_id: task_id.to_string(),
                        file_size: None,
                    };

                    match engine.download(&download_item, |_, _| {}).await {
                        Ok(result) => {
                            info!(
                                "[TaskService] 附属文件已下载: {} ({} bytes)",
                                result.path.display(),
                                result.file_size
                            );
                            downloaded_paths.push(result.path.to_string_lossy().to_string());
                        }
                        Err(error) => warn!(
                            "[TaskService] 附属文件下载失败: {}, error={}",
                            accessory.output.filename, error
                        ),
                    }
                }
            }
        }

        downloaded_paths
    }

    /// 清理取消信号
    fn cleanup_cancel_signal(
        cancel_signals: &Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
        task_id: &str,
    ) {
        let mut signals = cancel_signals.lock();
        signals.remove(task_id);
    }

    /// 启动 Rust 原生直播录制。
    ///
    /// Python 只负责解析直播元数据和 FULL_HD1 流地址；Rust 负责录制、取消、
    /// 事件和数据库持久化。
    pub async fn start_live_record(&self, url: &str) -> Result<String, String> {
        let task_id = Uuid::new_v4().to_string();
        let task_id = task_id[..8].to_string();

        let new_task = crate::db::NewDownloadTask {
            id: task_id.clone(),
            mode: DownloadMode::Live.as_str().to_string(),
            url: url.to_string(),
            title: None,
            author_nickname: None,
        };
        self.db()
            .create_task(&new_task)
            .map_err(|e| format!("创建直播任务失败: {}", e))?;
        self.db()
            .update_task_status(&task_id, "starting", None)
            .map_err(|e| format!("更新直播任务状态失败: {}", e))?;

        self.adapters.event_sink.emit(TaskEvent {
            event_type: TaskEventType::Started,
            task_id: task_id.clone(),
            mode: Some(DownloadMode::Live),
            url: Some(url.to_string()),
            patch: TaskPatch::new(&task_id).with_status(TaskStatus::Starting),
        });

        let cancel_signal = self.state.register_cancel_signal(&task_id);
        let db = self.state.db.clone();
        let cancel_signals = self.state.cancel_signals.clone();
        let app_config = self.state.config.lock().get_douyin_config();
        let live_resolver = self.adapters.live_resolver.clone();
        let live_recorder = self.adapters.live_recorder.clone();
        let event_sink = self.adapters.event_sink.clone();
        let task_id_clone = task_id.clone();
        let url = url.to_string();

        tokio::spawn(async move {
            let plan = match live_resolver.resolve(url.clone()).await {
                Ok(plan) => plan,
                Err(error) => {
                    match db.update_task_status(&task_id_clone, "error", Some(&error)) {
                        Ok(()) => event_sink.emit(TaskEvent {
                            event_type: TaskEventType::Finished,
                            task_id: task_id_clone.clone(),
                            mode: Some(DownloadMode::Live),
                            url: Some(url.clone()),
                            patch: TaskPatch::new(&task_id_clone).with_error(error),
                        }),
                        Err(db_error) => error!(
                            "[TaskService] 直播解析失败后状态持久化失败: task_id={}, error={}",
                            task_id_clone, db_error
                        ),
                    }
                    Self::cleanup_cancel_signal(&cancel_signals, &task_id_clone);
                    return;
                }
            };

            let recording_started_at = unix_timestamp();
            let target_path = plan.full_path().to_string_lossy().to_string();
            let recording = RecordingLiveRecord {
                task_id: task_id_clone.clone(),
                room_id: plan.room_id.clone(),
                web_rid: plan.web_rid.clone(),
                title: plan.title.clone(),
                nickname: plan.nickname.clone(),
                sec_user_id: plan.sec_user_id.clone(),
                cover_url: plan.cover_url.clone(),
                file_path: target_path,
                started_at: recording_started_at,
            };
            if let Err(db_error) = db.create_recording_live_record(&recording) {
                let message = format!("创建直播 recording 状态失败: {db_error}");
                if db
                    .update_task_status(&task_id_clone, "error", Some(&message))
                    .is_ok()
                {
                    event_sink.emit(TaskEvent {
                        event_type: TaskEventType::Finished,
                        task_id: task_id_clone.clone(),
                        mode: Some(DownloadMode::Live),
                        url: Some(url.clone()),
                        patch: TaskPatch::new(&task_id_clone).with_error(message),
                    });
                } else {
                    error!(
                        "[TaskService] 创建直播状态失败且无法记录 task error: task_id={}",
                        task_id_clone
                    );
                }
                Self::cleanup_cancel_signal(&cancel_signals, &task_id_clone);
                return;
            }

            let active_status = if cancel_signal.load(Ordering::Relaxed) {
                TaskStatus::Stopping
            } else {
                TaskStatus::Recording
            };
            event_sink.emit(TaskEvent {
                event_type: TaskEventType::Progress,
                task_id: task_id_clone.clone(),
                mode: Some(DownloadMode::Live),
                url: Some(url.clone()),
                patch: TaskPatch::new(&task_id_clone)
                    .with_status(active_status)
                    .with_live_metadata(
                        &plan.title,
                        &plan.nickname,
                        &plan.room_id,
                        &plan.web_rid,
                        &plan.cover_url,
                    ),
            });

            let progress_task_id = task_id_clone.clone();
            let progress_sink = event_sink.clone();
            let progress: LiveProgressCallback = Arc::new(move |downloaded| {
                let mut patch =
                    TaskPatch::new(&progress_task_id).with_status(TaskStatus::Recording);
                patch.file_size = Some(downloaded as i64);
                progress_sink.emit(TaskEvent {
                    event_type: TaskEventType::Progress,
                    task_id: progress_task_id.clone(),
                    mode: Some(DownloadMode::Live),
                    url: None,
                    patch,
                });
            });
            let outcome = live_recorder
                .record(app_config, cancel_signal, plan.clone(), progress)
                .await;
            let output = outcome.output().clone();
            let (terminal_status, outcome_error) = match &outcome {
                LiveRecorderOutcome::Completed { reason, .. } => {
                    info!(
                        "[TaskService] live recorder completed: task_id={}, reason={:?}",
                        task_id_clone, reason
                    );
                    (LiveTerminalStatus::Completed, None)
                }
                LiveRecorderOutcome::Failed { kind, error, .. } => {
                    warn!(
                        "[TaskService] live recorder failed: task_id={}, kind={:?}, error={}",
                        task_id_clone, kind, error
                    );
                    (LiveTerminalStatus::Error, Some(error.clone()))
                }
            };
            let terminal = LiveTerminalCommit {
                task_id: task_id_clone.clone(),
                status: terminal_status,
                file_path: output.path.to_string_lossy().to_string(),
                file_size: output.file_size as i64,
                duration_sec: output.duration_sec(),
                started_at: output.started_at,
                ended_at: output.ended_at,
                error_msg: outcome_error.clone(),
            };

            let committed = match db.commit_live_terminal(&terminal) {
                Ok(()) => Some(outcome_error),
                Err(commit_error) => {
                    let original = outcome_error
                        .as_deref()
                        .unwrap_or("recorder completed successfully");
                    let persistence_error = format!(
                        "直播终态持久化失败: {commit_error}; 原始 recorder outcome: {original}"
                    );
                    let error_terminal = LiveTerminalCommit {
                        task_id: task_id_clone.clone(),
                        status: LiveTerminalStatus::Error,
                        file_path: output.path.to_string_lossy().to_string(),
                        file_size: output.file_size as i64,
                        duration_sec: output.duration_sec(),
                        started_at: output.started_at,
                        ended_at: output.ended_at,
                        error_msg: Some(persistence_error.clone()),
                    };
                    match db.commit_live_terminal(&error_terminal) {
                        Ok(()) => Some(Some(persistence_error)),
                        Err(fallback_error) => {
                            error!(
                                "[TaskService] 直播终态及 fallback 均无法持久化: task_id={}, terminal_error={}, fallback_error={}, recorder={}",
                                task_id_clone, commit_error, fallback_error, original
                            );
                            None
                        }
                    }
                }
            };

            if let Some(error_state) = committed {
                let mut patch = TaskPatch::new(&task_id_clone)
                    .with_status(if error_state.is_some() {
                        TaskStatus::Error
                    } else {
                        TaskStatus::Completed
                    })
                    .with_live_metadata(
                        &plan.title,
                        &plan.nickname,
                        &plan.room_id,
                        &plan.web_rid,
                        &plan.cover_url,
                    )
                    .with_live_result(
                        output.path.to_string_lossy().to_string(),
                        output.file_size as i64,
                        output.duration_sec(),
                        output.started_at,
                        output.ended_at,
                    );
                if let Some(message) = error_state {
                    patch = patch.with_error(message);
                }
                event_sink.emit(TaskEvent {
                    event_type: TaskEventType::Finished,
                    task_id: task_id_clone.clone(),
                    mode: Some(DownloadMode::Live),
                    url: Some(url),
                    patch,
                });
            }

            Self::cleanup_cancel_signal(&cancel_signals, &task_id_clone);
        });

        Ok(task_id)
    }

    /// 请求停止直播。只写 stopping 和设置 token；后台录制器是唯一终态 owner。
    pub fn stop_live_record(&self, task_id: &str) -> Result<(), String> {
        let signal = self
            .state
            .get_cancel_signal(task_id)
            .ok_or_else(|| "录制任务不存在或已完成".to_string())?;
        self.db()
            .request_live_stop(task_id)
            .map_err(|error| format!("请求停止直播失败: {error}"))?;
        signal.store(true, Ordering::Relaxed);
        self.adapters.event_sink.emit(TaskEvent {
            event_type: TaskEventType::Progress,
            task_id: task_id.to_string(),
            mode: Some(DownloadMode::Live),
            url: None,
            patch: TaskPatch::new(task_id).with_status(TaskStatus::Stopping),
        });
        Ok(())
    }

    // ============================================================
    // 统一下载入口（mode=one）
    // ============================================================

    fn typed_media_task_item(
        task_id: &str,
        item: &SingleDownloadItem,
    ) -> Result<NewTaskItem, String> {
        Ok(NewTaskItem {
            task_id: task_id.to_string(),
            aweme_id: Some(item.aweme_id.clone()),
            media_key: Some(item.media_key.clone()),
            media_kind: Some(
                match item.kind {
                    SingleMediaKind::Video => "video",
                    SingleMediaKind::Image => "image",
                    SingleMediaKind::LivePhoto => "live_photo",
                }
                .to_string(),
            ),
            media_index: Some(item.media_index()?),
            title: item.metadata.desc.clone(),
            author_nickname: item.metadata.author_nickname.clone(),
            author_sec_uid: item.metadata.author_sec_uid.clone(),
            cover_url: item.metadata.cover_url.clone(),
        })
    }

    /// 通过结果级事务接口提交 mode=one 媒体项。
    ///
    /// 首次事务失败时，原事务已经回滚。这里使用同一接口发起独立的
    /// failed 结果事务，尽最大可能留下可见 item 错误；恢复写入失败会被
    /// 明确记录并合并到返回错误，最终由顶层任务终态协调器处理。
    fn persist_single_media_item_outcome<'result>(
        db: &Database,
        task_item: &NewTaskItem,
        video_info: &VideoInfo,
        outcome: MediaItemOutcome<'result>,
    ) -> Result<(), String> {
        let result = MediaItemResult {
            item: task_item,
            outcome,
            video_info: Some(video_info),
            user_info: None,
        };
        if let Err(db_error) = db.commit_media_item_result(&result) {
            let message = format!("提交媒体项结果失败: {db_error}");
            error!(
                "[TaskService] {}: task_id={}, aweme_id={}",
                message,
                task_item.task_id,
                task_item.aweme_id.as_deref().unwrap_or("<missing>")
            );

            let recovery_message = match outcome {
                MediaItemOutcome::Failed { error_msg } => {
                    format!("{message}; 原始媒体错误: {error_msg}")
                }
                MediaItemOutcome::Completed { .. } | MediaItemOutcome::Skipped { .. } => {
                    format!("{message}; 已下载文件可能仍保留在磁盘")
                }
            };
            let recovery_result = MediaItemResult {
                item: task_item,
                outcome: MediaItemOutcome::Failed {
                    error_msg: &recovery_message,
                },
                video_info: None,
                user_info: None,
            };
            if let Err(recovery_error) = db.commit_media_item_result(&recovery_result) {
                error!(
                    "[TaskService] 媒体项错误状态恢复写入失败: task_id={}, aweme_id={}, error={}",
                    task_item.task_id,
                    task_item.aweme_id.as_deref().unwrap_or("<missing>"),
                    recovery_error
                );
                return Err(format!(
                    "{message}; 媒体项错误状态恢复写入失败: {recovery_error}"
                ));
            }
            return Err(message);
        }

        Ok(())
    }

    /// 将 mode=one 执行结果转换为唯一允许发射的终态语义。
    ///
    /// `Completed` 仅在 task completed 状态成功提交后返回，因此调用者按
    /// 返回值发事件时不会在数据库失败后误发 finished/completed。
    fn finalize_task(
        db: &Database,
        task_id: &str,
        execution_result: Result<(), String>,
        cancelled: bool,
    ) -> TaskTerminal {
        let is_explicit_cancellation = cancelled
            && execution_result
                .as_ref()
                .is_err_and(|message| message == "下载已取消");
        if is_explicit_cancellation {
            return match db.update_task_status(task_id, "cancelled", None) {
                Ok(()) => TaskTerminal::Cancelled,
                Err(db_error) => {
                    let message = format!("更新取消状态失败: {db_error}");
                    error!("[TaskService] task_id={}, {}", task_id, message);
                    TaskTerminal::Error(message)
                }
            };
        }

        match execution_result {
            Ok(()) => match db.update_task_status(task_id, "completed", None) {
                Ok(()) => TaskTerminal::Completed,
                Err(db_error) => {
                    let mut message = format!("提交任务完成状态失败: {db_error}");
                    error!("[TaskService] task_id={}, {}", task_id, message);
                    if let Err(error_status_error) =
                        db.update_task_status(task_id, "error", Some(&message))
                    {
                        error!(
                            "[TaskService] 任务完成状态失败后的 error 标记也失败: task_id={}, error={}",
                            task_id, error_status_error
                        );
                        message
                            .push_str(&format!("; 任务 error 状态写入失败: {error_status_error}"));
                    }
                    TaskTerminal::Error(message)
                }
            },
            Err(mut message) => {
                if let Err(db_error) = db.update_task_status(task_id, "error", Some(&message)) {
                    error!(
                        "[TaskService] 更新任务错误状态失败: task_id={}, error={}",
                        task_id, db_error
                    );
                    message.push_str(&format!("; 任务 error 状态写入失败: {db_error}"));
                }
                TaskTerminal::Error(message)
            }
        }
    }

    /// Emit item-count progress only after the corresponding result transaction
    /// has committed. HTTP byte progress must not be written into task item count
    /// fields; the frontend treats these values as durable media-row counts.
    fn emit_persisted_item_counts(
        db: &Database,
        task_id: &str,
        event_sink: &Arc<dyn TaskEventSink>,
    ) {
        match db.get_task_item_counts(task_id) {
            Ok(counts) => {
                event_sink.emit(TaskEvent::progress(TaskPatch::new(task_id).with_counts(
                    counts.total,
                    counts.completed,
                    counts.skipped,
                    counts.failed,
                )))
            }
            Err(error) => warn!(
                "[TaskService] 读取已提交任务计数失败，跳过进度事件: task_id={}, error={}",
                task_id, error
            ),
        }
    }

    /// Execute a batch of typed media items with a shared engine and event sink.
    ///
    /// Shared between `execute_single_download_plan` and the typed paged runner.
    /// Returns the number of completed and failed items.
    async fn execute_media_items(
        db: &Database,
        task_id: &str,
        items: &[SingleDownloadItem],
        save_dir: &str,
        cancel_signal: &Arc<AtomicBool>,
        app_config: &crate::config::AppConfig,
        event_sink: &Arc<dyn TaskEventSink>,
    ) -> Result<(i64, i64), String> {
        let engine = Self::create_engine(app_config, cancel_signal.clone());
        let total = items.len() as i64;
        let mut completed = 0_i64;
        let mut failed = 0_i64;

        for item in items.iter() {
            if cancel_signal.load(Ordering::Relaxed) {
                info!(
                    "[TaskService] 下载被取消: task_id={}, 已完成 {}/{}",
                    task_id,
                    completed + failed,
                    total
                );
                return Err("下载已取消".to_string());
            }

            let download_item = build_single_download_item(item, save_dir, task_id);
            match engine
                .download(&download_item, |_downloaded, _total_size| {})
                .await
            {
                Ok(result) => {
                    let file_path = result.path.to_string_lossy().to_string();
                    let file_size = result.file_size as i64;
                    let _ = Self::download_single_accessories(
                        &engine,
                        &item.accessories,
                        save_dir,
                        &item.output.folder_name,
                        task_id,
                        app_config,
                    )
                    .await;

                    let task_item = Self::typed_media_task_item(task_id, item)?;
                    let outcome = if result.skipped {
                        MediaItemOutcome::Skipped {
                            file_path: &file_path,
                            file_size,
                        }
                    } else {
                        MediaItemOutcome::Completed {
                            file_path: &file_path,
                            file_size,
                        }
                    };
                    Self::persist_single_media_item_outcome(
                        db,
                        &task_item,
                        &item.metadata,
                        outcome,
                    )?;

                    completed += 1;
                    Self::emit_persisted_item_counts(db, task_id, event_sink);
                }
                Err(error) => {
                    if matches!(error, super::engine::DownloadError::Cancelled) {
                        info!(
                            "[TaskService] 下载被取消: task_id={}, aweme_id={}",
                            task_id, item.aweme_id
                        );
                        return Err("下载已取消".to_string());
                    }
                    warn!(
                        "[TaskService] 下载失败: aweme_id={}, error={}",
                        item.aweme_id, error
                    );
                    let task_item = Self::typed_media_task_item(task_id, item)?;
                    let error_message = error.to_string();
                    Self::persist_single_media_item_outcome(
                        db,
                        &task_item,
                        &item.metadata,
                        MediaItemOutcome::Failed {
                            error_msg: &error_message,
                        },
                    )?;
                    failed += 1;
                    Self::emit_persisted_item_counts(db, task_id, event_sink);
                }
            }
        }

        Ok((completed, failed))
    }

    async fn execute_single_download_plan(
        db: &Database,
        task_id: &str,
        plan: SingleDownloadPlanV1,
        cancel_signal: &Arc<AtomicBool>,
        app_config: &crate::config::AppConfig,
        event_sink: Arc<dyn TaskEventSink>,
    ) -> Result<(), String> {
        let save_dir = plan.save_dir;
        tokio::fs::create_dir_all(&save_dir)
            .await
            .map_err(|error| format!("创建保存目录失败: {error}"))?;

        let (completed, failed) = Self::execute_media_items(
            db,
            task_id,
            &plan.items,
            &save_dir,
            cancel_signal,
            app_config,
            &event_sink,
        )
        .await?;

        if failed > 0 && completed == 0 {
            return Err(format!(
                "所有下载均失败: {failed}/{} failed",
                plan.items.len()
            ));
        }

        info!(
            "[TaskService] 单视频下载完成: task_id={}, total={}, completed={}, failed={}",
            task_id,
            plan.items.len(),
            completed,
            failed
        );
        Ok(())
    }

    /// Typed paged download runner for post mode.
    ///
    /// Python returns one versioned page at a time; Rust drives pagination,
    /// calls the shared media executor, and handles protocol errors.
    ///
    /// When `aweme_ids` is non-empty, creates a `SelectionTracker` that is the
    /// source of truth for requested/seen/planned state and classifies terminal
    /// missing/unavailable outcomes.
    async fn execute_paged_download_plan(
        db: &Database,
        task_id: &str,
        request: PagedRunRequest<'_>,
        cancel_signal: &Arc<AtomicBool>,
        app_config: &crate::config::AppConfig,
        adapters: &TaskRuntimeAdapters,
    ) -> Result<(), String> {
        let expected_mode = request.mode.as_str().to_string();
        let page_counts = app_config.page_counts as i64;
        let mut cursor: i64 = 0;
        let mut total_completed = 0_i64;
        let mut total_failed = 0_i64;
        let mut page_index: i64 = 0;
        let mut save_dir: Option<String> = None;
        let mut seen_cursors = HashSet::from([cursor]);
        let mut seen_media_keys = HashSet::new();
        let mut seen_aweme_ids: HashSet<String> = HashSet::new();
        let event_sink = adapters.event_sink.clone();
        let max_counts = app_config.max_counts as usize;
        let mut tracker = SelectionTracker::new(request.aweme_ids.to_vec());
        let selection_active = tracker.is_active();

        loop {
            if cancel_signal.load(Ordering::Relaxed) {
                info!("[TaskService] 分页下载被取消 (typed): task_id={}", task_id);
                return Err("下载已取消".to_string());
            }

            page_index += 1;
            info!(
                "[TaskService] typed 分页解析第 {} 页: task_id={}, cursor={}",
                page_index, task_id, cursor
            );

            // When selection is active, override max_counts: requested set is the limit.
            let request_count = if selection_active || max_counts == 0 {
                page_counts
            } else {
                let remaining = max_counts.saturating_sub(seen_aweme_ids.len());
                if remaining == 0 {
                    break;
                }
                page_counts.min(remaining as i64)
            };

            let plan = adapters
                .paged_resolver
                .resolve_page(
                    expected_mode.clone(),
                    request.url.to_string(),
                    cursor,
                    request_count,
                )
                .await
                .map_err(|error| {
                    tracker.contextualize(format!(
                        "第 {page_index} 页解析失败 (cursor={cursor}): {error}"
                    ))
                })?;

            if cancel_signal.load(Ordering::Relaxed) {
                return Err("下载已取消".to_string());
            }
            if plan.mode != expected_mode {
                return Err(tracker.contextualize(format!(
                    "第 {page_index} 页 mode 漂移: expected={expected_mode}, actual={}",
                    plan.mode
                )));
            }

            if page_index == 1 {
                save_dir = Some(plan.save_dir.clone());
                tokio::fs::create_dir_all(plan.save_dir.as_str())
                    .await
                    .map_err(|error| format!("创建保存目录失败: {error}"))?;

                if let Some(user_info) = &plan.user_profile {
                    db.get_user_by_sec_uid(&user_info.sec_user_id)
                        .map_err(|error| format!("查询用户资料失败: {error}"))?;
                    db.save_user(user_info)
                        .map_err(|error| format!("保存用户资料失败: {error}"))?;
                    let nickname = user_info.nickname.as_deref();
                    let title = nickname.or(Some("用户作品"));
                    db.update_task_metadata(task_id, title, nickname)
                        .map_err(|error| format!("更新任务资料失败: {error}"))?;
                }
            } else {
                if plan.save_dir != *save_dir.as_ref().expect("first page sets save_dir") {
                    return Err(tracker.contextualize(format!("第 {page_index} 页 save_dir 漂移")));
                }
                if plan.user_profile.is_some() {
                    return Err(tracker
                        .contextualize(format!("第 {page_index} 页不得重复返回 user_profile")));
                }
            }

            let current_save_dir = save_dir
                .as_ref()
                .ok_or_else(|| "分页解析未返回 save_dir".to_string())?;

            if plan.has_more && plan.next_cursor.is_none() {
                return Err(tracker.contextualize(format!(
                    "第 {} 页 has_more=true 但 next_cursor 为空",
                    page_index
                )));
            }

            if plan.page_aweme_ids.is_empty() && plan.has_more {
                return Err(tracker.contextualize(format!(
                    "第 {} 页 page_aweme_ids 为空但 has_more=true (重复 cursor 或协议错误)",
                    page_index
                )));
            }

            // Update selection tracker before item filtering.
            tracker.mark_seen(&plan.page_aweme_ids);

            let mut page_media_keys = HashSet::new();
            for item in &plan.items {
                if !plan.page_aweme_ids.contains(&item.aweme_id) {
                    return Err(tracker.contextualize(format!(
                        "第 {page_index} 页媒体 {} 不属于 page_aweme_ids",
                        item.media_key
                    )));
                }
                item.media_index()
                    .map_err(|error| tracker.contextualize(error))?;
                if !page_media_keys.insert(item.media_key.as_str()) {
                    return Err(tracker.contextualize(format!(
                        "第 {page_index} 页重复 media_key: {}",
                        item.media_key
                    )));
                }
                if !selection_active && !seen_media_keys.insert(item.media_key.clone()) {
                    return Err(format!("跨页重复 media_key: {}", item.media_key));
                }
            }

            // Filter items: selection uses tracker, non-selection uses existing max_counts logic.
            let page_items: Vec<_> = if selection_active {
                tracker.take_unplanned_items(plan.items.clone(), &|item| &item.aweme_id)
            } else {
                let mut allowed_work_ids = HashSet::new();
                for aweme_id in &plan.page_aweme_ids {
                    if seen_aweme_ids.contains(aweme_id) {
                        allowed_work_ids.insert(aweme_id.as_str());
                        continue;
                    }
                    if max_counts != 0 && seen_aweme_ids.len() >= max_counts {
                        break;
                    }
                    seen_aweme_ids.insert(aweme_id.clone());
                    allowed_work_ids.insert(aweme_id.as_str());
                }
                let filtered: Vec<_> = plan
                    .items
                    .iter()
                    .filter(|item| allowed_work_ids.contains(item.aweme_id.as_str()))
                    .cloned()
                    .collect();
                if filtered.is_empty() && max_counts != 0 && seen_aweme_ids.len() >= max_counts {
                    break;
                }
                filtered
            };

            let (completed, failed) = Self::execute_media_items(
                db,
                task_id,
                &page_items,
                current_save_dir,
                cancel_signal,
                app_config,
                &event_sink,
            )
            .await?;

            total_completed += completed;
            total_failed += failed;

            info!(
                "[TaskService] 第 {} 页下载完成: completed={}, failed={}, 累计: {}/{}",
                page_index,
                completed,
                failed,
                total_completed,
                total_completed + total_failed
            );

            // Post-download max_counts early stop (non-selection).
            if !selection_active && max_counts != 0 && seen_aweme_ids.len() >= max_counts {
                break;
            }

            // Selection early-stop: all requested IDs seen.
            if selection_active && tracker.all_seen() {
                info!(
                    "[TaskService] 所有选中作品已命中，停止分页: task_id={}",
                    task_id
                );
                break;
            }

            if !plan.has_more {
                info!(
                    "[TaskService] has_more=false，停止分页 (共 {} 页)",
                    page_index
                );
                break;
            }

            let next = plan.next_cursor.ok_or_else(|| {
                tracker.contextualize(format!(
                    "第 {page_index} 页 has_more=true 但 next_cursor 为空"
                ))
            })?;
            if !seen_cursors.insert(next) {
                return Err(
                    tracker.contextualize(format!("第 {page_index} 页 next_cursor 重复: {next}"))
                );
            }
            cursor = next;
        }

        // Post-pagination terminal logic.
        if selection_active {
            // Check for missing/unavailable — never report these on cancel (already returned).
            if let Some(summary) = tracker.summary_error() {
                if total_completed == 0 && total_failed == 0 {
                    return Err(format!("选择下载未发现任何媒体: {summary}"));
                }
                let message = format!("选择下载部分完成: {summary}");
                warn!("[TaskService] {}: task_id={}", message, task_id);
                return Err(message);
            }
        }
        if total_completed == 0 && total_failed == 0 {
            return Err("未发现可下载媒体".to_string());
        }
        if total_failed > 0 && total_completed == 0 {
            return Err(format!(
                "所有下载均失败: {total_failed}/{} failed",
                total_failed + total_completed
            ));
        }
        if selection_active {
            info!(
                "[TaskService] 选择下载完成 (typed): task_id={}, pages={}, completed={}, failed={}",
                task_id, page_index, total_completed, total_failed
            );
        } else {
            info!(
                "[TaskService] 分页下载完成 (typed): task_id={}, pages={}, completed={}, failed={}",
                task_id, page_index, total_completed, total_failed
            );
        }
        Ok(())
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
        self.adapters
            .event_sink
            .emit(TaskEvent::started(&task_id, request.mode, &request.url));

        // 3. 注册取消信号
        let cancel_signal = self.state.register_cancel_signal(&task_id);

        // 4. 克隆数据用于后台任务
        let db = self.state.db.clone();
        let cancel_signals = self.state.cancel_signals.clone();
        let app_config = self.state.config.lock().get_douyin_config();
        let task_id_clone = task_id.clone();
        let mode = request.mode;
        let url = request.url;
        let adapters = self.adapters.clone();

        // 5. 启动后台下载任务
        tokio::spawn(async move {
            let result = Self::execute_download(
                &db,
                &task_id_clone,
                mode,
                &url,
                &cancel_signal,
                &app_config,
                &adapters,
            )
            .await;

            match Self::finalize_task(
                &db,
                &task_id_clone,
                result,
                cancel_signal.load(Ordering::Relaxed),
            ) {
                TaskTerminal::Completed => {
                    adapters.event_sink.emit(TaskEvent::finished(
                        TaskPatch::new(&task_id_clone).with_status(TaskStatus::Completed),
                    ));
                    info!("[TaskService] 下载任务完成: task_id={}", task_id_clone);
                }
                TaskTerminal::Cancelled => {
                    adapters.event_sink.emit(TaskEvent::finished(
                        TaskPatch::new(&task_id_clone).with_status(TaskStatus::Cancelled),
                    ));
                    info!("[TaskService] 下载任务已取消: task_id={}", task_id_clone);
                }
                TaskTerminal::Error(message) => {
                    error!(
                        "[TaskService] 下载任务失败: task_id={}, error={}",
                        task_id_clone, message
                    );
                    adapters.event_sink.emit(TaskEvent::finished(
                        TaskPatch::new(&task_id_clone)
                            .with_status(TaskStatus::Error)
                            .with_error(&message),
                    ));
                }
            }

            // 清理取消信号
            Self::cleanup_cancel_signal(&cancel_signals, &task_id_clone);
        });

        Ok(task_id)
    }

    /// 执行单视频下载（typed, mode=one）
    ///
    /// 其他 mode 走 start_batch_download_mode/start_music_download/start_live_record。
    async fn execute_download(
        db: &Database,
        task_id: &str,
        mode: DownloadMode,
        url: &str,
        cancel_signal: &Arc<AtomicBool>,
        app_config: &crate::config::AppConfig,
        adapters: &TaskRuntimeAdapters,
    ) -> Result<(), String> {
        if mode != DownloadMode::One {
            return Err(format!("execute_download 不支持 mode={}", mode.as_str()));
        }

        let plan = adapters.single_resolver.resolve(url.to_string()).await?;
        Self::execute_single_download_plan(
            db,
            task_id,
            plan,
            cancel_signal,
            app_config,
            adapters.event_sink.clone(),
        )
        .await
    }

    // execute_paged_download removed in issue 08 — all paged modes use execute_paged_download_plan.

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
        aweme_ids: &[String],
    ) -> Result<String, String> {
        let task_id = Uuid::new_v4().to_string();
        let task_id = task_id[..8].to_string();

        info!(
            "[TaskService] start_batch_download_mode: task_id={}, mode={}, aweme_ids={:?}",
            task_id, mode, aweme_ids
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
        self.adapters
            .event_sink
            .emit(TaskEvent::started(&task_id, mode, url));

        // 3. 注册取消信号
        let cancel_signal = self.state.register_cancel_signal(&task_id);

        // 4. 克隆数据用于后台任务
        let db = self.state.db.clone();
        let cancel_signals = self.state.cancel_signals.clone();
        let app_config = self.state.config.lock().get_douyin_config();
        let task_id_clone = task_id.clone();
        let mode_val = mode;
        let url_val = url.to_string();
        let aweme_ids_val: Vec<String> = aweme_ids.to_vec();
        let adapters = self.adapters.clone();

        // 5. 启动后台下载任务（分页模式）
        tokio::spawn(async move {
            // All paged video modes (post/like/mix/collects) use one typed runner.
            let result = Self::execute_paged_download_plan(
                &db,
                &task_id_clone,
                PagedRunRequest {
                    mode: mode_val,
                    url: &url_val,
                    aweme_ids: &aweme_ids_val,
                },
                &cancel_signal,
                &app_config,
                &adapters,
            )
            .await;

            match Self::finalize_task(
                &db,
                &task_id_clone,
                result,
                cancel_signal.load(Ordering::Relaxed),
            ) {
                TaskTerminal::Completed => {
                    adapters.event_sink.emit(TaskEvent::finished(
                        TaskPatch::new(&task_id_clone).with_status(TaskStatus::Completed),
                    ));
                    info!("[TaskService] 批量下载完成: task_id={}", task_id_clone);
                }
                TaskTerminal::Cancelled => {
                    adapters.event_sink.emit(TaskEvent::finished(
                        TaskPatch::new(&task_id_clone).with_status(TaskStatus::Cancelled),
                    ));
                    info!("[TaskService] 批量下载已取消: task_id={}", task_id_clone);
                }
                TaskTerminal::Error(message) => {
                    error!(
                        "[TaskService] 批量下载失败: task_id={}, error={}",
                        task_id_clone, message
                    );
                    adapters.event_sink.emit(TaskEvent::finished(
                        TaskPatch::new(&task_id_clone)
                            .with_status(TaskStatus::Error)
                            .with_error(message),
                    ));
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
                    if let Err(e) = db.update_task_status(&task_id_clone, "completed", None) {
                        error!(
                            "[TaskService] 更新音乐任务完成状态失败: task_id={}, error={}",
                            task_id_clone, e
                        );
                    }
                    events::emit_finished(
                        TaskPatch::new(&task_id_clone).with_status(TaskStatus::Completed),
                    );
                    info!("[TaskService] 音乐下载完成: task_id={}", task_id_clone);
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
                        info!("[TaskService] 音乐下载已取消: task_id={}", task_id_clone);
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
        let resolved = Self::resolve_music_urls(url).await?;
        if !resolved.success {
            let err = resolved.error.unwrap_or_else(|| "解析失败".to_string());
            return Err(err);
        }

        let items = resolved.items;
        let save_dir = resolved
            .save_dir
            .unwrap_or_else(|| "./Download/music".to_string());

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

            let download_item = build_music_download_item(item, &save_dir, task_id);

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
                            media_key: None,
                            media_kind: None,
                            media_index: None,
                            title: title.clone(),
                            author_nickname: author.clone(),
                            author_sec_uid: None,
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
                            media_key: None,
                            media_kind: None,
                            media_index: None,
                            title: title.clone(),
                            author_nickname: author.clone(),
                            author_sec_uid: None,
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
                    }

                    completed += 1;
                    progress.update((completed + failed) as u64, total as u64);
                }
                Err(e) => {
                    warn!(
                        "[TaskService] 音乐下载失败: music_id={}, error={}",
                        item.aweme_id, e
                    );

                    let new_item = NewTaskItem {
                        task_id: task_id.to_string(),
                        aweme_id: Some(item.aweme_id.clone()),
                        media_key: None,
                        media_kind: None,
                        media_index: None,
                        title: None,
                        author_nickname: None,
                        author_sec_uid: None,
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

#[cfg(test)]
mod single_media_outcome_tests {
    use std::path::PathBuf;

    use serde_json::json;
    use uuid::Uuid;

    use super::{TaskApplicationService, TaskTerminal};
    use crate::db::{Database, MediaItemOutcome, NewDownloadTask, NewTaskItem, VideoInfo};

    fn test_database(task_id: &str) -> (Database, PathBuf) {
        let path = std::env::temp_dir().join(format!(
            "douyin-crawler-media-outcome-{}.sqlite",
            Uuid::new_v4()
        ));
        let db = Database::open(&path).expect("temporary database should open");
        db.create_task(&NewDownloadTask {
            id: task_id.to_string(),
            mode: "one".to_string(),
            url: "https://www.douyin.com/video/test".to_string(),
            title: None,
            author_nickname: None,
        })
        .expect("task should be created");
        (db, path)
    }

    fn task_item(task_id: &str, aweme_id: &str) -> NewTaskItem {
        NewTaskItem {
            task_id: task_id.to_string(),
            aweme_id: Some(aweme_id.to_string()),
            media_key: Some(format!("{aweme_id}:video:0")),
            media_kind: Some("video".to_string()),
            media_index: Some(0),
            title: Some("transactional result".to_string()),
            author_nickname: Some("tester".to_string()),
            author_sec_uid: Some("sec-user".to_string()),
            cover_url: Some("https://cdn.example/cover.jpeg".to_string()),
        }
    }

    fn video(aweme_id: &str) -> VideoInfo {
        serde_json::from_value(json!({
            "aweme_id": aweme_id,
            "desc": "transactional result",
            "author_nickname": "tester",
            "author_sec_uid": "sec-user"
        }))
        .expect("video fixture should deserialize")
    }

    fn remove_database_files(path: &PathBuf) {
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(format!("{}-wal", path.display()));
        let _ = std::fs::remove_file(format!("{}-shm", path.display()));
    }

    fn has_video(db: &Database, aweme_id: &str) -> bool {
        db.get_videos(100, 0, None, None, None, None, None)
            .expect("video query should succeed")
            .iter()
            .any(|stored| stored.aweme_id == aweme_id)
    }

    #[test]
    fn completed_single_media_outcome_commits_item_metadata_and_counts() {
        let task_id = "completed-outcome";
        let aweme_id = "aweme-completed";
        let (db, path) = test_database(task_id);

        TaskApplicationService::persist_single_media_item_outcome(
            &db,
            &task_item(task_id, aweme_id),
            &video(aweme_id),
            MediaItemOutcome::Completed {
                file_path: "/tmp/video.mp4",
                file_size: 42,
            },
        )
        .expect("completed outcome should persist");

        let detail = db
            .get_task_detail(task_id)
            .expect("task detail query should succeed")
            .expect("task should exist");
        assert_eq!(detail.task.status, "running");
        assert_eq!(detail.task.total, 1);
        assert_eq!(detail.task.completed, 1);
        assert_eq!(detail.task.skipped, 0);
        assert_eq!(detail.task.failed, 0);
        assert_eq!(detail.items.len(), 1);
        assert_eq!(detail.items[0].status, "completed");
        assert_eq!(detail.items[0].file_path.as_deref(), Some("/tmp/video.mp4"));
        assert_eq!(detail.items[0].file_size, 42);
        assert!(has_video(&db, aweme_id));

        assert_eq!(
            TaskApplicationService::finalize_task(&db, task_id, Ok(()), false),
            TaskTerminal::Completed
        );
        assert_eq!(
            db.get_task_by_id(task_id)
                .unwrap()
                .expect("task should exist")
                .status,
            "completed"
        );

        drop(db);
        remove_database_files(&path);
    }

    #[test]
    fn skipped_single_media_outcome_commits_path_metadata_and_skipped_count() {
        let task_id = "skipped-outcome";
        let aweme_id = "aweme-skipped";
        let (db, path) = test_database(task_id);

        TaskApplicationService::persist_single_media_item_outcome(
            &db,
            &task_item(task_id, aweme_id),
            &video(aweme_id),
            MediaItemOutcome::Skipped {
                file_path: "/tmp/existing.mp4",
                file_size: 84,
            },
        )
        .expect("skipped outcome should persist");

        let detail = db.get_task_detail(task_id).unwrap().unwrap();
        assert_eq!(detail.task.status, "running");
        assert_eq!(detail.task.total, 1);
        assert_eq!(detail.task.completed, 0);
        assert_eq!(detail.task.skipped, 1);
        assert_eq!(detail.task.failed, 0);
        assert_eq!(detail.items[0].status, "skipped");
        assert_eq!(
            detail.items[0].file_path.as_deref(),
            Some("/tmp/existing.mp4")
        );
        assert_eq!(detail.items[0].file_size, 84);
        assert!(has_video(&db, aweme_id));

        assert_eq!(
            TaskApplicationService::finalize_task(&db, task_id, Ok(()), false),
            TaskTerminal::Completed
        );

        drop(db);
        remove_database_files(&path);
    }

    #[test]
    fn failed_single_media_outcome_commits_error_metadata_counts_and_task_error() {
        let task_id = "failed-outcome";
        let aweme_id = "aweme-failed";
        let (db, path) = test_database(task_id);
        let download_error = "HTTP 503";

        TaskApplicationService::persist_single_media_item_outcome(
            &db,
            &task_item(task_id, aweme_id),
            &video(aweme_id),
            MediaItemOutcome::Failed {
                error_msg: download_error,
            },
        )
        .expect("download failure outcome should persist");

        let terminal = TaskApplicationService::finalize_task(
            &db,
            task_id,
            Err(format!("所有下载均失败: {download_error}")),
            false,
        );
        assert!(matches!(terminal, TaskTerminal::Error(_)));

        let detail = db.get_task_detail(task_id).unwrap().unwrap();
        assert_eq!(detail.task.status, "error");
        assert_eq!(detail.task.total, 1);
        assert_eq!(detail.task.completed, 0);
        assert_eq!(detail.task.skipped, 0);
        assert_eq!(detail.task.failed, 1);
        assert!(detail
            .task
            .error_msg
            .as_deref()
            .is_some_and(|message| message.contains(download_error)));
        assert_eq!(detail.items[0].status, "failed");
        assert_eq!(detail.items[0].error_msg.as_deref(), Some(download_error));
        assert_eq!(detail.items[0].file_path, None);
        assert!(has_video(&db, aweme_id));

        drop(db);
        remove_database_files(&path);
    }

    #[test]
    fn metadata_db_failure_rolls_back_success_and_recovers_visible_item_error() {
        let task_id = "metadata-db-failure";
        let aweme_id = "aweme-db-failure";
        let (db, path) = test_database(task_id);
        db.with_transaction(|tx| {
            tx.execute_batch(
                "CREATE TRIGGER fail_video_insert \
                 BEFORE INSERT ON video_info \
                 BEGIN SELECT RAISE(ABORT, 'forced metadata failure'); END;",
            )?;
            Ok(())
        })
        .expect("failure trigger should be installed");

        let execution_error = TaskApplicationService::persist_single_media_item_outcome(
            &db,
            &task_item(task_id, aweme_id),
            &video(aweme_id),
            MediaItemOutcome::Completed {
                file_path: "/tmp/downloaded-before-db-failure.mp4",
                file_size: 128,
            },
        )
        .expect_err("metadata failure must fail the result transaction");
        assert!(execution_error.contains("forced metadata failure"));

        let terminal =
            TaskApplicationService::finalize_task(&db, task_id, Err(execution_error), true);
        assert!(matches!(terminal, TaskTerminal::Error(_)));

        let detail = db.get_task_detail(task_id).unwrap().unwrap();
        assert_eq!(detail.task.status, "error");
        assert_eq!(detail.task.completed, 0);
        assert_eq!(detail.task.skipped, 0);
        assert_eq!(detail.task.failed, 1);
        assert_eq!(detail.task.total, 1);
        assert_eq!(detail.items.len(), 1);
        assert_eq!(detail.items[0].status, "failed");
        assert_eq!(detail.items[0].file_path, None);
        assert_eq!(detail.items[0].file_size, 0);
        assert!(detail.items[0]
            .error_msg
            .as_deref()
            .is_some_and(|message| message.contains("forced metadata failure")));
        assert!(!has_video(&db, aweme_id));

        drop(db);
        remove_database_files(&path);
    }

    #[test]
    fn fallback_failure_is_reported_while_task_error_update_remains_visible() {
        let task_id = "fallback-db-failure";
        let aweme_id = "aweme-fallback-failure";
        let (db, path) = test_database(task_id);
        db.with_transaction(|tx| {
            tx.execute_batch(
                "CREATE TRIGGER fail_item_insert \
                 BEFORE INSERT ON download_task_items \
                 BEGIN SELECT RAISE(ABORT, 'forced item failure'); END;",
            )?;
            Ok(())
        })
        .expect("failure trigger should be installed");

        let execution_error = TaskApplicationService::persist_single_media_item_outcome(
            &db,
            &task_item(task_id, aweme_id),
            &video(aweme_id),
            MediaItemOutcome::Completed {
                file_path: "/tmp/untracked.mp4",
                file_size: 256,
            },
        )
        .expect_err("both result and fallback writes must fail");
        assert!(execution_error.contains("forced item failure"));
        assert!(execution_error.contains("错误状态恢复写入失败"));

        let terminal =
            TaskApplicationService::finalize_task(&db, task_id, Err(execution_error), false);
        assert!(matches!(terminal, TaskTerminal::Error(_)));
        let detail = db.get_task_detail(task_id).unwrap().unwrap();
        assert_eq!(detail.task.status, "error");
        assert!(detail
            .task
            .error_msg
            .as_deref()
            .is_some_and(|message| message.contains("错误状态恢复写入失败")));
        assert!(detail.items.is_empty());

        drop(db);
        remove_database_files(&path);
    }

    #[test]
    fn completed_terminal_signal_is_gated_by_completed_status_commit() {
        let task_id = "terminal-gate";
        let (db, path) = test_database(task_id);
        db.with_transaction(|tx| {
            tx.execute_batch(
                "CREATE TRIGGER fail_completed_status \
                 BEFORE UPDATE OF status ON download_tasks \
                 WHEN NEW.status = 'completed' \
                 BEGIN SELECT RAISE(ABORT, 'forced completed status failure'); END;",
            )?;
            Ok(())
        })
        .expect("failure trigger should be installed");

        let terminal = TaskApplicationService::finalize_task(&db, task_id, Ok(()), false);
        let TaskTerminal::Error(message) = terminal else {
            panic!("completed status DB failure must not produce completed semantics");
        };
        assert!(message.contains("forced completed status failure"));
        let task = db.get_task_by_id(task_id).unwrap().unwrap();
        assert_eq!(task.status, "error");
        assert!(task
            .error_msg
            .as_deref()
            .is_some_and(|error| error.contains("forced completed status failure")));

        drop(db);
        remove_database_files(&path);
    }

    #[test]
    fn explicit_cancellation_does_not_create_a_failed_item() {
        let task_id = "cancelled-outcome";
        let (db, path) = test_database(task_id);

        let terminal = TaskApplicationService::finalize_task(
            &db,
            task_id,
            Err("下载已取消".to_string()),
            true,
        );

        assert_eq!(terminal, TaskTerminal::Cancelled);
        let detail = db.get_task_detail(task_id).unwrap().unwrap();
        assert_eq!(detail.task.status, "cancelled");
        assert_eq!(detail.task.total, 0);
        assert_eq!(detail.task.failed, 0);
        assert!(detail.items.is_empty());

        drop(db);
        remove_database_files(&path);
    }
}
