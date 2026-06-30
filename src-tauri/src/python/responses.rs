//! Python↔Rust 桥接响应结构体
//!
//! 每个结构体对应 core/models/responses.py 中的同名 dataclass。
//! 修改时必须两边同步，否则 serde_json::from_value 会在运行时失败。
//!
//! 下载类结构体（PythonDownloadResult 等）定义在 services/download/mod.rs，
//! 此文件仅定义查询类 + 直播类结构体。

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{AppError, ErrorCode};

// ============================================================
// 默认值函数（serde 缺省字段）
// ============================================================

fn default_success() -> bool { false }
fn default_error_code() -> String { "unknown".to_string() }
fn default_empty_string() -> String { String::new() }
fn default_false() -> bool { false }
fn default_zero_i64() -> i64 { 0 }
fn default_empty_vec_value() -> Vec<Value> { Vec::new() }

/// 桥接响应 trait — 提供 success 检查和 AppError 转换
pub trait BridgeResponse: Sized {
    fn is_success(&self) -> bool;
    fn error_code_str(&self) -> &str;
    fn error_message(&self) -> &str;

    /// 检查 success，失败时转为 Err(AppError)
    fn into_result(self) -> Result<Self, AppError> {
        if self.is_success() {
            Ok(self)
        } else {
            let code = parse_error_code(self.error_code_str());
            Err(AppError::new(code, self.error_message()))
        }
    }
}

/// 将字符串 ErrorCode 转为 Rust ErrorCode 枚举
fn parse_error_code(s: &str) -> ErrorCode {
    match s {
        "network_timeout" => ErrorCode::NetworkTimeout,
        "network_error" => ErrorCode::NetworkError,
        "rate_limited" => ErrorCode::RateLimited,
        "proxy_error" => ErrorCode::ProxyError,
        "cookie_expired" => ErrorCode::CookieExpired,
        "cookie_invalid" => ErrorCode::CookieInvalid,
        "login_required" => ErrorCode::LoginRequired,
        "video_not_found" => ErrorCode::VideoNotFound,
        "user_not_found" => ErrorCode::UserNotFound,
        "content_deleted" => ErrorCode::ContentDeleted,
        "signature_error" => ErrorCode::SignatureError,
        "parse_error" => ErrorCode::ParseError,
        "database_error" => ErrorCode::DatabaseError,
        "file_system_error" => ErrorCode::FileSystemError,
        "config_error" => ErrorCode::ConfigError,
        _ => ErrorCode::Unknown,
    }
}

/// 为所有包含 success/error_code/error 字段的结构体实现 BridgeResponse
/// 使用宏减少重复代码
macro_rules! impl_bridge_response {
    ($struct_name:ident) => {
        impl BridgeResponse for $struct_name {
            fn is_success(&self) -> bool { self.success }
            fn error_code_str(&self) -> &str { &self.error_code }
            fn error_message(&self) -> &str { &self.error }
        }
    };
}

// ============================================================
// 查询类响应
// ============================================================

/// parse_video() 返回值 — 对齐 Python VideoParseResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoParseResult {
    #[serde(default = "default_success")]
    pub success: bool,
    #[serde(default = "default_error_code")]
    pub error_code: String,
    #[serde(default = "default_empty_string")]
    pub error: String,
    pub detail: Option<Value>,
    pub path: Option<String>,
    pub paths: Option<Vec<String>>,
}

/// get_user_profile() 返回值 — 对齐 Python UserProfileResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileResult {
    #[serde(default = "default_success")]
    pub success: bool,
    #[serde(default = "default_error_code")]
    pub error_code: String,
    #[serde(default = "default_empty_string")]
    pub error: String,
    pub profile: Option<Value>,
}

/// get_user_posts() / get_user_likes() 返回值 — 对齐 Python UserPostsResult / UserLikesResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPostsResult {
    #[serde(default = "default_success")]
    pub success: bool,
    #[serde(default = "default_error_code")]
    pub error_code: String,
    #[serde(default = "default_empty_string")]
    pub error: String,
    #[serde(default = "default_empty_vec_value")]
    pub videos: Vec<Value>,
    #[serde(default = "default_false")]
    pub has_more: bool,
    #[serde(default = "default_zero_i64")]
    pub cursor: i64,
}

/// get_live_info() 返回值 — 对齐 Python LiveInfoResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveInfoResult {
    #[serde(default = "default_success")]
    pub success: bool,
    #[serde(default = "default_error_code")]
    pub error_code: String,
    #[serde(default = "default_empty_string")]
    pub error: String,
    pub live_info: Option<Value>,
}

/// get_music_collection() 返回值 — 对齐 Python MusicCollectionResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicCollectionResult {
    #[serde(default = "default_success")]
    pub success: bool,
    #[serde(default = "default_error_code")]
    pub error_code: String,
    #[serde(default = "default_empty_string")]
    pub error: String,
    #[serde(default = "default_empty_vec_value")]
    pub music_list: Vec<Value>,
    #[serde(default = "default_false")]
    pub has_more: bool,
    #[serde(default = "default_zero_i64")]
    pub cursor: i64,
}

/// get_comments() 返回值 — 对齐 Python CommentsResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentsResult {
    #[serde(default = "default_success")]
    pub success: bool,
    #[serde(default = "default_error_code")]
    pub error_code: String,
    #[serde(default = "default_empty_string")]
    pub error: String,
    #[serde(default = "default_empty_vec_value")]
    pub comments: Vec<Value>,
    #[serde(default = "default_false")]
    pub has_more: bool,
    #[serde(default = "default_zero_i64")]
    pub cursor: i64,
}

/// get_following_list() 返回值 — 对齐 Python FollowingListResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowingListResult {
    #[serde(default = "default_success")]
    pub success: bool,
    #[serde(default = "default_error_code")]
    pub error_code: String,
    #[serde(default = "default_empty_string")]
    pub error: String,
    #[serde(default = "default_empty_vec_value")]
    pub followings: Vec<Value>,
    #[serde(default = "default_false")]
    pub has_more: bool,
    #[serde(default = "default_zero_i64")]
    pub cursor: i64,
}

/// get_follower_list() 返回值 — 对齐 Python FollowerListResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowerListResult {
    #[serde(default = "default_success")]
    pub success: bool,
    #[serde(default = "default_error_code")]
    pub error_code: String,
    #[serde(default = "default_empty_string")]
    pub error: String,
    #[serde(default = "default_empty_vec_value")]
    pub followers: Vec<Value>,
    #[serde(default = "default_false")]
    pub has_more: bool,
    #[serde(default = "default_zero_i64")]
    pub cursor: i64,
}

/// get_collects_list() 返回值 — 对齐 Python CollectsListResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectsListResult {
    #[serde(default = "default_success")]
    pub success: bool,
    #[serde(default = "default_error_code")]
    pub error_code: String,
    #[serde(default = "default_empty_string")]
    pub error: String,
    #[serde(default = "default_empty_vec_value")]
    pub collects: Vec<Value>,
}

/// get_collects_video_list() 返回值 — 对齐 Python CollectsVideoListResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectsVideoListResult {
    #[serde(default = "default_success")]
    pub success: bool,
    #[serde(default = "default_error_code")]
    pub error_code: String,
    #[serde(default = "default_empty_string")]
    pub error: String,
    #[serde(default = "default_empty_vec_value")]
    pub videos: Vec<Value>,
    #[serde(default = "default_false")]
    pub has_more: bool,
    #[serde(default = "default_zero_i64")]
    pub cursor: i64,
}

/// get_mix_info() 返回值 — 对齐 Python MixInfoResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixInfoResult {
    #[serde(default = "default_success")]
    pub success: bool,
    #[serde(default = "default_error_code")]
    pub error_code: String,
    #[serde(default = "default_empty_string")]
    pub error: String,
    #[serde(default = "default_empty_vec_value")]
    pub videos: Vec<Value>,
    #[serde(default = "default_false")]
    pub has_more: bool,
    #[serde(default = "default_zero_i64")]
    pub cursor: i64,
}

/// search_videos() 返回值 — 对齐 Python SearchResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    #[serde(default = "default_success")]
    pub success: bool,
    #[serde(default = "default_error_code")]
    pub error_code: String,
    #[serde(default = "default_empty_string")]
    pub error: String,
    #[serde(default = "default_empty_vec_value")]
    pub videos: Vec<Value>,
    #[serde(default = "default_false")]
    pub has_more: bool,
    #[serde(default = "default_zero_i64")]
    pub cursor: i64,
}

/// get_tab_feed() 返回值 — 对齐 Python TabFeedResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabFeedResult {
    #[serde(default = "default_success")]
    pub success: bool,
    #[serde(default = "default_error_code")]
    pub error_code: String,
    #[serde(default = "default_empty_string")]
    pub error: String,
    #[serde(default = "default_empty_vec_value")]
    pub videos: Vec<Value>,
    #[serde(default = "default_false")]
    pub has_more: bool,
    #[serde(default = "default_zero_i64")]
    pub cursor: i64,
    #[serde(default = "default_zero_i64")]
    pub next_cursor: i64,
}

/// get_follow_feed() 返回值 — 对齐 Python FollowFeedResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowFeedResult {
    #[serde(default = "default_success")]
    pub success: bool,
    #[serde(default = "default_error_code")]
    pub error_code: String,
    #[serde(default = "default_empty_string")]
    pub error: String,
    #[serde(default = "default_empty_vec_value")]
    pub videos: Vec<Value>,
    #[serde(default = "default_false")]
    pub has_more: bool,
    #[serde(default = "default_zero_i64")]
    pub cursor: i64,
}

/// get_friend_feed() 返回值 — 对齐 Python FriendFeedResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendFeedResult {
    #[serde(default = "default_success")]
    pub success: bool,
    #[serde(default = "default_error_code")]
    pub error_code: String,
    #[serde(default = "default_empty_string")]
    pub error: String,
    #[serde(default = "default_empty_vec_value")]
    pub videos: Vec<Value>,
    #[serde(default = "default_false")]
    pub has_more: bool,
    #[serde(default = "default_zero_i64")]
    pub cursor: i64,
}

/// get_user_likes() 返回值 — 对齐 Python UserLikesResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLikesResult {
    #[serde(default = "default_success")]
    pub success: bool,
    #[serde(default = "default_error_code")]
    pub error_code: String,
    #[serde(default = "default_empty_string")]
    pub error: String,
    #[serde(default = "default_empty_vec_value")]
    pub videos: Vec<Value>,
    #[serde(default = "default_false")]
    pub has_more: bool,
    #[serde(default = "default_zero_i64")]
    pub cursor: i64,
}

/// get_post_stats() 返回值 — 对齐 Python PostStatsResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostStatsResult {
    #[serde(default = "default_success")]
    pub success: bool,
    #[serde(default = "default_error_code")]
    pub error_code: String,
    #[serde(default = "default_empty_string")]
    pub error: String,
    pub stats: Option<Value>,
}

// ============================================================
// 下载类响应 — 对齐 Python DownloadResult / BatchDownloadResult / MusicBatchResult
// ============================================================

/// download_video() 返回值 — 对齐 Python DownloadResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadResult {
    #[serde(default = "default_success")]
    pub success: bool,
    #[serde(default = "default_error_code")]
    pub error_code: String,
    #[serde(default = "default_empty_string")]
    pub error: String,
    pub detail: Option<Value>,
    pub path: Option<String>,
    pub paths: Option<Vec<String>>,
}

/// download_batch() 返回值 — 对齐 Python BatchDownloadResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchDownloadResult {
    #[serde(default = "default_success")]
    pub success: bool,
    #[serde(default = "default_error_code")]
    pub error_code: String,
    #[serde(default = "default_empty_string")]
    pub error: String,
    #[serde(default = "default_zero_i64")]
    pub count: i64,
    #[serde(default = "default_empty_vec_value")]
    pub results: Vec<Value>,
}

/// download_music_batch() 返回值 — 对齐 Python MusicBatchResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicBatchResult {
    #[serde(default = "default_success")]
    pub success: bool,
    #[serde(default = "default_error_code")]
    pub error_code: String,
    #[serde(default = "default_empty_string")]
    pub error: String,
    #[serde(default = "default_empty_vec_value")]
    pub music_list: Vec<Value>,
    #[serde(default = "default_empty_vec_value")]
    pub results: Vec<Value>,
}

// ============================================================
// 直播类响应
// ============================================================

/// start_live_record() 返回值 — 对齐 Python LiveRecordResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveRecordResult {
    #[serde(default = "default_success")]
    pub success: bool,
    #[serde(default = "default_error_code")]
    pub error_code: String,
    #[serde(default = "default_empty_string")]
    pub error: String,
    #[serde(default = "default_empty_string")]
    pub task_id: String,
}

/// get_live_status() 返回值 — 对齐 Python LiveStatusResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveStatusResult {
    #[serde(default = "default_success")]
    pub success: bool,
    #[serde(default = "default_error_code")]
    pub error_code: String,
    #[serde(default = "default_empty_string")]
    pub error: String,
    #[serde(default = "default_empty_vec_value")]
    pub tasks: Vec<Value>,
}

/// get_following_live() 返回值 — 对齐 Python FollowingLiveResult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowingLiveResult {
    #[serde(default = "default_success")]
    pub success: bool,
    #[serde(default = "default_error_code")]
    pub error_code: String,
    #[serde(default = "default_empty_string")]
    pub error: String,
    #[serde(default = "default_empty_vec_value")]
    pub lives: Vec<Value>,
}

// ============================================================
// BridgeResponse trait 实现
// ============================================================

impl_bridge_response!(VideoParseResult);
impl_bridge_response!(UserProfileResult);
impl_bridge_response!(UserPostsResult);
impl_bridge_response!(LiveInfoResult);
impl_bridge_response!(MusicCollectionResult);
impl_bridge_response!(CommentsResult);
impl_bridge_response!(FollowingListResult);
impl_bridge_response!(FollowerListResult);
impl_bridge_response!(CollectsListResult);
impl_bridge_response!(CollectsVideoListResult);
impl_bridge_response!(MixInfoResult);
impl_bridge_response!(SearchResult);
impl_bridge_response!(TabFeedResult);
impl_bridge_response!(FollowFeedResult);
impl_bridge_response!(FriendFeedResult);
impl_bridge_response!(UserLikesResult);
impl_bridge_response!(PostStatsResult);
impl_bridge_response!(DownloadResult);
impl_bridge_response!(BatchDownloadResult);
impl_bridge_response!(MusicBatchResult);
impl_bridge_response!(LiveRecordResult);
impl_bridge_response!(LiveStatusResult);
impl_bridge_response!(FollowingLiveResult);
