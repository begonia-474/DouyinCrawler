//! Schema migration module
//!
//! Contains all CREATE TABLE SQL, column definitions, and migration logic.

use rusqlite::{Connection, Result};
use log::info;
use std::time::{SystemTime, UNIX_EPOCH};

pub const CREATE_TABLES_SQL: &str = "
    CREATE TABLE IF NOT EXISTS _metadata (
        name TEXT PRIMARY KEY,
        value TEXT
    );
    CREATE TABLE IF NOT EXISTS user_info (
        sec_user_id TEXT PRIMARY KEY,
        nickname TEXT,
        uid TEXT,
        avatar_url TEXT,
        unique_id TEXT,
        signature TEXT,
        aweme_count INTEGER DEFAULT 0,
        follower_count INTEGER DEFAULT 0,
        following_count INTEGER DEFAULT 0,
        total_favorited INTEGER DEFAULT 0,
        ip_location TEXT,
        live_status INTEGER DEFAULT 0,
        room_id TEXT,
        updated_at INTEGER DEFAULT 0
    );
    CREATE TABLE IF NOT EXISTS video_info (
        aweme_id TEXT PRIMARY KEY,
        desc TEXT,
        aweme_type INTEGER DEFAULT 0,
        author_nickname TEXT,
        author_sec_uid TEXT,
        author_uid TEXT,
        create_time INTEGER,
        duration INTEGER DEFAULT 0,
        video_url TEXT,
        cover_url TEXT,
        music_title TEXT,
        digg_count INTEGER DEFAULT 0,
        comment_count INTEGER DEFAULT 0,
        share_count INTEGER DEFAULT 0,
        collect_count INTEGER DEFAULT 0,
        mix_id TEXT,
        mix_name TEXT,
        updated_at INTEGER DEFAULT 0
    );
    CREATE TABLE IF NOT EXISTS download_history (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        aweme_id TEXT,
        download_type TEXT NOT NULL DEFAULT 'video',
        title TEXT,
        author_nickname TEXT,
        author_sec_uid TEXT,
        file_path TEXT,
        file_size INTEGER DEFAULT 0,
        cover_url TEXT,
        status TEXT NOT NULL DEFAULT 'completed',
        error_msg TEXT,
        created_at INTEGER NOT NULL
    );
    CREATE TABLE IF NOT EXISTS live_records (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        room_id TEXT,
        web_rid TEXT,
        title TEXT,
        nickname TEXT,
        sec_user_id TEXT,
        file_path TEXT,
        file_size INTEGER DEFAULT 0,
        duration_sec INTEGER DEFAULT 0,
        status TEXT NOT NULL DEFAULT 'completed',
        started_at INTEGER,
        ended_at INTEGER
    );
    CREATE TABLE IF NOT EXISTS music_collection (
        music_id TEXT PRIMARY KEY,
        mid TEXT,
        title TEXT,
        author TEXT,
        owner_nickname TEXT,
        duration INTEGER DEFAULT 0,
        cover TEXT,
        play_url TEXT,
        file_path TEXT,
        status TEXT NOT NULL DEFAULT 'collected',
        created_at INTEGER NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_download_created ON download_history(created_at);
    CREATE INDEX IF NOT EXISTS idx_download_type ON download_history(download_type);
    CREATE INDEX IF NOT EXISTS idx_download_status ON download_history(status);
    CREATE INDEX IF NOT EXISTS idx_download_author ON download_history(author_sec_uid);
    CREATE INDEX IF NOT EXISTS idx_live_started ON live_records(started_at);
    CREATE INDEX IF NOT EXISTS idx_music_created ON music_collection(created_at);
";

// === 显式列名（与 row_to_video / row_to_user mapper 的位置索引一一对应）===
// 顺序 = 基表列 + 迁移追加列，mapper 按 index 读取，不可调换
pub const VIDEO_COLUMNS: &str = "aweme_id, desc, aweme_type, \
    author_nickname, author_sec_uid, author_uid, create_time, duration, \
    video_url, cover_url, music_title, digg_count, comment_count, share_count, \
    collect_count, mix_id, mix_name, updated_at, \
    author_nickname_raw, author_short_id, author_unique_id, desc_raw, \
    is_ads, is_story, is_top, is_long_video, video_bit_rate, animated_cover, \
    private_status, is_delete, music_author, music_author_raw, music_duration, \
    music_id, music_mid, pgc_author, pgc_author_title, pgc_music_type, \
    music_status, music_owner_handle, music_owner_id, music_owner_nickname, \
    music_play_url, is_commerce_music, mix_desc, mix_create_time, mix_pic_type, \
    mix_type, mix_share_url, can_comment, can_forward, can_share, download_setting, \
    allow_douplus, allow_share, admire_count, hashtag_ids, hashtag_names, images, \
    region, is_prohibited";

pub const USER_COLUMNS: &str = "sec_user_id, nickname, uid, avatar_url, unique_id, \
    signature, aweme_count, follower_count, following_count, total_favorited, \
    ip_location, live_status, room_id, updated_at, \
    city, country, favoriting_count, gender, is_ban, is_block, is_blocked, \
    is_star, mix_count, mplatform_followers_count, nickname_raw, school_name, \
    short_id, signature_raw, user_age, custom_verify";

// === Schema 迁移 ===

pub const MIGRATE_V1_USER_INFO: &[&str] = &[
    "ALTER TABLE user_info ADD COLUMN city TEXT",
    "ALTER TABLE user_info ADD COLUMN country TEXT",
    "ALTER TABLE user_info ADD COLUMN favoriting_count INTEGER DEFAULT 0",
    "ALTER TABLE user_info ADD COLUMN gender INTEGER DEFAULT 0",
    "ALTER TABLE user_info ADD COLUMN is_ban INTEGER DEFAULT 0",
    "ALTER TABLE user_info ADD COLUMN is_block INTEGER DEFAULT 0",
    "ALTER TABLE user_info ADD COLUMN is_blocked INTEGER DEFAULT 0",
    "ALTER TABLE user_info ADD COLUMN is_star INTEGER DEFAULT 0",
    "ALTER TABLE user_info ADD COLUMN mix_count INTEGER DEFAULT 0",
    "ALTER TABLE user_info ADD COLUMN mplatform_followers_count INTEGER DEFAULT 0",
    "ALTER TABLE user_info ADD COLUMN nickname_raw TEXT",
    "ALTER TABLE user_info ADD COLUMN school_name TEXT",
    "ALTER TABLE user_info ADD COLUMN short_id TEXT",
    "ALTER TABLE user_info ADD COLUMN signature_raw TEXT",
    "ALTER TABLE user_info ADD COLUMN user_age INTEGER DEFAULT 0",
    "ALTER TABLE user_info ADD COLUMN custom_verify TEXT",
];

pub const MIGRATE_V2_VIDEO_INFO: &[&str] = &[
    "ALTER TABLE video_info ADD COLUMN author_nickname_raw TEXT",
    "ALTER TABLE video_info ADD COLUMN author_short_id TEXT",
    "ALTER TABLE video_info ADD COLUMN author_unique_id TEXT",
    "ALTER TABLE video_info ADD COLUMN desc_raw TEXT",
    "ALTER TABLE video_info ADD COLUMN is_ads INTEGER DEFAULT 0",
    "ALTER TABLE video_info ADD COLUMN is_story INTEGER DEFAULT 0",
    "ALTER TABLE video_info ADD COLUMN is_top INTEGER DEFAULT 0",
    "ALTER TABLE video_info ADD COLUMN is_long_video INTEGER DEFAULT 0",
    "ALTER TABLE video_info ADD COLUMN video_bit_rate TEXT",
    "ALTER TABLE video_info ADD COLUMN animated_cover TEXT",
    "ALTER TABLE video_info ADD COLUMN private_status INTEGER DEFAULT 0",
    "ALTER TABLE video_info ADD COLUMN is_delete INTEGER DEFAULT 0",
    "ALTER TABLE video_info ADD COLUMN music_author TEXT",
    "ALTER TABLE video_info ADD COLUMN music_author_raw TEXT",
    "ALTER TABLE video_info ADD COLUMN music_duration INTEGER DEFAULT 0",
    "ALTER TABLE video_info ADD COLUMN music_id TEXT",
    "ALTER TABLE video_info ADD COLUMN music_mid TEXT",
    "ALTER TABLE video_info ADD COLUMN pgc_author TEXT",
    "ALTER TABLE video_info ADD COLUMN pgc_author_title TEXT",
    "ALTER TABLE video_info ADD COLUMN pgc_music_type INTEGER DEFAULT 0",
    "ALTER TABLE video_info ADD COLUMN music_status INTEGER DEFAULT 0",
    "ALTER TABLE video_info ADD COLUMN music_owner_handle TEXT",
    "ALTER TABLE video_info ADD COLUMN music_owner_id TEXT",
    "ALTER TABLE video_info ADD COLUMN music_owner_nickname TEXT",
    "ALTER TABLE video_info ADD COLUMN music_play_url TEXT",
    "ALTER TABLE video_info ADD COLUMN is_commerce_music INTEGER DEFAULT 0",
    "ALTER TABLE video_info ADD COLUMN mix_desc TEXT",
    "ALTER TABLE video_info ADD COLUMN mix_create_time INTEGER DEFAULT 0",
    "ALTER TABLE video_info ADD COLUMN mix_pic_type INTEGER DEFAULT 0",
    "ALTER TABLE video_info ADD COLUMN mix_type INTEGER DEFAULT 0",
    "ALTER TABLE video_info ADD COLUMN mix_share_url TEXT",
    "ALTER TABLE video_info ADD COLUMN can_comment INTEGER DEFAULT 1",
    "ALTER TABLE video_info ADD COLUMN can_forward INTEGER DEFAULT 1",
    "ALTER TABLE video_info ADD COLUMN can_share INTEGER DEFAULT 1",
    "ALTER TABLE video_info ADD COLUMN download_setting INTEGER DEFAULT 0",
    "ALTER TABLE video_info ADD COLUMN allow_douplus INTEGER DEFAULT 0",
    "ALTER TABLE video_info ADD COLUMN allow_share INTEGER DEFAULT 1",
    "ALTER TABLE video_info ADD COLUMN admire_count INTEGER DEFAULT 0",
    "ALTER TABLE video_info ADD COLUMN hashtag_ids TEXT",
    "ALTER TABLE video_info ADD COLUMN hashtag_names TEXT",
    "ALTER TABLE video_info ADD COLUMN images TEXT",
    "ALTER TABLE video_info ADD COLUMN region TEXT",
    "ALTER TABLE video_info ADD COLUMN is_prohibited INTEGER DEFAULT 0",
];

pub const MIGRATE_V3_INDEXES: &[&str] = &[
    "CREATE INDEX IF NOT EXISTS idx_video_author_sec_uid ON video_info(author_sec_uid)",
    "CREATE INDEX IF NOT EXISTS idx_video_create_time ON video_info(create_time)",
    "CREATE INDEX IF NOT EXISTS idx_user_nickname ON user_info(nickname)",
];

pub const MIGRATE_V4_LIVE_COVER: &[&str] = &[
    "ALTER TABLE live_records ADD COLUMN cover_url TEXT",
];

pub const MIGRATE_V5_DOWNLOAD_UNIQUE: &[&str] = &[
    "CREATE UNIQUE INDEX IF NOT EXISTS idx_download_unique ON download_history(aweme_id, file_path)",
];

pub const MIGRATE_V6_LIVE_UNIQUE: &[&str] = &[
    "CREATE UNIQUE INDEX IF NOT EXISTS idx_live_unique ON live_records(room_id, started_at)",
];

pub const MIGRATE_V7_TASK_TABLES: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS download_tasks (
        id TEXT PRIMARY KEY,
        mode TEXT NOT NULL,
        url TEXT NOT NULL,
        title TEXT,
        status TEXT NOT NULL DEFAULT 'running',
        total INTEGER NOT NULL DEFAULT 0,
        completed INTEGER NOT NULL DEFAULT 0,
        skipped INTEGER NOT NULL DEFAULT 0,
        failed INTEGER NOT NULL DEFAULT 0,
        error_msg TEXT,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
    )",
    "CREATE INDEX IF NOT EXISTS idx_task_status ON download_tasks(status)",
    "CREATE INDEX IF NOT EXISTS idx_task_mode ON download_tasks(mode)",
    "CREATE INDEX IF NOT EXISTS idx_task_created ON download_tasks(created_at)",
    "CREATE TABLE IF NOT EXISTS download_task_items (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        task_id TEXT NOT NULL,
        aweme_id TEXT,
        title TEXT,
        author_nickname TEXT,
        cover_url TEXT,
        file_path TEXT,
        file_size INTEGER DEFAULT 0,
        status TEXT NOT NULL DEFAULT 'pending',
        error_msg TEXT,
        created_at INTEGER NOT NULL,
        FOREIGN KEY (task_id) REFERENCES download_tasks(id) ON DELETE CASCADE
    )",
    "CREATE INDEX IF NOT EXISTS idx_item_task ON download_task_items(task_id)",
    "CREATE INDEX IF NOT EXISTS idx_item_status ON download_task_items(status)",
    "CREATE UNIQUE INDEX IF NOT EXISTS idx_item_unique ON download_task_items(task_id, aweme_id)",
];

pub const MIGRATE_V8_TASK_AUTHOR: &[&str] = &[
    "ALTER TABLE download_tasks ADD COLUMN author_nickname TEXT",
];

/// Execute migration SQLs, ignoring duplicate column/index errors
pub fn run_migration_sql(conn: &Connection, sqls: &[&str]) -> Result<()> {
    for sql in sqls {
        match conn.execute(sql, []) {
            Ok(_) => {}
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("duplicate column")
                    || msg.contains("already exists")
                    || msg.contains("SQLITE_ERROR") && msg.contains("duplicate")
                {
                    // duplicate column/index — 可忽略
                } else {
                    return Err(e);
                }
            }
        }
    }
    Ok(())
}

/// Run all schema migrations based on _metadata version
pub fn migrate(conn: &Connection) -> Result<()> {
    let version: i64 = conn
        .query_row(
            "SELECT COALESCE(CAST(value AS INTEGER), 0) FROM _metadata WHERE name = 'schema_version'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if version < 1 {
        info!("[DB] 迁移 v1: 扩展 user_info 表");
        run_migration_sql(conn, MIGRATE_V1_USER_INFO)?;
    }
    if version < 2 {
        info!("[DB] 迁移 v2: 扩展 video_info 表");
        run_migration_sql(conn, MIGRATE_V2_VIDEO_INFO)?;
    }
    if version < 3 {
        info!("[DB] 迁移 v3: 添加索引");
        run_migration_sql(conn, MIGRATE_V3_INDEXES)?;
    }
    if version < 4 {
        info!("[DB] 迁移 v4: live_records 添加 cover_url");
        run_migration_sql(conn, MIGRATE_V4_LIVE_COVER)?;
    }
    if version < 5 {
        info!("[DB] 迁移 v5: download_history 添加唯一索引");
        run_migration_sql(conn, MIGRATE_V5_DOWNLOAD_UNIQUE)?;
    }
    if version < 6 {
        info!("[DB] 迁移 v6: live_records 添加唯一索引 (room_id + started_at)");
        run_migration_sql(conn, MIGRATE_V6_LIVE_UNIQUE)?;
    }
    if version < 7 {
        info!("[DB] 迁移 v7: 新增 download_tasks / download_task_items 表");
        run_migration_sql(conn, MIGRATE_V7_TASK_TABLES)?;
    }
    if version < 8 {
        info!("[DB] 迁移 v8: download_tasks 添加 author_nickname");
        run_migration_sql(conn, MIGRATE_V8_TASK_AUTHOR)?;
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    conn.execute(
        "INSERT OR REPLACE INTO _metadata (name, value) VALUES ('schema_version', '8')",
        [],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO _metadata (name, value) VALUES ('schema_updated_at', ?1)",
        rusqlite::params![now.to_string()],
    )?;
    Ok(())
}
