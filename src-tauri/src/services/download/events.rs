//! 任务事件发射
//!
//! 通过 Tauri 事件系统将类型化 TaskEvent 推送到前端。
//! 替代当前 Python→emit.rs 的非类型化事件路径。

use std::sync::OnceLock;
use log::{info, warn};
use tauri::{AppHandle, Emitter};

use super::{DownloadMode, TaskEvent, TaskPatch, TaskStatus};

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

/// 初始化事件模块，在应用 setup 时调用
pub fn init(app_handle: &AppHandle) {
    let _ = APP_HANDLE.set(app_handle.clone());
}

/// 发射类型化任务事件到前端
///
/// 事件名: "task-update"
/// payload: { task_id, task_type: "typed", data: TaskEvent }
pub fn emit_task_event(event: &TaskEvent) {
    let app_handle = match APP_HANDLE.get() {
        Some(h) => h,
        None => {
            warn!("[tasks/events] APP_HANDLE 未初始化，无法发射事件");
            return;
        }
    };

    let payload = serde_json::json!({
        "task_id": event.task_id,
        "task_type": "typed",
        "data": event,
    });

    match app_handle.emit("task-update", &payload) {
        Ok(_) => info!(
            "[tasks/events] 事件已发射: type={:?}, task_id={}",
            event.event_type, event.task_id
        ),
        Err(e) => warn!("[tasks/events] 事件发射失败: {}", e),
    }
}

/// 发射任务启动事件
pub fn emit_started(task_id: &str, mode: DownloadMode, url: &str) {
    let event = TaskEvent::started(task_id, mode, url);
    emit_task_event(&event);
}

/// 发射任务进度事件
///
/// 用于实时更新下载进度到前端
pub fn emit_progress(task_id: &str, downloaded: u64, total: u64) {
    let patch = TaskPatch::new(task_id)
        .with_counts(total as i64, downloaded as i64, 0, 0);
    let event = TaskEvent::progress(patch);
    emit_task_event(&event);
}

/// 发射任务完成事件
pub fn emit_finished(patch: TaskPatch) {
    let event = TaskEvent::finished(patch);
    emit_task_event(&event);
}

/// 发射任务错误事件
pub fn emit_error(task_id: &str, error: &str) {
    let patch = TaskPatch::new(task_id)
        .with_status(TaskStatus::Error)
        .with_error(error);
    let event = TaskEvent::finished(patch);
    emit_task_event(&event);
}

/// 发射任务取消事件
pub fn emit_cancelled(task_id: &str) {
    let patch = TaskPatch::new(task_id)
        .with_status(TaskStatus::Cancelled);
    let event = TaskEvent::finished(patch);
    emit_task_event(&event);
}

/// 发射直播终态事件，并保留 mode=live，供前端刷新直播记录查询。
pub fn emit_live_finished(patch: TaskPatch) {
    emit_task_event(&TaskEvent {
        event_type: super::TaskEventType::Finished,
        task_id: patch.task_id.clone(),
        mode: Some(DownloadMode::Live),
        url: None,
        patch,
    });
}

pub fn emit_live_error(task_id: &str, error: &str) {
    emit_live_finished(
        TaskPatch::new(task_id)
            .with_status(TaskStatus::Error)
            .with_error(error),
    );
}

pub fn emit_live_cancelled(task_id: &str) {
    emit_live_finished(
        TaskPatch::new(task_id).with_status(TaskStatus::Cancelled),
    );
}
