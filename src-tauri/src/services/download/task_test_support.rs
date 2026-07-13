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
use super::task_service::{
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

    pub(crate) fn paged_plan(&self, url: &str, aweme_ids: &[&str], has_more: bool, next_cursor: Option<i64>) -> PagedDownloadPlanV1 {
        let items: Vec<SingleDownloadItem> = aweme_ids.iter().map(|aweme_id| {
            SingleDownloadItem {
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
            }
        }).collect();
        PagedDownloadPlanV1 {
            success: true,
            contract_version: 1,
            mode: "post".to_string(),
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
        plan.page_aweme_ids = items.iter().map(|(aweme_id, _)| (*aweme_id).to_string()).collect();
        plan.items = items
            .iter()
            .map(|(aweme_id, url)| self.plan(url, aweme_id).items.into_iter().next().unwrap())
            .collect();
        plan
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
        TaskApplicationService::with_paged_test_adapters(
            &self.state,
            Arc::new(resolver),
            self.events.clone(),
        )
        .start_batch_download_mode(
            DownloadMode::Post,
            "https://fixture.invalid/user",
            &[],
        )
        .await
        .unwrap()
    }

    pub(crate) async fn start_paged_selection(&self, resolver: FixedPagedResolver, aweme_ids: &[&str]) -> String {
        TaskApplicationService::with_paged_test_adapters(
            &self.state,
            Arc::new(resolver),
            self.events.clone(),
        )
        .start_batch_download_mode(
            DownloadMode::Post,
            "https://fixture.invalid/user",
            &aweme_ids.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        )
        .await
        .unwrap()
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

fn assert_started_event(
    harness: &TaskHarness,
    task_id: &str,
    mode: DownloadMode,
    url: &str,
) {
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
        .filter(|event| {
            event.task_id == task_id && event.event_type == TaskEventType::Progress
        })
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
    assert!(task.error_msg.as_deref().unwrap_or("").contains("next_cursor"));
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
    assert!(task.error_msg.as_deref().unwrap_or("").contains("next_cursor 重复"));
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
    let mut first = harness.paged_plan(
        "http://127.0.0.1:1/unused",
        &[],
        true,
        Some(50),
    );
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
    assert!(mode_result.error_msg.as_deref().unwrap_or("").contains("mode 漂移"));

    let dir_harness = TaskHarness::new();
    let first = dir_harness.paged_plan(server.url(), &["dir-one"], true, Some(20));
    let mut second = dir_harness.paged_plan(server.url(), &["dir-two"], false, None);
    second.save_dir = dir_harness.root.join("different").to_string_lossy().to_string();
    let dir_task = dir_harness
        .start_paged(FixedPagedResolver::multi(vec![
            PagedResolverResult::Plan(first),
            PagedResolverResult::Plan(second),
        ]))
        .await;
    let dir_result = dir_harness.wait_for_terminal(&dir_task).await;
    assert_eq!(dir_result.status, "error");
    assert_eq!(dir_result.completed, 1);
    assert!(dir_result.error_msg.as_deref().unwrap_or("").contains("save_dir 漂移"));
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
    assert!(task.error_msg.as_deref().unwrap_or("").contains("跨页重复 media_key"));
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
    assert!(items.iter().all(|item| item.aweme_id.as_deref() == Some("work-one")));
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
    assert!(task.error_msg.as_deref().unwrap_or("").contains("所有下载均失败"));
}

#[tokio::test]
async fn public_paged_task_cancelled_after_resolver_returns() {
    let harness = TaskHarness::new();
    let plan = harness.paged_plan("http://127.0.0.1:1/unused", &["cancel-after-parse"], false, None);
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
    assert!(task.error_msg.as_deref().unwrap_or("").contains("harness user failure"));
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
    let plan = harness.paged_plan(
        "http://127.0.0.1:1/unused",
        &["terminal-db"],
        false,
        None,
    );
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
    let task_id = harness.start_paged(FixedPagedResolver::error("fixture paged error")).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert_eq!(task.total, 0);
    assert!(task.error_msg.as_deref().unwrap_or("").contains("fixture paged error"));
    assert_terminal_event(&harness, &task_id, TaskStatus::Error);
}

#[tokio::test]
async fn public_paged_task_cancelled_during_page() {
    let server = LocalHttpServer::start(HttpBehavior::Chunked {
        chunks: 50,
        chunk_size: 16 * 1024,
        delay: std::time::Duration::from_millis(20),
    }).await;
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
    let task_id = harness.start_paged_selection(FixedPagedResolver::single(plan), &["target"]).await;

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
    let task_id = harness.start_paged_selection(FixedPagedResolver::single(plan), &["keep"]).await;

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
    let task_id = harness.start_paged_selection(FixedPagedResolver::single(plan), &["present", "never-appears"]).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert!(task.error_msg.as_deref().unwrap_or("").contains("missing_aweme_ids"));
    assert!(task.error_msg.as_deref().unwrap_or("").contains("never-appears"));
    // Successful files are retained despite missing IDs
    assert!(harness.media_path("present").exists());
}

#[tokio::test]
async fn public_paged_selection_all_missing_is_error_with_zero_counts() {
    let harness = TaskHarness::new();
    let plan = harness.paged_plan("http://127.0.0.1:1/unused", &[], false, None);
    let task_id = harness.start_paged_selection(FixedPagedResolver::single(plan), &["missing-1", "missing-2"]).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert_eq!(task.total, 0);
    assert!(task.error_msg.as_deref().unwrap_or("").contains("未发现任何媒体"));
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
    let task_id = harness.start_paged_selection(resolver.clone(), &["page1", "page2"]).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "completed");
    assert_eq!(task.completed, 2);
    assert_eq!(
        resolver.calls().iter().map(|c| c.cursor).collect::<Vec<_>>(),
        [0, 50]
    );
}

#[tokio::test]
async fn public_paged_selection_duplicate_input_ids_no_duplicate_download() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"dedup")).await;
    let harness = TaskHarness::new();
    let plan = harness.paged_plan(server.url(), &["item"], false, None);
    let task_id = harness.start_paged_selection(FixedPagedResolver::single(plan), &["item", "item", "item"]).await;

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
    let task_id = harness.start_paged_selection(resolver, &["hit", "never-seen"]).await;
    wait_until(|| harness.state.get_cancel_signal(&task_id).is_some()).await;
    tokio::time::sleep(Duration::from_millis(30)).await;
    assert!(harness.state.cancel_task(&task_id));

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "cancelled");
    // Cancelled tasks should not report missing IDs
    assert!(task.error_msg.is_none() || !task.error_msg.as_deref().unwrap_or("").contains("missing"));
}

#[tokio::test]
async fn public_paged_selection_unavailable_ids_error() {
    let server = LocalHttpServer::start(HttpBehavior::Success(b"avail")).await;
    let harness = TaskHarness::new();
    // First item has media, second item is in page_aweme_ids but has no items
    let mut plan = harness.paged_plan(server.url(), &["has-media"], false, None);
    plan.page_aweme_ids = vec!["has-media".to_string(), "no-media".to_string()];
    let task_id = harness.start_paged_selection(FixedPagedResolver::single(plan), &["has-media", "no-media"]).await;

    let task = harness.wait_for_terminal(&task_id).await;
    assert_eq!(task.status, "error");
    assert!(task.error_msg.as_deref().unwrap_or("").contains("unavailable_aweme_ids"));
    assert!(task.error_msg.as_deref().unwrap_or("").contains("no-media"));
    // Successful files are retained
    assert_eq!(task.completed, 1);
}

#[tokio::test]
async fn public_paged_selection_media_free_nonterminal_page_is_unavailable_not_protocol_error() {
    let harness = TaskHarness::new();
    let mut plan = harness.paged_plan(
        "http://127.0.0.1:1/unused",
        &[],
        true,
        Some(50),
    );
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
    assert!(!task
        .error_msg
        .as_deref()
        .unwrap_or("")
        .contains("协议错误"));
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
    let task_id = harness
        .start_paged_selection(resolver, &["target"])
        .await;

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
        .start_paged_selection(
            FixedPagedResolver::single(plan),
            &["selected", "not-seen"],
        )
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
