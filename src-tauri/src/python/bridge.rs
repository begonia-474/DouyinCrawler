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

static INIT: Once = Once::new();

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

            info!("Python 路径初始化完成");
            Ok(())
        })
    }

}

// 线程安全：PythonBridge 可以在多线程中使用
unsafe impl Send for PythonBridge {}
unsafe impl Sync for PythonBridge {}
