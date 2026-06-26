//! 数据库桥接模块 — Rust 侧注入
//!
//! 在应用启动时（`register_db_bridge()`），将 5 个 PyO3 闭包注入到 Python 的 `core.db_bridge` 模块：
//! - `_save_download_record`: 提取字段 → `NewDownloadRecord` → `db.save_download()`
//! - `_save_video_info`: JSON 中转 → `VideoInfo` → `db.save_video()`
//! - `_save_user_info`: JSON 中转 → `UserInfo` → `db.save_user()`
//! - `_save_live_record`: 提取字段 → `NewLiveRecord` → `db.save_live_record()`
//! - `_has_user`: 查询 `db.get_user_by_sec_uid()` → 返回 bool
//!
//! 架构边界：Python 不直接执行 SQL。所有 DB 写入最终由本模块的闭包通过 rusqlite 执行。
//! 前端也有两条直接写入路径（不经 Python）：live_records、music_collection（见 lib.rs Tauri commands）。

use pyo3::prelude::*;
use serde_json::Value;
use tauri::Manager;
use log::{info, warn};

/// 递归将 JSON 中的 bool 值转为 int（Python True/False → 1/0）
///
/// Python 的 bool 是 int 子类，json.dumps 输出 true/false，
/// 但 Rust 的 i32/i64 无法直接反序列化 JSON bool。
/// 此函数在 serde_json::from_value 前统一处理。
fn bool_to_int(v: &mut Value) {
    match v {
        Value::Bool(b) => *v = Value::Number(serde_json::Number::from(if *b { 1i64 } else { 0i64 })),
        Value::Array(arr) => arr.iter_mut().for_each(bool_to_int),
        Value::Object(map) => map.values_mut().for_each(bool_to_int),
        _ => {}
    }
}

/// 注册数据库方法到 Python core.db_bridge 模块
pub fn register_db_bridge() {
    Python::with_gil(|py| {
        let db_bridge = match py.import_bound("core.db_bridge") {
            Ok(m) => m,
            Err(e) => {
                warn!("[db_bridge] 导入 core.db_bridge 失败: {}", e);
                return;
            }
        };

        // 注册 save_download_record
        let save_download_fn = pyo3::types::PyCFunction::new_closure_bound(
            py,
            None,
            None,
            move |args: &Bound<'_, pyo3::types::PyTuple>, _kwargs: Option<&Bound<'_, pyo3::types::PyDict>>| -> PyResult<()> {
                let data = args.get_item(0)?;
                let app_handle = crate::APP_HANDLE.get()
                    .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("AppHandle 未初始化"))?;

                // 从 Python dict 提取字段
                let aweme_id: Option<String> = data.get_item("aweme_id").ok().and_then(|v| v.extract().ok());
                let download_type: String = data.get_item("download_type")?.extract()?;
                let title: Option<String> = data.get_item("title").ok().and_then(|v| v.extract().ok());
                let author_nickname: Option<String> = data.get_item("author_nickname").ok().and_then(|v| v.extract().ok());
                let author_sec_uid: Option<String> = data.get_item("author_sec_uid").ok().and_then(|v| v.extract().ok());
                let file_path: Option<String> = data.get_item("file_path").ok().and_then(|v| v.extract().ok());
                let file_size: i64 = data.get_item("file_size").ok().and_then(|v| v.extract().ok()).unwrap_or(0);
                let cover_url: Option<String> = data.get_item("cover_url").ok().and_then(|v| v.extract().ok());
                let status: String = data.get_item("status")?.extract()?;
                let error_msg: Option<String> = data.get_item("error_msg").ok().and_then(|v| v.extract().ok());

                // 获取数据库实例
                let db = app_handle.state::<crate::db::Database>();
                let record = crate::db::NewDownloadRecord {
                    aweme_id,
                    download_type,
                    title,
                    author_nickname,
                    author_sec_uid,
                    file_path,
                    file_size,
                    cover_url,
                    status,
                    error_msg,
                };

                match db.save_download(&record) {
                    Ok(id) => {
                        info!("[db_bridge] save_download_record 成功: id={}", id);
                    }
                    Err(e) => {
                        return Err(pyo3::exceptions::PyRuntimeError::new_err(format!("保存下载记录失败: {}", e)));
                    }
                }

                Ok(())
            },
        ).expect("创建 save_download_record 闭包失败");

        // 注册 save_video_info
        let save_video_fn = pyo3::types::PyCFunction::new_closure_bound(
            py,
            None,
            None,
            move |args: &Bound<'_, pyo3::types::PyTuple>, _kwargs: Option<&Bound<'_, pyo3::types::PyDict>>| -> PyResult<()> {
                let data = args.get_item(0)?;
                let app_handle = crate::APP_HANDLE.get()
                    .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("AppHandle 未初始化"))?;

                // 将 Python dict 转为 JSON，修复 bool→int，再反序列化为 VideoInfo
                let mut json_value = super::bridge::py_to_json_value(&data)?;
                bool_to_int(&mut json_value);
                let video_info: crate::db::VideoInfo = serde_json::from_value(json_value)
                    .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("反序列化 VideoInfo 失败: {}", e)))?;

                let db = app_handle.state::<crate::db::Database>();
                match db.save_video(&video_info) {
                    Ok(_) => {
                        info!("[db_bridge] save_video_info 成功: aweme_id={}", video_info.aweme_id);
                    }
                    Err(e) => {
                        return Err(pyo3::exceptions::PyRuntimeError::new_err(format!("保存视频信息失败: {}", e)));
                    }
                }

                Ok(())
            },
        ).expect("创建 save_video_info 闭包失败");

        // 注册 save_user_info
        let save_user_fn = pyo3::types::PyCFunction::new_closure_bound(
            py,
            None,
            None,
            move |args: &Bound<'_, pyo3::types::PyTuple>, _kwargs: Option<&Bound<'_, pyo3::types::PyDict>>| -> PyResult<()> {
                let data = args.get_item(0)?;
                let app_handle = crate::APP_HANDLE.get()
                    .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("AppHandle 未初始化"))?;

                let mut json_value = super::bridge::py_to_json_value(&data)?;
                bool_to_int(&mut json_value);
                let user_info: crate::db::UserInfo = serde_json::from_value(json_value)
                    .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("反序列化 UserInfo 失败: {}", e)))?;

                let db = app_handle.state::<crate::db::Database>();
                match db.save_user(&user_info) {
                    Ok(_) => {
                        info!("[db_bridge] save_user_info 成功: sec_user_id={}", user_info.sec_user_id);
                    }
                    Err(e) => {
                        return Err(pyo3::exceptions::PyRuntimeError::new_err(format!("保存用户信息失败: {}", e)));
                    }
                }

                Ok(())
            },
        ).expect("创建 save_user_info 闭包失败");

        // 注册 save_live_record
        let save_live_record_fn = pyo3::types::PyCFunction::new_closure_bound(
            py,
            None,
            None,
            move |args: &Bound<'_, pyo3::types::PyTuple>, _kwargs: Option<&Bound<'_, pyo3::types::PyDict>>| -> PyResult<()> {
                let data = args.get_item(0)?;
                let app_handle = crate::APP_HANDLE.get()
                    .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("AppHandle 未初始化"))?;

                let room_id: Option<String> = data.get_item("room_id").ok().and_then(|v| v.extract().ok());
                let web_rid: Option<String> = data.get_item("web_rid").ok().and_then(|v| v.extract().ok());
                let title: Option<String> = data.get_item("title").ok().and_then(|v| v.extract().ok());
                let nickname: Option<String> = data.get_item("nickname").ok().and_then(|v| v.extract().ok());
                let sec_user_id: Option<String> = data.get_item("sec_user_id").ok().and_then(|v| v.extract().ok());
                let file_path: Option<String> = data.get_item("file_path").ok().and_then(|v| v.extract().ok());
                let file_size: i64 = data.get_item("file_size").ok().and_then(|v| v.extract().ok()).unwrap_or(0);
                let duration_sec: i64 = data.get_item("duration_sec").ok().and_then(|v| v.extract().ok()).unwrap_or(0);
                let status: String = data.get_item("status").ok().and_then(|v| v.extract().ok()).unwrap_or_else(|| "completed".to_string());
                let started_at: Option<i64> = data.get_item("started_at").ok().and_then(|v| v.extract().ok());
                let ended_at: Option<i64> = data.get_item("ended_at").ok().and_then(|v| v.extract().ok());
                let cover_url: Option<String> = data.get_item("cover_url").ok().and_then(|v| v.extract().ok());

                let db = app_handle.state::<crate::db::Database>();
                let record = crate::db::NewLiveRecord {
                    room_id,
                    web_rid,
                    title,
                    nickname,
                    sec_user_id,
                    file_path,
                    file_size,
                    duration_sec,
                    status,
                    started_at,
                    ended_at,
                    cover_url,
                };

                match db.save_live_record(&record) {
                    Ok(id) => {
                        info!("[db_bridge] save_live_record 成功: id={}", id);
                    }
                    Err(e) => {
                        return Err(pyo3::exceptions::PyRuntimeError::new_err(format!("保存直播记录失败: {}", e)));
                    }
                }

                Ok(())
            },
        ).expect("创建 save_live_record 闭包失败");

        // 注册 has_user（查询用户是否已存在）
        let has_user_fn = pyo3::types::PyCFunction::new_closure_bound(
            py,
            None,
            None,
            move |args: &Bound<'_, pyo3::types::PyTuple>, _kwargs: Option<&Bound<'_, pyo3::types::PyDict>>| -> PyResult<bool> {
                let sec_user_id: String = args.get_item(0)?.extract()?;
                let app_handle = crate::APP_HANDLE.get()
                    .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("AppHandle 未初始化"))?;

                let db = app_handle.state::<crate::db::Database>();
                match db.get_user_by_sec_uid(&sec_user_id) {
                    Ok(Some(_)) => Ok(true),
                    Ok(None) => Ok(false),
                    Err(e) => {
                        warn!("[db_bridge] has_user 查询失败: {}", e);
                        Ok(false)
                    }
                }
            },
        ).expect("创建 has_user 闭包失败");

        // 注入到 db_bridge 模块
        if let Err(e) = db_bridge.setattr("_save_download_record", save_download_fn) {
            warn!("[db_bridge] 注入 save_download_record 失败: {}", e);
        }
        if let Err(e) = db_bridge.setattr("_save_video_info", save_video_fn) {
            warn!("[db_bridge] 注入 save_video_info 失败: {}", e);
        }
        if let Err(e) = db_bridge.setattr("_save_user_info", save_user_fn) {
            warn!("[db_bridge] 注入 save_user_info 失败: {}", e);
        }
        if let Err(e) = db_bridge.setattr("_save_live_record", save_live_record_fn) {
            warn!("[db_bridge] 注入 save_live_record 失败: {}", e);
        }
        if let Err(e) = db_bridge.setattr("_has_user", has_user_fn) {
            warn!("[db_bridge] 注入 has_user 失败: {}", e);
        }

        // === 任务管理闭包 ===

        // 注册 create_task
        let create_task_fn = pyo3::types::PyCFunction::new_closure_bound(
            py,
            None,
            None,
            move |args: &Bound<'_, pyo3::types::PyTuple>, _kwargs: Option<&Bound<'_, pyo3::types::PyDict>>| -> PyResult<()> {
                let data = args.get_item(0)?;
                let app_handle = crate::APP_HANDLE.get()
                    .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("AppHandle 未初始化"))?;

                let id: String = data.get_item("id")?.extract()?;
                let mode: String = data.get_item("mode")?.extract()?;
                let url: String = data.get_item("url")?.extract()?;
                let title: Option<String> = data.get_item("title").ok().and_then(|v| v.extract().ok());
                let author_nickname: Option<String> = data.get_item("author_nickname").ok().and_then(|v| v.extract().ok());

                let db = app_handle.state::<crate::db::Database>();
                let task = crate::db::NewDownloadTask { id, mode, url, title, author_nickname };
                db.create_task(&task)
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("创建任务失败: {}", e)))?;
                Ok(())
            },
        ).expect("创建 create_task 闭包失败");

        // 注册 update_task_status
        let update_task_status_fn = pyo3::types::PyCFunction::new_closure_bound(
            py,
            None,
            None,
            move |args: &Bound<'_, pyo3::types::PyTuple>, _kwargs: Option<&Bound<'_, pyo3::types::PyDict>>| -> PyResult<()> {
                let task_id: String = args.get_item(0)?.extract()?;
                let status: String = args.get_item(1)?.extract()?;
                let error_msg: Option<String> = args.get_item(2).ok().and_then(|v| v.extract().ok());

                let app_handle = crate::APP_HANDLE.get()
                    .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("AppHandle 未初始化"))?;

                let db = app_handle.state::<crate::db::Database>();
                db.update_task_status(&task_id, &status, error_msg.as_deref())
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("更新任务状态失败: {}", e)))?;
                Ok(())
            },
        ).expect("创建 update_task_status 闭包失败");

        // 注册 create_task_item
        let create_task_item_fn = pyo3::types::PyCFunction::new_closure_bound(
            py,
            None,
            None,
            move |args: &Bound<'_, pyo3::types::PyTuple>, _kwargs: Option<&Bound<'_, pyo3::types::PyDict>>| -> PyResult<()> {
                let data = args.get_item(0)?;
                let app_handle = crate::APP_HANDLE.get()
                    .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("AppHandle 未初始化"))?;

                let task_id: String = data.get_item("task_id")?.extract()?;
                let aweme_id: Option<String> = data.get_item("aweme_id").ok().and_then(|v| v.extract().ok());
                let title: Option<String> = data.get_item("title").ok().and_then(|v| v.extract().ok());
                let author_nickname: Option<String> = data.get_item("author_nickname").ok().and_then(|v| v.extract().ok());
                let cover_url: Option<String> = data.get_item("cover_url").ok().and_then(|v| v.extract().ok());

                let db = app_handle.state::<crate::db::Database>();
                let item = crate::db::NewTaskItem { task_id, aweme_id, title, author_nickname, cover_url };
                db.create_task_item(&item)
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("创建子项失败: {}", e)))?;
                Ok(())
            },
        ).expect("创建 create_task_item 闭包失败");

        // 注册 update_task_item_status
        let update_task_item_status_fn = pyo3::types::PyCFunction::new_closure_bound(
            py,
            None,
            None,
            move |args: &Bound<'_, pyo3::types::PyTuple>, _kwargs: Option<&Bound<'_, pyo3::types::PyDict>>| -> PyResult<()> {
                let task_id: String = args.get_item(0)?.extract()?;
                let aweme_id: String = args.get_item(1)?.extract()?;
                let status: String = args.get_item(2)?.extract()?;
                let file_path: Option<String> = args.get_item(3).ok().and_then(|v| v.extract().ok());
                let file_size: i64 = args.get_item(4).ok().and_then(|v| v.extract().ok()).unwrap_or(0);
                let error_msg: Option<String> = args.get_item(5).ok().and_then(|v| v.extract().ok());

                let app_handle = crate::APP_HANDLE.get()
                    .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("AppHandle 未初始化"))?;

                let db = app_handle.state::<crate::db::Database>();
                db.update_task_item_status(&task_id, &aweme_id, &status, file_path.as_deref(), file_size, error_msg.as_deref())
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("更新子项状态失败: {}", e)))?;
                // 自动刷新任务聚合计数
                db.update_task_counts(&task_id)
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("更新任务计数失败: {}", e)))?;
                Ok(())
            },
        ).expect("创建 update_task_item_status 闭包失败");

        // 注入任务管理闭包
        if let Err(e) = db_bridge.setattr("_create_task", create_task_fn) {
            warn!("[db_bridge] 注入 create_task 失败: {}", e);
        }
        if let Err(e) = db_bridge.setattr("_update_task_status", update_task_status_fn) {
            warn!("[db_bridge] 注入 update_task_status 失败: {}", e);
        }
        if let Err(e) = db_bridge.setattr("_create_task_item", create_task_item_fn) {
            warn!("[db_bridge] 注入 create_task_item 失败: {}", e);
        }
        if let Err(e) = db_bridge.setattr("_update_task_item_status", update_task_item_status_fn) {
            warn!("[db_bridge] 注入 update_task_item_status 失败: {}", e);
        }

        info!("[db_bridge] 数据库桥接方法已注入");
    });
}
