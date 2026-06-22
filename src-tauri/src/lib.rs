mod db;

use db::{Database, DownloadRecord, DownloadStats, LiveRecord};
use std::path::PathBuf;
use tauri::{Manager, State};

// ============================================================
// Tauri Commands
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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_downloads,
            get_download_stats,
            get_live_records,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn resolve_db_path(app: &tauri::App) -> PathBuf {
    // 获取当前工作目录（开发时是 src-tauri）
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    println!("[DB] 当前工作目录: {:?}", current_dir);

    // 开发时：工作目录是 src-tauri，需要向上一级到项目根目录
    let project_root = current_dir.parent().unwrap_or(&current_dir);
    let dev_path = project_root.join("data").join("douyin.db");
    println!("[DB] 检查项目根目录路径: {:?}, 存在: {}", dev_path, dev_path.exists());

    if dev_path.exists() {
        println!("[DB] 使用项目根目录路径: {:?}", dev_path);
        return dev_path;
    }

    // 也检查当前目录下的 data/douyin.db（可能在项目根目录直接运行）
    let current_data_path = current_dir.join("data").join("douyin.db");
    println!("[DB] 检查当前目录路径: {:?}, 存在: {}", current_data_path, current_data_path.exists());
    if current_data_path.exists() {
        println!("[DB] 使用当前目录路径: {:?}", current_data_path);
        return current_data_path;
    }

    // 打包后使用 app_data_dir
    if let Ok(data_dir) = app.path().app_data_dir() {
        let app_path = data_dir.join("douyin.db");
        println!("[DB] 使用 app_data_dir 路径: {:?}", app_path);
        return app_path;
    }

    println!("[DB] 使用默认路径: data/douyin.db");
    PathBuf::from("data").join("douyin.db")
}
