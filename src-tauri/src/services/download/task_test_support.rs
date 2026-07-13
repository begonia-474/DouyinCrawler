//! Reusable task-level regression harness for download lifecycle tests.

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use uuid::Uuid;

use super::contract::{
    PagedDownloadPlanV1, SingleAccessory, SingleAccessoryKind, SingleDownloadItem,
    SingleDownloadPlanV1, SingleMediaKind, SingleOutputSpec,
};
use super::live::{
    LiveCompletionReason, LiveFailureKind, LiveOutputFacts, LiveOutputV1, LivePlanV1,
    LiveRecorderOutcome,
};
use super::task_service::{
    LivePlanFuture, LiveProgressCallback, LiveRecorderFuture, LiveRecorderRunner, LiveResolver,
    PagedDownloadResolver, PagedPlanFuture, SingleDownloadResolver, SinglePlanFuture,
    TaskApplicationService, TaskEventSink,
};
use super::{DownloadMode, DownloadRequest, TaskEvent, TaskEventType, TaskStatus};
use crate::config::{AppConfig, ConfigManager};
use crate::db::{Database, DownloadTask, UserInfo, VideoInfo};
use crate::python::PythonBridge;
use crate::state::AppState;

#[derive(Clone)]
pub(crate) enum ResolverResult {
    Plan(SingleDownloadPlanV1),
    Error(String),
}

pub(crate) struct FixedSingleResolver {
    result: ResolverResult,
}

impl FixedSingleResolver {
    pub(crate) fn plan(plan: SingleDownloadPlanV1) -> Self {
        Self {
            result: ResolverResult::Plan(plan),
        }
    }

    pub(crate) fn error(message: impl Into<String>) -> Self {
        Self {
            result: ResolverResult::Error(message.into()),
        }
    }
}

impl SingleDownloadResolver for FixedSingleResolver {
    fn resolve(&self, _url: String) -> SinglePlanFuture {
        let result = self.result.clone();
        Box::pin(async move {
            match result {
                ResolverResult::Plan(plan) => Ok(plan),
                ResolverResult::Error(message) => Err(message),
            }
        })
    }
}

// ============================================================
// Live resolver / recorder fixtures
// ============================================================

#[derive(Clone)]
pub(crate) enum LiveResolverResult {
    Plan(Box<LivePlanV1>),
    Error(String),
}

#[derive(Clone)]
pub(crate) struct FixedLiveResolver {
    result: LiveResolverResult,
}

impl FixedLiveResolver {
    pub(crate) fn plan(plan: LivePlanV1) -> Self {
        Self {
            result: LiveResolverResult::Plan(Box::new(plan)),
        }
    }

    pub(crate) fn error(message: impl Into<String>) -> Self {
        Self {
            result: LiveResolverResult::Error(message.into()),
        }
    }
}

impl LiveResolver for FixedLiveResolver {
    fn resolve(&self, _url: String) -> LivePlanFuture {
        let result = self.result.clone();
        Box::pin(async move {
            match result {
                LiveResolverResult::Plan(plan) => Ok(*plan),
                LiveResolverResult::Error(error) => Err(error),
            }
        })
    }
}

#[derive(Clone)]
pub(crate) enum LiveRecorderBehavior {
    Complete {
        bytes: Vec<u8>,
        delay: Duration,
    },
    Fail {
        kind: LiveFailureKind,
        error: String,
        bytes: Vec<u8>,
    },
    WaitForStop {
        bytes: Vec<u8>,
    },
}

#[derive(Clone)]
pub(crate) struct FixedLiveRecorder {
    behavior: LiveRecorderBehavior,
}

impl FixedLiveRecorder {
    pub(crate) fn complete(bytes: &[u8], delay: Duration) -> Self {
        Self {
            behavior: LiveRecorderBehavior::Complete {
                bytes: bytes.to_vec(),
                delay,
            },
        }
    }

    pub(crate) fn fail(kind: LiveFailureKind, error: impl Into<String>) -> Self {
        Self {
            behavior: LiveRecorderBehavior::Fail {
                kind,
                error: error.into(),
                bytes: Vec::new(),
            },
        }
    }

    pub(crate) fn wait_for_stop(bytes: &[u8]) -> Self {
        Self {
            behavior: LiveRecorderBehavior::WaitForStop {
                bytes: bytes.to_vec(),
            },
        }
    }
}

impl LiveRecorderRunner for FixedLiveRecorder {
    fn record(
        &self,
        _config: AppConfig,
        cancel_signal: Arc<std::sync::atomic::AtomicBool>,
        plan: LivePlanV1,
        progress: LiveProgressCallback,
    ) -> LiveRecorderFuture {
        let behavior = self.behavior.clone();
        Box::pin(async move {
            let started_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            let path = plan.full_path();
            let write_bytes = |bytes: &[u8]| {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).unwrap();
                }
                if !bytes.is_empty() {
                    std::fs::write(&path, bytes).unwrap();
                    progress(bytes.len() as u64);
                }
            };
            match behavior {
                LiveRecorderBehavior::Complete { bytes, delay } => {
                    write_bytes(&bytes);
                    tokio::time::sleep(delay).await;
                    LiveRecorderOutcome::Completed {
                        reason: LiveCompletionReason::StreamEnded,
                        output: LiveOutputFacts {
                            path,
                            file_size: bytes.len() as u64,
                            started_at,
                            ended_at: started_at + 2,
                        },
                    }
                }
                LiveRecorderBehavior::Fail { kind, error, bytes } => {
                    write_bytes(&bytes);
                    LiveRecorderOutcome::Failed {
                        kind,
                        error,
                        output: LiveOutputFacts {
                            path,
                            file_size: bytes.len() as u64,
                            started_at,
                            ended_at: started_at + 1,
                        },
                    }
                }
                LiveRecorderBehavior::WaitForStop { bytes } => {
                    write_bytes(&bytes);
                    while !cancel_signal.load(std::sync::atomic::Ordering::Relaxed) {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                    let output = LiveOutputFacts {
                        path,
                        file_size: bytes.len() as u64,
                        started_at,
                        ended_at: started_at + 1,
                    };
                    if bytes.is_empty() {
                        LiveRecorderOutcome::Failed {
                            kind: LiveFailureKind::StoppedBeforeUsableBytes,
                            error: "用户停止录制时尚未产生可用字节".to_string(),
                            output,
                        }
                    } else {
                        LiveRecorderOutcome::Completed {
                            reason: LiveCompletionReason::UserStopped,
                            output,
                        }
                    }
                }
            }
        })
    }
}

// ============================================================
// Paged resolver fixture
// ============================================================

#[derive(Clone)]
pub(crate) enum PagedResolverResult {
    Plan(PagedDownloadPlanV1),
    DelayedPlan(PagedDownloadPlanV1, Duration),
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PagedResolverCall {
    pub(crate) mode: String,
    pub(crate) url: String,
    pub(crate) cursor: i64,
    pub(crate) count: i64,
}

#[derive(Clone)]
pub(crate) struct FixedPagedResolver {
    pages: Arc<Vec<PagedResolverResult>>,
    index: Arc<std::sync::Mutex<usize>>,
    calls: Arc<std::sync::Mutex<Vec<PagedResolverCall>>>,
}

impl FixedPagedResolver {
    pub(crate) fn single(plan: PagedDownloadPlanV1) -> Self {
        Self {
            pages: Arc::new(vec![PagedResolverResult::Plan(plan)]),
            index: Arc::new(std::sync::Mutex::new(0)),
            calls: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub(crate) fn multi(pages: Vec<PagedResolverResult>) -> Self {
        Self {
            pages: Arc::new(pages),
            index: Arc::new(std::sync::Mutex::new(0)),
            calls: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub(crate) fn error(message: impl Into<String>) -> Self {
        Self {
            pages: Arc::new(vec![PagedResolverResult::Error(message.into())]),
            index: Arc::new(std::sync::Mutex::new(0)),
            calls: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub(crate) fn calls(&self) -> Vec<PagedResolverCall> {
        self.calls.lock().unwrap().clone()
    }
}

impl PagedDownloadResolver for FixedPagedResolver {
    fn resolve_page(&self, mode: String, url: String, cursor: i64, count: i64) -> PagedPlanFuture {
        self.calls.lock().unwrap().push(PagedResolverCall {
            mode,
            url,
            cursor,
            count,
        });
        let mut idx = self.index.lock().unwrap();
        let result = if *idx < self.pages.len() {
            self.pages[*idx].clone()
        } else {
            PagedResolverResult::Error("no more pages".to_string())
        };
        *idx += 1;
        Box::pin(async move {
            match result {
                PagedResolverResult::Plan(plan) => Ok(plan),
                PagedResolverResult::DelayedPlan(plan, delay) => {
                    tokio::time::sleep(delay).await;
                    Ok(plan)
                }
                PagedResolverResult::Error(message) => Err(message),
            }
        })
    }
}

#[derive(Default)]
pub(crate) struct EventCapture {
    events: Mutex<Vec<TaskEvent>>,
}

impl EventCapture {
    pub(crate) fn snapshot(&self) -> Vec<TaskEvent> {
        self.events.lock().clone()
    }

    pub(crate) fn terminal(&self, task_id: &str) -> Option<TaskEvent> {
        self.events
            .lock()
            .iter()
            .rev()
            .find(|event| event.task_id == task_id && event.event_type == TaskEventType::Finished)
            .cloned()
    }
}

impl TaskEventSink for EventCapture {
    fn emit(&self, event: TaskEvent) {
        self.events.lock().push(event);
    }
}

#[derive(Clone, Copy)]
pub(crate) enum HttpBehavior {
    Success(&'static [u8]),
    NotFound,
    Timeout,
    Chunked {
        chunks: usize,
        chunk_size: usize,
        delay: Duration,
    },
}

pub(crate) struct LocalHttpServer {
    url: String,
    request_count: Arc<AtomicUsize>,
    task: tokio::task::JoinHandle<()>,
}

impl LocalHttpServer {
    pub(crate) async fn start(behavior: HttpBehavior) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("local HTTP listener should bind");
        let address = listener.local_addr().unwrap();
        let request_count = Arc::new(AtomicUsize::new(0));
        let server_request_count = request_count.clone();
        let task = tokio::spawn(async move {
            // Accept multiple connections (handles multi-item downloads)
            loop {
                let (mut socket, _) = match listener.accept().await {
                    Ok(socket) => socket,
                    Err(_) => break,
                };
                server_request_count.fetch_add(1, AtomicOrdering::Relaxed);
                let mut request = [0_u8; 2048];
                let _ = socket.read(&mut request).await;
                match behavior {
                    HttpBehavior::Success(body) => {
                        let header = format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            body.len()
                        );
                        let _ = socket.write_all(header.as_bytes()).await;
                        let _ = socket.write_all(body).await;
                    }
                    HttpBehavior::NotFound => {
                        let _ = socket
                            .write_all(
                                b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                            )
                            .await;
                    }
                    HttpBehavior::Timeout => {
                        tokio::time::sleep(Duration::from_secs(3)).await;
                    }
                    HttpBehavior::Chunked {
                        chunks,
                        chunk_size,
                        delay,
                    } => {
                        let total = chunks * chunk_size;
                        let header = format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {total}\r\nConnection: close\r\n\r\n"
                        );
                        let _ = socket.write_all(header.as_bytes()).await;
                        let chunk = vec![b'x'; chunk_size];
                        for _ in 0..chunks {
                            if socket.write_all(&chunk).await.is_err() {
                                break;
                            }
                            tokio::time::sleep(delay).await;
                        }
                    }
                }
            }
        });

        Self {
            url: format!("http://{address}/media"),
            request_count,
            task,
        }
    }

    pub(crate) fn url(&self) -> &str {
        &self.url
    }

    pub(crate) fn request_count(&self) -> usize {
        self.request_count.load(AtomicOrdering::Relaxed)
    }
}

impl Drop for LocalHttpServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub(crate) struct TaskHarness {
    pub(crate) state: AppState,
    pub(crate) events: Arc<EventCapture>,
    pub(crate) root: PathBuf,
    database_path: PathBuf,
}

impl TaskHarness {
    pub(crate) fn new() -> Self {
        Self::with_config(AppConfig {
            timeout: 1,
            max_retries: 1,
            music: false,
            cover: false,
            desc: false,
            ..AppConfig::default()
        })
    }

    pub(crate) fn with_max_counts(max_counts: u32) -> Self {
        Self::with_config(AppConfig {
            timeout: 1,
            max_retries: 1,
            max_counts,
            music: false,
            cover: false,
            desc: false,
            ..AppConfig::default()
        })
    }

    fn with_config(mut config: AppConfig) -> Self {
        let root =
            std::env::temp_dir().join(format!("douyin-crawler-task-harness-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let database_path = root.join("tasks.sqlite");
        let db = Database::open(&database_path).unwrap();
        config.download_path = root.to_string_lossy().to_string();
        let state = AppState::new(
            db,
            Arc::new(Mutex::new(ConfigManager::for_test(config))),
            Arc::new(PythonBridge::for_test()),
        );
        Self {
            state,
            events: Arc::new(EventCapture::default()),
            root,
            database_path,
        }
    }

    pub(crate) fn plan(&self, url: &str, aweme_id: &str) -> SingleDownloadPlanV1 {
        SingleDownloadPlanV1 {
            success: true,
            contract_version: 1,
            mode: "one".to_string(),
            save_dir: self.root.to_string_lossy().to_string(),
            items: vec![SingleDownloadItem {
                media_key: format!("{aweme_id}:video:0"),
                aweme_id: aweme_id.to_string(),
                urls: vec![url.to_string()],
                kind: SingleMediaKind::Video,
                output: SingleOutputSpec {
                    filename: aweme_id.to_string(),
                    suffix: ".mp4".to_string(),
                    folder_name: None,
                },
                headers: Default::default(),
                accessories: Vec::new(),
                metadata: video(aweme_id),
            }],
            total: 1,
        }
    }

    pub(crate) fn paged_plan(
        &self,
        url: &str,
        aweme_ids: &[&str],
        has_more: bool,
        next_cursor: Option<i64>,
    ) -> PagedDownloadPlanV1 {
        self.paged_plan_for_mode("post", url, aweme_ids, has_more, next_cursor)
    }

    pub(crate) fn paged_plan_for_mode(
        &self,
        mode: &str,
        url: &str,
        aweme_ids: &[&str],
        has_more: bool,
        next_cursor: Option<i64>,
    ) -> PagedDownloadPlanV1 {
        let items: Vec<SingleDownloadItem> = aweme_ids
            .iter()
            .map(|aweme_id| SingleDownloadItem {
                media_key: format!("{aweme_id}:video:0"),
                aweme_id: aweme_id.to_string(),
                urls: vec![url.to_string()],
                kind: SingleMediaKind::Video,
                output: SingleOutputSpec {
                    filename: aweme_id.to_string(),
                    suffix: ".mp4".to_string(),
                    folder_name: None,
                },
                headers: Default::default(),
                accessories: Vec::new(),
                metadata: video(aweme_id),
            })
            .collect();
        PagedDownloadPlanV1 {
            success: true,
            contract_version: 1,
            mode: mode.to_string(),
            save_dir: self.root.to_string_lossy().to_string(),
            items,
            next_cursor,
            has_more,
            page_aweme_ids: aweme_ids.iter().map(|s| s.to_string()).collect(),
            user_profile: None,
        }
    }

    pub(crate) fn paged_plan_with_urls(
        &self,
        items: &[(&str, &str)],
        has_more: bool,
        next_cursor: Option<i64>,
    ) -> PagedDownloadPlanV1 {
        let mut plan = self.paged_plan("http://127.0.0.1:1/unused", &[], has_more, next_cursor);
        plan.page_aweme_ids = items
            .iter()
            .map(|(aweme_id, _)| (*aweme_id).to_string())
            .collect();
        plan.items = items
            .iter()
            .map(|(aweme_id, url)| self.plan(url, aweme_id).items.into_iter().next().unwrap())
            .collect();
        plan
    }

    pub(crate) fn live_plan(&self) -> LivePlanV1 {
        LivePlanV1 {
            success: true,
            contract_version: 1,
            mode: "live".to_string(),
            web_rid: "web-live".to_string(),
            room_id: "room-live".to_string(),
            title: "fixture live".to_string(),
            nickname: "fixture anchor".to_string(),
            sec_user_id: "sec-live".to_string(),
            user_id: Some("uid-live".to_string()),
            cover_url: "https://example.com/live-cover.jpg".to_string(),
            user_count: 10,
            m3u8_url: "https://example.com/FULL_HD1.m3u8".to_string(),
            output: LiveOutputV1 {
                save_dir: self.root.to_string_lossy().to_string(),
                filename: "fixture_live".to_string(),
                suffix: ".flv".to_string(),
            },
            headers: Default::default(),
        }
    }

    pub(crate) async fn start(&self, resolver: FixedSingleResolver) -> String {
        TaskApplicationService::with_test_adapters(
            &self.state,
            Arc::new(resolver),
            self.events.clone(),
        )
        .start_download(DownloadRequest {
            mode: DownloadMode::One,
            url: "https://fixture.invalid/video".to_string(),
            aweme_ids: Vec::new(),
        })
        .await
        .unwrap()
    }

    pub(crate) async fn start_paged(&self, resolver: FixedPagedResolver) -> String {
        self.start_paged_for_mode(
            DownloadMode::Post,
            "https://fixture.invalid/user",
            resolver,
            &[],
        )
        .await
    }

    pub(crate) async fn start_paged_selection(
        &self,
        resolver: FixedPagedResolver,
        aweme_ids: &[&str],
    ) -> String {
        self.start_paged_for_mode(
            DownloadMode::Post,
            "https://fixture.invalid/user",
            resolver,
            aweme_ids,
        )
        .await
    }

    pub(crate) async fn start_paged_for_mode(
        &self,
        mode: DownloadMode,
        url: &str,
        resolver: FixedPagedResolver,
        aweme_ids: &[&str],
    ) -> String {
        TaskApplicationService::with_paged_test_adapters(
            &self.state,
            Arc::new(resolver),
            self.events.clone(),
        )
        .start_batch_download_mode(
            mode,
            url,
            &aweme_ids.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        )
        .await
        .unwrap()
    }

    pub(crate) async fn start_live(
        &self,
        resolver: FixedLiveResolver,
        recorder: FixedLiveRecorder,
    ) -> String {
        TaskApplicationService::with_live_test_adapters(
            &self.state,
            Arc::new(resolver),
            Arc::new(recorder),
            self.events.clone(),
        )
        .start_live_record("https://fixture.invalid/live")
        .await
        .unwrap()
    }

    pub(crate) fn stop_live(&self, task_id: &str) -> Result<(), String> {
        TaskApplicationService::with_live_test_adapters(
            &self.state,
            Arc::new(FixedLiveResolver::error("unused")),
            Arc::new(FixedLiveRecorder::fail(
                LiveFailureKind::InvalidPlan,
                "unused",
            )),
            self.events.clone(),
        )
        .stop_live_record(task_id)
    }

    pub(crate) async fn wait_for_terminal(&self, task_id: &str) -> DownloadTask {
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            if let Some(task) = self.state.db.get_task_by_id(task_id).unwrap() {
                let is_terminal = matches!(
                    task.status.as_str(),
                    "completed" | "error" | "cancelled" | "interrupted"
                );
                // Startup recovery intentionally emits no historical terminal event.
                if is_terminal
                    && (task.status == "interrupted" || self.events.terminal(task_id).is_some())
                {
                    return task;
                }
            }
            assert!(
                Instant::now() < deadline,
                "task did not reach a terminal state"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    pub(crate) fn media_path(&self, aweme_id: &str) -> PathBuf {
        self.root.join(format!("{aweme_id}.mp4"))
    }

    pub(crate) fn install_video_failure(&self) {
        self.state
            .db
            .with_transaction(|tx| {
                tx.execute_batch(
                    "CREATE TRIGGER harness_fail_video BEFORE INSERT ON video_info \
                     BEGIN SELECT RAISE(ABORT, 'harness metadata failure'); END;",
                )?;
                Ok(())
            })
            .unwrap();
    }

    pub(crate) fn install_user_failure(&self) {
        self.state
            .db
            .with_transaction(|tx| {
                tx.execute_batch(
                    "CREATE TRIGGER harness_fail_user BEFORE INSERT ON user_info \
                     BEGIN SELECT RAISE(ABORT, 'harness user failure'); END;",
                )?;
                Ok(())
            })
            .unwrap();
    }

    pub(crate) fn install_completed_status_failure(&self) {
        self.state
            .db
            .with_transaction(|tx| {
                tx.execute_batch(
                    "CREATE TRIGGER harness_fail_completed BEFORE UPDATE OF status ON download_tasks \
                     WHEN NEW.status = 'completed' \
                     BEGIN SELECT RAISE(ABORT, 'harness completed status failure'); END;",
                )?;
                Ok(())
            })
            .unwrap();
    }

    pub(crate) fn install_live_terminal_failure(&self) {
        self.state
            .db
            .with_transaction(|tx| {
                tx.execute_batch(
                    "CREATE TRIGGER harness_fail_live_terminal BEFORE UPDATE OF status ON live_records \
                     WHEN NEW.status IN ('completed', 'error') \
                     BEGIN SELECT RAISE(ABORT, 'harness live terminal failure'); END;",
                )?;
                Ok(())
            })
            .unwrap();
    }
}

impl Drop for TaskHarness {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.database_path);
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn video(aweme_id: &str) -> VideoInfo {
    serde_json::from_value(serde_json::json!({
        "aweme_id": aweme_id,
        "desc": "task harness",
        "author_nickname": "fixture"
    }))
    .unwrap()
}

fn user_profile(sec_user_id: &str, nickname: &str) -> UserInfo {
    serde_json::from_value(serde_json::json!({
        "sec_user_id": sec_user_id,
        "nickname": nickname
    }))
    .unwrap()
}

async fn wait_until(predicate: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while !predicate() {
        assert!(Instant::now() < deadline, "condition did not become true");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

fn assert_terminal_event(harness: &TaskHarness, task_id: &str, status: TaskStatus) {
    let event = harness
        .events
        .terminal(task_id)
        .expect("terminal event should be captured");
    assert_eq!(event.patch.status, Some(status));
}

fn assert_started_event(harness: &TaskHarness, task_id: &str, mode: DownloadMode, url: &str) {
    let events = harness.events.snapshot();
    let started = events
        .iter()
        .find(|event| event.task_id == task_id && event.event_type == TaskEventType::Started)
        .expect("started event should be captured");
    assert_eq!(started.mode, Some(mode));
    assert_eq!(started.url.as_deref(), Some(url));
    assert_eq!(started.patch.status, Some(TaskStatus::Running));
}

#[tokio::test]
async fn public_task_success_writes_file_database_and_events() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"video-data")).await;
    let harness = TaskHarness::new();
    let plan = harness.plan(server.url(), "success-item");
    let task_id = harness.start(FixedSingleResolver::plan(plan)).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.completed, 1);
    assert_eq!(
        std::fs::read(harness.media_path("success-item")).unwrap(),
        b"video-data"
    );
    let detail = harness.state.db.get_task_detail(&task_id).unwrap().unwrap();
    assert_eq!(detail.items.len(), 1);
    assert_eq!(detail.items[0].status, "completed");
    assert_eq!(
        detail.items[0].file_path.as_deref(),
        harness.media_path("success-item").to_str()
    );
    assert!(harness
        .state
        .db
        .get_videos(10, 0, None, None, None, None, None)
        .unwrap()
        .iter()
        .any(|video| video.aweme_id == "success-item"));
    assert_started_event(
        &harness,
        &task_id,
        DownloadMode::One,
        "https://fixture.invalid/video",
    );
    assert_terminal_event(&harness, &task_id, TaskStatus::Completed);
}

#[tokio::test]
async fn public_task_existing_file_is_skipped() {
    let harness = TaskHarness::new();
    std::fs::write(harness.media_path("skipped-item"), b"existing").unwrap();
    let plan = harness.plan("http://127.0.0.1:1/unused", "skipped-item");
    let task_id = harness.start(FixedSingleResolver::plan(plan)).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.skipped, 1);
    assert_terminal_event(&harness, &task_id, TaskStatus::Completed);
}

#[tokio::test]
async fn public_task_404_records_failed_item_and_error_event() {
    let server = LocalHttpServer::start(HttpBehavior::NotFound).await;
    let harness = TaskHarness::new();
    let plan = harness.plan(server.url(), "missing-item");
    let task_id = harness.start(FixedSingleResolver::plan(plan)).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert_eq!(task.failed, 1);
    let detail = harness.state.db.get_task_detail(&task_id).unwrap().unwrap();
    assert_eq!(detail.items.len(), 1);
    assert_eq!(detail.items[0].status, "failed");
    assert!(detail.items[0].error_msg.is_some());
    assert_terminal_event(&harness, &task_id, TaskStatus::Error);
}

#[tokio::test]
async fn public_task_timeout_records_failed_item() {
    let server = LocalHttpServer::start(HttpBehavior::Timeout).await;
    let harness = TaskHarness::new();
    let plan = harness.plan(server.url(), "timeout-item");
    let task_id = harness.start(FixedSingleResolver::plan(plan)).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert_eq!(task.failed, 1);
}

#[tokio::test]
async fn public_task_chunked_download_can_be_cancelled() {
    let server = LocalHttpServer::start(HttpBehavior::Chunked {
        chunks: 100,
        chunk_size: 16 * 1024,
        delay: Duration::from_millis(20),
    })
    .await;
    let harness = TaskHarness::new();
    let plan = harness.plan(server.url(), "cancelled-item");
    let task_id = harness.start(FixedSingleResolver::plan(plan)).await;
    wait_until(|| harness.state.get_cancel_signal(&task_id).is_some()).await;
    tokio::time::sleep(Duration::from_millis(80)).await;
    assert!(harness.state.cancel_task(&task_id));

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "cancelled");
    assert_eq!(task.failed, 0);
    assert!(harness
        .state
        .db
        .get_task_items(&task_id, None)
        .unwrap()
        .is_empty());
    assert_terminal_event(&harness, &task_id, TaskStatus::Cancelled);
}

#[tokio::test]
async fn public_task_database_failure_keeps_error_visible_and_never_finishes() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"downloaded")).await;
    let harness = TaskHarness::new();
    harness.install_video_failure();
    let plan = harness.plan(server.url(), "db-failure-item");
    let task_id = harness.start(FixedSingleResolver::plan(plan)).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert_eq!(task.failed, 1);
    assert!(task
        .error_msg
        .as_deref()
        .unwrap()
        .contains("harness metadata failure"));
    let items = harness.state.db.get_task_items(&task_id, None).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].status, "failed");
    assert!(items[0]
        .error_msg
        .as_deref()
        .unwrap()
        .contains("harness metadata failure"));
    assert_terminal_event(&harness, &task_id, TaskStatus::Error);
    assert!(!harness.events.snapshot().iter().any(|event| {
        event.task_id == task_id && event.patch.status == Some(TaskStatus::Completed)
    }));
}

#[tokio::test]
async fn public_task_resolver_error_never_contacts_http_or_creates_items() {
    let harness = TaskHarness::new();
    let task_id = harness
        .start(FixedSingleResolver::error("fixture resolver failure"))
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert_eq!(task.total, 0);
    assert!(task
        .error_msg
        .as_deref()
        .unwrap()
        .contains("fixture resolver failure"));
    assert_terminal_event(&harness, &task_id, TaskStatus::Error);
}

/// Verify recovery works across a simulated process restart
/// by seeding a running task + downloading item in a temporary DB,
/// then opening a fresh handle on the same path.
#[tokio::test]
async fn recover_simulates_process_restart() {
    let root = std::env::temp_dir().join(format!("recovery-restart-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let db_path = root.join("tasks.sqlite");

    // Phase 1: seed a running task and downloading item
    {
        let db = Database::open(&db_path).unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Direct SQL insert to simulate what a running task looks like in DB
        {
            let conn = db.conn.lock();
            conn.execute_batch(&format!(
                "INSERT INTO download_tasks (id, mode, url, status, total, completed, skipped, failed, created_at, updated_at) \
                 VALUES ('crash-task', 'one', 'https://example.com/video', 'running', 1, 0, 0, 0, {}, {}); \
                 INSERT INTO download_task_items (task_id, aweme_id, status, file_path, file_size, created_at) \
                 VALUES ('crash-task', 'vid-1', 'downloading', '/tmp/vid.mp4', 512, {}); \
                 INSERT INTO download_tasks (id, mode, url, status, total, completed, skipped, failed, created_at, updated_at) \
                 VALUES ('done-task', 'one', 'https://example.com/done', 'completed', 0, 0, 0, 0, {}, {});",
                now, now, now, now, now
            )).unwrap();
        }
    }

    // Phase 2: a fresh handle executes the same recovery step used by app setup.
    {
        let db = Database::open(&db_path).unwrap();
        let recovered = db.recover_interrupted_tasks().unwrap();
        assert_eq!(recovered.len(), 1, "should recover the crashed task");
        assert_eq!(recovered[0].task_id, "crash-task");

        let task = db.get_task_by_id("crash-task").unwrap().unwrap();
        assert_eq!(task.status, "interrupted");
        assert_eq!(task.total, 1);

        let items = db.get_task_items("crash-task", None).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].status, "interrupted");
        assert_eq!(items[0].file_path.as_deref(), Some("/tmp/vid.mp4"));
        assert_eq!(items[0].file_size, 512);

        // Completed task should be untouched
        let done = db.get_task_by_id("done-task").unwrap().unwrap();
        assert_eq!(done.status, "completed");
    }

    let _ = std::fs::remove_dir_all(&root);
}

// ============================================================
// Public live lifecycle tests
// ============================================================

#[tokio::test]
async fn public_live_normal_completion_updates_linked_row_before_finished_event() {
    let harness = TaskHarness::new();
    let plan = harness.live_plan();
    let task_id = harness
        .start_live(
            FixedLiveResolver::plan(plan.clone()),
            FixedLiveRecorder::complete(b"FLV-DATA", Duration::from_millis(120)),
        )
        .await;

    wait_until(|| {
        harness
            .state
            .db
            .get_live_record_by_task_id(&task_id)
            .unwrap()
            .is_some_and(|record| record.status == "recording")
    })
    .await;
    let recording_id = harness
        .state
        .db
        .get_live_record_by_task_id(&task_id)
        .unwrap()
        .unwrap()
        .id;

    let task = harness.wait_for_terminal(&task_id).await;
    let live = harness
        .state
        .db
        .get_live_record_by_task_id(&task_id)
        .unwrap()
        .unwrap();
    assert_eq!(task.status, "completed");
    assert_eq!(live.id, recording_id);
    assert_eq!(live.status, "completed");
    assert_eq!(live.file_size, 8);
    assert_eq!(std::fs::read(plan.full_path()).unwrap(), b"FLV-DATA");
    let finished = harness
        .events
        .snapshot()
        .into_iter()
        .filter(|event| event.task_id == task_id && event.event_type == TaskEventType::Finished)
        .collect::<Vec<_>>();
    assert_eq!(finished.len(), 1);
    assert_eq!(finished[0].patch.status, Some(TaskStatus::Completed));
}

#[tokio::test]
async fn public_live_stop_after_bytes_completes_and_preserves_partial_file() {
    let harness = TaskHarness::new();
    let plan = harness.live_plan();
    let task_id = harness
        .start_live(
            FixedLiveResolver::plan(plan.clone()),
            FixedLiveRecorder::wait_for_stop(b"PARTIAL-FLV"),
        )
        .await;
    wait_until(|| {
        harness
            .state
            .db
            .get_live_record_by_task_id(&task_id)
            .unwrap()
            .is_some_and(|record| record.status == "recording")
    })
    .await;

    harness.stop_live(&task_id).unwrap();
    let task = harness.wait_for_terminal(&task_id).await;
    let live = harness
        .state
        .db
        .get_live_record_by_task_id(&task_id)
        .unwrap()
        .unwrap();
    assert_eq!(task.status, "completed");
    assert_eq!(live.status, "completed");
    assert_eq!(std::fs::read(plan.full_path()).unwrap(), b"PARTIAL-FLV");
    assert!(harness.events.snapshot().iter().any(|event| {
        event.task_id == task_id
            && event.event_type == TaskEventType::Progress
            && event.patch.status == Some(TaskStatus::Stopping)
    }));
}

#[tokio::test]
async fn public_live_stop_before_bytes_is_error() {
    let harness = TaskHarness::new();
    let task_id = harness
        .start_live(
            FixedLiveResolver::plan(harness.live_plan()),
            FixedLiveRecorder::wait_for_stop(b""),
        )
        .await;
    wait_until(|| {
        harness
            .state
            .db
            .get_live_record_by_task_id(&task_id)
            .unwrap()
            .is_some()
    })
    .await;

    harness.stop_live(&task_id).unwrap();
    let task = harness.wait_for_terminal(&task_id).await;
    let live = harness
        .state
        .db
        .get_live_record_by_task_id(&task_id)
        .unwrap()
        .unwrap();
    assert_eq!(task.status, "error");
    assert_eq!(live.status, "error");
    assert!(live
        .error_msg
        .as_deref()
        .unwrap()
        .contains("尚未产生可用字节"));
}

#[tokio::test]
async fn public_live_resolver_failure_creates_no_live_row() {
    let harness = TaskHarness::new();
    let task_id = harness
        .start_live(
            FixedLiveResolver::error("FULL_HD1 missing"),
            FixedLiveRecorder::fail(LiveFailureKind::InvalidPlan, "unused"),
        )
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert!(harness
        .state
        .db
        .get_live_record_by_task_id(&task_id)
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn public_live_recorder_failure_updates_both_rows_to_error() {
    let harness = TaskHarness::new();
    let task_id = harness
        .start_live(
            FixedLiveResolver::plan(harness.live_plan()),
            FixedLiveRecorder::fail(
                LiveFailureKind::SegmentRetryExhausted,
                "segment retries exhausted",
            ),
        )
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    let live = harness
        .state
        .db
        .get_live_record_by_task_id(&task_id)
        .unwrap()
        .unwrap();
    assert_eq!(task.status, "error");
    assert_eq!(live.status, "error");
    assert!(task
        .error_msg
        .as_deref()
        .unwrap()
        .contains("segment retries"));
    assert_eq!(
        harness
            .events
            .snapshot()
            .iter()
            .filter(|event| {
                event.task_id == task_id && event.event_type == TaskEventType::Finished
            })
            .count(),
        1
    );
}

#[tokio::test]
async fn public_live_terminal_database_failure_emits_no_finished_event() {
    let harness = TaskHarness::new();
    harness.install_live_terminal_failure();
    let task_id = harness
        .start_live(
            FixedLiveResolver::plan(harness.live_plan()),
            FixedLiveRecorder::complete(b"FLV", Duration::from_millis(20)),
        )
        .await;

    wait_until(|| harness.state.get_cancel_signal(&task_id).is_none()).await;
    let task = harness.state.db.get_task_by_id(&task_id).unwrap().unwrap();
    let live = harness
        .state
        .db
        .get_live_record_by_task_id(&task_id)
        .unwrap()
        .unwrap();
    assert_eq!(task.status, "recording");
    assert_eq!(live.status, "recording");
    assert!(!harness
        .events
        .snapshot()
        .iter()
        .any(|event| { event.task_id == task_id && event.event_type == TaskEventType::Finished }));
}

#[tokio::test]
async fn recover_linked_live_task_preserves_partial_file_and_is_idempotent() {
    let root = std::env::temp_dir().join(format!("live-recovery-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let db_path = root.join("tasks.sqlite");
    let partial = root.join("partial.flv");
    std::fs::write(&partial, b"PARTIAL").unwrap();
    {
        let db = Database::open(&db_path).unwrap();
        db.create_task(&crate::db::NewDownloadTask {
            id: "live-crash".to_string(),
            mode: "live".to_string(),
            url: "https://fixture.invalid/live".to_string(),
            title: None,
            author_nickname: None,
        })
        .unwrap();
        db.update_task_status("live-crash", "starting", None)
            .unwrap();
        db.create_recording_live_record(&crate::db::RecordingLiveRecord {
            task_id: "live-crash".to_string(),
            room_id: "room".to_string(),
            web_rid: "web".to_string(),
            title: "title".to_string(),
            nickname: "anchor".to_string(),
            sec_user_id: "sec".to_string(),
            cover_url: String::new(),
            file_path: partial.to_string_lossy().to_string(),
            started_at: 100,
        })
        .unwrap();
    }
    {
        let db = Database::open(&db_path).unwrap();
        let recovered = db.recover_interrupted_tasks().unwrap();
        assert_eq!(recovered.len(), 1);
        let task = db.get_task_by_id("live-crash").unwrap().unwrap();
        let live = db
            .get_live_record_by_task_id("live-crash")
            .unwrap()
            .unwrap();
        assert_eq!(task.status, "interrupted");
        assert_eq!(live.status, "interrupted");
        assert_eq!(live.file_path.as_deref(), partial.to_str());
        assert_eq!(std::fs::read(&partial).unwrap(), b"PARTIAL");
        assert!(db.recover_interrupted_tasks().unwrap().is_empty());
    }
    let _ = std::fs::remove_dir_all(root);
}

// ============================================================
// Paged task tests (post mode with typed PagedDownloadPlanV1)
// ============================================================

#[tokio::test]
async fn public_paged_task_single_page_success() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"page-data")).await;
    let harness = TaskHarness::new();
    let plan = harness.paged_plan(server.url(), &["item-1", "item-2"], false, None);
    let task_id = harness.start_paged(FixedPagedResolver::single(plan)).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.completed, 2);
    assert_eq!(task.skipped, 0);
    assert_eq!(
        std::fs::read(harness.media_path("item-1")).unwrap(),
        b"page-data"
    );
    assert_eq!(
        std::fs::read(harness.media_path("item-2")).unwrap(),
        b"page-data"
    );
    let detail = harness.state.db.get_task_detail(&task_id).unwrap().unwrap();
    assert_eq!(detail.items.len(), 2);
    assert_eq!(detail.items[0].media_key.as_deref(), Some("item-1:video:0"));
    assert_eq!(detail.items[1].media_key.as_deref(), Some("item-2:video:0"));
    assert_started_event(
        &harness,
        &task_id,
        DownloadMode::Post,
        "https://fixture.invalid/user",
    );
    assert_terminal_event(&harness, &task_id, TaskStatus::Completed);
}

#[tokio::test]
async fn public_paged_task_two_pages_success() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"page-data")).await;
    let harness = TaskHarness::new();
    let plan1 = harness.paged_plan(server.url(), &["item-1"], true, Some(100));
    let plan2 = harness.paged_plan(server.url(), &["item-2"], false, None);
    let resolver = FixedPagedResolver::multi(vec![
        PagedResolverResult::Plan(plan1),
        PagedResolverResult::Plan(plan2),
    ]);
    let task_id = harness.start_paged(resolver.clone()).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.completed, 2);
    assert_eq!(
        resolver
            .calls()
            .iter()
            .map(|call| (call.mode.as_str(), call.cursor))
            .collect::<Vec<_>>(),
        [("post", 0), ("post", 100)]
    );
    let progress_events = harness
        .events
        .snapshot()
        .into_iter()
        .filter(|event| event.task_id == task_id && event.event_type == TaskEventType::Progress)
        .collect::<Vec<_>>();
    assert_eq!(progress_events.len(), 2);
    assert_eq!(progress_events[0].patch.total, Some(1));
    assert_eq!(progress_events[0].patch.completed, Some(1));
    assert_eq!(progress_events[0].patch.skipped, Some(0));
    assert_eq!(progress_events[0].patch.failed, Some(0));
    assert_eq!(progress_events[1].patch.total, Some(2));
    assert_eq!(progress_events[1].patch.completed, Some(2));
    assert_eq!(progress_events[1].patch.skipped, Some(0));
    assert_eq!(progress_events[1].patch.failed, Some(0));
    assert_terminal_event(&harness, &task_id, TaskStatus::Completed);
}

#[tokio::test]
async fn public_paged_task_second_page_error_retains_first_page_and_errors() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"page-one")).await;
    let harness = TaskHarness::new();
    let first = harness.paged_plan(server.url(), &["first-item"], true, Some(100));
    let resolver = FixedPagedResolver::multi(vec![
        PagedResolverResult::Plan(first),
        PagedResolverResult::Error("second page fixture failure".to_string()),
    ]);
    let task_id = harness.start_paged(resolver).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert_eq!(task.completed, 1);
    assert!(task
        .error_msg
        .as_deref()
        .unwrap_or("")
        .contains("第 2 页解析失败 (cursor=100)"));
    assert_eq!(
        std::fs::read(harness.media_path("first-item")).unwrap(),
        b"page-one"
    );
    assert_terminal_event(&harness, &task_id, TaskStatus::Error);
}

#[tokio::test]
async fn public_paged_task_missing_next_cursor_is_protocol_error() {
    let harness = TaskHarness::new();
    let plan = harness.paged_plan("http://127.0.0.1:1/unused", &["item"], true, None);
    let task_id = harness.start_paged(FixedPagedResolver::single(plan)).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert_eq!(task.total, 0);
    assert!(task
        .error_msg
        .as_deref()
        .unwrap_or("")
        .contains("next_cursor"));
}

#[tokio::test]
async fn public_paged_task_repeated_cursor_is_protocol_error() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"first-page")).await;
    let harness = TaskHarness::new();
    let plan = harness.paged_plan(server.url(), &["item"], true, Some(0));
    let task_id = harness.start_paged(FixedPagedResolver::single(plan)).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert_eq!(task.completed, 1);
    assert!(task
        .error_msg
        .as_deref()
        .unwrap_or("")
        .contains("next_cursor 重复"));
}

#[tokio::test]
async fn public_paged_task_empty_page_with_has_more_is_protocol_error() {
    let harness = TaskHarness::new();
    let plan = harness.paged_plan("http://127.0.0.1:1/unused", &[], true, Some(10));
    let task_id = harness.start_paged(FixedPagedResolver::single(plan)).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert!(task
        .error_msg
        .as_deref()
        .unwrap_or("")
        .contains("page_aweme_ids 为空"));
}

#[tokio::test]
async fn public_paged_task_media_free_source_page_continues_to_next_page() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"after-unavailable")).await;
    let harness = TaskHarness::new();
    let mut first = harness.paged_plan("http://127.0.0.1:1/unused", &[], true, Some(50));
    first.page_aweme_ids = vec!["unavailable".to_string()];
    let second = harness.paged_plan(server.url(), &["downloadable"], false, None);
    let resolver = FixedPagedResolver::multi(vec![
        PagedResolverResult::Plan(first),
        PagedResolverResult::Plan(second),
    ]);
    let task_id = harness.start_paged(resolver.clone()).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.total, 1);
    assert_eq!(task.completed, 1);
    assert_eq!(resolver.calls().len(), 2);
    assert!(harness.media_path("downloadable").exists());
}

#[tokio::test]
async fn public_paged_task_rejects_mode_and_save_dir_drift() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"page-data")).await;

    let mode_harness = TaskHarness::new();
    let mut wrong_mode = mode_harness.paged_plan(server.url(), &["mode-item"], false, None);
    wrong_mode.mode = "like".to_string();
    let mode_task = mode_harness
        .start_paged(FixedPagedResolver::single(wrong_mode))
        .await;
    let mode_result = mode_harness.wait_for_terminal(&mode_task).await;
    assert_eq!(mode_result.status, "error");
    assert!(mode_result
        .error_msg
        .as_deref()
        .unwrap_or("")
        .contains("mode 漂移"));

    let dir_harness = TaskHarness::new();
    let first = dir_harness.paged_plan(server.url(), &["dir-one"], true, Some(20));
    let mut second = dir_harness.paged_plan(server.url(), &["dir-two"], false, None);
    second.save_dir = dir_harness
        .root
        .join("different")
        .to_string_lossy()
        .to_string();
    let dir_task = dir_harness
        .start_paged(FixedPagedResolver::multi(vec![
            PagedResolverResult::Plan(first),
            PagedResolverResult::Plan(second),
        ]))
        .await;
    let dir_result = dir_harness.wait_for_terminal(&dir_task).await;
    assert_eq!(dir_result.status, "error");
    assert_eq!(dir_result.completed, 1);
    assert!(dir_result
        .error_msg
        .as_deref()
        .unwrap_or("")
        .contains("save_dir 漂移"));
}

#[tokio::test]
async fn public_paged_task_rejects_cross_page_duplicate_media_key() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"page-data")).await;
    let harness = TaskHarness::new();
    let first = harness.paged_plan(server.url(), &["duplicate"], true, Some(10));
    let second = harness.paged_plan(server.url(), &["duplicate"], false, None);
    let task_id = harness
        .start_paged(FixedPagedResolver::multi(vec![
            PagedResolverResult::Plan(first),
            PagedResolverResult::Plan(second),
        ]))
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert_eq!(task.completed, 1);
    assert!(task
        .error_msg
        .as_deref()
        .unwrap_or("")
        .contains("跨页重复 media_key"));
}

#[tokio::test]
async fn public_paged_task_max_counts_keeps_complete_work_group() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"image-data")).await;
    let harness = TaskHarness::with_max_counts(1);
    let mut plan = harness.paged_plan(server.url(), &["work-one", "work-two"], true, Some(100));
    let mut image_one = plan.items[0].clone();
    image_one.media_key = "work-one:image:1".to_string();
    image_one.kind = SingleMediaKind::Image;
    image_one.output.filename = "work-one-image-1".to_string();
    image_one.output.suffix = ".webp".to_string();
    let mut image_two = image_one.clone();
    image_two.media_key = "work-one:image:2".to_string();
    image_two.output.filename = "work-one-image-2".to_string();
    let work_two = plan.items[1].clone();
    plan.items = vec![image_one, image_two, work_two];
    let resolver = FixedPagedResolver::single(plan);
    let task_id = harness.start_paged(resolver.clone()).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.total, 2);
    assert_eq!(task.completed, 2);
    let items = harness.state.db.get_task_items(&task_id, None).unwrap();
    assert!(items
        .iter()
        .all(|item| item.aweme_id.as_deref() == Some("work-one")));
    assert_eq!(resolver.calls()[0].count, 1);
}

#[tokio::test]
async fn public_paged_task_partial_media_failure_can_complete() {
    let success = LocalHttpServer::start(HttpBehavior::Success(b"ok")).await;
    let failure = LocalHttpServer::start(HttpBehavior::NotFound).await;
    let harness = TaskHarness::new();
    let plan = harness.paged_plan_with_urls(
        &[("good", success.url()), ("bad", failure.url())],
        false,
        None,
    );
    let task_id = harness.start_paged(FixedPagedResolver::single(plan)).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.completed, 1);
    assert_eq!(task.failed, 1);
}

#[tokio::test]
async fn public_paged_task_all_media_failure_is_error() {
    let failure = LocalHttpServer::start(HttpBehavior::NotFound).await;
    let harness = TaskHarness::new();
    let plan = harness.paged_plan(failure.url(), &["bad"], false, None);
    let task_id = harness.start_paged(FixedPagedResolver::single(plan)).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert_eq!(task.failed, 1);
    assert!(task
        .error_msg
        .as_deref()
        .unwrap_or("")
        .contains("所有下载均失败"));
}

#[tokio::test]
async fn public_paged_task_cancelled_after_resolver_returns() {
    let harness = TaskHarness::new();
    let plan = harness.paged_plan(
        "http://127.0.0.1:1/unused",
        &["cancel-after-parse"],
        false,
        None,
    );
    let resolver = FixedPagedResolver::multi(vec![PagedResolverResult::DelayedPlan(
        plan,
        Duration::from_millis(150),
    )]);
    let task_id = harness.start_paged(resolver).await;
    wait_until(|| harness.state.get_cancel_signal(&task_id).is_some()).await;
    tokio::time::sleep(Duration::from_millis(30)).await;
    assert!(harness.state.cancel_task(&task_id));

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "cancelled");
    assert_eq!(task.total, 0);
}

#[tokio::test]
async fn public_paged_task_profile_failure_is_error_without_completed_event() {
    let harness = TaskHarness::new();
    harness.install_user_failure();
    let mut plan = harness.paged_plan("http://127.0.0.1:1/unused", &["profile-item"], false, None);
    plan.user_profile = Some(user_profile("sec-profile", "profile user"));
    let task_id = harness.start_paged(FixedPagedResolver::single(plan)).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert_eq!(task.total, 0);
    assert!(task
        .error_msg
        .as_deref()
        .unwrap_or("")
        .contains("harness user failure"));
    assert!(!harness.events.snapshot().iter().any(|event| {
        event.task_id == task_id && event.patch.status == Some(TaskStatus::Completed)
    }));
}

#[tokio::test]
async fn public_paged_task_profile_updates_task_metadata() {
    let harness = TaskHarness::new();
    std::fs::write(harness.media_path("profile-success"), b"existing").unwrap();
    let mut plan = harness.paged_plan(
        "http://127.0.0.1:1/unused",
        &["profile-success"],
        false,
        None,
    );
    plan.user_profile = Some(user_profile("sec-profile-success", "profile user"));
    let task_id = harness.start_paged(FixedPagedResolver::single(plan)).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.title.as_deref(), Some("profile user"));
    assert_eq!(task.author_nickname.as_deref(), Some("profile user"));
    assert!(harness
        .state
        .db
        .get_user_by_sec_uid("sec-profile-success")
        .unwrap()
        .is_some());
}

#[tokio::test]
async fn public_paged_task_terminal_db_failure_never_emits_completed() {
    let harness = TaskHarness::new();
    std::fs::write(harness.media_path("terminal-db"), b"existing").unwrap();
    harness.install_completed_status_failure();
    let plan = harness.paged_plan("http://127.0.0.1:1/unused", &["terminal-db"], false, None);
    let task_id = harness.start_paged(FixedPagedResolver::single(plan)).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert!(task
        .error_msg
        .as_deref()
        .unwrap_or("")
        .contains("harness completed status failure"));
    assert!(!harness.events.snapshot().iter().any(|event| {
        event.task_id == task_id && event.patch.status == Some(TaskStatus::Completed)
    }));
}

#[tokio::test]
async fn public_paged_task_first_page_resolver_error() {
    let harness = TaskHarness::new();
    let task_id = harness
        .start_paged(FixedPagedResolver::error("fixture paged error"))
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert_eq!(task.total, 0);
    assert!(task
        .error_msg
        .as_deref()
        .unwrap_or("")
        .contains("fixture paged error"));
    assert_terminal_event(&harness, &task_id, TaskStatus::Error);
}

#[tokio::test]
async fn public_paged_task_cancelled_during_page() {
    let server = LocalHttpServer::start(HttpBehavior::Chunked {
        chunks: 50,
        chunk_size: 16 * 1024,
        delay: std::time::Duration::from_millis(20),
    })
    .await;
    let harness = TaskHarness::new();
    let plan = harness.paged_plan(server.url(), &["item-cancel"], false, None);
    let task_id = harness.start_paged(FixedPagedResolver::single(plan)).await;

    wait_until(|| harness.state.get_cancel_signal(&task_id).is_some()).await;
    tokio::time::sleep(std::time::Duration::from_millis(80)).await;
    assert!(harness.state.cancel_task(&task_id));

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "cancelled");
    assert_eq!(task.failed, 0);
    assert_terminal_event(&harness, &task_id, TaskStatus::Cancelled);
}

#[tokio::test]
async fn public_paged_task_existing_file_is_skipped() {
    let harness = TaskHarness::new();
    std::fs::write(harness.media_path("existing-item"), b"existing").unwrap();
    let plan = harness.paged_plan("http://127.0.0.1:1/unused", &["existing-item"], false, None);
    let task_id = harness.start_paged(FixedPagedResolver::single(plan)).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.skipped, 1);
    assert_terminal_event(&harness, &task_id, TaskStatus::Completed);
}

// ============================================================
// Selection task tests (post mode with aweme_ids)
// ============================================================

#[tokio::test]
async fn public_paged_selection_single_hit() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"sel-data")).await;
    let harness = TaskHarness::new();
    let plan = harness.paged_plan(server.url(), &["target"], false, None);
    let task_id = harness
        .start_paged_selection(FixedPagedResolver::single(plan), &["target"])
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.completed, 1);
    assert_eq!(
        std::fs::read(harness.media_path("target")).unwrap(),
        b"sel-data"
    );
    assert_terminal_event(&harness, &task_id, TaskStatus::Completed);
}

#[tokio::test]
async fn public_paged_selection_skips_non_requested_items() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"keep-only")).await;
    let harness = TaskHarness::new();
    let mut plan = harness.paged_plan(server.url(), &["keep", "discard"], false, None);
    plan.page_aweme_ids = vec!["keep".to_string(), "discard".to_string()];
    let task_id = harness
        .start_paged_selection(FixedPagedResolver::single(plan), &["keep"])
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.completed, 1);
    assert!(harness.media_path("keep").exists());
    assert!(!harness.media_path("discard").exists());
}

#[tokio::test]
async fn public_paged_selection_missing_ids_error() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"present-data")).await;
    let harness = TaskHarness::new();
    let plan = harness.paged_plan(server.url(), &["present"], false, None);
    let task_id = harness
        .start_paged_selection(
            FixedPagedResolver::single(plan),
            &["present", "never-appears"],
        )
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert!(task
        .error_msg
        .as_deref()
        .unwrap_or("")
        .contains("missing_aweme_ids"));
    assert!(task
        .error_msg
        .as_deref()
        .unwrap_or("")
        .contains("never-appears"));
    // Successful files are retained despite missing IDs
    assert!(harness.media_path("present").exists());
}

#[tokio::test]
async fn public_paged_selection_all_missing_is_error_with_zero_counts() {
    let harness = TaskHarness::new();
    let plan = harness.paged_plan("http://127.0.0.1:1/unused", &[], false, None);
    let task_id = harness
        .start_paged_selection(
            FixedPagedResolver::single(plan),
            &["missing-1", "missing-2"],
        )
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert_eq!(task.total, 0);
    assert!(task
        .error_msg
        .as_deref()
        .unwrap_or("")
        .contains("未发现任何媒体"));
}

#[tokio::test]
async fn public_paged_selection_cross_page_hit() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"cross-page")).await;
    let harness = TaskHarness::new();
    let plan1 = harness.paged_plan(server.url(), &["page1"], true, Some(50));
    let plan2 = harness.paged_plan(server.url(), &["page2"], false, None);
    let resolver = FixedPagedResolver::multi(vec![
        PagedResolverResult::Plan(plan1),
        PagedResolverResult::Plan(plan2),
    ]);
    let task_id = harness
        .start_paged_selection(resolver.clone(), &["page1", "page2"])
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.completed, 2);
    assert_eq!(
        resolver
            .calls()
            .iter()
            .map(|c| c.cursor)
            .collect::<Vec<_>>(),
        [0, 50]
    );
}

#[tokio::test]
async fn public_paged_selection_duplicate_input_ids_no_duplicate_download() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"dedup")).await;
    let harness = TaskHarness::new();
    let plan = harness.paged_plan(server.url(), &["item"], false, None);
    let task_id = harness
        .start_paged_selection(FixedPagedResolver::single(plan), &["item", "item", "item"])
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.completed, 1);
}

#[tokio::test]
async fn public_paged_selection_cancelled_after_resolver_does_not_report_missing() {
    let harness = TaskHarness::new();
    let plan = harness.paged_plan("http://127.0.0.1:1/unused", &["hit"], false, None);
    let resolver = FixedPagedResolver::multi(vec![PagedResolverResult::DelayedPlan(
        plan,
        Duration::from_millis(150),
    )]);
    let task_id = harness
        .start_paged_selection(resolver, &["hit", "never-seen"])
        .await;
    wait_until(|| harness.state.get_cancel_signal(&task_id).is_some()).await;
    tokio::time::sleep(Duration::from_millis(30)).await;
    assert!(harness.state.cancel_task(&task_id));

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "cancelled");
    // Cancelled tasks should not report missing IDs
    assert!(
        task.error_msg.is_none() || !task.error_msg.as_deref().unwrap_or("").contains("missing")
    );
}

#[tokio::test]
async fn public_paged_selection_unavailable_ids_error() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"avail")).await;
    let harness = TaskHarness::new();
    // First item has media, second item is in page_aweme_ids but has no items
    let mut plan = harness.paged_plan(server.url(), &["has-media"], false, None);
    plan.page_aweme_ids = vec!["has-media".to_string(), "no-media".to_string()];
    let task_id = harness
        .start_paged_selection(FixedPagedResolver::single(plan), &["has-media", "no-media"])
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert!(task
        .error_msg
        .as_deref()
        .unwrap_or("")
        .contains("unavailable_aweme_ids"));
    assert!(task.error_msg.as_deref().unwrap_or("").contains("no-media"));
    // Successful files are retained
    assert_eq!(task.completed, 1);
}

#[tokio::test]
async fn public_paged_selection_media_free_nonterminal_page_is_unavailable_not_protocol_error() {
    let harness = TaskHarness::new();
    let mut plan = harness.paged_plan("http://127.0.0.1:1/unused", &[], true, Some(50));
    plan.page_aweme_ids = vec!["no-media".to_string()];
    let resolver = FixedPagedResolver::single(plan);
    let task_id = harness
        .start_paged_selection(resolver.clone(), &["no-media"])
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert_eq!(task.total, 0);
    assert!(task
        .error_msg
        .as_deref()
        .unwrap_or("")
        .contains("unavailable_aweme_ids=[no-media]"));
    assert!(!task.error_msg.as_deref().unwrap_or("").contains("协议错误"));
    assert_eq!(resolver.calls().len(), 1);
}

#[tokio::test]
async fn public_paged_selection_repeated_selected_page_does_not_redownload() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"selection-repeat")).await;
    let harness = TaskHarness::new();
    let first = harness.paged_plan(server.url(), &["repeat"], true, Some(50));
    let second = harness.paged_plan(server.url(), &["repeat", "later"], false, None);
    let resolver = FixedPagedResolver::multi(vec![
        PagedResolverResult::Plan(first),
        PagedResolverResult::Plan(second),
    ]);
    let task_id = harness
        .start_paged_selection(resolver, &["repeat", "later"])
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.total, 2);
    assert_eq!(task.completed, 2);
    assert_eq!(task.skipped, 0);
    let items = harness.state.db.get_task_items(&task_id, None).unwrap();
    assert_eq!(items.len(), 2);
}

#[tokio::test]
async fn public_paged_selection_ignores_repeated_unselected_media_key() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"selected-only")).await;
    let harness = TaskHarness::new();
    let first = harness.paged_plan(server.url(), &["other"], true, Some(50));
    let second = harness.paged_plan(server.url(), &["other", "target"], false, None);
    let resolver = FixedPagedResolver::multi(vec![
        PagedResolverResult::Plan(first),
        PagedResolverResult::Plan(second),
    ]);
    let task_id = harness.start_paged_selection(resolver, &["target"]).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.total, 1);
    assert_eq!(task.completed, 1);
    assert!(!harness.media_path("other").exists());
    assert!(harness.media_path("target").exists());
}

#[tokio::test]
async fn public_paged_selection_overrides_global_max_counts() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"selection-max")).await;
    let harness = TaskHarness::with_max_counts(1);
    let first = harness.paged_plan(server.url(), &["first"], true, Some(50));
    let second = harness.paged_plan(server.url(), &["second"], false, None);
    let resolver = FixedPagedResolver::multi(vec![
        PagedResolverResult::Plan(first),
        PagedResolverResult::Plan(second),
    ]);
    let task_id = harness
        .start_paged_selection(resolver.clone(), &["first", "second"])
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.total, 2);
    assert_eq!(task.completed, 2);
    assert_eq!(resolver.calls().len(), 2);
    assert!(resolver.calls().iter().all(|call| call.count > 1));
}

#[tokio::test]
async fn public_paged_selection_resolver_error_includes_current_selection_state() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"selection-state")).await;
    let harness = TaskHarness::new();
    let first = harness.paged_plan(server.url(), &["first"], true, Some(50));
    let resolver = FixedPagedResolver::multi(vec![
        PagedResolverResult::Plan(first),
        PagedResolverResult::Error("later page failed".to_string()),
    ]);
    let task_id = harness
        .start_paged_selection(resolver, &["first", "missing"])
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    let error = task.error_msg.as_deref().unwrap_or("");
    assert!(error.contains("第 2 页解析失败 (cursor=50)"), "{error}");
    assert!(error.contains("selection_state"), "{error}");
    assert!(error.contains("requested=[first,missing]"), "{error}");
    assert!(error.contains("seen=[first]"), "{error}");
    assert!(error.contains("planned=[first]"), "{error}");
}

#[tokio::test]
async fn public_paged_selection_keeps_complete_media_group_for_selected_work() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"media-group")).await;
    let harness = TaskHarness::new();
    let mut plan = harness.paged_plan(server.url(), &["gallery"], false, None);
    let mut image = plan.items[0].clone();
    image.media_key = "gallery:image:1".to_string();
    image.kind = SingleMediaKind::Image;
    image.output.filename = "gallery-image-1".to_string();
    image.output.suffix = ".webp".to_string();
    plan.items.push(image);
    let task_id = harness
        .start_paged_selection(FixedPagedResolver::single(plan), &["gallery"])
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.total, 2);
    assert_eq!(task.completed, 2);
    let items = harness.state.db.get_task_items(&task_id, None).unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].aweme_id.as_deref(), Some("gallery"));
    assert_eq!(items[1].aweme_id.as_deref(), Some("gallery"));
    assert!(harness.media_path("gallery").exists());
    assert!(harness.root.join("gallery-image-1.webp").exists());
}

#[tokio::test]
async fn public_paged_selection_cancelled_during_media_does_not_report_missing() {
    let server = LocalHttpServer::start(HttpBehavior::Chunked {
        chunks: 50,
        chunk_size: 16 * 1024,
        delay: Duration::from_millis(20),
    })
    .await;
    let harness = TaskHarness::new();
    let plan = harness.paged_plan(server.url(), &["selected"], false, None);
    let task_id = harness
        .start_paged_selection(FixedPagedResolver::single(plan), &["selected", "not-seen"])
        .await;

    wait_until(|| harness.state.get_cancel_signal(&task_id).is_some()).await;
    tokio::time::sleep(Duration::from_millis(80)).await;
    assert!(harness.state.cancel_task(&task_id));

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "cancelled");
    assert_eq!(task.failed, 0);
    assert!(task
        .error_msg
        .as_deref()
        .is_none_or(|error| !error.contains("missing_aweme_ids")));
    assert_terminal_event(&harness, &task_id, TaskStatus::Cancelled);
}

// ============================================================
// Gallery media item tests (issue 07)
// ============================================================

fn gallery_plan(
    harness: &TaskHarness,
    aweme_id: &str,
    url: &str,
    n_images: i64,
) -> SingleDownloadPlanV1 {
    use super::contract::SingleDownloadItem;
    let items: Vec<SingleDownloadItem> = (1..=n_images)
        .map(|i| SingleDownloadItem {
            media_key: format!("{aweme_id}:image:{i}"),
            aweme_id: aweme_id.to_string(),
            urls: vec![url.to_string()],
            kind: SingleMediaKind::Image,
            output: SingleOutputSpec {
                filename: format!("{aweme_id}_image_{i}"),
                suffix: ".webp".to_string(),
                folder_name: None,
            },
            headers: Default::default(),
            accessories: Vec::new(),
            metadata: video(aweme_id),
        })
        .collect();
    SingleDownloadPlanV1 {
        success: true,
        contract_version: 1,
        mode: "one".to_string(),
        save_dir: harness.root.to_string_lossy().to_string(),
        items,
        total: n_images,
    }
}

fn paged_gallery_plan(
    harness: &TaskHarness,
    aweme_id: &str,
    url: &str,
    n_images: i64,
    n_live: i64,
    has_more: bool,
    next_cursor: Option<i64>,
) -> PagedDownloadPlanV1 {
    use super::contract::SingleDownloadItem;
    let mut items: Vec<SingleDownloadItem> = Vec::new();
    for i in 1..=n_live {
        items.push(SingleDownloadItem {
            media_key: format!("{aweme_id}:live_photo:{i}"),
            aweme_id: aweme_id.to_string(),
            urls: vec![url.to_string()],
            kind: SingleMediaKind::LivePhoto,
            output: SingleOutputSpec {
                filename: format!("{aweme_id}_live_{i}"),
                suffix: ".mp4".to_string(),
                folder_name: None,
            },
            headers: Default::default(),
            accessories: Vec::new(),
            metadata: video(aweme_id),
        });
    }
    for i in 1..=n_images {
        items.push(SingleDownloadItem {
            media_key: format!("{aweme_id}:image:{i}"),
            aweme_id: aweme_id.to_string(),
            urls: vec![url.to_string()],
            kind: SingleMediaKind::Image,
            output: SingleOutputSpec {
                filename: format!("{aweme_id}_image_{i}"),
                suffix: ".webp".to_string(),
                folder_name: None,
            },
            headers: Default::default(),
            accessories: Vec::new(),
            metadata: video(aweme_id),
        });
    }
    PagedDownloadPlanV1 {
        success: true,
        contract_version: 1,
        mode: "post".to_string(),
        save_dir: harness.root.to_string_lossy().to_string(),
        items,
        next_cursor,
        has_more,
        page_aweme_ids: vec![aweme_id.to_string()],
        user_profile: None,
    }
}

#[tokio::test]
async fn public_gallery_single_work_multi_row_persistence() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"gallery-data")).await;
    let harness = TaskHarness::new();
    let plan = gallery_plan(&harness, "gallery-multi", server.url(), 3);
    let task_id = harness.start(FixedSingleResolver::plan(plan)).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.total, 3);
    assert_eq!(task.completed, 3);

    let detail = harness.state.db.get_task_detail(&task_id).unwrap().unwrap();
    assert_eq!(detail.items.len(), 3);
    assert_eq!(detail.items[0].aweme_id.as_deref(), Some("gallery-multi"));
    assert_eq!(detail.items[1].aweme_id.as_deref(), Some("gallery-multi"));
    assert_eq!(detail.items[2].aweme_id.as_deref(), Some("gallery-multi"));
    assert_eq!(detail.items[0].status, "completed");
    assert_eq!(detail.items[1].status, "completed");
    assert_eq!(detail.items[2].status, "completed");
    assert!(harness.root.join("gallery-multi_image_1.webp").exists());
    assert!(harness.root.join("gallery-multi_image_2.webp").exists());
    assert!(harness.root.join("gallery-multi_image_3.webp").exists());
}

#[tokio::test]
async fn public_gallery_paged_multi_row_persistence() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"gallery-page")).await;
    let harness = TaskHarness::new();
    let plan = paged_gallery_plan(&harness, "gallery-page", server.url(), 2, 1, false, None);
    let task_id = harness.start_paged(FixedPagedResolver::single(plan)).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.total, 3);
    assert_eq!(task.completed, 3);

    let detail = harness.state.db.get_task_detail(&task_id).unwrap().unwrap();
    assert_eq!(detail.items.len(), 3);
    assert_eq!(detail.items[0].media_kind.as_deref(), Some("live_photo"));
    assert_eq!(
        detail.items[0].media_key.as_deref(),
        Some("gallery-page:live_photo:1")
    );
    assert_eq!(detail.items[1].media_kind.as_deref(), Some("image"));
    assert_eq!(
        detail.items[1].media_key.as_deref(),
        Some("gallery-page:image:1")
    );
    assert_eq!(detail.items[2].media_kind.as_deref(), Some("image"));
    assert_eq!(
        detail.items[2].media_key.as_deref(),
        Some("gallery-page:image:2")
    );
    assert!(harness.root.join("gallery-page_live_1.mp4").exists());
    assert!(harness.root.join("gallery-page_image_1.webp").exists());
    assert!(harness.root.join("gallery-page_image_2.webp").exists());
}

#[tokio::test]
async fn public_gallery_partial_failure_retains_successful_files() {
    let success = LocalHttpServer::start(HttpBehavior::Success(b"gallery-ok")).await;
    let failure = LocalHttpServer::start(HttpBehavior::NotFound).await;
    let harness = TaskHarness::new();
    let mut plan = gallery_plan(&harness, "gallery-partial", success.url(), 3);
    plan.items[1].urls = vec![failure.url().to_string()];
    let task_id = harness.start(FixedSingleResolver::plan(plan)).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.completed, 2);
    assert_eq!(task.failed, 1);
    assert!(harness.root.join("gallery-partial_image_1.webp").exists());
    assert!(!harness.root.join("gallery-partial_image_2.webp").exists());
    assert!(harness.root.join("gallery-partial_image_3.webp").exists());
}

#[tokio::test]
async fn public_gallery_selection_keeps_complete_group() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"sel-gallery")).await;
    let harness = TaskHarness::new();
    let plan = paged_gallery_plan(&harness, "sel-gallery", server.url(), 2, 0, false, None);
    let task_id = harness
        .start_paged_selection(FixedPagedResolver::single(plan), &["sel-gallery"])
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.total, 2);
    assert_eq!(task.completed, 2);
    let items = harness.state.db.get_task_items(&task_id, None).unwrap();
    assert_eq!(items.len(), 2);
    assert!(items
        .iter()
        .all(|item| item.aweme_id.as_deref() == Some("sel-gallery")));
    assert!(harness.root.join("sel-gallery_image_1.webp").exists());
    assert!(harness.root.join("sel-gallery_image_2.webp").exists());
}

#[tokio::test]
async fn public_gallery_webp_and_mp4_mixed_download() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"gallery-mixed")).await;
    let harness = TaskHarness::new();
    let plan = paged_gallery_plan(&harness, "gallery-mixed", server.url(), 1, 1, false, None);
    let task_id = harness.start_paged(FixedPagedResolver::single(plan)).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.completed, 2);
    assert!(harness.root.join("gallery-mixed_live_1.mp4").exists());
    assert!(harness.root.join("gallery-mixed_image_1.webp").exists());
}

#[tokio::test]
async fn public_gallery_downloads_first_item_accessory_once() {
    let media_server = LocalHttpServer::start(HttpBehavior::Success(b"gallery-media")).await;
    let accessory_server = LocalHttpServer::start(HttpBehavior::Success(b"gallery-music")).await;
    let harness = TaskHarness::with_config(AppConfig {
        music: true,
        cover: false,
        desc: false,
        ..AppConfig::default()
    });
    let mut plan = gallery_plan(&harness, "gallery-accessory", media_server.url(), 2);
    plan.items[0].accessories.push(SingleAccessory {
        kind: SingleAccessoryKind::Music,
        output: SingleOutputSpec {
            filename: "gallery-accessory_music".to_string(),
            suffix: ".mp3".to_string(),
            folder_name: None,
        },
        url: Some(accessory_server.url().to_string()),
        content: None,
    });

    let task_id = harness.start(FixedSingleResolver::plan(plan)).await;
    let task = harness.wait_for_terminal(&task_id).await;

    assert_eq!(task.status, "completed");
    assert_eq!(task.total, 2);
    assert_eq!(task.completed, 2);
    assert_eq!(accessory_server.request_count(), 1);
    assert!(harness.root.join("gallery-accessory_music.mp3").exists());
}

#[tokio::test]
async fn public_gallery_skipped_file_does_not_overwrite() {
    let harness = TaskHarness::new();
    std::fs::write(
        harness.root.join("gallery-skip_image_1.webp"),
        b"existing-1",
    )
    .unwrap();
    std::fs::write(
        harness.root.join("gallery-skip_image_2.webp"),
        b"existing-2",
    )
    .unwrap();
    let plan = gallery_plan(&harness, "gallery-skip", "http://127.0.0.1:1/unused", 2);
    let task_id = harness.start(FixedSingleResolver::plan(plan)).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.skipped, 2);
    assert_eq!(
        std::fs::read(harness.root.join("gallery-skip_image_1.webp")).unwrap(),
        b"existing-1"
    );
    assert_eq!(
        std::fs::read(harness.root.join("gallery-skip_image_2.webp")).unwrap(),
        b"existing-2"
    );
}

#[tokio::test]
async fn public_gallery_all_failure_is_error() {
    let failure = LocalHttpServer::start(HttpBehavior::NotFound).await;
    let harness = TaskHarness::new();
    let plan = gallery_plan(&harness, "gallery-all-fail", failure.url(), 2);
    let task_id = harness.start(FixedSingleResolver::plan(plan)).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert_eq!(task.failed, 2);
    assert!(task
        .error_msg
        .as_deref()
        .unwrap_or("")
        .contains("所有下载均失败"));
}

// ============================================================
// Cross-mode paged task tests (issue 08: like/mix/collects)
// ============================================================

/// Helper to run a two-page success test for a given mode.
async fn assert_two_page_mode_success(mode: DownloadMode, url: &str) {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"page-data")).await;
    let harness = TaskHarness::new();
    let plan1 =
        harness.paged_plan_for_mode(mode.as_str(), server.url(), &["item-1"], true, Some(100));
    let plan2 = harness.paged_plan_for_mode(mode.as_str(), server.url(), &["item-2"], false, None);
    let resolver = FixedPagedResolver::multi(vec![
        PagedResolverResult::Plan(plan1),
        PagedResolverResult::Plan(plan2),
    ]);
    let task_id = harness
        .start_paged_for_mode(mode, url, resolver.clone(), &[])
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.mode, mode.as_str());
    assert_eq!(task.completed, 2);
    assert_eq!(
        resolver
            .calls()
            .iter()
            .map(|call| call.cursor)
            .collect::<Vec<_>>(),
        [0, 100]
    );
    assert!(resolver
        .calls()
        .iter()
        .all(|call| call.mode == mode.as_str()));
    let detail = harness.state.db.get_task_detail(&task_id).unwrap().unwrap();
    assert_eq!(detail.items.len(), 2);
    assert_started_event(&harness, &task_id, mode, url);
    assert_terminal_event(&harness, &task_id, TaskStatus::Completed);
    let finished_count = harness
        .events
        .snapshot()
        .iter()
        .filter(|event| event.task_id == task_id && event.event_type == TaskEventType::Finished)
        .count();
    assert_eq!(finished_count, 1);
}

#[tokio::test]
async fn public_paged_like_two_pages_success() {
    assert_two_page_mode_success(DownloadMode::Like, "https://fixture.invalid/like-user").await;
}

#[tokio::test]
async fn public_paged_mix_two_pages_success() {
    assert_two_page_mode_success(DownloadMode::Mix, "https://fixture.invalid/mix").await;
}

#[tokio::test]
async fn public_paged_collects_two_pages_success() {
    assert_two_page_mode_success(DownloadMode::Collects, "https://fixture.invalid/collects").await;
}

/// Cross-mode table-driven tests: selected work on later page
#[tokio::test]
async fn public_paged_cross_mode_selection_on_later_page() {
    let modes = [
        (DownloadMode::Like, "https://fixture.invalid/like"),
        (DownloadMode::Mix, "https://fixture.invalid/mix"),
        (DownloadMode::Collects, "https://fixture.invalid/collects"),
    ];
    for (mode, url) in &modes {
        let server = LocalHttpServer::start(HttpBehavior::Success(b"late-data")).await;
        let harness = TaskHarness::new();
        let plan1 = harness.paged_plan_for_mode(
            mode.as_str(),
            "http://127.0.0.1:1/unused",
            &["other"],
            true,
            Some(50),
        );
        let mut plan2 =
            harness.paged_plan_for_mode(mode.as_str(), server.url(), &["target"], false, None);
        plan2.items[0].media_key = "target:image:1".to_string();
        plan2.items[0].kind = SingleMediaKind::Image;
        plan2.items[0].output.filename = "target_image_1".to_string();
        plan2.items[0].output.suffix = ".webp".to_string();
        let mut second_image = plan2.items[0].clone();
        second_image.media_key = "target:image:2".to_string();
        second_image.output.filename = "target_image_2".to_string();
        plan2.items.push(second_image);
        let resolver = FixedPagedResolver::multi(vec![
            PagedResolverResult::Plan(plan1),
            PagedResolverResult::Plan(plan2),
        ]);
        let task_id = harness
            .start_paged_for_mode(*mode, url, resolver, &["target"])
            .await;

        let task = harness.wait_for_terminal(&task_id).await;
        assert_eq!(task.status, "completed", "mode={}", mode.as_str());
        assert_eq!(task.completed, 2, "mode={}", mode.as_str());
        assert!(
            harness.root.join("target_image_1.webp").exists(),
            "mode={}",
            mode.as_str()
        );
        assert!(
            harness.root.join("target_image_2.webp").exists(),
            "mode={}",
            mode.as_str()
        );
    }
}

/// Cross-mode test: returned mode drift is rejected
#[tokio::test]
async fn public_paged_cross_mode_rejects_mode_drift() {
    let harness = TaskHarness::new();
    let mode = DownloadMode::Like;
    let mut plan = harness.paged_plan_for_mode(
        mode.as_str(),
        "http://127.0.0.1:1/unused",
        &["item"],
        false,
        None,
    );
    plan.mode = "mix".to_string();
    let task_id = harness
        .start_paged_for_mode(
            mode,
            "https://fixture.invalid/like",
            FixedPagedResolver::single(plan),
            &[],
        )
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert!(
        task.error_msg
            .as_deref()
            .unwrap_or("")
            .contains("mode 漂移"),
        "mode drift must be detected"
    );
}

/// Cross-mode test: max_counts counts works not media files
#[tokio::test]
async fn public_paged_cross_mode_max_counts_works_not_media() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"max-data")).await;
    let harness = TaskHarness::with_max_counts(1);
    let mode = DownloadMode::Collects;
    let mut plan = harness.paged_plan_for_mode(
        mode.as_str(),
        server.url(),
        &["work-a", "work-b"],
        true,
        Some(100),
    );
    plan.items[0].media_key = "work-a:image:1".to_string();
    plan.items[0].kind = SingleMediaKind::Image;
    plan.items[0].output.filename = "work-a_image_1".to_string();
    plan.items[0].output.suffix = ".webp".to_string();
    let mut extra_item = plan.items[0].clone();
    extra_item.media_key = "work-a:image:2".to_string();
    extra_item.output.filename = "work-a_image_2".to_string();
    plan.items.push(extra_item);
    let resolver = FixedPagedResolver::single(plan);
    let task_id = harness
        .start_paged_for_mode(
            mode,
            "https://fixture.invalid/collects",
            resolver.clone(),
            &[],
        )
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.total, 2, "must keep complete media group");
    let items = harness.state.db.get_task_items(&task_id, None).unwrap();
    assert!(items
        .iter()
        .all(|item| item.aweme_id.as_deref() == Some("work-a")));
    assert_eq!(resolver.calls()[0].count, 1);
}

/// Cross-mode test: existing files are skipped
#[tokio::test]
async fn public_paged_cross_mode_existing_file_skipped() {
    let harness = TaskHarness::new();
    std::fs::write(harness.media_path("skip-me"), b"existing").unwrap();
    let plan = harness.paged_plan_for_mode(
        "like",
        "http://127.0.0.1:1/unused",
        &["skip-me"],
        false,
        None,
    );
    let task_id = harness
        .start_paged_for_mode(
            DownloadMode::Like,
            "https://fixture.invalid/like",
            FixedPagedResolver::single(plan),
            &[],
        )
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.skipped, 1);
}

/// Cross-mode test: media-free source page with has_more=true continues
#[tokio::test]
async fn public_paged_cross_mode_media_free_page_continues() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"after-free")).await;
    let harness = TaskHarness::new();
    let mut first =
        harness.paged_plan_for_mode("mix", "http://127.0.0.1:1/unused", &[], true, Some(50));
    first.page_aweme_ids = vec!["no-media".to_string()];
    let second = harness.paged_plan_for_mode("mix", server.url(), &["downloadable"], false, None);
    let resolver = FixedPagedResolver::multi(vec![
        PagedResolverResult::Plan(first),
        PagedResolverResult::Plan(second),
    ]);
    let task_id = harness
        .start_paged_for_mode(
            DownloadMode::Mix,
            "https://fixture.invalid/mix",
            resolver.clone(),
            &[],
        )
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.total, 1);
    assert_eq!(task.completed, 1);
    assert_eq!(resolver.calls().len(), 2);
}

/// Cross-mode test: empty source page with has_more=true is protocol error
#[tokio::test]
async fn public_paged_cross_mode_empty_page_with_has_more_is_protocol_error() {
    let harness = TaskHarness::new();
    let plan =
        harness.paged_plan_for_mode("collects", "http://127.0.0.1:1/unused", &[], true, Some(10));
    let task_id = harness
        .start_paged_for_mode(
            DownloadMode::Collects,
            "https://fixture.invalid/collects",
            FixedPagedResolver::single(plan),
            &[],
        )
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert!(task
        .error_msg
        .as_deref()
        .unwrap_or("")
        .contains("page_aweme_ids 为空"));
}

/// Cross-mode test: later-page resolver error retains earlier files
#[tokio::test]
async fn public_paged_cross_mode_later_page_error_retains_earlier() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"page-one")).await;
    let harness = TaskHarness::new();
    let first = harness.paged_plan_for_mode("like", server.url(), &["first"], true, Some(100));
    let resolver = FixedPagedResolver::multi(vec![
        PagedResolverResult::Plan(first),
        PagedResolverResult::Error("later page failure".to_string()),
    ]);
    let task_id = harness
        .start_paged_for_mode(
            DownloadMode::Like,
            "https://fixture.invalid/like",
            resolver,
            &[],
        )
        .await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert_eq!(task.completed, 1);
    assert!(harness.media_path("first").exists());
    assert_terminal_event(&harness, &task_id, TaskStatus::Error);
}

/// Cross-mode test: cancellation does not report false missing/unavailable
#[tokio::test]
async fn public_paged_cross_mode_cancelled_no_false_missing() {
    let harness = TaskHarness::new();
    let plan = harness.paged_plan_for_mode(
        "like",
        "http://127.0.0.1:1/unused",
        &["will-cancel"],
        false,
        None,
    );
    let resolver = FixedPagedResolver::multi(vec![PagedResolverResult::DelayedPlan(
        plan,
        Duration::from_millis(150),
    )]);
    let task_id = harness
        .start_paged_for_mode(
            DownloadMode::Like,
            "https://fixture.invalid/like",
            resolver,
            &["will-cancel", "not-seen"],
        )
        .await;

    wait_until(|| harness.state.get_cancel_signal(&task_id).is_some()).await;
    tokio::time::sleep(Duration::from_millis(30)).await;
    assert!(harness.state.cancel_task(&task_id));

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "cancelled");
    assert!(task.error_msg.as_deref().is_none_or(|error| {
        !error.contains("missing_aweme_ids") && !error.contains("unavailable_aweme_ids")
    }));
}
