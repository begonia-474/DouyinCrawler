//! 下载服务
//!
//! - `mod.rs`（本文件）：类型化 DTO 定义
//! - `task_service.rs`：TaskApplicationService — 任务生命周期管理
//! - `events.rs`：类型化事件发射
//! - `python_adapter.rs`：PythonDownloadAdapter — 封装 GIL 管理

pub mod events;
pub mod task_service;
pub mod python_adapter;

use serde::{Deserialize, Serialize};

// ============================================================
// 下载模式
// ============================================================

/// 下载模式枚举（对齐 Python core.constants.DownloadMode）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum DownloadMode {
    One,
    Post,
    Like,
    Mix,
    Collects,
    Live,
    Music,
}

impl DownloadMode {
    /// 从字符串解析，未知模式返回 None
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "one" => Some(Self::One),
            "post" => Some(Self::Post),
            "like" => Some(Self::Like),
            "mix" => Some(Self::Mix),
            "collects" => Some(Self::Collects),
            "live" => Some(Self::Live),
            "music" => Some(Self::Music),
            _ => None,
        }
    }

    /// 转为字符串（与 Python 侧 mode 参数一致）
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::One => "one",
            Self::Post => "post",
            Self::Like => "like",
            Self::Mix => "mix",
            Self::Collects => "collects",
            Self::Live => "live",
            Self::Music => "music",
        }
    }
}

impl std::fmt::Display for DownloadMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ============================================================
// 任务状态
// ============================================================

/// 任务状态枚举（对齐 DB download_tasks.status）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Starting,
    Running,
    Recording,
    Stopping,
    Completed,
    Error,
    Cancelled,
}

#[allow(dead_code)]
impl TaskStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "starting" => Some(Self::Starting),
            "running" => Some(Self::Running),
            "recording" => Some(Self::Recording),
            "stopping" => Some(Self::Stopping),
            "completed" => Some(Self::Completed),
            "error" => Some(Self::Error),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Recording => "recording",
            Self::Stopping => "stopping",
            Self::Completed => "completed",
            Self::Error => "error",
            Self::Cancelled => "cancelled",
        }
    }

    /// 是否为终态（不再变化）
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Error | Self::Cancelled)
    }
}

// ============================================================
// 下载请求（前端 → Rust）
// ============================================================

/// 统一下载请求（前端 invoke 参数）
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DownloadRequest {
    pub mode: DownloadMode,
    pub url: String,
}

// ============================================================
// 任务快照（DB → 前端）
// ============================================================

// ============================================================
// 任务补丁（事件 → 前端 store 合并）
// ============================================================

/// 任务补丁（事件携带的部分更新，前端 patch 语义合并）
/// None 字段不覆盖现有值
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct TaskPatch {
    pub task_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TaskStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_msg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_item: Option<String>,
}

// ============================================================
// 任务事件（Rust → 前端 Tauri event）
// ============================================================

/// 任务事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum TaskEventType {
    Started,
    Progress,
    Finished,
}

/// 类型化任务事件（通过 Tauri event 系统发射到前端）
#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct TaskEvent {
    pub event_type: TaskEventType,
    pub task_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<DownloadMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(flatten)]
    pub patch: TaskPatch,
}

// ============================================================
// Python 返回值（Python → Rust）
// ============================================================

/// Python 单视频下载/解析结果
/// 对齐 core.py_bridge.download_video / parse_video 返回的 JSON
#[derive(Debug, Clone, Deserialize)]
pub struct PythonDownloadResult {
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub paths: Option<Vec<String>>,
    #[serde(default)]
    pub detail: Option<serde_json::Value>,
    #[serde(default)]
    pub user_profile: Option<serde_json::Value>,
}


/// Python 批量下载单条结果
#[derive(Debug, Clone, Deserialize)]
pub struct PythonBatchItem {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub detail: Option<serde_json::Value>,
}

/// Python 批量下载结果（新路径，不写 task DB 表）
#[derive(Debug, Clone, Deserialize)]
pub struct PythonBatchDownloadResult {
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub results: Option<Vec<PythonBatchItem>>,
    #[serde(default)]
    pub user_profile: Option<serde_json::Value>,
}

/// Python 音乐批量下载单条结果
#[derive(Debug, Clone, Deserialize)]
pub struct PythonMusicItem {
    pub music_id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub file_size: Option<i64>,
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
}

/// Python 音乐批量下载结果
#[derive(Debug, Clone, Deserialize)]
pub struct PythonMusicBatchResult {
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub results: Option<Vec<PythonMusicItem>>,
}

// ============================================================
// 构建器方法
// ============================================================

impl TaskPatch {
    /// 创建仅包含 task_id 的空补丁
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            status: None,
            total: None,
            completed: None,
            skipped: None,
            failed: None,
            error_msg: None,
            current_item: None,
        }
    }

    /// 设置状态
    pub fn with_status(mut self, status: TaskStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// 设置计数器
    pub fn with_counts(mut self, total: i64, completed: i64, failed: i64, skipped: i64) -> Self {
        self.total = Some(total);
        self.completed = Some(completed);
        self.failed = Some(failed);
        self.skipped = Some(skipped);
        self
    }

    /// 设置错误信息
    pub fn with_error(mut self, msg: impl Into<String>) -> Self {
        self.error_msg = Some(msg.into());
        self.status = Some(TaskStatus::Error);
        self
    }
}

#[allow(dead_code)]
impl TaskEvent {
    /// 创建任务启动事件
    pub fn started(task_id: impl Into<String>, mode: DownloadMode, url: impl Into<String>) -> Self {
        let task_id = task_id.into();
        Self {
            event_type: TaskEventType::Started,
            task_id: task_id.clone(),
            mode: Some(mode),
            url: Some(url.into()),
            patch: TaskPatch::new(task_id).with_status(TaskStatus::Running),
        }
    }

    /// 创建任务进度事件
    pub fn progress(patch: TaskPatch) -> Self {
        Self {
            event_type: TaskEventType::Progress,
            task_id: patch.task_id.clone(),
            mode: None,
            url: None,
            patch,
        }
    }

    /// 创建任务完成事件
    pub fn finished(patch: TaskPatch) -> Self {
        Self {
            event_type: TaskEventType::Finished,
            task_id: patch.task_id.clone(),
            mode: None,
            url: None,
            patch,
        }
    }
}
