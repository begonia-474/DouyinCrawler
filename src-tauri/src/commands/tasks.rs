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
use crate::services::download::{DownloadMode, DownloadRequest};

/// 统一下载入口（Rust-owned）
///
/// 所有模式通过 TaskApplicationService 走 Rust 路径，
/// 使用 resolve_urls 解析下载 URL + DownloadEngine 执行下载。
#[tauri::command(rename_all = "snake_case")]
pub async fn start_download(
    state: State<'_, AppState>,
    mode: String,
    url: String,
) -> Result<Value, String> {
    let download_mode = DownloadMode::from_str(&mode)
        .ok_or_else(|| format!("未知的下载模式: {}", mode))?;

    let service = TaskApplicationService::new(&state);

    match download_mode {
        DownloadMode::One => {
            let request = DownloadRequest {
                mode: download_mode,
                url,
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
                .start_batch_download_mode(download_mode, &url)
                .await?;
            Ok(serde_json::json!({
                "success": true,
                "task_id": task_id,
            }))
        }
        DownloadMode::Live => {
            // live 当前通过 Python bridge 实现（后续将改为 Rust 原生路径）
            let result = crate::python::handler::start_download(&mode, &url)
                .map_err(|e| format!("Python 调用失败: {}", e))?;
            Ok(result)
        }
    }
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
