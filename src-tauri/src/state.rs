//! 应用全局状态 — 单一状态容器
//!
//! 替代分散的 app.manage() 调用和 APP_HANDLE 全局变量。
//! 在 lib.rs setup() 中初始化，通过 tauri::State<AppState> 注入命令。
//!
//! 设计参考: Tauri 2 生产应用 (QuickClipboard, knowledge-base)

use std::sync::Arc;
use parking_lot::Mutex;
use crate::db::Database;
use crate::config::ConfigManager;
use crate::python::PythonBridge;

/// 应用全局状态
///
/// 包含所有需要跨命令共享的长期对象。
/// 通过 `app.manage(state)` 注册，命令通过 `State<AppState>` 获取。
///
/// # 示例
///
/// ```ignore
/// #[tauri::command]
/// fn my_command(state: tauri::State<'_, AppState>) -> Result<(), String> {
///     let db = &state.db;
///     let config = state.config.lock();
///     // ...
/// }
/// ```
pub struct AppState {
    pub db: Database,
    pub config: Arc<Mutex<ConfigManager>>,
    pub python: Arc<PythonBridge>,
}

impl AppState {
    pub fn new(db: Database, config: Arc<Mutex<ConfigManager>>, python: Arc<PythonBridge>) -> Self {
        Self { db, config, python }
    }
}
