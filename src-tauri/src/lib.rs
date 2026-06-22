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
    // 优先使用 app_data_dir（打包后可写位置）
    if let Ok(data_dir) = app.path().app_data_dir() {
        return data_dir.join("douyin.db");
    }
    // 回退到当前工作目录
    PathBuf::from("data").join("douyin.db")
}
