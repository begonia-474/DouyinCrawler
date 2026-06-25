//! Python 处理器模块
//!
//! 封装对 core.handler 的调用

use pyo3::prelude::*;
use serde_json::Value;
use log::info;

/// 解析视频信息
pub fn parse_video(url: &str) -> PyResult<Value> {
    info!("[handler.rs] parse_video 开始, url={}", &url[..url.len().min(60)]);
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method1("parse_video", (url,))?;
        info!("[handler.rs] parse_video Python 调用完成");

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;
        info!("[handler.rs] parse_video JSON 序列化完成, 长度={}", json_str.len());

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 下载单个视频
pub fn download_video(url: &str) -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method1("download_video", (url,))?;

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 获取直播信息
pub fn get_live_info(url: &str) -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method1("get_live_info", (url,))?;

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 开始批量下载
pub fn start_batch_download(url: &str, download_type: &str) -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method1("start_batch_download", (url, download_type))?;

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 获取用户信息
pub fn get_user_profile(url: &str) -> PyResult<Value> {
    info!("[handler.rs] get_user_profile 开始, url={}", &url[..url.len().min(60)]);
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method1("get_user_profile", (url,))?;
        info!("[handler.rs] get_user_profile Python 调用完成");

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;
        info!("[handler.rs] get_user_profile JSON 序列化完成, 长度={}", json_str.len());

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 获取用户作品列表（单页）
pub fn get_user_posts(url: &str, cursor: i64, count: i64) -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method1("get_user_posts", (url, cursor, count))?;

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 搜索视频
pub fn search_videos(keyword: &str, offset: i64, count: i64) -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method1("search_videos", (keyword, offset, count))?;

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 获取合集信息（单页）
pub fn get_mix_info(url: &str, cursor: i64, count: i64) -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method1("get_mix_info", (url, cursor, count))?;

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 获取收藏夹列表
pub fn get_collects_list() -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method0("get_collects_list")?;

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 获取收藏夹视频列表（单页）
pub fn get_collects_video_list(collects_id: &str, cursor: i64, count: i64) -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method1("get_collects_video_list", (collects_id, cursor, count))?;

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 获取关注列表
pub fn get_following_list(url: &str, offset: i64, count: i64) -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method1("get_following_list", (url, offset, count))?;

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 获取粉丝列表
pub fn get_follower_list(url: &str, offset: i64, count: i64) -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method1("get_follower_list", (url, offset, count))?;

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 获取音乐收藏
pub fn get_music_collection(cursor: i64, count: i64) -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method1("get_music_collection", (cursor, count))?;

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 下载音乐
pub fn download_music(play_url: &str, title: &str, author: &str) -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method1("download_music", (play_url, title, author))?;

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 获取关注直播列表
pub fn get_following_live() -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method0("get_following_live")?;

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 获取评论
pub fn get_comments(url: &str, cursor: i64, count: i64) -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method1("get_comments", (url, cursor, count))?;

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 获取评论回复
pub fn get_comment_replies(url: &str, comment_id: &str, cursor: i64, count: i64) -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method1("get_comment_replies", (url, comment_id, cursor, count))?;

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 获取推荐 Feed
pub fn get_tab_feed(count: i64) -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method1("get_tab_feed", (count,))?;

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 获取关注 Feed
pub fn get_follow_feed(cursor: i64, count: i64) -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method1("get_follow_feed", (cursor, count))?;

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 获取好友 Feed
pub fn get_friend_feed(cursor: i64, count: i64) -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method1("get_friend_feed", (cursor, count))?;

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 获取用户点赞列表（单页）
pub fn get_user_likes(url: &str, cursor: i64, count: i64) -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method1("get_user_likes", (url, cursor, count))?;

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 获取作品统计
pub fn get_post_stats(url: &str) -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method1("get_post_stats", (url,))?;

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 开始直播录制
pub fn start_live_record(url: &str) -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method1("start_live_record", (url,))?;

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 停止直播录制
pub fn stop_live_record(task_id: &str) -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method1("stop_live_record", (task_id,))?;

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 获取直播录制状态
pub fn get_live_status() -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method0("get_live_status")?;

        // 转换为 JSON
        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 获取批量下载状态
pub fn get_batch_status() -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method0("get_batch_status")?;

        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}

/// 测试 Tauri 事件发射
pub fn test_emit() -> PyResult<Value> {
    Python::with_gil(|py| {
        let handler = py.import_bound("core.py_bridge")?;
        let result = handler.call_method0("test_emit")?;

        let json = py.import_bound("json")?;
        let json_str: String = json.call_method1("dumps", (result,))?.extract()?;

        Ok(serde_json::from_str(&json_str).unwrap_or_default())
    })
}
