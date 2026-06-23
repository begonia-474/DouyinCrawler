mod db;
mod proxy;

use db::{
    AppConfig, Database, DownloadRecord, DownloadStats, LiveRecord,
    MusicCollection, NewDownloadRecord, NewLiveRecord, NewMusicCollection,
    UserInfo, VideoInfo, VideoStats, UserStats,
};
use proxy::PythonProxy;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::{Manager, State};
use serde_json::Value;

// ============================================================
// Tauri Commands - 配置
// ============================================================

#[tauri::command]
fn get_config(db: State<'_, Database>) -> Result<AppConfig, String> {
    db.get_config().map_err(|e| e.to_string())
}

#[tauri::command]
fn set_config(
    db: State<'_, Database>,
    updates: HashMap<String, String>,
) -> Result<(), String> {
    db.set_config(&updates).map_err(|e| e.to_string())
}

// ============================================================
// Tauri Commands - HTTP 代理
// ============================================================

#[tauri::command]
async fn proxy_post(
    proxy: State<'_, PythonProxy>,
    path: String,
    body: Value,
) -> Result<Value, String> {
    proxy.post(&path, body).await
}

#[tauri::command]
async fn proxy_get(
    proxy: State<'_, PythonProxy>,
    path: String,
) -> Result<Value, String> {
    proxy.get(&path).await
}

#[tauri::command]
async fn health_check(proxy: State<'_, PythonProxy>) -> Result<bool, String> {
    Ok(proxy.health_check().await)
}

// ============================================================
// Tauri Commands - 数据库读取
// ============================================================

#[tauri::command]
fn get_downloads(
    db: State<'_, Database>,
    limit: i64,
    offset: i64,
    status: Option<String>,
    download_type: Option<String>,
) -> Result<Vec<DownloadRecord>, String> {
    db.get_downloads(limit, offset, status, download_type)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_download_stats(db: State<'_, Database>) -> Result<DownloadStats, String> {
    db.get_download_stats().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_live_records(
    db: State<'_, Database>,
    limit: i64,
    offset: i64,
) -> Result<Vec<LiveRecord>, String> {
    db.get_live_records(limit, offset)
        .map_err(|e| e.to_string())
}

// === 数据库读取 - video_info / user_info ===

#[tauri::command]
fn get_videos(
    db: State<'_, Database>,
    limit: i64,
    offset: i64,
    keyword: Option<String>,
    author_sec_uid: Option<String>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    post_type: Option<String>,
) -> Result<Vec<VideoInfo>, String> {
    db.get_videos(limit, offset, keyword, author_sec_uid, sort_by, sort_order, post_type)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_video_count(
    db: State<'_, Database>,
    keyword: Option<String>,
    author_sec_uid: Option<String>,
    post_type: Option<String>,
) -> Result<i64, String> {
    db.get_video_count(keyword, author_sec_uid, post_type)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_users(
    db: State<'_, Database>,
    limit: i64,
    offset: i64,
    keyword: Option<String>,
    sort_by: Option<String>,
    sort_order: Option<String>,
) -> Result<Vec<UserInfo>, String> {
    db.get_users(limit, offset, keyword, sort_by, sort_order)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_user_count(
    db: State<'_, Database>,
    keyword: Option<String>,
) -> Result<i64, String> {
    db.get_user_count(keyword)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_user_by_sec_uid(
    db: State<'_, Database>,
    sec_user_id: String,
) -> Result<Option<UserInfo>, String> {
    db.get_user_by_sec_uid(&sec_user_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_video_stats(db: State<'_, Database>) -> Result<VideoStats, String> {
    db.get_video_stats().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_user_stats(db: State<'_, Database>) -> Result<UserStats, String> {
    db.get_user_stats().map_err(|e| e.to_string())
}

fn delete_local_path(path: Option<String>) -> Result<(), String> {
    let Some(path) = path.filter(|value| !value.trim().is_empty()) else {
        return Ok(());
    };
    let local_path = Path::new(&path);
    if !local_path.exists() {
        return Ok(());
    }
    if local_path.is_dir() {
        std::fs::remove_dir_all(local_path).map_err(|e| format!("删除本地文件夹失败: {}", e))
    } else {
        std::fs::remove_file(local_path).map_err(|e| format!("删除本地文件失败: {}", e))
    }
}

// ============================================================
// Tauri Commands - 代理 + 自动写入数据库
// ============================================================

#[tauri::command]
async fn download_one_and_save(
    proxy: State<'_, PythonProxy>,
    db: State<'_, Database>,
    url: String,
) -> Result<Value, String> {
    // 1. 调用 Python 下载
    let body = serde_json::json!({ "url": url });
    let result = proxy.post("/api/download/one", body).await?;

    // 2. 检查是否成功
    let success = result.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
    if !success {
        return Ok(result);
    }

    // 3. 解析结果（Python safe_call 包了一层 data）
    let data = result.get("data").unwrap_or(&result);

    let download_type = data.get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("video")
        .to_string();

    let file_path = if download_type == "images" {
        data.get("paths")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    } else {
        data.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };

    let detail = data.get("detail");

    // 辅助闭包：从 detail 中取值
    let dget = |key: &str| detail.and_then(|d| d.get(key));
    let dstr_opt = |key: &str| dget(key).and_then(|v| v.as_str()).map(String::from);
    let di64 = |key: &str| dget(key).and_then(|v| v.as_i64()).unwrap_or(0);
    let di32 = |key: &str| dget(key).and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    let aweme_id = dstr_opt("aweme_id");
    let author_nickname = dstr_opt("author_nickname");
    let author_sec_uid = dstr_opt("author_sec_uid");

    // 4. 写入 download_history
    let record = NewDownloadRecord {
        aweme_id: aweme_id.clone(),
        download_type,
        title: dstr_opt("desc"),
        author_nickname: author_nickname.clone(),
        author_sec_uid: author_sec_uid.clone(),
        file_path: Some(file_path),
        file_size: 0,
        cover_url: dstr_opt("cover_url"),
        status: "completed".to_string(),
        error_msg: None,
    };
    if let Err(e) = db.save_download(&record) {
        eprintln!("[Rust] 保存下载记录失败: {}", e);
    }

    // 5. 写入 video_info
    if let Some(aweme_id) = aweme_id.as_ref().filter(|s| !s.is_empty()) {
        let video = VideoInfo {
            aweme_id: aweme_id.clone(),
            desc: dstr_opt("desc"),
            aweme_type: di32("aweme_type"),
            author_nickname: author_nickname.clone(),
            author_sec_uid: author_sec_uid.clone(),
            author_uid: dstr_opt("author_uid"),
            create_time: dget("create_time").and_then(|v| v.as_i64()),
            duration: di32("duration"),
            video_url: dstr_opt("video_url"),
            cover_url: dstr_opt("cover_url"),
            music_title: dstr_opt("music_title"),
            digg_count: di64("digg_count"),
            comment_count: di64("comment_count"),
            share_count: di64("share_count"),
            collect_count: di64("collect_count"),
            mix_id: dstr_opt("mix_id"),
            mix_name: dstr_opt("mix_name"),
            author_nickname_raw: dstr_opt("author_nickname_raw"),
            author_short_id: dstr_opt("author_short_id"),
            author_unique_id: dstr_opt("author_unique_id"),
            desc_raw: dstr_opt("desc_raw"),
            is_ads: di32("is_ads"),
            is_story: di32("is_story"),
            is_top: di32("is_top"),
            is_long_video: di32("is_long_video"),
            video_bit_rate: dstr_opt("video_bit_rate"),
            animated_cover: dstr_opt("animated_cover"),
            private_status: di32("private_status"),
            is_delete: di32("is_delete"),
            music_author: dstr_opt("music_author"),
            music_author_raw: dstr_opt("music_author_raw"),
            music_duration: di32("music_duration"),
            music_id: dstr_opt("music_id"),
            music_mid: dstr_opt("music_mid"),
            pgc_author: dstr_opt("pgc_author"),
            pgc_author_title: dstr_opt("pgc_author_title"),
            pgc_music_type: di32("pgc_music_type"),
            music_status: di32("music_status"),
            music_owner_handle: dstr_opt("music_owner_handle"),
            music_owner_id: dstr_opt("music_owner_id"),
            music_owner_nickname: dstr_opt("music_owner_nickname"),
            music_play_url: dstr_opt("music_play_url"),
            is_commerce_music: di32("is_commerce_music"),
            mix_desc: dstr_opt("mix_desc"),
            mix_create_time: di64("mix_create_time"),
            mix_pic_type: di32("mix_pic_type"),
            mix_type: di32("mix_type"),
            mix_share_url: dstr_opt("mix_share_url"),
            can_comment: di32("can_comment"),
            can_forward: di32("can_forward"),
            can_share: di32("can_share"),
            download_setting: di32("download_setting"),
            allow_douplus: di32("allow_douplus"),
            allow_share: di32("allow_share"),
            admire_count: di64("admire_count"),
            hashtag_ids: dstr_opt("hashtag_ids"),
            hashtag_names: dstr_opt("hashtag_names"),
            images: dstr_opt("images"),
            region: dstr_opt("region"),
            is_prohibited: di32("is_prohibited"),
        };
        if let Err(e) = db.save_video(&video) {
            eprintln!("[Rust] 保存视频信息失败: {}", e);
        }
    }

    // 6. 写入 user_info（先尝试获取完整用户资料，失败则用 detail 中的部分数据）
    if let Some(sec_uid) = author_sec_uid.as_ref().filter(|s| !s.is_empty()) {
        let user = fetch_and_build_user_info(&proxy, sec_uid, detail).await;
        if let Err(e) = db.save_user(&user) {
            eprintln!("[Rust] 保存用户信息失败: {}", e);
        }
    }

    Ok(result)
}

/// 保存单个视频的下载记录和视频信息
async fn save_download_result(
    db: &Database,
    proxy: &PythonProxy,
    path: &str,
    detail: &Value,
    download_type: &str,
) {
    let dget = |key: &str| detail.get(key);
    let dstr_opt = |key: &str| dget(key).and_then(|v| v.as_str()).map(String::from);
    let di64 = |key: &str| dget(key).and_then(|v| v.as_i64()).unwrap_or(0);
    let di32 = |key: &str| dget(key).and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    let aweme_id = dstr_opt("aweme_id");
    let author_nickname = dstr_opt("author_nickname");
    let author_sec_uid = dstr_opt("author_sec_uid");

    // 写入 download_history
    let record = NewDownloadRecord {
        aweme_id: aweme_id.clone(),
        download_type: download_type.to_string(),
        title: dstr_opt("desc"),
        author_nickname: author_nickname.clone(),
        author_sec_uid: author_sec_uid.clone(),
        file_path: Some(path.to_string()),
        file_size: 0,
        cover_url: dstr_opt("cover_url"),
        status: "completed".to_string(),
        error_msg: None,
    };
    if let Err(e) = db.save_download(&record) {
        eprintln!("[Rust] 保存下载记录失败: {}", e);
    }

    // 写入 video_info
    if let Some(aweme_id) = aweme_id.as_ref().filter(|s| !s.is_empty()) {
        let video = VideoInfo {
            aweme_id: aweme_id.clone(),
            desc: dstr_opt("desc"),
            aweme_type: di32("aweme_type"),
            author_nickname: author_nickname.clone(),
            author_sec_uid: author_sec_uid.clone(),
            author_uid: dstr_opt("author_uid"),
            create_time: dget("create_time").and_then(|v| v.as_i64()),
            duration: di32("duration"),
            video_url: dstr_opt("video_url"),
            cover_url: dstr_opt("cover_url"),
            music_title: dstr_opt("music_title"),
            digg_count: di64("digg_count"),
            comment_count: di64("comment_count"),
            share_count: di64("share_count"),
            collect_count: di64("collect_count"),
            mix_id: dstr_opt("mix_id"),
            mix_name: dstr_opt("mix_name"),
            author_nickname_raw: dstr_opt("author_nickname_raw"),
            author_short_id: dstr_opt("author_short_id"),
            author_unique_id: dstr_opt("author_unique_id"),
            desc_raw: dstr_opt("desc_raw"),
            is_ads: di32("is_ads"),
            is_story: di32("is_story"),
            is_top: di32("is_top"),
            is_long_video: di32("is_long_video"),
            video_bit_rate: dstr_opt("video_bit_rate"),
            animated_cover: dstr_opt("animated_cover"),
            private_status: di32("private_status"),
            is_delete: di32("is_delete"),
            music_author: dstr_opt("music_author"),
            music_author_raw: dstr_opt("music_author_raw"),
            music_duration: di32("music_duration"),
            music_id: dstr_opt("music_id"),
            music_mid: dstr_opt("music_mid"),
            pgc_author: dstr_opt("pgc_author"),
            pgc_author_title: dstr_opt("pgc_author_title"),
            pgc_music_type: di32("pgc_music_type"),
            music_status: di32("music_status"),
            music_owner_handle: dstr_opt("music_owner_handle"),
            music_owner_id: dstr_opt("music_owner_id"),
            music_owner_nickname: dstr_opt("music_owner_nickname"),
            music_play_url: dstr_opt("music_play_url"),
            is_commerce_music: di32("is_commerce_music"),
            mix_desc: dstr_opt("mix_desc"),
            mix_create_time: di64("mix_create_time"),
            mix_pic_type: di32("mix_pic_type"),
            mix_type: di32("mix_type"),
            mix_share_url: dstr_opt("mix_share_url"),
            can_comment: di32("can_comment"),
            can_forward: di32("can_forward"),
            can_share: di32("can_share"),
            download_setting: di32("download_setting"),
            allow_douplus: di32("allow_douplus"),
            allow_share: di32("allow_share"),
            admire_count: di64("admire_count"),
            hashtag_ids: dstr_opt("hashtag_ids"),
            hashtag_names: dstr_opt("hashtag_names"),
            images: dstr_opt("images"),
            region: dstr_opt("region"),
            is_prohibited: di32("is_prohibited"),
        };
        if let Err(e) = db.save_video(&video) {
            eprintln!("[Rust] 保存视频信息失败: {}", e);
        }
    }

    // 写入 user_info
    if let Some(sec_uid) = author_sec_uid.as_ref().filter(|s| !s.is_empty()) {
        let user = fetch_and_build_user_info(proxy, sec_uid, Some(detail)).await;
        if let Err(e) = db.save_user(&user) {
            eprintln!("[Rust] 保存用户信息失败: {}", e);
        }
    }
}

/// 从 detail JSON 中提取字符串值
fn detail_str(detail: &Option<&Value>, key: &str) -> Option<String> {
    detail.and_then(|d| d.get(key)).and_then(|v| v.as_str()).map(String::from)
}

/// 从 detail JSON 中提取 i64 值
fn detail_i64(detail: &Option<&Value>, key: &str) -> i64 {
    detail.and_then(|d| d.get(key)).and_then(|v| v.as_i64()).unwrap_or(0)
}

/// 尝试通过 /api/user/profile 获取完整用户资料，失败则用 detail 中的部分数据
async fn fetch_and_build_user_info(
    proxy: &PythonProxy,
    sec_uid: &str,
    detail: Option<&Value>,
) -> UserInfo {
    // 尝试获取完整用户资料
    let url = format!("https://www.douyin.com/user/{}", sec_uid);
    let body = serde_json::json!({ "url": url });
    if let Ok(result) = proxy.post("/api/user/profile", body).await {
        let success = result.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
        if success {
            let profile = result.get("data")
                .and_then(|d| d.get("profile"))
                .cloned()
                .unwrap_or(result);
            let pget = |key: &str| profile.get(key);
            let pstr = |key: &str| pget(key).and_then(|v| v.as_str()).map(String::from);
            let pi64 = |key: &str| pget(key).and_then(|v| v.as_i64()).unwrap_or(0);
            let pi32 = |key: &str| pget(key).and_then(|v| v.as_i64()).unwrap_or(0) as i32;

            return UserInfo {
                sec_user_id: sec_uid.to_string(),
                nickname: pstr("nickname").or_else(|| detail_str(&detail, "author_nickname")),
                uid: pstr("uid").or_else(|| detail_str(&detail, "author_uid")),
                avatar_url: pstr("avatar_url").or_else(|| detail_str(&detail, "author_avatar_url")),
                unique_id: pstr("unique_id").or_else(|| detail_str(&detail, "author_unique_id")),
                signature: pstr("signature").or_else(|| detail_str(&detail, "author_signature")),
                aweme_count: {
                    let v = pi64("aweme_count");
                    if v > 0 { v } else { detail_i64(&detail, "author_aweme_count") }
                },
                follower_count: {
                    let v = pi64("follower_count");
                    if v > 0 { v } else { detail_i64(&detail, "author_follower_count") }
                },
                following_count: {
                    let v = pi64("following_count");
                    if v > 0 { v } else { detail_i64(&detail, "author_following_count") }
                },
                total_favorited: {
                    let v = pi64("total_favorited");
                    if v > 0 { v } else { detail_i64(&detail, "author_total_favorited") }
                },
                ip_location: pstr("ip_location").or_else(|| detail_str(&detail, "author_ip_location")),
                live_status: pi32("live_status"),
                room_id: pstr("room_id"),
                city: pstr("city"),
                country: pstr("country"),
                favoriting_count: pi64("favoriting_count"),
                gender: pi32("gender"),
                is_ban: pi32("is_ban"),
                is_block: pi32("is_block"),
                is_blocked: pi32("is_blocked"),
                is_star: pi32("is_star"),
                mix_count: pi32("mix_count"),
                mplatform_followers_count: pi64("mplatform_followers_count"),
                nickname_raw: pstr("nickname_raw").or_else(|| detail_str(&detail, "author_nickname_raw")),
                school_name: pstr("school_name"),
                short_id: pstr("short_id").or_else(|| detail_str(&detail, "author_short_id")),
                signature_raw: pstr("signature_raw"),
                user_age: pi32("user_age"),
                custom_verify: pstr("custom_verify"),
            };
        }
    }

    // 回退：用 detail 中的部分数据
    UserInfo {
        sec_user_id: sec_uid.to_string(),
        nickname: detail_str(&detail, "author_nickname"),
        uid: detail_str(&detail, "author_uid"),
        avatar_url: detail_str(&detail, "author_avatar_url"),
        unique_id: detail_str(&detail, "author_unique_id"),
        signature: detail_str(&detail, "author_signature"),
        aweme_count: detail_i64(&detail, "author_aweme_count"),
        follower_count: detail_i64(&detail, "author_follower_count"),
        following_count: detail_i64(&detail, "author_following_count"),
        total_favorited: detail_i64(&detail, "author_total_favorited"),
        ip_location: detail_str(&detail, "author_ip_location"),
        live_status: 0,
        room_id: None,
        city: None, country: None, favoriting_count: 0, gender: 0,
        is_ban: 0, is_block: 0, is_blocked: 0, is_star: 0,
        mix_count: 0, mplatform_followers_count: 0,
        nickname_raw: detail_str(&detail, "author_nickname_raw"),
        school_name: None, short_id: detail_str(&detail, "author_short_id"),
        signature_raw: None, user_age: 0, custom_verify: None,
    }
}

#[tauri::command]
async fn download_music_and_save(
    proxy: State<'_, PythonProxy>,
    db: State<'_, Database>,
    play_url: String,
    title: String,
    author: String,
) -> Result<Value, String> {
    // 1. 调用 Python 下载音乐
    let body = serde_json::json!({
        "play_url": play_url,
        "title": title,
        "author": author,
    });
    let result = proxy.post("/api/music/download", body).await?;

    // 2. 检查是否成功
    let success = result.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
    if !success {
        return Ok(result);
    }

    // 3. 提取路径并保存（Python safe_call 包了一层 data）
    let data = result.get("data").unwrap_or(&result);
    let file_path = data.get("path")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let record = NewDownloadRecord {
        aweme_id: None,
        download_type: "music".to_string(),
        title: Some(title),
        author_nickname: Some(author),
        author_sec_uid: None,
        file_path: Some(file_path),
        file_size: 0,
        cover_url: None,
        status: "completed".to_string(),
        error_msg: None,
    };

    if let Err(e) = db.save_download(&record) {
        eprintln!("[Rust] 保存音乐下载记录失败: {}", e);
    }

    Ok(result)
}

#[tauri::command]
async fn download_batch_and_save(
    proxy: State<'_, PythonProxy>,
    db: State<'_, Database>,
    path: String,
    body: Value,
) -> Result<Value, String> {
    // 1. 调用 Python 批量下载
    let result = proxy.post(&path, body).await?;

    // 2. 检查是否成功
    let success = result.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
    if !success {
        return Ok(result);
    }

    // 3. 解析结果并保存
    let data = result.get("data").unwrap_or(&result);
    let results = data.get("results").and_then(|v| v.as_array());

    if let Some(results) = results {
        // 确定下载类型
        let download_type = if path.contains("/posts") {
            "user_post"
        } else if path.contains("/likes") {
            "user_like"
        } else if path.contains("/mix") {
            "mix"
        } else if path.contains("/collects") {
            "collects"
        } else {
            "batch"
        };

        for item in results {
            let file_path = item.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let detail = item.get("detail").unwrap_or(&serde_json::Value::Null);
            save_download_result(&db, &proxy, file_path, detail, download_type).await;
        }
    }

    Ok(result)
}

// ============================================================
// Tauri Commands - 数据库写入
// ============================================================

#[tauri::command]
fn save_download_record(
    db: State<'_, Database>,
    record: NewDownloadRecord,
) -> Result<i64, String> {
    db.save_download(&record).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_live_record_record(
    db: State<'_, Database>,
    record: NewLiveRecord,
) -> Result<i64, String> {
    db.save_live_record(&record).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_user_info(
    db: State<'_, Database>,
    user: UserInfo,
) -> Result<(), String> {
    db.save_user(&user).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_video_info(
    db: State<'_, Database>,
    video: VideoInfo,
) -> Result<(), String> {
    db.save_video(&video).map_err(|e| e.to_string())
}

#[tauri::command]
fn is_video_downloaded(
    db: State<'_, Database>,
    aweme_id: String,
) -> Result<bool, String> {
    db.is_video_downloaded(&aweme_id).map_err(|e| e.to_string())
}

// === 音乐收藏 ===

#[tauri::command]
fn save_music_collection(
    db: State<'_, Database>,
    music: NewMusicCollection,
) -> Result<(), String> {
    db.save_music_collection(&music).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_music_collection_batch(
    db: State<'_, Database>,
    musics: Vec<NewMusicCollection>,
) -> Result<(), String> {
    db.save_music_collection_batch(&musics).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_music_collection(
    db: State<'_, Database>,
    limit: i64,
    offset: i64,
    keyword: Option<String>,
    status: Option<String>,
) -> Result<Vec<MusicCollection>, String> {
    db.get_music_collection(limit, offset, keyword, status).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_music_collection_count(
    db: State<'_, Database>,
    keyword: Option<String>,
    status: Option<String>,
) -> Result<i64, String> {
    db.get_music_collection_count(keyword, status).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_music_file_path(
    db: State<'_, Database>,
    music_id: String,
    file_path: String,
) -> Result<(), String> {
    db.update_music_file_path(&music_id, &file_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_download_record(
    db: State<'_, Database>,
    id: i64,
    delete_file: bool,
) -> Result<(), String> {
    if delete_file {
        let file_path = db.get_download_file_path(id).map_err(|e| e.to_string())?;
        delete_local_path(file_path)?;
    }
    db.delete_download(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_live_record(
    db: State<'_, Database>,
    id: i64,
    delete_file: bool,
) -> Result<(), String> {
    if delete_file {
        let file_path = db.get_live_record_file_path(id).map_err(|e| e.to_string())?;
        delete_local_path(file_path)?;
    }
    db.delete_live_record(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_video_info(
    db: State<'_, Database>,
    aweme_id: String,
) -> Result<(), String> {
    db.delete_video(&aweme_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_user_info(
    db: State<'_, Database>,
    sec_user_id: String,
) -> Result<(), String> {
    db.delete_user(&sec_user_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_music_collection(
    db: State<'_, Database>,
    music_id: String,
    delete_file: bool,
) -> Result<(), String> {
    if delete_file {
        let file_path = db.get_music_file_path(&music_id).map_err(|e| e.to_string())?;
        delete_local_path(file_path)?;
    }
    db.delete_music_collection(&music_id).map_err(|e| e.to_string())
}

// ============================================================
// Entry Point
// ============================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db_path = resolve_db_path(app);
            let database = Database::open(&db_path)
                .expect("Failed to open database");
            app.manage(database);

            let proxy = PythonProxy::new(8765);
            app.manage(proxy);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 配置
            get_config,
            set_config,
            // 代理
            proxy_post,
            proxy_get,
            health_check,
            // 代理 + 自动写入数据库
            download_one_and_save,
            download_music_and_save,
            download_batch_and_save,
            // 数据库读取
            get_downloads,
            get_download_stats,
            get_live_records,
            get_videos,
            get_video_count,
            get_users,
            get_user_count,
            get_user_by_sec_uid,
            get_video_stats,
            get_user_stats,
            get_music_collection,
            get_music_collection_count,
            // 数据库写入
            save_download_record,
            save_live_record_record,
            save_user_info,
            save_video_info,
            save_music_collection,
            save_music_collection_batch,
            update_music_file_path,
            delete_download_record,
            delete_live_record,
            delete_video_info,
            delete_user_info,
            delete_music_collection,
            is_video_downloaded,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn resolve_db_path(app: &tauri::App) -> PathBuf {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_root = current_dir.parent().unwrap_or(&current_dir);
    let dev_path = project_root.join("data").join("douyin.db");

    if dev_path.exists() {
        return dev_path;
    }

    let current_data_path = current_dir.join("data").join("douyin.db");
    if current_data_path.exists() {
        return current_data_path;
    }

    if let Ok(data_dir) = app.path().app_data_dir() {
        return data_dir.join("douyin.db");
    }

    PathBuf::from("data").join("douyin.db")
}
