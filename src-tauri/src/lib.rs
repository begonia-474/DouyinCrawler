mod db;
mod python;
mod config;

use db::{
    Database, DownloadRecord, DownloadStats, LiveRecord,
    MusicCollection, NewDownloadRecord, NewLiveRecord, NewMusicCollection,
    UserInfo, VideoInfo, VideoStats, UserStats,
};
use config::{AppConfig, ConfigManager};
use python::PythonBridge;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use tauri::{Emitter, Manager, State};
use serde_json::Value;
use log::{info, warn, error};

/// 全局 AppHandle，供 Python 通过 PyO3 发送 Tauri 事件
static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();

// ============================================================
// Tauri Commands - 配置
// ============================================================

#[tauri::command]
fn get_config(config_manager: State<'_, Arc<Mutex<ConfigManager>>>) -> Result<AppConfig, String> {
    let manager = config_manager.lock().map_err(|e| e.to_string())?;
    Ok(manager.get_douyin_config())
}

#[tauri::command]
fn set_config(
    config_manager: State<'_, Arc<Mutex<ConfigManager>>>,
    _python_bridge: State<'_, Arc<PythonBridge>>,
    updates: HashMap<String, String>,
) -> Result<(), String> {
    info!("[set_config] 收到 {} 个配置更新", updates.len());
    for (k, v) in &updates {
        if k == "cookie" {
            info!("[set_config] cookie 长度={}, 前60字符: {:?}", v.len(), &v[..v.len().min(60)]);
        } else {
            info!("[set_config] {} = {:?}", k, v);
        }
    }

    // 1. 更新配置文件
    let mut manager = config_manager.lock().map_err(|e| e.to_string())?;
    manager.update_douyin_config(&updates)?;

    // 2. 同步到 Python
    let config = manager.get_douyin_config();
    info!("[set_config] 同步到 Python, cookie 长度={}", config.cookie.len());
    python::init_config(&config).map_err(|e| e.to_string())?;

    Ok(())
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
// Tauri Commands - PyO3 直接调用
// ============================================================

#[tauri::command]
fn py_parse_video(url: String) -> Result<Value, String> {
    python::parse_video(&url).map_err(|e| e.to_string())
}

#[tauri::command]
fn py_download_video(url: String) -> Result<Value, String> {
    python::download_video(&url).map_err(|e| e.to_string())
}

#[tauri::command]
fn py_get_live_info(url: String) -> Result<Value, String> {
    python::get_live_info(&url).map_err(|e| e.to_string())
}

#[tauri::command]
fn py_start_batch_download(url: String, download_type: String) -> Result<Value, String> {
    python::start_batch_download(&url, &download_type).map_err(|e| e.to_string())
}

#[tauri::command]
fn py_get_user_profile(url: String) -> Result<Value, String> {
    python::get_user_profile(&url).map_err(|e| e.to_string())
}

#[tauri::command]
fn py_get_user_posts(url: String, cursor: Option<i64>, count: Option<i64>) -> Result<Value, String> {
    python::get_user_posts(&url, cursor.unwrap_or(0), count.unwrap_or(20)).map_err(|e| e.to_string())
}

#[tauri::command]
fn py_search_videos(keyword: String, offset: i64, count: i64) -> Result<Value, String> {
    python::search_videos(&keyword, offset, count).map_err(|e| e.to_string())
}

#[tauri::command]
fn py_get_mix_info(url: String, cursor: Option<i64>, count: Option<i64>) -> Result<Value, String> {
    python::get_mix_info(&url, cursor.unwrap_or(0), count.unwrap_or(20)).map_err(|e| e.to_string())
}

#[tauri::command]
fn py_get_collects_list() -> Result<Value, String> {
    python::get_collects_list().map_err(|e| e.to_string())
}

#[tauri::command]
fn py_get_collects_video_list(collects_id: String, cursor: Option<i64>, count: Option<i64>) -> Result<Value, String> {
    python::get_collects_video_list(&collects_id, cursor.unwrap_or(0), count.unwrap_or(20)).map_err(|e| e.to_string())
}

#[tauri::command]
fn py_get_following_list(url: String, offset: i64, count: i64) -> Result<Value, String> {
    python::get_following_list(&url, offset, count).map_err(|e| e.to_string())
}

#[tauri::command]
fn py_get_follower_list(url: String, offset: i64, count: i64) -> Result<Value, String> {
    python::get_follower_list(&url, offset, count).map_err(|e| e.to_string())
}

#[tauri::command]
fn py_get_music_collection(cursor: i64, count: i64) -> Result<Value, String> {
    python::get_music_collection(cursor, count).map_err(|e| e.to_string())
}

#[tauri::command]
fn py_download_music(play_url: String, title: String, author: String) -> Result<Value, String> {
    python::download_music(&play_url, &title, &author).map_err(|e| e.to_string())
}

#[tauri::command]
fn py_get_following_live() -> Result<Value, String> {
    python::get_following_live().map_err(|e| e.to_string())
}

#[tauri::command]
fn py_get_comments(url: String, cursor: i64, count: i64) -> Result<Value, String> {
    python::get_comments(&url, cursor, count).map_err(|e| e.to_string())
}

#[tauri::command]
fn py_get_comment_replies(url: String, comment_id: String, cursor: i64, count: i64) -> Result<Value, String> {
    python::get_comment_replies(&url, &comment_id, cursor, count).map_err(|e| e.to_string())
}

#[tauri::command]
fn py_get_tab_feed(count: i64) -> Result<Value, String> {
    python::get_tab_feed(count).map_err(|e| e.to_string())
}

#[tauri::command]
fn py_get_follow_feed(cursor: i64, count: i64) -> Result<Value, String> {
    python::get_follow_feed(cursor, count).map_err(|e| e.to_string())
}

#[tauri::command]
fn py_get_friend_feed(cursor: i64, count: i64) -> Result<Value, String> {
    python::get_friend_feed(cursor, count).map_err(|e| e.to_string())
}

#[tauri::command]
fn py_get_user_likes(url: String, cursor: Option<i64>, count: Option<i64>) -> Result<Value, String> {
    python::get_user_likes(&url, cursor.unwrap_or(0), count.unwrap_or(20)).map_err(|e| e.to_string())
}

#[tauri::command]
fn py_get_post_stats(url: String) -> Result<Value, String> {
    python::get_post_stats(&url).map_err(|e| e.to_string())
}

#[tauri::command]
fn py_start_live_record(url: String) -> Result<Value, String> {
    python::start_live_record(&url).map_err(|e| e.to_string())
}

#[tauri::command]
fn py_stop_live_record(task_id: String) -> Result<Value, String> {
    python::stop_live_record(&task_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn py_get_live_status() -> Result<Value, String> {
    python::get_live_status().map_err(|e| e.to_string())
}

#[tauri::command]
fn py_test_emit() -> Result<Value, String> {
    python::test_emit().map_err(|e| e.to_string())
}

// ============================================================
// Tauri Commands - 数据库写入
// ============================================================

#[tauri::command]
fn save_download_record(
    db: State<'_, Database>,
    record: NewDownloadRecord,
) -> Result<i64, String> {
    info!("[save_download_record] 收到请求: aweme_id={:?}, file_path={:?}", record.aweme_id, record.file_path);
    db.save_download(&record).map_err(|e| {
        error!("[save_download_record] 保存失败: {}", e);
        e.to_string()
    })
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
    // 初始化日志系统
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("启动 DouyinCrawler Desktop");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db_path = resolve_db_path(app);
            info!("数据库路径: {:?}", db_path);

            let database = Database::open(&db_path)
                .expect("Failed to open database");
            app.manage(database);

            // 初始化配置管理器
            let config_manager = Arc::new(Mutex::new(ConfigManager::new()));
            app.manage(config_manager.clone());

            // 初始化 Python 桥接器
            let python_bridge = match PythonBridge::new() {
                Ok(bridge) => {
                    info!("Python 桥接器初始化成功");
                    Arc::new(bridge)
                }
                Err(e) => {
                    error!("Python 桥接器初始化失败: {}", e);
                    Arc::new(PythonBridge::new().unwrap_or_else(|_| {
                        panic!("无法创建 Python 桥接器")
                    }))
                }
            };
            app.manage(python_bridge);

            // 同步配置到 Python
            let config = config_manager.lock().unwrap().get_douyin_config();
            if let Err(e) = python::init_config(&config) {
                warn!("同步配置到 Python 失败: {}", e);
            }

            // 存储全局 AppHandle，供 Python 发送 Tauri 事件
            let _ = APP_HANDLE.set(app.handle().clone());
            python::register_app_handle();
            info!("AppHandle 已注册到 Python 模块");

            // 注册数据库桥接方法，供 Python 直接写入数据库
            python::register_db_bridge();
            info!("数据库桥接已注册到 Python 模块");

            info!("应用初始化完成");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 配置
            get_config,
            set_config,
            // PyO3 直接调用
            py_parse_video,
            py_download_video,
            py_get_live_info,
            py_start_batch_download,
            py_get_user_profile,
            py_get_user_posts,
            py_search_videos,
            py_get_mix_info,
            py_get_collects_list,
            py_get_collects_video_list,
            py_get_following_list,
            py_get_follower_list,
            py_get_music_collection,
            py_download_music,
            py_get_following_live,
            py_get_comments,
            py_get_comment_replies,
            py_get_tab_feed,
            py_get_follow_feed,
            py_get_friend_feed,
            py_get_user_likes,
            py_get_post_stats,
            py_start_live_record,
            py_stop_live_record,
            py_get_live_status,
            py_test_emit,
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

    // 优先使用项目根目录下的 data/ （开发模式和生产模式统一）
    let project_data = project_root.join("data");
    let dev_path = project_data.join("douyin.db");

    if dev_path.exists() {
        info!("使用项目数据库: {:?}", dev_path);
        return dev_path;
    }

    // 不存在时创建目录并使用项目路径（不回退到 AppData）
    if let Err(e) = std::fs::create_dir_all(&project_data) {
        warn!("创建 data 目录失败: {}, 回退到 AppData", e);
        if let Ok(data_dir) = app.path().app_data_dir() {
            return data_dir.join("douyin.db");
        }
    }

    info!("新建项目数据库: {:?}", dev_path);
    dev_path
}
