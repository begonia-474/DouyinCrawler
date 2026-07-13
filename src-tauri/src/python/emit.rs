//! Tauri 事件格式化模块
//!
//! 提供任务事件 payload 格式化工具函数。
//! Python 广播已删除；所有事件由 Rust task_service 直接发射。

use serde_json::Value;

/// 将任务状态事件格式化为 TaskEvent 兼容的 JSON payload
///
/// 统一规则：
/// - task_type 始终为 "typed"
/// - data 包含 TaskEvent 的所有字段（event_type, task_id, mode, url, patch.*）
fn format_event_payload(
    task_id: &str,
    _python_task_type: &str,
    data: &Value,
) -> Value {
    let event_type_str = data
        .get("event_type")
        .and_then(|v| v.as_str())
        .unwrap_or("progress");
    let event_type = match event_type_str {
        "started" => "started",
        "finished" => "completed",
        "progress" => "progress",
        _ => "progress",
    };

    let status_str = data
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("running");
    let mode_str = data
        .get("mode")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let url = data
        .get("url")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let mut patch = serde_json::Map::new();
    patch.insert(
        "task_id".to_string(),
        Value::String(task_id.to_string()),
    );
    patch.insert(
        "status".to_string(),
        Value::String(status_str.to_string()),
    );

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

    for field in &[
        "title",
        "nickname",
        "room_id",
        "web_rid",
        "file",
        "file_size",
        "duration_sec",
        "started_at",
        "ended_at",
        "cover_url",
        "type",
    ] {
        if let Some(val) = data.get(*field) {
            patch.insert(field.to_string(), val.clone());
        }
    }

    let mut event_data = serde_json::Map::new();
    event_data.insert(
        "event_type".to_string(),
        Value::String(event_type.to_string()),
    );
    event_data.insert(
        "task_id".to_string(),
        Value::String(task_id.to_string()),
    );

    if let Some(mode) = &mode_str {
        event_data.insert("mode".to_string(), Value::String(mode.clone()));
    }
    if let Some(url) = &url {
        event_data.insert("url".to_string(), Value::String(url.clone()));
    }

    for (k, v) in patch {
        event_data.insert(k, v);
    }

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
            assert_eq!(
                result["task_type"], "typed",
                "task_type 应为 typed 对于输入: {}",
                data
            );
        }
    }
}
