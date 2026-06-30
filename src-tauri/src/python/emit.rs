//! Tauri 事件发射模块
//!
//! 供 Python 通过 PyO3 调用，将任务状态推送到前端

use pyo3::prelude::*;
use tauri::{AppHandle, Emitter};
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
        let emit_fn = pyo3::types::PyCFunction::new_closure_bound(
            py,
            None,
            None,
            move |args: &Bound<'_, pyo3::types::PyTuple>, _kwargs: Option<&Bound<'_, pyo3::types::PyDict>>| -> PyResult<()> {
                let task_id: String = args.get_item(0)?.extract()?;
                let task_type: String = args.get_item(1)?.extract()?;
                let data = args.get_item(2)?;

                info!("[emit] Python 调用 emit: task_id={}, task_type={}", task_id, task_type);

                // 将 Python dict 转为 JSON（使用统一桥接辅助函数）
                let json_value = super::bridge::py_to_json_value(&data)?;

                let payload = serde_json::json!({
                    "task_id": task_id,
                    "task_type": task_type,
                    "data": json_value,
                });

                info!("[emit] 发送 Tauri 事件: task-update, payload={}", payload);
                handle.emit("task-update", &payload)
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("事件发射失败: {}", e)))?;
                info!("[emit] Tauri 事件发送成功");

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
