//! Python 桥接核心模块
//!
//! 提供 Python 运行时初始化和通用调用接口

use pyo3::prelude::*;
use pyo3::types::PyTuple;
use std::path::PathBuf;
use std::sync::Once;
use log::{info, error};

static INIT: Once = Once::new();
static mut PYTHON_PATH_INITIALIZED: bool = false;

/// Python 桥接器
pub struct PythonBridge {
    // 未来可以添加更多状态
}

impl PythonBridge {
    /// 创建新的 Python 桥接器
    pub fn new() -> PyResult<Self> {
        // 确保 Python 路径只初始化一次
        INIT.call_once(|| {
            if let Err(e) = Self::init_python_path() {
                eprintln!("[PythonBridge] 初始化 Python 路径失败: {}", e);
            }
        });

        Ok(Self {})
    }

    /// 初始化 Python 路径
    fn init_python_path() -> PyResult<()> {
        Python::with_gil(|py| {
            let sys = py.import_bound("sys")?;
            let path = sys.getattr("path")?;

            // 获取项目根目录（src-tauri 的父目录）
            let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let project_root = manifest_dir.parent().unwrap_or(&manifest_dir);

            // 添加项目根目录
            if let Some(root_str) = project_root.to_str() {
                path.call_method1("append", (root_str,))?;
                info!("添加项目根目录: {}", root_str);
            }

            // 添加 backend 目录
            let backend_dir = project_root.join("backend");
            if let Some(backend_str) = backend_dir.to_str() {
                path.call_method1("append", (backend_str,))?;
                info!("添加 backend 目录: {}", backend_str);
            }

            // 添加 core 目录
            let core_dir = project_root.join("core");
            if let Some(core_str) = core_dir.to_str() {
                path.call_method1("append", (core_str,))?;
                info!("添加 core 目录: {}", core_str);
            }

            unsafe {
                PYTHON_PATH_INITIALIZED = true;
            }

            info!("Python 路径初始化完成");
            Ok(())
        })
    }

    /// 调用 Python 函数并返回 JSON 字符串
    pub fn call_json(&self, module: &str, method: &str, args: impl IntoPy<Py<PyTuple>>) -> PyResult<String> {
        Python::with_gil(|py| {
            let module = py.import_bound(module)?;
            let result = module.call_method1(method, args)?;

            // 调用 json.dumps 将结果转换为字符串
            let json = py.import_bound("json")?;
            let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

            Ok(json_str)
        })
    }
}

// 线程安全：PythonBridge 可以在多线程中使用
unsafe impl Send for PythonBridge {}
unsafe impl Sync for PythonBridge {}
