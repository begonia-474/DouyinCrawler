//! Tauri commands — 任务系统（Rust-owned）
//!
//! 新的任务命令通过 TaskApplicationService 路由，
//! Rust 拥有任务生命周期和 DB 写入。
//!
//! Phase 3.1: 所有模式走 Rust-owned 路径，通过 resolve_urls + DownloadEngine。

use serde_json::Value;
use tauri::State;

use crate::state::AppState;
use crate::services::download::task_service::TaskApplicationService;
use crate::services::download::{DownloadMode, DownloadRequest, TaskEvent, TaskPatch, TaskStatus};
use crate::services::download::events;

/// 统一下载入口（Rust-owned）
///
/// 所有模式通过 TaskApplicationService 走 Rust 路径，
/// 使用 resolve_urls 解析下载 URL + DownloadEngine 执行下载。
#[tauri::command(rename_all = "snake_case")]
pub async fn start_download(
    state: State<'_, AppState>,
    mode: String,
    url: String,
    aweme_ids: Option<Vec<String>>,
) -> Result<Value, String> {
    let download_mode = DownloadMode::from_str(&mode)
        .ok_or_else(|| format!("未知的下载模式: {}", mode))?;

    let aweme_ids = aweme_ids.unwrap_or_default();
    let service = TaskApplicationService::new(&state);

    match download_mode {
        DownloadMode::One => {
            let request = DownloadRequest {
                mode: download_mode,
                url,
                aweme_ids,
            };
            let task_id = service.start_download(request).await?;
            Ok(serde_json::json!({
                "success": true,
                "task_id": task_id,
            }))
        }
        DownloadMode::Music => {
            let task_id = service.start_music_download(&url).await?;
            Ok(serde_json::json!({
                "success": true,
                "task_id": task_id,
            }))
        }
        DownloadMode::Post | DownloadMode::Like | DownloadMode::Mix | DownloadMode::Collects => {
            let task_id = service
                .start_batch_download_mode(download_mode, &url, &aweme_ids)
                .await?;
            Ok(serde_json::json!({
                "success": true,
                "task_id": task_id,
            }))
        }
        DownloadMode::Live => {
            let task_id = service.start_live_record(&url).await?;
            Ok(serde_json::json!({
                "success": true,
                "task_id": task_id,
            }))
        }
    }
}

/// 启动 Rust 原生直播录制。
#[tauri::command(rename_all = "snake_case")]
pub async fn start_live_record(
    state: State<'_, AppState>,
    url: String,
) -> Result<Value, String> {
    let service = TaskApplicationService::new(&state);
    let task_id = service.start_live_record(&url).await?;
    Ok(serde_json::json!({
        "success": true,
        "data": { "task_id": task_id },
    }))
}

/// 停止直播录制。停止信号由 Rust 录制循环消费，已写入的文件会正常入库。
#[tauri::command(rename_all = "snake_case")]
pub fn stop_live_record(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<Value, String> {
    if !state.cancel_task(&task_id) {
        return Err("录制任务不存在或已完成".to_string());
    }

    state
        .db
        .update_task_status(&task_id, "stopping", None)
        .map_err(|e| e.to_string())?;
    events::emit_task_event(&TaskEvent::progress(
        TaskPatch::new(&task_id).with_status(TaskStatus::Stopping),
    ));

    Ok(serde_json::json!({
        "success": true,
        "data": { "task_id": task_id },
    }))
}

/// 返回仍在运行的 Rust 直播任务，供页面刷新后恢复停止按钮。
#[tauri::command(rename_all = "snake_case")]
pub fn get_live_status(state: State<'_, AppState>) -> Result<Value, String> {
    let tasks = state
        .db
        .get_tasks(100, 0, None, Some("live".to_string()))
        .map_err(|e| e.to_string())?;
    let mut active = serde_json::Map::new();

    for task in tasks {
        if !matches!(
            task.status.as_str(),
            "starting" | "running" | "recording" | "stopping"
        ) || state.get_cancel_signal(&task.id).is_none()
        {
            continue;
        }
        active.insert(
            task.id.clone(),
            serde_json::json!({
                "task_id": task.id,
                "status": task.status,
                "url": task.url,
                "title": task.title,
                "nickname": task.author_nickname,
                "error": task.error_msg,
            }),
        );
    }

    Ok(serde_json::json!({
        "success": true,
        "data": active,
    }))
}

/// 取消任务
///
/// 设置任务的取消信号，下载引擎会在下一个 chunk 检测到并停止
#[tauri::command(rename_all = "snake_case")]
pub async fn cancel_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<Value, String> {
    let cancelled = state.cancel_task(&task_id);

    if cancelled {
        Ok(serde_json::json!({
            "success": true,
            "message": "取消信号已发送",
        }))
    } else {
        Err("任务不存在或已完成".to_string())
    }
}
