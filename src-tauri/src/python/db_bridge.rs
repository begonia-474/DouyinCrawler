//! 数据库桥接模块 — Rust 侧注入
//!
//! 在应用启动时（`register_db_bridge()`），将 3 个 PyO3 闭包注入到 Python 的 `core.db_bridge` 模块：
//! - `_save_video_info`: JSON 中转 → `VideoInfo` → `db.save_video()`
//! - `_save_user_info`: JSON 中转 → `UserInfo` → `db.save_user()`
//! - `_has_user`: 查询 `db.get_user_by_sec_uid()` → 返回 bool
//!
//! 架构边界：Python 不直接执行 SQL。所有 DB 写入最终由本模块的闭包通过 rusqlite 执行。
//! live_records 生命周期由 Rust TaskApplicationService 独占，不再向 Python 注入写入口。
//!
//! 注意：任务生命周期（_create_task, _update_task_status, _create_task_item, _update_task_item_status）
//! 已迁移到 Rust TaskApplicationService，不再通过 Python 桥接注册。

use log::{info, warn};
use pyo3::prelude::*;
use serde_json::Value;
use tauri::{AppHandle, Manager};

/// 递归将 JSON 中的 bool 值转为 int（Python True/False → 1/0）
///
/// Python 的 bool 是 int 子类，json.dumps 输出 true/false，
/// 但 Rust 的 i32/i64 无法直接反序列化 JSON bool。
/// 此函数在 serde_json::from_value 前统一处理。
pub(crate) fn bool_to_int(v: &mut Value) {
    match v {
        Value::Bool(b) => {
            *v = Value::Number(serde_json::Number::from(if *b { 1i64 } else { 0i64 }))
        }
        Value::Array(arr) => arr.iter_mut().for_each(bool_to_int),
        Value::Object(map) => map.values_mut().for_each(bool_to_int),
        _ => {}
    }
}

/// 注册数据库方法到 Python core.db_bridge 模块
pub fn register_db_bridge(app_handle: &AppHandle) {
    Python::with_gil(|py| {
        let db_bridge = match py.import_bound("core.db_bridge") {
            Ok(m) => m,
            Err(e) => {
                warn!("[db_bridge] 导入 core.db_bridge 失败: {}", e);
                return;
            }
        };

        // 注册 save_video_info
        let h2 = app_handle.clone();
        let save_video_fn = pyo3::types::PyCFunction::new_closure_bound(
            py,
            None,
            None,
            move |args: &Bound<'_, pyo3::types::PyTuple>,
                  _kwargs: Option<&Bound<'_, pyo3::types::PyDict>>|
                  -> PyResult<()> {
                let py = args.py();
                let data = args.get_item(0)?;
                let state = h2.state::<crate::state::AppState>();

                // 将 Python dict 转为 JSON，修复 bool→int，再反序列化为 VideoInfo
                let mut json_value = super::bridge::py_to_json_value(&data)?;
                bool_to_int(&mut json_value);
                let video_info: crate::db::VideoInfo =
                    serde_json::from_value(json_value).map_err(|e| {
                        pyo3::exceptions::PyValueError::new_err(format!(
                            "反序列化 VideoInfo 失败: {}",
                            e
                        ))
                    })?;

                // 释放 GIL 后再锁 DB，避免 GIL+mutex 死锁
                match py.allow_threads(|| state.db.save_video(&video_info)) {
                    Ok(_) => {
                        info!(
                            "[db_bridge] save_video_info 成功: aweme_id={}",
                            video_info.aweme_id
                        );
                    }
                    Err(e) => {
                        return Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                            "保存视频信息失败: {}",
                            e
                        )));
                    }
                }

                Ok(())
            },
        )
        .expect("创建 save_video_info 闭包失败");

        // 注册 save_user_info
        let h3 = app_handle.clone();
        let save_user_fn = pyo3::types::PyCFunction::new_closure_bound(
            py,
            None,
            None,
            move |args: &Bound<'_, pyo3::types::PyTuple>,
                  _kwargs: Option<&Bound<'_, pyo3::types::PyDict>>|
                  -> PyResult<()> {
                let py = args.py();
                let data = args.get_item(0)?;
                let state = h3.state::<crate::state::AppState>();

                let mut json_value = super::bridge::py_to_json_value(&data)?;
                bool_to_int(&mut json_value);
                let user_info: crate::db::UserInfo =
                    serde_json::from_value(json_value).map_err(|e| {
                        pyo3::exceptions::PyValueError::new_err(format!(
                            "反序列化 UserInfo 失败: {}",
                            e
                        ))
                    })?;

                // 释放 GIL 后再锁 DB，避免 GIL+mutex 死锁
                match py.allow_threads(|| state.db.save_user(&user_info)) {
                    Ok(_) => {
                        info!(
                            "[db_bridge] save_user_info 成功: sec_user_id={}",
                            user_info.sec_user_id
                        );
                    }
                    Err(e) => {
                        return Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                            "保存用户信息失败: {}",
                            e
                        )));
                    }
                }

                Ok(())
            },
        )
        .expect("创建 save_user_info 闭包失败");

        // 注册 has_user（查询用户是否已存在）
        let h5 = app_handle.clone();
        let has_user_fn = pyo3::types::PyCFunction::new_closure_bound(
            py,
            None,
            None,
            move |args: &Bound<'_, pyo3::types::PyTuple>,
                  _kwargs: Option<&Bound<'_, pyo3::types::PyDict>>|
                  -> PyResult<bool> {
                let py = args.py();
                let sec_user_id: String = args.get_item(0)?.extract()?;
                let state = h5.state::<crate::state::AppState>();

                // 释放 GIL 后再锁 DB，避免 GIL+mutex 死锁
                match py.allow_threads(|| state.db.get_user_by_sec_uid(&sec_user_id)) {
                    Ok(Some(_)) => Ok(true),
                    Ok(None) => Ok(false),
                    Err(e) => {
                        warn!("[db_bridge] has_user 查询失败: {}", e);
                        Ok(false)
                    }
                }
            },
        )
        .expect("创建 has_user 闭包失败");

        // 注入到 db_bridge 模块
        if let Err(e) = db_bridge.setattr("_save_video_info", save_video_fn) {
            warn!("[db_bridge] 注入 save_video_info 失败: {}", e);
        }
        if let Err(e) = db_bridge.setattr("_save_user_info", save_user_fn) {
            warn!("[db_bridge] 注入 save_user_info 失败: {}", e);
        }
        if let Err(e) = db_bridge.setattr("_has_user", has_user_fn) {
            warn!("[db_bridge] 注入 has_user 失败: {}", e);
        }

        info!("[db_bridge] 视频/用户数据库桥已注入；直播生命周期仅由 Rust 管理");
    });
}
