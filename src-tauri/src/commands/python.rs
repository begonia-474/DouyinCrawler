//! Tauri commands — Python 桥接调用
//!
//! 26 个 py_* 命令，无状态注入，纯透传到 python:: 模块函数。

use serde_json::Value;

#[tauri::command(rename_all = "snake_case")]
pub fn py_parse_video(url: String) -> Result<Value, String> {
    crate::python::parse_video(&url).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_download_video(url: String) -> Result<Value, String> {
    crate::python::download_video(&url).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_get_live_info(url: String) -> Result<Value, String> {
    crate::python::get_live_info(&url).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_start_batch_download(url: String, download_type: String) -> Result<Value, String> {
    crate::python::start_batch_download(&url, &download_type).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_get_user_profile(url: String) -> Result<Value, String> {
    crate::python::get_user_profile(&url).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_get_user_posts(url: String, cursor: Option<i64>, count: Option<i64>) -> Result<Value, String> {
    crate::python::get_user_posts(&url, cursor.unwrap_or(0), count.unwrap_or(20)).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_search_videos(keyword: String, offset: i64, count: i64) -> Result<Value, String> {
    crate::python::search_videos(&keyword, offset, count).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_get_mix_info(url: String, cursor: Option<i64>, count: Option<i64>) -> Result<Value, String> {
    crate::python::get_mix_info(&url, cursor.unwrap_or(0), count.unwrap_or(20)).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_get_collects_list() -> Result<Value, String> {
    crate::python::get_collects_list().map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_get_collects_video_list(collects_id: String, cursor: Option<i64>, count: Option<i64>) -> Result<Value, String> {
    crate::python::get_collects_video_list(&collects_id, cursor.unwrap_or(0), count.unwrap_or(20)).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_get_following_list(url: String, offset: i64, count: i64) -> Result<Value, String> {
    crate::python::get_following_list(&url, offset, count).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_get_follower_list(url: String, offset: i64, count: i64) -> Result<Value, String> {
    crate::python::get_follower_list(&url, offset, count).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_get_music_collection(cursor: i64, count: i64) -> Result<Value, String> {
    crate::python::get_music_collection(cursor, count).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_download_music(play_url: String, title: String, author: String) -> Result<Value, String> {
    crate::python::download_music(&play_url, &title, &author).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_get_following_live() -> Result<Value, String> {
    crate::python::get_following_live().map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_get_comments(url: String, cursor: i64, count: i64) -> Result<Value, String> {
    crate::python::get_comments(&url, cursor, count).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_get_comment_replies(url: String, comment_id: String, cursor: i64, count: i64) -> Result<Value, String> {
    crate::python::get_comment_replies(&url, &comment_id, cursor, count).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_get_tab_feed(count: i64) -> Result<Value, String> {
    crate::python::get_tab_feed(count).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_get_follow_feed(cursor: i64, count: i64) -> Result<Value, String> {
    crate::python::get_follow_feed(cursor, count).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_get_friend_feed(cursor: i64, count: i64) -> Result<Value, String> {
    crate::python::get_friend_feed(cursor, count).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_get_user_likes(url: String, cursor: Option<i64>, count: Option<i64>) -> Result<Value, String> {
    crate::python::get_user_likes(&url, cursor.unwrap_or(0), count.unwrap_or(20)).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_get_post_stats(url: String) -> Result<Value, String> {
    crate::python::get_post_stats(&url).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn py_start_live_record(url: String) -> Result<Value, String> {
    crate::python::start_live_record(&url).map_err(|e| e.to_string())
}

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
