//! Python 桥接核心模块
//!
//! 提供 Python 运行时初始化和通用调用接口

use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyList, PyLong, PyString, PyTuple};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Once;
use log::info;

/// 将 Python 对象直接转为 serde_json::Value（无 JSON string 中转）
///
/// 递归遍历 Python 对象树，直接构建 `serde_json::Value`：
/// - dict → Value::Object
/// - list/tuple → Value::Array
/// - bool → Value::Bool（必须在 int 之前，Python bool 是 int 子类）
/// - int → Value::Number(i64)，溢出时回退到 f64
/// - float → Value::Number(f64)
/// - str → Value::String
/// - None → Value::Null
/// - 其他类型 → fallback 到 json.dumps()
pub fn py_to_json_value(obj: &Bound<'_, PyAny>) -> PyResult<Value> {
    let py = obj.py();

    // None → Null（必须最早检查）
    if obj.is_none() {
        return Ok(Value::Null);
    }

    // bool 在 int 之前（Python bool 是 int 子类）
    if obj.is_instance_of::<PyBool>() {
        return Ok(Value::Bool(obj.extract::<bool>()?));
    }

    // dict → Object
    if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut map = serde_json::Map::with_capacity(dict.len());
        for (k, v) in dict.iter() {
            let key = if k.is_instance_of::<PyString>() {
                k.extract::<String>()?
            } else {
                // 非字符串键转为字符串（Python dict 允许任意 hashable key）
                k.str()?.to_string()
            };
            map.insert(key, py_to_json_value(&v)?);
        }
        return Ok(Value::Object(map));
    }

    // list → Array
    if let Ok(list) = obj.downcast::<PyList>() {
        let mut arr = Vec::with_capacity(list.len());
        for item in list.iter() {
            arr.push(py_to_json_value(&item)?);
        }
        return Ok(Value::Array(arr));
    }

    // tuple → Array
    if let Ok(tuple) = obj.downcast::<PyTuple>() {
        let mut arr = Vec::with_capacity(tuple.len());
        for item in tuple.iter() {
            arr.push(py_to_json_value(&item)?);
        }
        return Ok(Value::Array(arr));
    }

    // int → Number(i64)，溢出回退 f64
    if obj.is_instance_of::<PyLong>() {
        if let Ok(n) = obj.extract::<i64>() {
            return Ok(Value::Number(n.into()));
        }
        if let Ok(f) = obj.extract::<f64>() {
            return Ok(Value::Number(serde_json::Number::from_f64(f)
                .unwrap_or(serde_json::Number::from(0))));
        }
        return Err(pyo3::exceptions::PyValueError::new_err(
            format!("Python int 溢出，无法转为 i64/f64: {:?}", obj),
        ));
    }

    // float → Number(f64)
    if obj.is_instance_of::<PyFloat>() {
        let f = obj.extract::<f64>()?;
        return Ok(Value::Number(serde_json::Number::from_f64(f)
            .unwrap_or(serde_json::Number::from(0))));
    }

    // str → String
    if obj.is_instance_of::<PyString>() {
        return Ok(Value::String(obj.extract::<String>()?));
    }

    // fallback：不支持的类型走 json.dumps（如 datetime、Decimal、自定义对象）
    let json = py.import_bound("json")?;
    let json_str: String = json.call_method1("dumps", (obj,))?.extract()?;
    serde_json::from_str(&json_str)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("JSON fallback 反序列化失败: {}", e)))
}

// ============================================================
// Windows DLL 搜索目录辅助
// ============================================================

#[cfg(target_os = "windows")]
extern "system" {
    fn SetDllDirectoryW(lpPathName: *const u16) -> i32;
}

#[cfg(target_os = "windows")]
fn set_dll_directory(path: &std::path::Path) {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    unsafe {
        let result = SetDllDirectoryW(wide.as_ptr());
        if result != 0 {
            info!("[bridge] SetDllDirectoryW: {:?}", path);
        } else {
            eprintln!("[bridge] SetDllDirectoryW 失败: {:?}", path);
        }
    }
}

// ============================================================
// PythonBridge
// ============================================================

static INIT: Once = Once::new();

/// Python 桥接器
pub struct PythonBridge {
    // 未来可以添加更多状态
}

impl PythonBridge {
    #[cfg(test)]
    pub(crate) fn for_test() -> Self {
        Python::with_gil(|py| {
            Self::init_python_path_for_test(py).ok();
        });
        Self {}
    }

    /// 创建新的 Python 桥接器
    ///
    /// `resource_dir` — Tauri app.path().resource_dir() 的返回值。
    /// 当为 Some 且包含 `python/` 子目录时，启用嵌入式 Python 模式。
    /// 当为 None 或不包含嵌入式 Python 时，回退到开发模式（系统 Python + CARGO_MANIFEST_DIR）。
    pub fn new(resource_dir: Option<PathBuf>) -> PyResult<Self> {
        INIT.call_once(|| {
            Self::pre_init_python(&resource_dir);
            if let Err(e) = Self::init_python_path(&resource_dir) {
                eprintln!("[PythonBridge] 初始化 Python 路径失败: {}", e);
            }
        });
        Ok(Self {})
    }

    /// Python 初始化前配置：设置 PYTHONHOME 和 DLL 搜索目录（Windows）
    fn pre_init_python(resource_dir: &Option<PathBuf>) {
        let python_dir = resource_dir
            .as_ref()
            .map(|d| d.join("python"))
            .filter(|d| d.exists());

        if let Some(ref py_dir) = python_dir {
            info!("[bridge] 启用嵌入式 Python 模式, PYTHONHOME={:?}", py_dir);
            std::env::set_var("PYTHONHOME", py_dir);

            #[cfg(target_os = "windows")]
            set_dll_directory(py_dir);
        }
    }

    /// 初始化 Python sys.path
    fn init_python_path(resource_dir: &Option<PathBuf>) -> PyResult<()> {
        Python::with_gil(|py| {
            let sys = py.import_bound("sys")?;
            let path = sys.getattr("path")?;

            // 判断使用嵌入式还是开发模式
            let use_bundled = resource_dir
                .as_ref()
                .map(|d| d.join("python").exists() && d.join("core").exists())
                .unwrap_or(false);

            if use_bundled {
                let base = resource_dir.as_ref().unwrap();
                append_sys_path(&path, base)?;
                append_sys_path(&path, &base.join("core"))?;
                info!("[bridge] 嵌入式模式: sys.path 已配置, base={:?}", base);
            } else {
                // 开发模式：使用 CARGO_MANIFEST_DIR 的相对路径
                let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                let project_root = manifest_dir.parent().unwrap_or(&manifest_dir);

                append_sys_path(&path, project_root)?;
                let backend_dir = project_root.join("backend");
                if backend_dir.exists() {
                    append_sys_path(&path, &backend_dir)?;
                }
                append_sys_path(&path, &project_root.join("core"))?;
                info!("[bridge] 开发模式: sys.path 已配置, root={:?}", project_root);
            }

            info!("[bridge] Python 路径初始化完成");
            Ok(())
        })
    }

    #[cfg(test)]
    fn init_python_path_for_test(py: Python<'_>) -> PyResult<()> {
        let sys = py.import_bound("sys")?;
        let path = sys.getattr("path")?;
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let project_root = manifest_dir.parent().unwrap_or(&manifest_dir);
        append_sys_path(&path, &project_root.join("core"))?;
        Ok(())
    }
}

/// 将路径追加到 Python sys.path
fn append_sys_path(sys_path: &Bound<'_, PyAny>, dir: &std::path::Path) -> PyResult<()> {
    if let Some(s) = dir.to_str() {
        if s.is_empty() {
            return Ok(());
        }
        sys_path.call_method1("append", (s,))?;
        info!("[bridge] sys.path += {:?}", s);
    }
    Ok(())
}

// 线程安全：PythonBridge 可以在多线程中使用
unsafe impl Send for PythonBridge {}
unsafe impl Sync for PythonBridge {}
