//! Python 下载适配器
//!
//! 封装 PyO3 GIL 管理，将同步 Python 调用包装为 async 方法。
//! TaskApplicationService 通过此适配器调用 Python，不直接依赖 python::handler。
//!
//! 设计决策：
//! - 具体类型（非 trait）：PyO3 所有操作需要 GIL，trait 抽象不可行
//! - spawn_blocking：避免同步 Python 调用阻塞 async runtime

use log::error;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// Python 下载适配器 — 封装 GIL 管理，对外暴露 async 方法
pub struct PythonDownloadAdapter;

impl PythonDownloadAdapter {
    /// 调用 Python handler 函数并将 JSON 结果反序列化为目标类型
    async fn call_python<T: DeserializeOwned + Send>(
        handler_fn: impl FnOnce() -> Result<Value, pyo3::PyErr> + Send + 'static,
        label: &str,
    ) -> Result<T, String> {
        let result = tokio::task::spawn_blocking(move || {
            handler_fn().map_err(|e| format!("Python 调用失败: {}", e))
        })
        .await
        .map_err(|e| format!("spawn_blocking 失败: {}", e))?;

        let json_value = result?;
        serde_json::from_value::<T>(json_value)
            .map_err(|e| format!("Python 返回值解析失败: {}", e))
    }

    /// 单视频下载
    pub async fn download_video(
        url: &str,
    ) -> Result<crate::services::download::PythonDownloadResult, String> {
        let url = url.to_string();
        Self::call_python(
            move || crate::python::handler::download_video(&url),
            "download_video",
        )
        .await
    }

    /// 批量下载（user_post / user_like / mix / collects）
    pub async fn download_batch(
        mode: &str,
        url: &str,
    ) -> Result<crate::services::download::PythonBatchDownloadResult, String> {
        let mode = mode.to_string();
        let url = url.to_string();
        Self::call_python(
            move || crate::python::handler::download_batch(&mode, &url),
            "download_batch",
        )
        .await
    }

    /// 音乐批量下载
    pub async fn download_music_batch(
        url: &str,
    ) -> Result<crate::services::download::PythonMusicBatchResult, String> {
        let url = url.to_string();
        Self::call_python(
            move || crate::python::handler::download_music_batch(&url),
            "download_music_batch",
        )
        .await
    }
}
