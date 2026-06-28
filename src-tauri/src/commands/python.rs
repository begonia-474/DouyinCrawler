//! Tauri commands — Python 桥接调用
//!
//! py_* 命令分为两类：
//! - async：通过 `run_python_blocking` 在 spawn_blocking 线程中执行，不阻塞 tokio 运行时
//! - sync：直接调用，用于轻量操作（状态查询、停止任务等）
//!
//! 使用 `py_command_*!` 宏消除样板代码（覆盖 ~70% 命令）。

use serde_json::Value;
use crate::python::runtime::run_python_blocking;

// ============================================================
// 命令宏定义（消除样板代码）
// ============================================================

macro_rules! py_command_str {
    ($name:ident, $handler:path) => {
        #[tauri::command(rename_all = "snake_case")]
        pub async fn $name(url: String) -> Result<Value, String> {
            run_python_blocking(stringify!($name), move || $handler(&url)).await
        }
    };
}

macro_rules! py_command_str_opts {
    ($name:ident, $handler:path) => {
        #[tauri::command(rename_all = "snake_case")]
        pub async fn $name(url: String, cursor: Option<i64>, count: Option<i64>) -> Result<Value, String> {
            run_python_blocking(stringify!($name), move || {
                $handler(&url, cursor.unwrap_or(0), count.unwrap_or(20))
            }).await
        }
    };
}

macro_rules! py_command_str_ii {
    ($name:ident, $handler:path) => {
        #[tauri::command(rename_all = "snake_case")]
        pub async fn $name(url: String, arg1: i64, arg2: i64) -> Result<Value, String> {
            run_python_blocking(stringify!($name), move || $handler(&url, arg1, arg2)).await
        }
    };
}

macro_rules! py_command_i64 {
    ($name:ident, $handler:path) => {
        #[tauri::command(rename_all = "snake_case")]
        pub async fn $name(count: i64) -> Result<Value, String> {
            run_python_blocking(stringify!($name), move || $handler(count)).await
        }
    };
}

macro_rules! py_command_noargs {
    ($name:ident, $handler:path) => {
        #[tauri::command(rename_all = "snake_case")]
        pub async fn $name() -> Result<Value, String> {
            run_python_blocking(stringify!($name), || $handler()).await
        }
    };
}

// ============================================================
// async 命令 — 通过 spawn_blocking 执行（宏展开）
// ============================================================

// 模式 1: (url: String) — 5 commands
py_command_str!(py_parse_video, crate::python::parse_video);
py_command_str!(py_get_live_info, crate::python::get_live_info);
py_command_str!(py_get_user_profile, crate::python::get_user_profile);
py_command_str!(py_get_post_stats, crate::python::get_post_stats);
py_command_str!(py_start_live_record, crate::python::start_live_record);

// 模式 2: (url: String, cursor: Option<i64>, count: Option<i64>) — 4 commands
py_command_str_opts!(py_get_user_posts, crate::python::get_user_posts);
py_command_str_opts!(py_get_mix_info, crate::python::get_mix_info);
py_command_str_opts!(py_get_collects_video_list, crate::python::get_collects_video_list);
py_command_str_opts!(py_get_user_likes, crate::python::get_user_likes);

// 模式 3: (url: String, arg1: i64, arg2: i64) — 4 commands
py_command_str_ii!(py_search_videos, crate::python::search_videos);
py_command_str_ii!(py_get_following_list, crate::python::get_following_list);
py_command_str_ii!(py_get_follower_list, crate::python::get_follower_list);
py_command_str_ii!(py_get_comments, crate::python::get_comments);

// 模式 4: (count: i64) — 1 command
py_command_i64!(py_get_tab_feed, crate::python::get_tab_feed);

// 模式 5: () — 2 commands
py_command_noargs!(py_get_collects_list, crate::python::get_collects_list);
py_command_noargs!(py_get_following_live, crate::python::get_following_live);

// ============================================================
// 特殊参数模式 — 手动定义（参数组合独特，不值得单独建宏）
// ============================================================

#[tauri::command(rename_all = "snake_case")]
pub async fn py_start_download(mode: String, url: String) -> Result<Value, String> {
    run_python_blocking("start_download", move || {
        crate::python::start_download(&mode, &url)
    }).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_download_music(play_url: String, title: String, author: String) -> Result<Value, String> {
    run_python_blocking("download_music", move || {
        crate::python::download_music(&play_url, &title, &author)
    }).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_get_comment_replies(url: String, comment_id: String, cursor: i64, count: i64) -> Result<Value, String> {
    run_python_blocking("get_comment_replies", move || {
        crate::python::get_comment_replies(&url, &comment_id, cursor, count)
    }).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_get_music_collection(cursor: i64, count: i64) -> Result<Value, String> {
    run_python_blocking("get_music_collection", move || {
        crate::python::get_music_collection(cursor, count)
    }).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_get_follow_feed(cursor: i64, count: i64) -> Result<Value, String> {
    run_python_blocking("get_follow_feed", move || {
        crate::python::get_follow_feed(cursor, count)
    }).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_get_friend_feed(cursor: i64, count: i64) -> Result<Value, String> {
    run_python_blocking("get_friend_feed", move || {
        crate::python::get_friend_feed(cursor, count)
    }).await
}

// ============================================================
// sync 命令 — 轻量操作，无需 spawn_blocking
// ============================================================

#[tauri::command(rename_all = "snake_case")]
pub fn py_stop_live_record(task_id: String) -> Result<Value, String> {
    crate::python::stop_live_record(&task_id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_get_live_status() -> Result<Value, String> {
    crate::python::get_live_status().map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_test_emit() -> Result<Value, String> {
    crate::python::test_emit().map_err(|e| e.to_string())
}
