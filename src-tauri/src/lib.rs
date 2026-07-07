mod db;
mod database;
mod python;
mod config;
mod commands;
mod services;
mod state;
mod error;

use config::{AppConfig, ConfigManager};
use python::PythonBridge;
use state::AppState;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use parking_lot::Mutex;
use tauri::{Manager, State};
use log::{info, warn, error};

#[tauri::command(rename_all = "snake_case")]
fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let manager = state.config.lock();
    Ok(manager.get_douyin_config())
}

/// 存储数据库路径，供 db_health_check 使用
static DB_PATH: OnceLock<PathBuf> = OnceLock::new();

#[tauri::command(rename_all = "snake_case")]
fn get_db_path() -> Result<String, String> {
    DB_PATH.get()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| "数据库路径未初始化".to_string())
}

#[tauri::command(rename_all = "snake_case")]
fn set_config(
    state: State<'_, AppState>,
    updates: HashMap<String, String>,
) -> Result<(), String> {
    info!("[set_config] 收到 {} 个配置更新", updates.len());
    for (k, v) in &updates {
        if k == "cookie" {
            info!("[set_config] cookie (len={})", v.len());
        } else {
            info!("[set_config] {} = {:?}", k, v);
        }
    }

    // 1. 更新配置文件
    let mut manager = state.config.lock();
    manager.update_douyin_config(&updates)?;

    // 2. 同步到 Python（Rust 下载引擎在每次下载任务创建时从 AppState.config 读取，无需额外同步）
    let config = manager.get_douyin_config();
    info!("[set_config] 同步配置到 Python (timeout={}, max_connections={}, max_retries={})", config.timeout, config.max_connections, config.max_retries);
    python::init_config(&config).map_err(|e| e.to_string())?;

    Ok(())
}

// ============================================================
// 共享工具函数
// ============================================================

/// 校验路径是否在项目根目录内（防止 ../ 穿越）
///
/// 处理不存在的路径：向上查找最近存在的祖先目录，校验其在项目根目录内，
/// 再检查剩余路径段不含 `..`。
pub(crate) fn validate_path_in_project(path: &str) -> Result<PathBuf, String> {
    let local_path = Path::new(path);

    // 解析为绝对路径
    let abs_path = if local_path.is_absolute() {
        local_path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("获取当前目录失败: {}", e))?
            .join(local_path)
    };

    // 找到最近存在的祖先目录
    let mut ancestor = abs_path.as_path();
    while !ancestor.exists() {
        match ancestor.parent() {
            Some(p) => ancestor = p,
            None => return Err("路径无法解析".to_string()),
        }
    }

    let canonical_ancestor = ancestor
        .canonicalize()
        .map_err(|e| format!("路径规范化失败: {}", e))?;

    // 计算项目根目录
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_root = current_dir
        .parent()
        .unwrap_or(&current_dir)
        .canonicalize()
        .unwrap_or_else(|_| current_dir.clone());

    if !canonical_ancestor.starts_with(&project_root) {
        return Err(format!("安全拒绝: 路径不在项目目录内: {:?}", abs_path));
    }

    // 检查剩余路径段不含 ..
    if let Ok(remainder) = abs_path.strip_prefix(ancestor) {
        for component in remainder.components() {
            if let std::path::Component::ParentDir = component {
                return Err(format!("安全拒绝: 路径包含 .. 穿越: {:?}", abs_path));
            }
        }
    }

    Ok(abs_path)
}

/// 删除本地文件/目录（白名单校验：仅允许项目根目录下的路径，防止 ../ 穿越）
pub(crate) fn delete_local_path(path: Option<String>) -> Result<(), String> {
    let Some(path) = path.filter(|value| !value.trim().is_empty()) else {
        return Ok(());
    };

    // 先检查是否存在（不存在的路径删除是 no-op）
    if !Path::new(&path).exists() {
        return Ok(());
    }

    // 校验路径在项目内
    let canonical = validate_path_in_project(&path)?;

    if canonical.is_dir() {
        std::fs::remove_dir_all(&canonical).map_err(|e| format!("删除本地文件夹失败: {}", e))
    } else {
        std::fs::remove_file(&canonical).map_err(|e| format!("删除本地文件失败: {}", e))
    }
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

            let database = db::Database::open(&db_path)
                .expect("Failed to open database");

            // 存储数据库路径
            let _ = DB_PATH.set(db_path.clone());

            // 初始化配置管理器
            let config_manager = Arc::new(Mutex::new(ConfigManager::new()));

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

            // 创建并注册全局状态容器
            let state = AppState::new(database, config_manager, python_bridge);
            app.manage(state);

            // 同步配置到 Python
            let config = app.state::<AppState>().config.lock().get_douyin_config();
            if let Err(e) = python::init_config(&config) {
                warn!("同步配置到 Python 失败: {}", e);
            }

            // 初始化事件模块，供 Rust 侧 task_service 使用
            services::download::events::init(app.handle());

            // 注册 Python 事件桥接
            python::register_app_handle(app.handle());
            info!("AppHandle 已注册到 Python 模块");

            // 注册数据库桥接方法，供 Python 直接写入数据库
            python::register_db_bridge(app.handle());
            info!("数据库桥接已注册到 Python 模块");

            info!("应用初始化完成");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 配置
            get_config,
            set_config,
            get_db_path,
            // Python 桥接调用（commands/python.rs）
            commands::python::py_parse_video,
            commands::python::py_get_live_info,
            commands::python::py_get_user_profile,
            commands::python::py_get_user_posts,
            commands::python::py_search_videos,
            commands::python::py_get_mix_info,
            commands::python::py_get_collects_list,
            commands::python::py_get_collects_video_list,
            commands::python::py_get_following_list,
            commands::python::py_get_follower_list,
            commands::python::py_get_music_collection,
            commands::python::py_download_music,
            commands::python::py_get_following_live,
            commands::python::py_get_comments,
            commands::python::py_get_comment_replies,
            commands::python::py_get_tab_feed,
            commands::python::py_get_follow_feed,
            commands::python::py_get_friend_feed,
            commands::python::py_get_user_likes,
            commands::python::py_get_post_stats,
            commands::python::py_start_live_record,
            commands::python::py_stop_live_record,
            commands::python::py_get_live_status,
            // 数据库读取（commands/db.rs）
            commands::db::get_live_records,
            commands::db::get_live_record_count,
            commands::db::get_videos,
            commands::db::get_video_count,
            commands::db::get_users,
            commands::db::get_user_count,
            commands::db::get_user_by_sec_uid,
            commands::db::get_video_stats,
            commands::db::get_user_stats,
            commands::db::get_music_collection,
            commands::db::get_music_collection_count,
            // 数据库写入/删除（commands/db.rs）
            commands::db::save_live_record_record,
            commands::db::save_user_info,
            commands::db::save_video_info,
            commands::db::save_music_collection,
            commands::db::save_music_collection_batch,
            commands::db::update_music_file_path,
            commands::db::delete_live_record,
            commands::db::delete_video_info,
            commands::db::delete_user_info,
            commands::db::delete_music_collection,
            commands::db::delete_video_info_batch,
            commands::db::delete_user_info_batch,
            commands::db::delete_live_record_batch,
            commands::db::delete_music_collection_batch,
            // 高级统计（commands/db.rs）
            commands::db::get_download_trend,
            commands::db::get_top_authors,
            commands::db::get_storage_analysis,
            commands::db::db_health_check,
            // 数据导出（commands/db.rs）
            commands::db::export_data,
            // 打开文件夹（commands/db.rs）
            commands::db::get_download_dir_by_music_id,
            commands::db::get_download_dir_by_aweme_id,
            commands::db::get_user_download_dir,
            // 下载任务（commands/db.rs）
            commands::db::create_download_task,
            commands::db::get_download_tasks,
            commands::db::get_download_task_detail,
            commands::db::update_download_task_status,
            commands::db::create_download_task_item,
            commands::db::update_download_task_item_status,
            commands::db::get_download_task_items,
            commands::db::get_download_task_item_counts,
            commands::db::delete_download_task,
            // 任务系统（Rust-owned，Phase 3 新增）
            commands::tasks::start_download,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn resolve_db_path(_app: &tauri::App) -> PathBuf {
    // 生产模式优先使用 app_data_dir，开发模式使用项目相对路径
    #[cfg(not(debug_assertions))]
    {
        if let Ok(data_dir) = app.path().app_data_dir() {
            let prod_path = data_dir.join("douyin.db");
            // 如果 AppData 中已有数据库，直接使用
            if prod_path.exists() {
                info!("使用 AppData 数据库: {:?}", prod_path);
                return prod_path;
            }
            // 兼容旧版：检查项目相对路径是否有已有数据库
            let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let legacy_path = current_dir.parent().unwrap_or(&current_dir).join("data").join("douyin.db");
            if legacy_path.exists() {
                info!("迁移旧数据库: {:?} → {:?}", legacy_path, prod_path);
                let _ = std::fs::create_dir_all(&data_dir);
                let _ = std::fs::copy(&legacy_path, &prod_path);
                return prod_path;
            }
            // 新建到 AppData
            let _ = std::fs::create_dir_all(&data_dir);
            info!("新建 AppData 数据库: {:?}", prod_path);
            return prod_path;
        }
        warn!("获取 AppData 路径失败，回退到项目目录");
    }

    // 开发模式：使用项目根目录下的 data/
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_root = current_dir.parent().unwrap_or(&current_dir);
    let dev_path = project_root.join("data").join("douyin.db");

    if dev_path.exists() {
        info!("使用项目数据库: {:?}", dev_path);
        return dev_path;
    }

    if let Err(e) = std::fs::create_dir_all(dev_path.parent().unwrap_or(&project_root.join("data"))) {
        warn!("创建 data 目录失败: {}", e);
    }

    info!("新建项目数据库: {:?}", dev_path);
    dev_path
}
