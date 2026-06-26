//! Tauri commands — 数据库 CRUD
//!
//! 25 个数据库命令，全部注入 `State<'_, Database>`。
//! 包含读取（get_*）、写入（save_*）、更新（update_*）、删除（delete_*）、查询（is_*）。

use tauri::State;
use log::{info, error};

use crate::db::{
    Database, DownloadRecord, DownloadStats, LiveRecord,
    MusicCollection, NewDownloadRecord, NewLiveRecord, NewMusicCollection,
    UserInfo, VideoInfo, VideoStats, UserStats,
};

// ============================================================
// 读取
// ============================================================

#[tauri::command(rename_all = "snake_case")]
pub fn get_downloads(
    db: State<'_, Database>,
    limit: i64,
    offset: i64,
    status: Option<String>,
    download_type: Option<String>,
) -> Result<Vec<DownloadRecord>, String> {
    db.get_downloads(limit, offset, status, download_type)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_download_stats(db: State<'_, Database>) -> Result<DownloadStats, String> {
    db.get_download_stats().map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_live_records(
    db: State<'_, Database>,
    limit: i64,
    offset: i64,
) -> Result<Vec<LiveRecord>, String> {
    db.get_live_records(limit, offset)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_live_record_count(
    db: State<'_, Database>,
) -> Result<i64, String> {
    db.get_live_records_count().map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_videos(
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

#[tauri::command(rename_all = "snake_case")]
pub fn get_video_count(
    db: State<'_, Database>,
    keyword: Option<String>,
    author_sec_uid: Option<String>,
    post_type: Option<String>,
) -> Result<i64, String> {
    db.get_video_count(keyword, author_sec_uid, post_type)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_users(
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

#[tauri::command(rename_all = "snake_case")]
pub fn get_user_count(
    db: State<'_, Database>,
    keyword: Option<String>,
) -> Result<i64, String> {
    db.get_user_count(keyword)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_user_by_sec_uid(
    db: State<'_, Database>,
    sec_user_id: String,
) -> Result<Option<UserInfo>, String> {
    db.get_user_by_sec_uid(&sec_user_id)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_video_stats(db: State<'_, Database>) -> Result<VideoStats, String> {
    db.get_video_stats().map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_user_stats(db: State<'_, Database>) -> Result<UserStats, String> {
    db.get_user_stats().map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_music_collection(
    db: State<'_, Database>,
    limit: i64,
    offset: i64,
    keyword: Option<String>,
    status: Option<String>,
) -> Result<Vec<MusicCollection>, String> {
    db.get_music_collection(limit, offset, keyword, status).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_music_collection_count(
    db: State<'_, Database>,
    keyword: Option<String>,
    status: Option<String>,
) -> Result<i64, String> {
    db.get_music_collection_count(keyword, status).map_err(|e| e.to_string())
}

// ============================================================
// 写入
// ============================================================

#[tauri::command(rename_all = "snake_case")]
pub fn save_download_record(
    db: State<'_, Database>,
    record: NewDownloadRecord,
) -> Result<i64, String> {
    info!("[save_download_record] 收到请求: aweme_id={:?}, file_path={:?}", record.aweme_id, record.file_path);
    db.save_download(&record).map_err(|e| {
        error!("[save_download_record] 保存失败: {}", e);
        e.to_string()
    })
}

#[tauri::command(rename_all = "snake_case")]
pub fn save_live_record_record(
    db: State<'_, Database>,
    record: NewLiveRecord,
) -> Result<i64, String> {
    db.save_live_record(&record).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn save_user_info(
    db: State<'_, Database>,
    user: UserInfo,
) -> Result<(), String> {
    db.save_user(&user).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn save_video_info(
    db: State<'_, Database>,
    video: VideoInfo,
) -> Result<(), String> {
    db.save_video(&video).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn is_video_downloaded(
    db: State<'_, Database>,
    aweme_id: String,
) -> Result<bool, String> {
    db.is_video_downloaded(&aweme_id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn save_music_collection(
    db: State<'_, Database>,
    music: NewMusicCollection,
) -> Result<(), String> {
    db.save_music_collection(&music).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn save_music_collection_batch(
    db: State<'_, Database>,
    musics: Vec<NewMusicCollection>,
) -> Result<(), String> {
    db.save_music_collection_batch(&musics).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn update_music_file_path(
    db: State<'_, Database>,
    music_id: String,
    file_path: String,
) -> Result<(), String> {
    db.update_music_file_path(&music_id, &file_path).map_err(|e| e.to_string())
}

// ============================================================
// 删除（使用 delete_local_path 白名单校验）
// ============================================================

#[tauri::command(rename_all = "snake_case")]
pub fn delete_download_record(
    db: State<'_, Database>,
    id: i64,
    delete_file: bool,
) -> Result<(), String> {
    if delete_file {
        let file_path = db.get_download_file_path(id).map_err(|e| e.to_string())?;
        crate::delete_local_path(file_path)?;
    }
    db.delete_download(id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_live_record(
    db: State<'_, Database>,
    id: i64,
    delete_file: bool,
) -> Result<(), String> {
    if delete_file {
        let file_path = db.get_live_record_file_path(id).map_err(|e| e.to_string())?;
        crate::delete_local_path(file_path)?;
    }
    db.delete_live_record(id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_video_info(
    db: State<'_, Database>,
    aweme_id: String,
) -> Result<(), String> {
    db.delete_video(&aweme_id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_user_info(
    db: State<'_, Database>,
    sec_user_id: String,
) -> Result<(), String> {
    db.delete_user(&sec_user_id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_music_collection(
    db: State<'_, Database>,
    music_id: String,
    delete_file: bool,
) -> Result<(), String> {
    if delete_file {
        let file_path = db.get_music_file_path(&music_id).map_err(|e| e.to_string())?;
        crate::delete_local_path(file_path)?;
    }
    db.delete_music_collection(&music_id).map_err(|e| e.to_string())
}
