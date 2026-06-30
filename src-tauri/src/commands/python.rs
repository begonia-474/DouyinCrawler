//! Tauri commands — Python 桥接调用
//!
//! py_* 命令分为两类：
//! - async：通过 `run_python_blocking` 在 spawn_blocking 线程中执行，不阻塞 tokio 运行时
//! - sync：直接调用，用于轻量操作（状态查询、停止任务等）
//!
//! 使用 `py_command_*!` 宏消除样板代码（覆盖 ~70% 命令）。

use serde::Serialize;
use serde_json::Value;
use crate::error::AppError;
use crate::python::runtime::run_python_blocking;
use crate::python::responses::*;

// ============================================================
// 命令宏定义（消除样板代码）
//
// 泛型参数 $ret 必须实现 BridgeResponse trait。
// 宏内部自动调用 into_result() 将 success: false 转为 Err(AppError)。
// ============================================================

macro_rules! py_command_str {
    ($name:ident, $handler:path, $ret:ty) => {
        #[tauri::command(rename_all = "snake_case")]
        pub async fn $name(url: String) -> Result<$ret, AppError> {
            run_python_blocking(stringify!($name), move || $handler(&url))
                .await
                .map_err(AppError::from)?
                .into_result()
        }
    };
}

macro_rules! py_command_str_opts {
    ($name:ident, $handler:path, $ret:ty) => {
        #[tauri::command(rename_all = "snake_case")]
        pub async fn $name(url: String, cursor: Option<i64>, count: Option<i64>) -> Result<$ret, AppError> {
            run_python_blocking(stringify!($name), move || {
                $handler(&url, cursor.unwrap_or(0), count.unwrap_or(20))
            }).await.map_err(AppError::from)?.into_result()
        }
    };
}

macro_rules! py_command_str_ii {
    ($name:ident, $handler:path, $ret:ty) => {
        #[tauri::command(rename_all = "snake_case")]
        pub async fn $name(url: String, arg1: i64, arg2: i64) -> Result<$ret, AppError> {
            run_python_blocking(stringify!($name), move || $handler(&url, arg1, arg2))
                .await
                .map_err(AppError::from)?
                .into_result()
        }
    };
}

macro_rules! py_command_i64 {
    ($name:ident, $handler:path, $ret:ty) => {
        #[tauri::command(rename_all = "snake_case")]
        pub async fn $name(count: i64) -> Result<$ret, AppError> {
            run_python_blocking(stringify!($name), move || $handler(count))
                .await
                .map_err(AppError::from)?
                .into_result()
        }
    };
}

macro_rules! py_command_noargs {
    ($name:ident, $handler:path, $ret:ty) => {
        #[tauri::command(rename_all = "snake_case")]
        pub async fn $name() -> Result<$ret, AppError> {
            run_python_blocking(stringify!($name), || $handler())
                .await
                .map_err(AppError::from)?
                .into_result()
        }
    };
}

// ============================================================
// async 命令 — 通过 spawn_blocking 执行（宏展开）
// ============================================================

// 模式 1: (url: String)
py_command_str!(py_parse_video, crate::python::handler::parse_video, VideoParseResult);
py_command_str!(py_get_live_info, crate::python::handler::get_live_info, LiveInfoResult);
py_command_str!(py_get_user_profile, crate::python::handler::get_user_profile, UserProfileResult);
py_command_str!(py_get_post_stats, crate::python::handler::get_post_stats, PostStatsResult);
py_command_str!(py_start_live_record, crate::python::handler::start_live_record, LiveRecordResult);

// 模式 2: (url: String, cursor: Option<i64>, count: Option<i64>)
py_command_str_opts!(py_get_user_posts, crate::python::handler::get_user_posts, UserPostsResult);
py_command_str_opts!(py_get_mix_info, crate::python::handler::get_mix_info, MixInfoResult);
py_command_str_opts!(py_get_collects_video_list, crate::python::handler::get_collects_video_list, CollectsVideoListResult);
py_command_str_opts!(py_get_user_likes, crate::python::handler::get_user_likes, UserLikesResult);

// 模式 3: (url: String, arg1: i64, arg2: i64)
py_command_str_ii!(py_search_videos, crate::python::handler::search_videos, SearchResult);
py_command_str_ii!(py_get_following_list, crate::python::handler::get_following_list, FollowingListResult);
py_command_str_ii!(py_get_follower_list, crate::python::handler::get_follower_list, FollowerListResult);
py_command_str_ii!(py_get_comments, crate::python::handler::get_comments, CommentsResult);

// 模式 4: (count: i64)
py_command_i64!(py_get_tab_feed, crate::python::handler::get_tab_feed, TabFeedResult);

// 模式 5: ()
py_command_noargs!(py_get_collects_list, crate::python::handler::get_collects_list, CollectsListResult);
py_command_noargs!(py_get_following_live, crate::python::handler::get_following_live, FollowingLiveResult);

// ============================================================
// 特殊参数模式 — 手动定义
// ============================================================

#[tauri::command(rename_all = "snake_case")]
pub async fn py_get_music_collection(cursor: i64, count: i64) -> Result<MusicCollectionResult, AppError> {
    run_python_blocking("get_music_collection", move || {
        crate::python::handler::get_music_collection(cursor, count)
    }).await.map_err(AppError::from)?.into_result()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_get_follow_feed(cursor: i64, count: i64) -> Result<FollowFeedResult, AppError> {
    run_python_blocking("get_follow_feed", move || {
        crate::python::handler::get_follow_feed(cursor, count)
    }).await.map_err(AppError::from)?.into_result()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_get_friend_feed(cursor: i64, count: i64) -> Result<FriendFeedResult, AppError> {
    run_python_blocking("get_friend_feed", move || {
        crate::python::handler::get_friend_feed(cursor, count)
    }).await.map_err(AppError::from)?.into_result()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_get_comment_replies(url: String, comment_id: String, cursor: i64, count: i64) -> Result<CommentsResult, AppError> {
    run_python_blocking("get_comment_replies", move || {
        crate::python::handler::get_comment_replies(&url, &comment_id, cursor, count)
    }).await.map_err(AppError::from)?.into_result()
}

// ============================================================
// 下载类命令 — 保持返回 Value（由 task_service.rs 处理）
// ============================================================

#[tauri::command(rename_all = "snake_case")]
pub async fn py_start_download(mode: String, url: String) -> Result<Value, AppError> {
    run_python_blocking("start_download", move || {
        crate::python::handler::start_download(&mode, &url)
    }).await.map_err(AppError::from)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_download_music(play_url: String, title: String, author: String) -> Result<Value, AppError> {
    run_python_blocking("download_music", move || {
        crate::python::handler::download_music(&play_url, &title, &author)
    }).await.map_err(AppError::from)
}

// ============================================================
// sync 命令 — 轻量操作，无需 spawn_blocking
// ============================================================

#[tauri::command(rename_all = "snake_case")]
pub fn py_stop_live_record(task_id: String) -> Result<LiveStatusResult, AppError> {
    crate::python::handler::stop_live_record(&task_id)
        .map_err(|e| AppError::from(e.to_string()))?
        .into_result()
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_get_live_status() -> Result<LiveStatusResult, AppError> {
    crate::python::handler::get_live_status()
        .map_err(|e| AppError::from(e.to_string()))?
        .into_result()
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_test_emit() -> Result<Value, AppError> {
    crate::python::handler::test_emit().map_err(|e| AppError::from(e.to_string()))
}
