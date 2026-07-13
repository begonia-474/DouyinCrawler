//! 下载服务
//!
//! - `mod.rs`（本文件）：类型化 DTO 定义
//! - `task_service.rs`：TaskApplicationService — 任务生命周期管理
//! - `events.rs`：类型化事件发射
//! - `engine.rs`：DownloadEngine — 核心下载引擎

pub mod events;
pub mod task_service;
pub mod engine;
pub mod live;
pub mod contract;
#[cfg(test)]
pub(crate) mod task_test_support;

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
    Interrupted,
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
            "interrupted" => Some(Self::Interrupted),
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
            Self::Interrupted => "interrupted",
        }
    }

    /// 是否为终态（不再变化）
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Error | Self::Cancelled | Self::Interrupted)
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
    #[serde(default)]
    pub aweme_ids: Vec<String>,
}

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_rid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_sec: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<i64>,
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
// resolve_urls 返回值（Python → Rust）
// ============================================================

/// resolve_urls 返回的附属文件
#[derive(Debug, Clone, Deserialize)]
pub struct ResolvedAccessory {
    pub url: Option<String>,
    pub filename: String,
    pub suffix: String,
    pub content_type: String,
    #[serde(default)]
    pub content: Option<String>,  // 文案内容
}


/// 自定义反序列化函数，支持字符串或字符串列表
fn deserialize_download_url<'de, D>(deserializer: D) -> Result<serde_json::Value, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;
    use serde_json::Value;
    
    struct DownloadUrlVisitor;
    
    impl<'de> de::Visitor<'de> for DownloadUrlVisitor {
        type Value = Value;
        
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or array of strings")
        }
        
        fn visit_str<E>(self, value: &str) -> Result<Value, E>
        where
            E: de::Error,
        {
            Ok(Value::String(value.to_string()))
        }
        
        fn visit_string<E>(self, value: String) -> Result<Value, E>
        where
            E: de::Error,
        {
            Ok(Value::String(value))
        }
        
        fn visit_seq<A>(self, seq: A) -> Result<Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let vec: Vec<Value> = serde::Deserialize::deserialize(de::value::SeqAccessDeserializer::new(seq))?;
            Ok(Value::Array(vec))
        }
    }
    
    deserializer.deserialize_any(DownloadUrlVisitor)
}

/// resolve_urls 返回的单个下载项
#[derive(Debug, Clone, Deserialize)]
pub struct ResolvedItem {
    pub aweme_id: String,
    #[serde(deserialize_with = "deserialize_download_url")]
    pub download_url: serde_json::Value,
    pub filename: String,
    pub suffix: String,
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    pub content_type: String,
    #[serde(default)]
    pub detail: Option<serde_json::Value>,
    #[serde(default)]
    pub accessories: Vec<ResolvedAccessory>,
    /// folderize 子目录名（Python 侧在 folderize=True 时设置）
    #[serde(default)]
    pub folder_name: Option<String>,
}

/// resolve_urls 返回的完整结果
#[derive(Debug, Clone, Deserialize)]
pub struct ResolvedUrls {
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub items: Vec<ResolvedItem>,
    /// 建议的保存目录
    #[serde(default)]
    pub save_dir: Option<String>,
    /// 总数
    #[serde(default)]
    pub total: Option<i64>,
    /// 用户资料（batch 模式下可能返回）
    #[serde(default)]
    pub user_profile: Option<serde_json::Value>,
    /// 下一页游标（resolve_page 返回）
    #[serde(default)]
    pub next_cursor: Option<i64>,
    /// 是否还有更多数据（resolve_page 返回）
    #[serde(default)]
    pub has_more: Option<bool>,
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
            title: None,
            nickname: None,
            room_id: None,
            web_rid: None,
            cover_url: None,
            file: None,
            file_size: None,
            duration_sec: None,
            started_at: None,
            ended_at: None,
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

    pub fn with_live_metadata(
        mut self,
        title: impl Into<String>,
        nickname: impl Into<String>,
        room_id: impl Into<String>,
        web_rid: impl Into<String>,
        cover_url: impl Into<String>,
    ) -> Self {
        self.title = Some(title.into());
        self.nickname = Some(nickname.into());
        self.room_id = Some(room_id.into());
        self.web_rid = Some(web_rid.into());
        self.cover_url = Some(cover_url.into());
        self
    }

    pub fn with_live_result(
        mut self,
        file: impl Into<String>,
        file_size: i64,
        duration_sec: i64,
        started_at: i64,
        ended_at: i64,
    ) -> Self {
        self.file = Some(file.into());
        self.file_size = Some(file_size);
        self.duration_sec = Some(duration_sec);
        self.started_at = Some(started_at);
        self.ended_at = Some(ended_at);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_status_interrupted_is_terminal() {
        assert!(TaskStatus::Interrupted.is_terminal());
        assert_eq!(TaskStatus::Interrupted.as_str(), "interrupted");
        assert_eq!(TaskStatus::from_str("interrupted"), Some(TaskStatus::Interrupted));
    }

    #[test]
    fn task_status_interrupted_serialization() {
        let status = TaskStatus::Interrupted;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"interrupted\"");
        let deserialized: TaskStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, TaskStatus::Interrupted);
    }

    #[test]
    fn live_task_patch_serializes_recording_metadata() {
        let patch = TaskPatch::new("live-1")
            .with_status(TaskStatus::Completed)
            .with_live_metadata("标题", "主播", "room-1", "web-1", "cover")
            .with_live_result("/tmp/live.flv", 1024, 60, 100, 160);

        let value = serde_json::to_value(patch).unwrap();
        assert_eq!(value["title"], "标题");
        assert_eq!(value["nickname"], "主播");
        assert_eq!(value["room_id"], "room-1");
        assert_eq!(value["file"], "/tmp/live.flv");
        assert_eq!(value["file_size"], 1024);
        assert_eq!(value["duration_sec"], 60);
    }
}
