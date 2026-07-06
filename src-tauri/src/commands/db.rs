//! Tauri commands — 数据库 CRUD
//!
//! 25 个数据库命令，全部通过 `State<'_, AppState>` 获取 Database。
//! 包含读取（get_*）、写入（save_*）、更新（update_*）、删除（delete_*）、查询（is_*）。

use tauri::State;
use log::info;

use crate::state::AppState;
use crate::db::{
    DownloadTask, DownloadTaskDetail,
    LiveRecord, MusicCollection, NewDownloadTask,
    NewLiveRecord, NewMusicCollection, NewTaskItem,
    TaskItem, TaskItemCounts, UserInfo, VideoInfo, VideoStats, UserStats,
    TrendPoint, AuthorStat, StorageStat, DbHealth,
};

// ============================================================
// 读取
// ============================================================

#[tauri::command(rename_all = "snake_case")]
pub fn get_live_records(
    state: State<'_, AppState>,
    limit: i64,
    offset: i64,
) -> Result<Vec<LiveRecord>, String> {
    state.db.get_live_records(limit, offset)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_live_record_count(
    state: State<'_, AppState>,
) -> Result<i64, String> {
    state.db.get_live_records_count().map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_videos(
    state: State<'_, AppState>,
    limit: i64,
    offset: i64,
    keyword: Option<String>,
    author_sec_uid: Option<String>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    post_type: Option<String>,
) -> Result<Vec<VideoInfo>, String> {
    state.db.get_videos(limit, offset, keyword, author_sec_uid, sort_by, sort_order, post_type)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_video_count(
    state: State<'_, AppState>,
    keyword: Option<String>,
    author_sec_uid: Option<String>,
    post_type: Option<String>,
) -> Result<i64, String> {
    state.db.get_video_count(keyword, author_sec_uid, post_type)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_users(
    state: State<'_, AppState>,
    limit: i64,
    offset: i64,
    keyword: Option<String>,
    sort_by: Option<String>,
    sort_order: Option<String>,
) -> Result<Vec<UserInfo>, String> {
    state.db.get_users(limit, offset, keyword, sort_by, sort_order)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_user_count(
    state: State<'_, AppState>,
    keyword: Option<String>,
) -> Result<i64, String> {
    state.db.get_user_count(keyword)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_user_by_sec_uid(
    state: State<'_, AppState>,
    sec_user_id: String,
) -> Result<Option<UserInfo>, String> {
    state.db.get_user_by_sec_uid(&sec_user_id)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_video_stats(state: State<'_, AppState>) -> Result<VideoStats, String> {
    state.db.get_video_stats().map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_user_stats(state: State<'_, AppState>) -> Result<UserStats, String> {
    state.db.get_user_stats().map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_music_collection(
    state: State<'_, AppState>,
    limit: i64,
    offset: i64,
    keyword: Option<String>,
    status: Option<String>,
) -> Result<Vec<MusicCollection>, String> {
    state.db.get_music_collection(limit, offset, keyword, status).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_music_collection_count(
    state: State<'_, AppState>,
    keyword: Option<String>,
    status: Option<String>,
) -> Result<i64, String> {
    state.db.get_music_collection_count(keyword, status).map_err(|e| e.to_string())
}

// ============================================================
// 写入
// ============================================================

#[tauri::command(rename_all = "snake_case")]
pub fn save_live_record_record(
    state: State<'_, AppState>,
    record: NewLiveRecord,
) -> Result<i64, String> {
    state.db.save_live_record(&record).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn save_user_info(
    state: State<'_, AppState>,
    user: UserInfo,
) -> Result<(), String> {
    state.db.save_user(&user).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn save_video_info(
    state: State<'_, AppState>,
    video: VideoInfo,
) -> Result<(), String> {
    state.db.save_video(&video).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn save_music_collection(
    state: State<'_, AppState>,
    music: NewMusicCollection,
) -> Result<(), String> {
    state.db.save_music_collection(&music).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn save_music_collection_batch(
    state: State<'_, AppState>,
    musics: Vec<NewMusicCollection>,
) -> Result<(), String> {
    state.db.save_music_collection_batch(&musics).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn update_music_file_path(
    state: State<'_, AppState>,
    music_id: String,
    file_path: String,
) -> Result<(), String> {
    state.db.update_music_file_path(&music_id, &file_path).map_err(|e| e.to_string())
}

/// 从下载文件路径中提取用户文件夹路径。
///
/// 路径格式: `{download_path}/{app_name}/{mode}/{nickname}/[{subfolder}/]{filename}`
/// 向上查找已知 mode 目录名，其下一级即为用户文件夹。
/// 返回 None 表示路径格式无法识别（回退到单文件删除）。
fn extract_user_download_dir(file_path: &str) -> Option<String> {
    const MODES: &[&str] = &["post", "like", "mix", "collects", "live", "music", "one"];
    let path = std::path::Path::new(file_path);
    let mut current = path;
    while let Some(parent) = current.parent() {
        if let Some(name) = parent.file_name() {
            let lower = name.to_string_lossy().to_lowercase();
            if MODES.contains(&lower.as_str()) {
                return Some(current.to_string_lossy().to_string());
            }
        }
        current = parent;
    }
    None
}

// ============================================================
// 删除（使用 delete_local_path 白名单校验）
// ============================================================

#[tauri::command(rename_all = "snake_case")]
pub fn delete_live_record(
    state: State<'_, AppState>,
    id: i64,
    delete_file: bool,
) -> Result<(), String> {
    if delete_file {
        let file_path = state.db.get_live_record_file_path(id).map_err(|e| e.to_string())?;
        crate::delete_local_path(file_path)?;
    }
    state.db.delete_live_record(id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_video_info(
    state: State<'_, AppState>,
    aweme_id: String,
) -> Result<(), String> {
    state.db.delete_video(&aweme_id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_user_info(
    state: State<'_, AppState>,
    sec_user_id: String,
    delete_file: bool,
) -> Result<(), String> {
    if delete_file {
        // 直播录制：单文件删除
        let live_paths = state.db.get_user_live_file_paths(&sec_user_id).map_err(|e| e.to_string())?;
        for path in live_paths {
            let _ = crate::delete_local_path(Some(path));
        }

        // 下载文件：删除整个用户文件夹（含附属文件、空文件夹）
        let download_paths = state.db.get_user_download_file_paths(&sec_user_id).map_err(|e| e.to_string())?;
        let mut user_dirs: Vec<String> = Vec::new();
        for path in &download_paths {
            if let Some(dir) = extract_user_download_dir(path) {
                if !user_dirs.contains(&dir) {
                    user_dirs.push(dir);
                }
            }
        }
        if user_dirs.is_empty() {
            // 回退：无法识别文件夹结构，逐文件删除
            for path in &download_paths {
                let _ = crate::delete_local_path(Some(path.clone()));
            }
        } else {
            for dir in user_dirs {
                let _ = crate::delete_local_path(Some(dir));
            }
        }
    }
    state.db.delete_user_cascade(&sec_user_id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_music_collection(
    state: State<'_, AppState>,
    music_id: String,
    delete_file: bool,
) -> Result<(), String> {
    if delete_file {
        let file_path = state.db.get_music_file_path(&music_id).map_err(|e| e.to_string())?;
        crate::delete_local_path(file_path)?;
    }
    state.db.delete_music_collection(&music_id).map_err(|e| e.to_string())
}

// ============================================================
// 批量删除（事务保证原子性）
// ============================================================

#[tauri::command(rename_all = "snake_case")]
pub fn delete_video_info_batch(
    state: State<'_, AppState>,
    aweme_ids: Vec<String>,
) -> Result<(), String> {
    state.db.delete_videos_batch(&aweme_ids).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_user_info_batch(
    state: State<'_, AppState>,
    sec_user_ids: Vec<String>,
    delete_file: bool,
) -> Result<(), String> {
    if delete_file {
        for id in &sec_user_ids {
            // 直播录制：单文件删除
            let live_paths = state.db.get_user_live_file_paths(id).map_err(|e| e.to_string())?;
            for path in live_paths {
                let _ = crate::delete_local_path(Some(path));
            }

            // 下载文件：删除整个用户文件夹（含附属文件、空文件夹）
            let download_paths = state.db.get_user_download_file_paths(id).map_err(|e| e.to_string())?;
            let mut user_dirs: Vec<String> = Vec::new();
            for path in &download_paths {
                if let Some(dir) = extract_user_download_dir(path) {
                    if !user_dirs.contains(&dir) {
                        user_dirs.push(dir);
                    }
                }
            }
            if user_dirs.is_empty() {
                for path in &download_paths {
                    let _ = crate::delete_local_path(Some(path.clone()));
                }
            } else {
                for dir in user_dirs {
                    let _ = crate::delete_local_path(Some(dir));
                }
            }
        }
    }
    state.db.delete_users_batch(&sec_user_ids).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_live_record_batch(
    state: State<'_, AppState>,
    ids: Vec<i64>,
    delete_file: bool,
) -> Result<(), String> {
    if delete_file {
        let file_paths = state.db.get_live_record_file_paths_batch(&ids).map_err(|e| e.to_string())?;
        for path in file_paths {
            let _ = crate::delete_local_path(Some(path));
        }
    }
    state.db.delete_live_records_batch(&ids).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_music_collection_batch(
    state: State<'_, AppState>,
    music_ids: Vec<String>,
    delete_file: bool,
) -> Result<(), String> {
    if delete_file {
        let file_paths = state.db.get_music_file_paths_batch(&music_ids).map_err(|e| e.to_string())?;
        for path in file_paths {
            let _ = crate::delete_local_path(Some(path));
        }
    }
    state.db.delete_music_collection_batch(&music_ids).map_err(|e| e.to_string())
}

// ============================================================
// 高级统计
// ============================================================

#[tauri::command(rename_all = "snake_case")]
pub fn get_download_trend(
    state: State<'_, AppState>,
    range: String,
) -> Result<Vec<TrendPoint>, String> {
    state.db.get_download_trend(&range).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_top_authors(
    state: State<'_, AppState>,
    limit: i64,
) -> Result<Vec<AuthorStat>, String> {
    state.db.get_top_authors(limit).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_storage_analysis(state: State<'_, AppState>) -> Result<Vec<StorageStat>, String> {
    state.db.get_storage_analysis().map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn db_health_check(
    state: State<'_, AppState>,
) -> Result<DbHealth, String> {
    state.db.db_health_check().map_err(|e| e.to_string())
}

// ============================================================
// 数据导出
// ============================================================

#[tauri::command(rename_all = "snake_case")]
pub fn export_data(
    state: State<'_, AppState>,
    data_type: String,
    save_path: String,
) -> Result<String, String> {
    // 路径校验：防止 ../ 穿越写入任意文件
    let safe_path = crate::validate_path_in_project(&save_path)?;

    let json = match data_type.as_str() {
        "videos" => {
            let records = state.db.get_videos(i64::MAX, 0, None, None, None, None, None)
                .map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&records).map_err(|e| e.to_string())?
        }
        "users" => {
            let records = state.db.get_users(i64::MAX, 0, None, None, None)
                .map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&records).map_err(|e| e.to_string())?
        }
        "live_records" => {
            let records = state.db.get_live_records(i64::MAX, 0)
                .map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&records).map_err(|e| e.to_string())?
        }
        "music" => {
            let records = state.db.get_music_collection(i64::MAX, 0, None, None)
                .map_err(|e| e.to_string())?;
            serde_json::to_string_pretty(&records).map_err(|e| e.to_string())?
        }
        _ => return Err(format!("不支持的导出类型: {}", data_type)),
    };

    // 确保父目录存在
    if let Some(parent) = safe_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }

    std::fs::write(&safe_path, &json)
        .map_err(|e| format!("写入文件失败: {}", e))?;

    info!("[export_data] 已导出 {} 到 {}", data_type, safe_path.display());
    Ok(safe_path.to_string_lossy().to_string())
}

// ============================================================
// 下载任务
// ============================================================

#[tauri::command(rename_all = "snake_case")]
pub fn create_download_task(
    state: State<'_, AppState>,
    task: NewDownloadTask,
) -> Result<(), String> {
    state.db.create_task(&task).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_download_tasks(
    state: State<'_, AppState>,
    limit: i64,
    offset: i64,
    status: Option<String>,
    mode: Option<String>,
) -> Result<Vec<DownloadTask>, String> {
    state.db.get_tasks(limit, offset, status, mode).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_download_task_detail(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<Option<DownloadTaskDetail>, String> {
    state.db.get_task_detail(&task_id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn update_download_task_status(
    state: State<'_, AppState>,
    task_id: String,
    status: String,
    error_msg: Option<String>,
) -> Result<(), String> {
    state.db.update_task_status(&task_id, &status, error_msg.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn create_download_task_item(
    state: State<'_, AppState>,
    item: NewTaskItem,
) -> Result<(), String> {
    state.db.create_task_item(&item).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn update_download_task_item_status(
    state: State<'_, AppState>,
    task_id: String,
    aweme_id: String,
    status: String,
    file_path: Option<String>,
    file_size: Option<i64>,
    error_msg: Option<String>,
) -> Result<(), String> {
    state.db.update_task_item_and_counts(
        &task_id, &aweme_id, &status,
        file_path.as_deref(), file_size.unwrap_or(0), error_msg.as_deref(),
    ).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_download_task_items(
    state: State<'_, AppState>,
    task_id: String,
    status: Option<String>,
) -> Result<Vec<TaskItem>, String> {
    state.db.get_task_items(&task_id, status).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_download_task_item_counts(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<TaskItemCounts, String> {
    state.db.get_task_item_counts(&task_id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_download_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<(), String> {
    state.db.delete_task(&task_id).map_err(|e| e.to_string())
}
