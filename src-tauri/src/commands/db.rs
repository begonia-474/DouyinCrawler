//! Tauri commands — 数据库 CRUD
//!
//! 25 个数据库命令，全部注入 `State<'_, Database>`。
//! 包含读取（get_*）、写入（save_*）、更新（update_*）、删除（delete_*）、查询（is_*）。

use tauri::State;
use log::{info, error};

use crate::db::{
    Database, DownloadRecord, DownloadStats, DownloadTask, DownloadTaskDetail,
    LiveRecord, MusicCollection, NewDownloadRecord, NewDownloadTask,
    NewLiveRecord, NewMusicCollection, NewTaskItem,
    TaskItem, TaskItemCounts, UserInfo, VideoInfo, VideoStats, UserStats,
    TrendPoint, AuthorStat, StorageStat, DbHealth,
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

// ============================================================
// 高级统计
// ============================================================

#[tauri::command(rename_all = "snake_case")]
pub fn get_download_trend(
    db: State<'_, Database>,
    range: String,
) -> Result<Vec<TrendPoint>, String> {
    db.get_download_trend(&range).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_top_authors(
    db: State<'_, Database>,
    limit: i64,
) -> Result<Vec<AuthorStat>, String> {
    db.get_top_authors(limit).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_storage_analysis(db: State<'_, Database>) -> Result<Vec<StorageStat>, String> {
    db.get_storage_analysis().map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn db_health_check(
    db: State<'_, Database>,
) -> Result<DbHealth, String> {
    db.db_health_check().map_err(|e| e.to_string())
}

// ============================================================
// 数据导出
// ============================================================

#[tauri::command(rename_all = "snake_case")]
pub fn export_data(
    db: State<'_, Database>,
    data_type: String,
    save_path: String,
) -> Result<String, String> {
    let json = match data_type.as_str() {
        "downloads" => {
            let records = db.get_downloads(i64::MAX, 0, None, None)
                .map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&records).map_err(|e| e.to_string())?
        }
        "videos" => {
            let records = db.get_videos(i64::MAX, 0, None, None, None, None, None)
                .map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&records).map_err(|e| e.to_string())?
        }
        "users" => {
            let records = db.get_users(i64::MAX, 0, None, None, None)
                .map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&records).map_err(|e| e.to_string())?
        }
        "live_records" => {
            let records = db.get_live_records(i64::MAX, 0)
                .map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&records).map_err(|e| e.to_string())?
        }
        "music" => {
            let records = db.get_music_collection(i64::MAX, 0, None, None)
                .map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&records).map_err(|e| e.to_string())?
        }
        _ => return Err(format!("不支持的导出类型: {}", data_type)),
    };

    // 确保父目录存在
    if let Some(parent) = std::path::Path::new(&save_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }

    std::fs::write(&save_path, &json)
        .map_err(|e| format!("写入文件失败: {}", e))?;

    info!("[export_data] 已导出 {} 到 {}", data_type, save_path);
    Ok(save_path)
}

// ============================================================
// 下载任务
// ============================================================

#[tauri::command(rename_all = "snake_case")]
pub fn create_download_task(
    db: State<'_, Database>,
    task: NewDownloadTask,
) -> Result<(), String> {
    db.create_task(&task).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_download_tasks(
    db: State<'_, Database>,
    limit: i64,
    offset: i64,
    status: Option<String>,
    mode: Option<String>,
) -> Result<Vec<DownloadTask>, String> {
    db.get_tasks(limit, offset, status, mode).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_download_task_detail(
    db: State<'_, Database>,
    task_id: String,
) -> Result<Option<DownloadTaskDetail>, String> {
    db.get_task_detail(&task_id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn update_download_task_status(
    db: State<'_, Database>,
    task_id: String,
    status: String,
    error_msg: Option<String>,
) -> Result<(), String> {
    db.update_task_status(&task_id, &status, error_msg.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn create_download_task_item(
    db: State<'_, Database>,
    item: NewTaskItem,
) -> Result<(), String> {
    db.create_task_item(&item).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn update_download_task_item_status(
    db: State<'_, Database>,
    task_id: String,
    aweme_id: String,
    status: String,
    file_path: Option<String>,
    file_size: Option<i64>,
    error_msg: Option<String>,
) -> Result<(), String> {
    db.update_task_item_and_counts(
        &task_id, &aweme_id, &status,
        file_path.as_deref(), file_size.unwrap_or(0), error_msg.as_deref(),
    ).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_download_task_items(
    db: State<'_, Database>,
    task_id: String,
    status: Option<String>,
) -> Result<Vec<TaskItem>, String> {
    db.get_task_items(&task_id, status).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_download_task_item_counts(
    db: State<'_, Database>,
    task_id: String,
) -> Result<TaskItemCounts, String> {
    db.get_task_item_counts(&task_id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_download_task(
    db: State<'_, Database>,
    task_id: String,
) -> Result<(), String> {
    db.delete_task(&task_id).map_err(|e| e.to_string())
}
