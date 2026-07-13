//! 应用全局状态 — 单一状态容器
//!
//! 替代分散的 app.manage() 调用和 APP_HANDLE 全局变量。
//! 在 lib.rs setup() 中初始化，通过 tauri::State<AppState> 注入命令。
//!
//! 设计参考: Tauri 2 生产应用 (QuickClipboard, knowledge-base)

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
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
    pub db: Arc<Database>,
    pub config: Arc<Mutex<ConfigManager>>,
    #[allow(dead_code)] // P2-01 下载入口统一后使用
    pub python: Arc<PythonBridge>,
    /// 任务取消信号映射表
    /// key: task_id, value: 取消信号（AtomicBool）
    pub cancel_signals: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
}

impl AppState {
    pub fn new(db: Database, config: Arc<Mutex<ConfigManager>>, python: Arc<PythonBridge>) -> Self {
        Self {
            db: Arc::new(db),
            config,
            python,
            cancel_signals: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// 注册任务的取消信号
    pub fn register_cancel_signal(&self, task_id: &str) -> Arc<AtomicBool> {
        let signal = Arc::new(AtomicBool::new(false));
        let mut signals = self.cancel_signals.lock();
        signals.insert(task_id.to_string(), signal.clone());
        signal
    }
    
    /// 获取任务的取消信号
    pub fn get_cancel_signal(&self, task_id: &str) -> Option<Arc<AtomicBool>> {
        let signals = self.cancel_signals.lock();
        signals.get(task_id).cloned()
    }
    
    /// 移除任务的取消信号（任务完成/错误/取消时调用）
    pub fn remove_cancel_signal(&self, task_id: &str) {
        let mut signals = self.cancel_signals.lock();
        signals.remove(task_id);
    }
    
    /// 取消任务（设置取消信号）
    pub fn cancel_task(&self, task_id: &str) -> bool {
        let signals = self.cancel_signals.lock();
        if let Some(signal) = signals.get(task_id) {
            signal.store(true, std::sync::atomic::Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    /// 应用退出时向所有后台任务发送停止信号。
    pub fn cancel_all_tasks(&self) {
        let signals = self.cancel_signals.lock();
        for signal in signals.values() {
            signal.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }
}
