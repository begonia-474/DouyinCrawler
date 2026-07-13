//! Python 配置管理模块
//!
//! 将 Rust 配置同步到 Python ParsingContext

use pyo3::prelude::*;
use pyo3::types::PyDict;
use crate::config::AppConfig;
use log::info;

/// 初始化 Python 配置
pub fn init_config(config: &AppConfig) -> PyResult<()> {
    let cookie_len = config.cookie.len();
    info!("[python/config] init_config 被调用, cookie 长度={}", cookie_len);

    Python::with_gil(|py| {
        let module = py.import_bound("core.bridge.parsing_context")?;
        let context = module.getattr("context")?;

        let kwargs = PyDict::new_bound(py);
        kwargs.set_item("cookie", &config.cookie)?;
        kwargs.set_item("download_path", &config.download_path)?;
        kwargs.set_item("naming", &config.naming)?;
        kwargs.set_item("encryption", &config.encryption)?;
        kwargs.set_item("proxy", &config.proxy)?;
        kwargs.set_item("app_name", &config.app_name)?;
        kwargs.set_item("folderize", config.folderize)?;
        kwargs.set_item("music", config.music)?;
        kwargs.set_item("cover", config.cover)?;
        kwargs.set_item("desc", config.desc)?;
        kwargs.set_item("interval", config.interval.as_deref())?;
        kwargs.set_item("page_counts", config.page_counts)?;
        kwargs.set_item("max_counts", config.max_counts)?;
        kwargs.set_item("timeout", config.timeout)?;
        kwargs.set_item("max_connections", config.max_connections)?;
        kwargs.set_item("max_retries", config.max_retries)?;
        kwargs.set_item("max_tasks", config.max_tasks)?;

        context.call_method("update_config", (), Some(&kwargs))?;

        info!("配置已同步到 Python ParsingContext");
        Ok(())
    })
}
