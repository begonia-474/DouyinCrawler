//! Python 调用运行时统一入口
//!
//! 所有 Tauri async command 通过 `run_python_blocking` 调用 Python，
//! 内部使用 `spawn_blocking` 将 GIL 阻塞移出 tokio 运行时线程。

use pyo3::prelude::*;
use serde::Serialize;
use serde_json::Value;
use log::info;

/// 在 spawn_blocking 线程中执行 Python 调用，避免阻塞 tokio 运行时。
///
/// 泛型版本：闭包可返回任意 `PyResult<T>` 类型。
/// 用法：
/// ```rust
/// let result: VideoParseResult = run_python_blocking("parse_video", || {
///     crate::python::handler::parse_video(&url)
/// }).await?;
/// ```
pub async fn run_python_blocking<T, F>(label: &str, f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> PyResult<T> + Send + 'static,
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
