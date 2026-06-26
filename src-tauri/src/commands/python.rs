//! Tauri commands — Python 桥接调用
//!
//! py_* 命令分为两类：
//! - async：通过 `run_python_blocking` 在 spawn_blocking 线程中执行，不阻塞 tokio 运行时
//! - sync：直接调用，用于轻量操作（状态查询、停止任务等）

use serde_json::Value;
use crate::python::runtime::run_python_blocking;

// ============================================================
// async 命令 — 通过 spawn_blocking 执行，释放 tokio 线程
// ============================================================

#[tauri::command(rename_all = "snake_case")]
pub async fn py_parse_video(url: String) -> Result<Value, String> {
    run_python_blocking("parse_video", move || {
        crate::python::parse_video(&url)
    }).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_download_video(url: String) -> Result<Value, String> {
    run_python_blocking("download_video", move || {
        crate::python::download_video(&url)
    }).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_get_live_info(url: String) -> Result<Value, String> {
    run_python_blocking("get_live_info", move || {
        crate::python::get_live_info(&url)
    }).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_start_batch_download(url: String, download_type: String) -> Result<Value, String> {
    run_python_blocking("start_batch_download", move || {
        crate::python::start_batch_download(&url, &download_type)
    }).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_get_user_profile(url: String) -> Result<Value, String> {
    run_python_blocking("get_user_profile", move || {
        crate::python::get_user_profile(&url)
    }).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_get_user_posts(url: String, cursor: Option<i64>, count: Option<i64>) -> Result<Value, String> {
    run_python_blocking("get_user_posts", move || {
        crate::python::get_user_posts(&url, cursor.unwrap_or(0), count.unwrap_or(20))
    }).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_search_videos(keyword: String, offset: i64, count: i64) -> Result<Value, String> {
    run_python_blocking("search_videos", move || {
        crate::python::search_videos(&keyword, offset, count)
    }).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_get_mix_info(url: String, cursor: Option<i64>, count: Option<i64>) -> Result<Value, String> {
    run_python_blocking("get_mix_info", move || {
        crate::python::get_mix_info(&url, cursor.unwrap_or(0), count.unwrap_or(20))
    }).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_get_collects_list() -> Result<Value, String> {
    run_python_blocking("get_collects_list", || {
        crate::python::get_collects_list()
    }).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_get_collects_video_list(collects_id: String, cursor: Option<i64>, count: Option<i64>) -> Result<Value, String> {
    run_python_blocking("get_collects_video_list", move || {
        crate::python::get_collects_video_list(&collects_id, cursor.unwrap_or(0), count.unwrap_or(20))
    }).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_get_following_list(url: String, offset: i64, count: i64) -> Result<Value, String> {
    run_python_blocking("get_following_list", move || {
        crate::python::get_following_list(&url, offset, count)
    }).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_get_follower_list(url: String, offset: i64, count: i64) -> Result<Value, String> {
    run_python_blocking("get_follower_list", move || {
        crate::python::get_follower_list(&url, offset, count)
    }).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_get_music_collection(cursor: i64, count: i64) -> Result<Value, String> {
    run_python_blocking("get_music_collection", move || {
        crate::python::get_music_collection(cursor, count)
    }).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_download_music(play_url: String, title: String, author: String) -> Result<Value, String> {
    run_python_blocking("download_music", move || {
        crate::python::download_music(&play_url, &title, &author)
    }).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_get_following_live() -> Result<Value, String> {
    run_python_blocking("get_following_live", || {
        crate::python::get_following_live()
    }).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_get_comments(url: String, cursor: i64, count: i64) -> Result<Value, String> {
    run_python_blocking("get_comments", move || {
        crate::python::get_comments(&url, cursor, count)
    }).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_get_comment_replies(url: String, comment_id: String, cursor: i64, count: i64) -> Result<Value, String> {
    run_python_blocking("get_comment_replies", move || {
        crate::python::get_comment_replies(&url, &comment_id, cursor, count)
    }).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_get_tab_feed(count: i64) -> Result<Value, String> {
    run_python_blocking("get_tab_feed", move || {
        crate::python::get_tab_feed(count)
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

#[tauri::command(rename_all = "snake_case")]
pub async fn py_get_user_likes(url: String, cursor: Option<i64>, count: Option<i64>) -> Result<Value, String> {
    run_python_blocking("get_user_likes", move || {
        crate::python::get_user_likes(&url, cursor.unwrap_or(0), count.unwrap_or(20))
    }).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_get_post_stats(url: String) -> Result<Value, String> {
    run_python_blocking("get_post_stats", move || {
        crate::python::get_post_stats(&url)
    }).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn py_start_live_record(url: String) -> Result<Value, String> {
    run_python_blocking("start_live_record", move || {
        crate::python::start_live_record(&url)
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
