//! Tauri commands — 任务系统（Rust-owned）
//!
//! 新的任务命令通过 TaskApplicationService 路由，
//! Rust 拥有任务生命周期和 DB 写入。
//!
//! Phase 3.1: mode=one 走新路径，其他 mode 暂时回退到 Python。

use serde_json::Value;
use tauri::State;

use crate::db::Database;
use crate::tasks::service::TaskApplicationService;
use crate::tasks::{DownloadMode, DownloadRequest};

/// 统一下载入口（Rust-owned）
///
/// mode=one 通过 TaskApplicationService 走 Rust 路径，
/// 其他 mode 暂时回退到 Python（Phase 5 逐步迁移）。
#[tauri::command(rename_all = "snake_case")]
pub async fn start_download(
    db: State<'_, Database>,
    mode: String,
    url: String,
) -> Result<Value, String> {
    let download_mode = DownloadMode::from_str(&mode)
        .ok_or_else(|| format!("未知的下载模式: {}", mode))?;

    match download_mode {
        DownloadMode::One => {
            // Rust-owned 路径：通过 TaskApplicationService
            let service = TaskApplicationService::new(&db);
            let request = DownloadRequest {
                mode: download_mode,
                url,
            };
            let task_id = service.start_download(request)?;
            Ok(serde_json::json!({
                "success": true,
                "task_id": task_id,
            }))
        }
        DownloadMode::Music => {
            // Phase 5.1: music 迁移到 Rust-owned 路径
            let service = TaskApplicationService::new(&db);
            let task_id = service.start_music_download(&url)?;
            Ok(serde_json::json!({
                "success": true,
                "task_id": task_id,
            }))
        }
        DownloadMode::Post | DownloadMode::Like | DownloadMode::Mix | DownloadMode::Collects => {
            // Phase 5.2: batch modes 迁移到 Rust-owned 路径
            // 注意：当前实现是同步的，会阻塞直到下载完成
            // TODO: 改为异步执行，立即返回 task_id
            let service = TaskApplicationService::new(&db);
            let task_id = service.start_batch_download_mode(download_mode, &url)?;
            Ok(serde_json::json!({
                "success": true,
                "task_id": task_id,
            }))
        }
        _ => {
            // live 暂时回退到 Python
            let result = crate::python::handler::start_download(&mode, &url)
                .map_err(|e| format!("Python 调用失败: {}", e))?;
            Ok(result)
        }
    }
}
