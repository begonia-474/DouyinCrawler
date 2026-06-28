//! 数据库模型 — serde DTOs
//!
//! 从 db.rs 提取，供所有 repository 模块使用。

use serde::{Deserialize, Serialize};

// === 下载记录 ===

#[derive(Serialize, Clone)]
pub struct DownloadRecord {
    pub id: i64,
    pub aweme_id: Option<String>,
    pub download_type: String,
    pub title: Option<String>,
    pub author_nickname: Option<String>,
    pub author_sec_uid: Option<String>,
    pub file_path: Option<String>,
    pub file_size: i64,
    pub cover_url: Option<String>,
    pub status: String,
    pub error_msg: Option<String>,
    pub created_at: i64,
}

#[derive(Serialize, Clone)]
pub struct DownloadStats {
    pub total_count: i64,
    pub total_size: i64,
    pub by_type: Vec<TypeStat>,
    pub by_day: Vec<DayStat>,
}

#[derive(Serialize, Clone)]
pub struct TypeStat {
    pub download_type: String,
    pub cnt: i64,
    pub size: i64,
}

#[derive(Serialize, Clone)]
pub struct DayStat {
    pub day: String,
    pub cnt: i64,
}

#[derive(Serialize, Clone)]
pub struct LiveRecord {
    pub id: i64,
    pub room_id: Option<String>,
    pub web_rid: Option<String>,
    pub title: Option<String>,
    pub nickname: Option<String>,
    pub sec_user_id: Option<String>,
    pub file_path: Option<String>,
    pub file_size: i64,
    pub duration_sec: i64,
    pub status: String,
    pub started_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub cover_url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NewDownloadRecord {
    pub aweme_id: Option<String>,
    pub download_type: String,
    pub title: Option<String>,
    pub author_nickname: Option<String>,
    pub author_sec_uid: Option<String>,
    pub file_path: Option<String>,
    pub file_size: i64,
    pub cover_url: Option<String>,
    pub status: String,
    pub error_msg: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NewLiveRecord {
    pub room_id: Option<String>,
    pub web_rid: Option<String>,
    pub title: Option<String>,
    pub nickname: Option<String>,
    pub sec_user_id: Option<String>,
    pub file_path: Option<String>,
    pub file_size: i64,
    pub duration_sec: i64,
    pub status: String,
    pub started_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub cover_url: Option<String>,
}

// === 下载任务 ===

#[derive(Serialize, Clone)]
pub struct DownloadTask {
    pub id: String,
    pub mode: String,
    pub url: String,
    pub title: Option<String>,
    pub author_nickname: Option<String>,
    pub status: String,
    pub total: i64,
    pub completed: i64,
    pub skipped: i64,
    pub failed: i64,
    pub error_msg: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NewDownloadTask {
    pub id: String,
    pub mode: String,
    pub url: String,
    pub title: Option<String>,
    pub author_nickname: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct TaskItem {
    pub id: i64,
    pub task_id: String,
    pub aweme_id: Option<String>,
    pub title: Option<String>,
    pub author_nickname: Option<String>,
    pub cover_url: Option<String>,
    pub file_path: Option<String>,
    pub file_size: i64,
    pub status: String,
    pub error_msg: Option<String>,
    pub created_at: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NewTaskItem {
    pub task_id: String,
    pub aweme_id: Option<String>,
    pub title: Option<String>,
    pub author_nickname: Option<String>,
    pub cover_url: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct TaskItemCounts {
    pub total: i64,
    pub completed: i64,
    pub skipped: i64,
    pub failed: i64,
    pub pending: i64,
}

#[derive(Serialize, Clone)]
pub struct DownloadTaskDetail {
    pub task: DownloadTask,
    pub items: Vec<TaskItem>,
}

// === 用户 ===

#[derive(Serialize, Deserialize, Clone)]
pub struct UserInfo {
    #[serde(alias = "author_sec_uid")]
    pub sec_user_id: String,
    #[serde(alias = "author_nickname")]
    pub nickname: Option<String>,
    #[serde(alias = "author_uid")]
    pub uid: Option<String>,
    #[serde(alias = "author_avatar_url")]
    pub avatar_url: Option<String>,
    #[serde(alias = "author_unique_id")]
    pub unique_id: Option<String>,
    #[serde(alias = "author_signature")]
    pub signature: Option<String>,
    #[serde(alias = "author_aweme_count", default)]
    pub aweme_count: i64,
    #[serde(alias = "author_follower_count", default)]
    pub follower_count: i64,
    #[serde(alias = "author_following_count", default)]
    pub following_count: i64,
    #[serde(alias = "author_total_favorited", default)]
    pub total_favorited: i64,
    #[serde(alias = "author_ip_location")]
    pub ip_location: Option<String>,
    #[serde(default)] pub live_status: i32,
    pub room_id: Option<String>,
    // f2 对齐字段
    #[serde(default)] pub city: Option<String>,
    #[serde(default)] pub country: Option<String>,
    #[serde(default)] pub favoriting_count: i64,
    #[serde(default)] pub gender: i32,
    #[serde(default)] pub is_ban: i32,
    #[serde(default)] pub is_block: i32,
    #[serde(default)] pub is_blocked: i32,
    #[serde(default)] pub is_star: i32,
    #[serde(default)] pub mix_count: i32,
    #[serde(default)] pub mplatform_followers_count: i64,
    #[serde(default)] pub nickname_raw: Option<String>,
    #[serde(default)] pub school_name: Option<String>,
    #[serde(default)] pub short_id: Option<String>,
    #[serde(default)] pub signature_raw: Option<String>,
    #[serde(default)] pub user_age: i32,
    #[serde(default)] pub custom_verify: Option<String>,
    #[serde(default)] pub updated_at: i64,
}

// === 视频 ===

#[derive(Serialize, Deserialize, Clone)]
pub struct VideoInfo {
    pub aweme_id: String,
    pub desc: Option<String>,
    #[serde(default)]
    pub aweme_type: i32,
    pub author_nickname: Option<String>,
    pub author_sec_uid: Option<String>,
    pub author_uid: Option<String>,
    pub create_time: Option<i64>,
    #[serde(default)]
    pub duration: i32,
    pub video_url: Option<String>,
    pub cover_url: Option<String>,
    pub music_title: Option<String>,
    #[serde(default)]
    pub digg_count: i64,
    #[serde(default)]
    pub comment_count: i64,
    #[serde(default)]
    pub share_count: i64,
    #[serde(default)]
    pub collect_count: i64,
    pub mix_id: Option<String>,
    pub mix_name: Option<String>,
    // f2 对齐字段 - 作者
    #[serde(default)] pub author_nickname_raw: Option<String>,
    #[serde(default)] pub author_short_id: Option<String>,
    #[serde(default)] pub author_unique_id: Option<String>,
    // f2 对齐字段 - 内容
    #[serde(default)] pub desc_raw: Option<String>,
    #[serde(default)] pub is_ads: i32,
    #[serde(default)] pub is_story: i32,
    #[serde(default)] pub is_top: i32,
    #[serde(default)] pub is_long_video: i32,
    // f2 对齐字段 - 视频
    #[serde(default)] pub video_bit_rate: Option<String>,
    #[serde(default)] pub animated_cover: Option<String>,
    #[serde(default)] pub private_status: i32,
    #[serde(default)] pub is_delete: i32,
    // f2 对齐字段 - 音乐
    #[serde(default)] pub music_author: Option<String>,
    #[serde(default)] pub music_author_raw: Option<String>,
    #[serde(default)] pub music_duration: i32,
    #[serde(default)] pub music_id: Option<String>,
    #[serde(default)] pub music_mid: Option<String>,
    #[serde(default)] pub pgc_author: Option<String>,
    #[serde(default)] pub pgc_author_title: Option<String>,
    #[serde(default)] pub pgc_music_type: i32,
    #[serde(default)] pub music_status: i32,
    #[serde(default)] pub music_owner_handle: Option<String>,
    #[serde(default)] pub music_owner_id: Option<String>,
    #[serde(default)] pub music_owner_nickname: Option<String>,
    #[serde(default)] pub music_play_url: Option<String>,
    #[serde(default)] pub is_commerce_music: i32,
    // f2 对齐字段 - 合集
    #[serde(default)] pub mix_desc: Option<String>,
    #[serde(default)] pub mix_create_time: i64,
    #[serde(default)] pub mix_pic_type: i32,
    #[serde(default)] pub mix_type: i32,
    #[serde(default)] pub mix_share_url: Option<String>,
    // f2 对齐字段 - 权限
    #[serde(default)] pub can_comment: i32,
    #[serde(default)] pub can_forward: i32,
    #[serde(default)] pub can_share: i32,
    #[serde(default)] pub download_setting: i32,
    #[serde(default)] pub allow_douplus: i32,
    #[serde(default)] pub allow_share: i32,
    // f2 对齐字段 - 统计/标签/其他
    #[serde(default)] pub admire_count: i64,
    #[serde(default)] pub hashtag_ids: Option<String>,
    #[serde(default)] pub hashtag_names: Option<String>,
    #[serde(default)] pub images: Option<String>,
    #[serde(default)] pub region: Option<String>,
    #[serde(default)] pub is_prohibited: i32,
    #[serde(default)] pub updated_at: i64,
}

// === 统计 ===

#[derive(Serialize, Clone)]
pub struct VideoStats {
    pub total_count: i64,
    pub total_digg: i64,
    pub total_comment: i64,
    pub total_share: i64,
    pub total_collect: i64,
    pub by_type: Vec<VideoTypeStat>,
}

#[derive(Serialize, Clone)]
pub struct VideoTypeStat {
    pub aweme_type: i32,
    pub cnt: i64,
}

#[derive(Serialize, Clone)]
pub struct UserStats {
    pub total_count: i64,
    pub total_follower: i64,
    pub total_aweme: i64,
}

#[derive(Serialize, Clone)]
pub struct TrendPoint {
    pub day: String,
    pub cnt: i64,
    pub size: i64,
}

#[derive(Serialize, Clone)]
pub struct AuthorStat {
    pub author_nickname: String,
    pub cnt: i64,
    pub total_size: i64,
}

#[derive(Serialize, Clone)]
pub struct StorageStat {
    pub download_type: String,
    pub cnt: i64,
    pub total_size: i64,
}

#[derive(Serialize, Clone)]
pub struct DbHealth {
    pub download_count: i64,
    pub video_count: i64,
    pub user_count: i64,
    pub live_count: i64,
    pub music_count: i64,
    pub task_count: i64,
    pub db_size_bytes: i64,
}

// === 音乐 ===

#[derive(Serialize, Clone)]
pub struct MusicCollection {
    pub music_id: String,
    pub mid: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub owner_nickname: Option<String>,
    pub duration: i32,
    pub cover: Option<String>,
    pub play_url: Option<String>,
    pub file_path: Option<String>,
    pub status: String,
    pub created_at: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NewMusicCollection {
    pub music_id: String,
    pub mid: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub owner_nickname: Option<String>,
    pub duration: i32,
    pub cover: Option<String>,
    pub play_url: Option<String>,
}
