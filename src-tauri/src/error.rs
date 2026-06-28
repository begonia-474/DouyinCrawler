//! 统一错误类型 — 全栈统一定义
//!
//! Python ↔ Rust ↔ Frontend 三端使用相同的错误码。
//! 修改 ErrorCode 枚举时必须同步更新:
//! - Python: core/models/responses.py (Phase C 创建)
//! - Frontend: tauri-specta 自动生成

use serde::{Deserialize, Serialize};

/// 跨语言错误码
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    // 网络层
    NetworkTimeout,
    NetworkError,
    RateLimited,
    ProxyError,
    // 认证层
    CookieExpired,
    CookieInvalid,
    LoginRequired,
    // 业务层
    VideoNotFound,
    UserNotFound,
    ContentDeleted,
    SignatureError,
    ParseError,
    // 系统层
    DatabaseError,
    FileSystemError,
    ConfigError,
    // 兜底
    Unknown,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self)
            .unwrap_or_else(|_| "\"unknown\"".to_string());
        write!(f, "{}", s.trim_matches('"'))
    }
}

/// 应用错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            detail: None,
        }
    }

    pub fn with_detail(code: ErrorCode, message: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            detail: Some(detail.into()),
        }
    }

    pub fn unknown(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Unknown, message)
    }

    pub fn from_py_err(e: impl ToString) -> Self {
        Self::unknown(e.to_string())
    }

    pub fn from_join_err(e: impl ToString) -> Self {
        Self::new(ErrorCode::Unknown, format!("spawn_blocking join error: {}", e.to_string()))
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl From<String> for AppError {
    fn from(msg: String) -> Self {
        Self::unknown(msg)
    }
}

impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        Self::unknown(msg)
    }
}
