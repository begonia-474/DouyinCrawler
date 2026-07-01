//! Tauri 事件发射模块
//!
//! 供 Python 通过 PyO3 调用，将任务状态推送到前端。
//! 统一事件格式：所有事件使用 task_type: "typed" + TaskEvent 兼容结构。

use pyo3::prelude::*;
use tauri::{AppHandle, Emitter, Manager};
use log::{info, warn};

/// 注册 emit_task_update 到 Python core.tauri_bridge 模块
pub fn register_app_handle(app_handle: &AppHandle) {
    let handle = app_handle.clone();
    Python::with_gil(|py| {
        let tauri_bridge = match py.import_bound("core.tauri_bridge") {
            Ok(m) => m,
            Err(e) => {
                warn!("[emit] 导入 core.tauri_bridge 失败: {}", e);
                return;
            }
        };

        // 创建一个 Python 可调用的闭包，包装 Rust 的 emit_task_update
        // 签名: _emit_func(task_id: str, task_type: str, data: dict) -> None
        let emit_fn = pyo3::types::PyCFunction::new_closure_bound(
            py,
            None,
            None,
            move |args: &Bound<'_, pyo3::types::PyTuple>, _kwargs: Option<&Bound<'_, pyo3::types::PyDict>>| -> PyResult<()> {
                let py = args.py();
                let task_id: String = args.get_item(0)?.extract()?;
                let task_type: String = args.get_item(1)?.extract()?;
                let data = args.get_item(2)?;

                info!("[emit] Python 调用 emit: task_id={}, task_type={}", task_id, task_type);

                // 将 Python dict 转为 JSON
                let json_value = super::bridge::py_to_json_value(&data)?;

                // 统一格式：所有事件使用 task_type: "typed"
                // 将 Python 的 batch/live 事件格式化为 TaskEvent 兼容结构
                let payload = format_event_payload(&task_id, &task_type, &json_value);

                // 以下操作不需要 GIL：Tauri emit + DB 写入
                py.allow_threads(|| {
                    info!("[emit] 发送 Tauri 事件: task-update, payload={}", payload);
                    handle.emit("task-update", &payload)
                        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("事件发射失败: {}", e)))?;
                    info!("[emit] Tauri 事件发送成功");

                    // 如果是 live completed 事件，持久化到 DB
                    if task_type == "live" {
                        if let Some(status) = json_value.get("status").and_then(|v| v.as_str()) {
                            if status == "completed" {
                                let state = handle.state::<crate::state::AppState>();
                                let record = crate::db::NewLiveRecord {
                                    room_id: json_value.get("room_id").and_then(|v| v.as_str()).map(String::from),
                                    web_rid: json_value.get("web_rid").and_then(|v| v.as_str()).map(String::from),
                                    title: json_value.get("title").and_then(|v| v.as_str()).map(String::from),
                                    nickname: json_value.get("nickname").and_then(|v| v.as_str()).map(String::from),
                                    sec_user_id: None,
                                    file_path: json_value.get("file").and_then(|v| v.as_str()).map(String::from),
                                    file_size: json_value.get("file_size").and_then(|v| v.as_i64()).unwrap_or(0),
                                    duration_sec: json_value.get("duration_sec").and_then(|v| v.as_i64()).unwrap_or(0),
                                    status: "completed".to_string(),
                                    started_at: json_value.get("started_at").and_then(|v| v.as_i64()),
                                    ended_at: json_value.get("ended_at").and_then(|v| v.as_i64()),
                                    cover_url: json_value.get("cover_url").and_then(|v| v.as_str()).map(String::from),
                                };
                                if let Err(e) = state.db.save_live_record(&record) {
                                    warn!("[emit] 保存 live 记录失败: {}", e);
                                } else {
                                    info!("[emit] live 记录已保存, task_id={}", task_id);
                                }
                            }
                        }
                    }

                    Ok::<(), pyo3::PyErr>(())
                })?;

                Ok(())
            },
        ).expect("创建 emit 闭包失败");

        // 方式1: 通过 setattr 设置模块属性
        if let Err(e) = tauri_bridge.setattr("_emit_func", emit_fn.clone()) {
            warn!("[emit] setattr 失败: {}", e);
        }

        // 方式2: 调用 Python 的 set_emit_func 函数（更可靠）
        if let Err(e) = tauri_bridge.call_method1("set_emit_func", (emit_fn,)) {
            warn!("[emit] set_emit_func 调用失败: {}", e);
        } else {
            info!("[emit] emit_task_update 已通过 set_emit_func 注入");
        }

        // 验证注入结果
        match tauri_bridge.getattr("_emit_func") {
            Ok(func) => {
                let is_none: bool = func.is_none();
                info!("[emit] 验证: _emit_func is_none={}", is_none);
            }
            Err(e) => {
                warn!("[emit] 验证失败: {}", e);
            }
        }
    });
}

/// 将 Python 事件格式化为 TaskEvent 兼容的 JSON payload
///
/// 统一规则：
/// - task_type 始终为 "typed"
/// - data 包含 TaskEvent 的所有字段（event_type, task_id, mode, url, patch.*）
/// - Python 特有字段（title, nickname 等）放在 data 顶层，前端 UnifiedTask 会处理
fn format_event_payload(task_id: &str, _python_task_type: &str, data: &serde_json::Value) -> serde_json::Value {
    // 从 Python data 中提取字段
    let event_type_str = data.get("event_type").and_then(|v| v.as_str()).unwrap_or("progress");
    let event_type = match event_type_str {
        "started" => "started",
        "finished" => "completed",
        "progress" => "progress",
        _ => "progress",
    };

    let status_str = data.get("status").and_then(|v| v.as_str()).unwrap_or("running");
    let mode_str = data.get("mode").and_then(|v| v.as_str()).map(|s| s.to_string());
    let url = data.get("url").and_then(|v| v.as_str()).map(|s| s.to_string());

    // 构建 patch 字段（TaskPatch 兼容）
    let mut patch = serde_json::Map::new();
    patch.insert("task_id".to_string(), serde_json::Value::String(task_id.to_string()));
    patch.insert("status".to_string(), serde_json::Value::String(status_str.to_string()));

    // 复制可选字段
    for field in &["total", "completed", "failed", "skipped"] {
        if let Some(val) = data.get(*field) {
            patch.insert(field.to_string(), val.clone());
        }
    }
    for field in &["current_item", "error_msg", "error"] {
        if let Some(val) = data.get(*field) {
            patch.insert(field.to_string(), val.clone());
        }
    }

    // Python 特有字段（live 事件等）也复制到 patch 中
    for field in &["title", "nickname", "room_id", "web_rid", "file", "file_size", "duration_sec", "started_at", "ended_at", "cover_url", "type"] {
        if let Some(val) = data.get(*field) {
            patch.insert(field.to_string(), val.clone());
        }
    }

    // 构建完整的 TaskEvent 兼容 payload
    let mut event_data = serde_json::Map::new();
    event_data.insert("event_type".to_string(), serde_json::Value::String(event_type.to_string()));
    event_data.insert("task_id".to_string(), serde_json::Value::String(task_id.to_string()));

    if let Some(mode) = &mode_str {
        event_data.insert("mode".to_string(), serde_json::Value::String(mode.clone()));
    }
    if let Some(url) = &url {
        event_data.insert("url".to_string(), serde_json::Value::String(url.clone()));
    }

    // 展开 patch 字段到 data 顶层（与 Rust TaskEvent 的 serde(flatten) 一致）
    for (k, v) in patch {
        event_data.insert(k, v);
    }

    // 统一使用 task_type: "typed"
    serde_json::json!({
        "task_id": task_id,
        "task_type": "typed",
        "data": event_data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_progress_event() {
        let data = json!({
            "event_type": "progress",
            "status": "running",
            "mode": "post",
            "url": "https://example.com",
            "total": 10,
            "completed": 5,
            "failed": 1,
            "skipped": 0,
            "current_item": "video_3",
        });
        let result = format_event_payload("task-123", "batch", &data);
        assert_eq!(result["task_type"], "typed");
        assert_eq!(result["task_id"], "task-123");
        let d = &result["data"];
        assert_eq!(d["event_type"], "progress");
        assert_eq!(d["status"], "running");
        assert_eq!(d["mode"], "post");
        assert_eq!(d["total"], 10);
        assert_eq!(d["completed"], 5);
        assert_eq!(d["failed"], 1);
        assert_eq!(d["current_item"], "video_3");
    }

    #[test]
    fn test_live_completed_event() {
        let data = json!({
            "event_type": "finished",
            "status": "completed",
            "mode": "live",
            "title": "主播名称",
            "nickname": "用户名",
            "file": "/path/to/file.mp4",
            "file_size": 12345,
            "duration_sec": 3600,
            "room_id": "123456",
            "type": "live_record",
        });
        let result = format_event_payload("task-456", "live", &data);
        assert_eq!(result["task_type"], "typed");
        let d = &result["data"];
        assert_eq!(d["event_type"], "completed");
        assert_eq!(d["title"], "主播名称");
        assert_eq!(d["file"], "/path/to/file.mp4");
        assert_eq!(d["file_size"], 12345);
    }

    #[test]
    fn test_unknown_event_type_fallback() {
        let data = json!({});
        let result = format_event_payload("task-789", "unknown", &data);
        assert_eq!(result["task_type"], "typed");
        let d = &result["data"];
        assert_eq!(d["event_type"], "progress");
        assert_eq!(d["status"], "running");
    }

    #[test]
    fn test_error_fields() {
        let data = json!({
            "event_type": "progress",
            "status": "error",
            "error": "Connection failed",
            "error_msg": "无法连接到服务器",
        });
        let result = format_event_payload("task-err", "batch", &data);
        let d = &result["data"];
        assert_eq!(d["error"], "Connection failed");
        assert_eq!(d["error_msg"], "无法连接到服务器");
    }

    #[test]
    fn test_live_started_fields_forwarded() {
        let data = json!({
            "event_type": "started",
            "status": "recording",
            "mode": "live",
            "room_id": "123456",
            "web_rid": "abc123",
            "cover_url": "https://example.com/cover.jpg",
            "started_at": "2026-07-01T00:00:00Z",
        });
        let result = format_event_payload("task-live", "live", &data);
        let d = &result["data"];
        assert_eq!(d["event_type"], "started");
        assert_eq!(d["room_id"], "123456");
        assert_eq!(d["web_rid"], "abc123");
        assert!(d.get("cover_url").is_some());
    }

    #[test]
    fn test_task_type_always_typed() {
        let cases = vec![
            json!({"event_type": "progress", "status": "running"}),
            json!({"event_type": "finished", "status": "completed"}),
            json!({"event_type": "started", "status": "starting"}),
            json!({}),
            json!({"event_type": "unknown_event_type"}),
        ];
        for data in cases {
            let result = format_event_payload("tid", "any", &data);
            assert_eq!(result["task_type"], "typed", "task_type 应为 typed 对于输入: {}", data);
        }
    }
}
