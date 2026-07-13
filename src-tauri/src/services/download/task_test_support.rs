//! Reusable task-level regression harness for download lifecycle tests.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use uuid::Uuid;

use super::contract::{
    SingleDownloadItem, SingleDownloadPlanV1, SingleMediaKind, SingleOutputSpec,
};
use super::task_service::{
    SingleDownloadResolver, SinglePlanFuture, TaskApplicationService, TaskEventSink,
};
use super::{DownloadMode, DownloadRequest, TaskEvent, TaskEventType, TaskStatus};
use crate::config::{AppConfig, ConfigManager};
use crate::db::{Database, DownloadTask, VideoInfo};
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
    task: tokio::task::JoinHandle<()>,
}

impl LocalHttpServer {
    pub(crate) async fn start(behavior: HttpBehavior) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("local HTTP listener should bind");
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            let Ok((mut socket, _)) = listener.accept().await else {
                return;
            };
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
        });

        Self {
            url: format!("http://{address}/media"),
            task,
        }
    }

    pub(crate) fn url(&self) -> &str {
        &self.url
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
        let root =
            std::env::temp_dir().join(format!("douyin-crawler-task-harness-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let database_path = root.join("tasks.sqlite");
        let db = Database::open(&database_path).unwrap();
        let config = AppConfig {
            download_path: root.to_string_lossy().to_string(),
            timeout: 1,
            max_retries: 1,
            music: false,
            cover: false,
            desc: false,
            ..AppConfig::default()
        };
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

fn assert_started_event(harness: &TaskHarness, task_id: &str) {
    let events = harness.events.snapshot();
    let started = events
        .iter()
        .find(|event| event.task_id == task_id && event.event_type == TaskEventType::Started)
        .expect("started event should be captured");
    assert_eq!(started.mode, Some(DownloadMode::One));
    assert_eq!(
        started.url.as_deref(),
        Some("https://fixture.invalid/video")
    );
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
    assert_started_event(&harness, &task_id);
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
