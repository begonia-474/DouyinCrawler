//! Python 调用运行时统一入口
//!
//! 所有 Tauri async command 通过 `run_python_blocking` 调用 Python，
//! 内部使用 `spawn_blocking` 将 GIL 阻塞移出 tokio 运行时线程。

use pyo3::prelude::*;
use serde_json::Value;
use log::info;

/// 在 spawn_blocking 线程中执行 Python 调用，避免阻塞 tokio 运行时。
///
/// 用法：
/// ```rust
/// let result = run_python_blocking("parse_video", || {
///     crate::python::parse_video(&url)
/// }).await?;
/// ```
pub async fn run_python_blocking<F>(label: &str, f: F) -> Result<Value, String>
where
    F: FnOnce() -> PyResult<Value> + Send + 'static,
{
    let label = label.to_string();
    let start = std::time::Instant::now();

    tokio::task::spawn_blocking(move || {
        let result = f();
        let elapsed = start.elapsed();
        info!("[python:{}] completed in {:.1}ms", label, elapsed.as_secs_f64() * 1000.0);
        result
    })
    .await
    .map_err(|e| format!("spawn_blocking join error: {}", e))?
    .map_err(|e| e.to_string())
}
