//! Python 处理器模块
//!
//! 封装对 core.py_bridge 的调用
//! 所有函数通过 call_py_json 通用桥接调用 Python，避免重复代码

use pyo3::prelude::*;
use pyo3::types::PyTuple;
use serde_json::Value;
use log::info;

use super::bridge::py_to_json_value;

/// 通用 Python 调用桥接：调用 core.py_bridge.method(*args)，返回 JSON Value
fn call_py_json(method: &str, args: impl IntoPy<Py<PyTuple>>) -> PyResult<Value> {
    Python::with_gil(|py| {
        let module = py.import_bound("core.py_bridge")?;
        let result = module.call_method1(method, args)?;
        py_to_json_value(&result)
    })
}

// ============================================================
// 视频相关
// ============================================================

/// 解析视频信息
pub fn parse_video(url: &str) -> PyResult<Value> {
    info!("[handler.rs] parse_video, url={}", &url[..url.len().min(60)]);
    call_py_json("parse_video", (url,))
}

/// 下载单个视频（仅下载，不写 DB，返回结果供 Rust 事务性完成）
pub fn download_video(url: &str) -> PyResult<Value> {
    info!("[handler.rs] download_video, url={}", &url[..url.len().min(60)]);
    call_py_json("download_video", (url,))
}

/// 获取作品统计
pub fn get_post_stats(url: &str) -> PyResult<Value> {
    call_py_json("get_post_stats", (url,))
}

// ============================================================
// 用户相关
// ============================================================

/// 获取用户信息
pub fn get_user_profile(url: &str) -> PyResult<Value> {
    info!("[handler.rs] get_user_profile, url={}", &url[..url.len().min(60)]);
    call_py_json("get_user_profile", (url,))
}

/// 获取用户作品列表（单页）
pub fn get_user_posts(url: &str, cursor: i64, count: i64) -> PyResult<Value> {
    call_py_json("get_user_posts", (url, cursor, count))
}

/// 获取用户点赞列表（单页）
pub fn get_user_likes(url: &str, cursor: i64, count: i64) -> PyResult<Value> {
    call_py_json("get_user_likes", (url, cursor, count))
}

/// 获取关注列表
pub fn get_following_list(url: &str, offset: i64, count: i64) -> PyResult<Value> {
    call_py_json("get_following_list", (url, offset, count))
}

/// 获取粉丝列表
pub fn get_follower_list(url: &str, offset: i64, count: i64) -> PyResult<Value> {
    call_py_json("get_follower_list", (url, offset, count))
}

// ============================================================
// 收藏夹 / 合集
// ============================================================

/// 获取收藏夹列表
pub fn get_collects_list() -> PyResult<Value> {
    call_py_json("get_collects_list", ())
}

/// 获取收藏夹视频列表（单页）
pub fn get_collects_video_list(collects_id: &str, cursor: i64, count: i64) -> PyResult<Value> {
    call_py_json("get_collects_video_list", (collects_id, cursor, count))
}

/// 获取合集信息（单页）
pub fn get_mix_info(url: &str, cursor: i64, count: i64) -> PyResult<Value> {
    call_py_json("get_mix_info", (url, cursor, count))
}

/// 统一下载入口（通过 mode 分发）
pub fn start_download(mode: &str, url: &str) -> PyResult<Value> {
    call_py_json("start_download", (mode, url))
}

/// 批量下载（不写 task DB 表，返回结果供 Rust 处理）
pub fn download_batch(mode: &str, url: &str) -> PyResult<Value> {
    info!("[handler.rs] download_batch, mode={}, url={}", mode, &url[..url.len().min(60)]);
    call_py_json("download_batch", (mode, url))
}

// ============================================================
// 搜索
// ============================================================

/// 搜索视频
pub fn search_videos(keyword: &str, offset: i64, count: i64) -> PyResult<Value> {
    call_py_json("search_videos", (keyword, offset, count))
}

// ============================================================
// 信息流
// ============================================================

/// 获取推荐 Feed
pub fn get_tab_feed(count: i64) -> PyResult<Value> {
    call_py_json("get_tab_feed", (count,))
}

/// 获取关注 Feed
pub fn get_follow_feed(cursor: i64, count: i64) -> PyResult<Value> {
    call_py_json("get_follow_feed", (cursor, count))
}

/// 获取好友 Feed
pub fn get_friend_feed(cursor: i64, count: i64) -> PyResult<Value> {
    call_py_json("get_friend_feed", (cursor, count))
}

// ============================================================
// 直播
// ============================================================

/// 获取直播信息
pub fn get_live_info(url: &str) -> PyResult<Value> {
    call_py_json("get_live_info", (url,))
}

/// 开始直播录制
pub fn start_live_record(url: &str) -> PyResult<Value> {
    call_py_json("start_live_record", (url,))
}

/// 停止直播录制
pub fn stop_live_record(task_id: &str) -> PyResult<Value> {
    call_py_json("stop_live_record", (task_id,))
}

/// 获取直播录制状态
pub fn get_live_status() -> PyResult<Value> {
    call_py_json("get_live_status", ())
}

/// 获取关注直播列表
pub fn get_following_live() -> PyResult<Value> {
    call_py_json("get_following_live", ())
}

// ============================================================
// 音乐
// ============================================================

/// 获取音乐收藏
pub fn get_music_collection(cursor: i64, count: i64) -> PyResult<Value> {
    call_py_json("get_music_collection", (cursor, count))
}

/// 下载音乐
pub fn download_music(play_url: &str, title: &str, author: &str) -> PyResult<Value> {
    call_py_json("download_music", (play_url, title, author))
}

/// 下载全部音乐（不写 task DB 表，返回结果供 Rust 处理）
pub fn download_music_batch(url: &str) -> PyResult<Value> {
    info!("[handler.rs] download_music_batch, url={}", &url[..url.len().min(60)]);
    call_py_json("download_music_batch", (url,))
}

// ============================================================
// 评论
// ============================================================

/// 获取评论
pub fn get_comments(url: &str, cursor: i64, count: i64) -> PyResult<Value> {
    call_py_json("get_comments", (url, cursor, count))
}

/// 获取评论回复
pub fn get_comment_replies(url: &str, comment_id: &str, cursor: i64, count: i64) -> PyResult<Value> {
    call_py_json("get_comment_replies", (url, comment_id, cursor, count))
}

// ============================================================
// 工具
// ============================================================

/// 测试 Tauri 事件发射
pub fn test_emit() -> PyResult<Value> {
    call_py_json("test_emit", ())
}
