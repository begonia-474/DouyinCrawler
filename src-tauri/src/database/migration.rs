//! Schema migration module
//!
//! Contains all CREATE TABLE SQL, column definitions, and migration logic.

use log::info;
use rusqlite::{Connection, Result};
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
    CREATE TABLE IF NOT EXISTS live_records (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        task_id TEXT,
        room_id TEXT,
        web_rid TEXT,
        title TEXT,
        nickname TEXT,
        sec_user_id TEXT,
        file_path TEXT,
        file_size INTEGER DEFAULT 0,
        duration_sec INTEGER DEFAULT 0,
        status TEXT NOT NULL DEFAULT 'completed',
        error_msg TEXT,
        started_at INTEGER,
        ended_at INTEGER,
        updated_at INTEGER NOT NULL DEFAULT 0
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
    CREATE INDEX IF NOT EXISTS idx_live_started ON live_records(started_at);
    CREATE INDEX IF NOT EXISTS idx_live_status ON live_records(status);
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

pub const MIGRATE_V4_LIVE_COVER: &[&str] = &["ALTER TABLE live_records ADD COLUMN cover_url TEXT"];

pub const MIGRATE_V6_LIVE_UNIQUE: &[&str] =
    &["CREATE UNIQUE INDEX IF NOT EXISTS idx_live_unique ON live_records(room_id, started_at)"];

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

pub const MIGRATE_V8_TASK_AUTHOR: &[&str] =
    &["ALTER TABLE download_tasks ADD COLUMN author_nickname TEXT"];

pub const MIGRATE_V9_LIVE_SEC_USER_INDEX: &[&str] =
    &["CREATE INDEX IF NOT EXISTS idx_live_sec_user_id ON live_records(sec_user_id)"];

pub const MIGRATE_V10_TASK_ITEMS_AUTHOR: &[&str] =
    &["ALTER TABLE download_task_items ADD COLUMN author_sec_uid TEXT"];

pub const MIGRATE_V11_MEDIA_KEY: &[&str] = &[
    "ALTER TABLE download_task_items ADD COLUMN media_key TEXT",
    "ALTER TABLE download_task_items ADD COLUMN media_kind TEXT",
    "ALTER TABLE download_task_items ADD COLUMN media_index INTEGER",
    "UPDATE download_task_items
     SET media_key = CASE
            WHEN aweme_id IS NOT NULL AND trim(aweme_id) <> ''
                THEN aweme_id || ':video:0'
            ELSE '__legacy_row:' || id
         END,
         media_kind = 'video',
         media_index = 0
     WHERE media_key IS NULL OR trim(media_key) = ''",
    "DROP INDEX IF EXISTS idx_item_unique",
    "CREATE UNIQUE INDEX IF NOT EXISTS idx_item_media_key ON download_task_items(task_id, media_key)",
    "CREATE INDEX IF NOT EXISTS idx_item_aweme ON download_task_items(task_id, aweme_id)",
];

pub const MIGRATE_V12_LIVE_LIFECYCLE: &[&str] = &[
    "ALTER TABLE live_records ADD COLUMN task_id TEXT",
    "ALTER TABLE live_records ADD COLUMN error_msg TEXT",
    "ALTER TABLE live_records ADD COLUMN updated_at INTEGER NOT NULL DEFAULT 0",
    "CREATE UNIQUE INDEX IF NOT EXISTS idx_live_task_unique ON live_records(task_id) WHERE task_id IS NOT NULL",
    "CREATE INDEX IF NOT EXISTS idx_live_status ON live_records(status)",
];

pub const MIGRATE_V10_DROP_DOWNLOAD_HISTORY: &[&str] = &[
    "DROP TABLE IF EXISTS download_history",
    "DROP INDEX IF EXISTS idx_download_created",
    "DROP INDEX IF EXISTS idx_download_type",
    "DROP INDEX IF EXISTS idx_download_status",
    "DROP INDEX IF EXISTS idx_download_author",
    "DROP INDEX IF EXISTS idx_download_unique",
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
    if version < 9 {
        info!("[DB] 迁移 v9: live_records 添加 sec_user_id 索引");
        run_migration_sql(conn, MIGRATE_V9_LIVE_SEC_USER_INDEX)?;
    }
    if version < 10 {
        info!("[DB] 迁移 v10: task_items 添加 author_sec_uid，移除 download_history");
        run_migration_sql(conn, MIGRATE_V10_TASK_ITEMS_AUTHOR)?;
        run_migration_sql(conn, MIGRATE_V10_DROP_DOWNLOAD_HISTORY)?;
    }
    if version < 11 {
        info!("[DB] 迁移 v11: task_items 添加 media_key/media_kind/media_index，唯一键改为 (task_id, media_key)");
        run_migration_sql(conn, MIGRATE_V11_MEDIA_KEY)?;
    }
    if version < 12 {
        info!("[DB] 迁移 v12: live_records 接入 task 生命周期");
        run_migration_sql(conn, MIGRATE_V12_LIVE_LIFECYCLE)?;
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    conn.execute(
        "INSERT OR REPLACE INTO _metadata (name, value) VALUES ('schema_version', '12')",
        [],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO _metadata (name, value) VALUES ('schema_updated_at', ?1)",
        rusqlite::params![now.to_string()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_v10_database() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE _metadata (name TEXT PRIMARY KEY, value TEXT);
             INSERT INTO _metadata (name, value) VALUES ('schema_version', '10');
             CREATE TABLE download_tasks (
                id TEXT PRIMARY KEY, mode TEXT NOT NULL, url TEXT NOT NULL,
                title TEXT, author_nickname TEXT, status TEXT NOT NULL DEFAULT 'running',
                total INTEGER NOT NULL DEFAULT 0, completed INTEGER NOT NULL DEFAULT 0,
                skipped INTEGER NOT NULL DEFAULT 0, failed INTEGER NOT NULL DEFAULT 0,
                error_msg TEXT, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL
             );
             CREATE TABLE download_task_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT, task_id TEXT NOT NULL,
                aweme_id TEXT, title TEXT, author_nickname TEXT, cover_url TEXT,
                file_path TEXT, file_size INTEGER DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'pending', error_msg TEXT,
                created_at INTEGER NOT NULL, author_sec_uid TEXT
             );
             CREATE UNIQUE INDEX idx_item_unique
                ON download_task_items(task_id, aweme_id);
             CREATE TABLE live_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                room_id TEXT, web_rid TEXT, title TEXT, nickname TEXT,
                sec_user_id TEXT, file_path TEXT, file_size INTEGER DEFAULT 0,
                duration_sec INTEGER DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'completed',
                started_at INTEGER, ended_at INTEGER, cover_url TEXT
             );",
        )
        .unwrap();
        conn
    }

    fn create_v11_database() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE _metadata (name TEXT PRIMARY KEY, value TEXT);
             INSERT INTO _metadata (name, value) VALUES ('schema_version', '11');
             CREATE TABLE live_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                room_id TEXT, web_rid TEXT, title TEXT, nickname TEXT,
                sec_user_id TEXT, file_path TEXT, file_size INTEGER DEFAULT 0,
                duration_sec INTEGER DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'completed',
                started_at INTEGER, ended_at INTEGER, cover_url TEXT
             );
             INSERT INTO live_records
                (room_id, title, file_path, status, started_at)
             VALUES ('historical-room', 'historical', '/tmp/old.flv', 'completed', 10);",
        )
        .unwrap();
        conn
    }

    fn index_columns(conn: &Connection, index: &str) -> Vec<String> {
        let mut stmt = conn
            .prepare(&format!("PRAGMA index_info('{index}')"))
            .unwrap();
        stmt.query_map([], |row| row.get(2))
            .unwrap()
            .collect::<Result<Vec<_>>>()
            .unwrap()
    }

    #[test]
    fn fresh_database_migrates_to_v12_with_live_lifecycle_columns() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(CREATE_TABLES_SQL).unwrap();

        migrate(&conn).unwrap();

        let version: String = conn
            .query_row(
                "SELECT value FROM _metadata WHERE name = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, "12");
        assert_eq!(
            index_columns(&conn, "idx_item_media_key"),
            ["task_id", "media_key"]
        );
        assert_eq!(
            index_columns(&conn, "idx_item_aweme"),
            ["task_id", "aweme_id"]
        );
        let live_columns: Vec<String> = conn
            .prepare("PRAGMA table_info('live_records')")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .collect::<Result<Vec<_>>>()
            .unwrap();
        assert!(live_columns.contains(&"task_id".to_string()));
        assert!(live_columns.contains(&"error_msg".to_string()));
        assert!(live_columns.contains(&"updated_at".to_string()));
        assert_eq!(index_columns(&conn, "idx_live_task_unique"), ["task_id"]);
    }

    #[test]
    fn v10_rows_are_backfilled_without_identity_collisions() {
        let conn = create_v10_database();
        conn.execute_batch(
            "INSERT INTO download_task_items (task_id, aweme_id, created_at)
                VALUES ('task', 'aweme', 1), ('task', NULL, 1), ('task', '', 1);",
        )
        .unwrap();

        migrate(&conn).unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT id, aweme_id, media_key, media_kind, media_index
                 FROM download_task_items ORDER BY id",
            )
            .unwrap();
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            })
            .unwrap()
            .collect::<Result<Vec<_>>>()
            .unwrap();
        assert_eq!(rows[0].2, "aweme:video:0");
        assert_eq!(rows[1].2, format!("__legacy_row:{}", rows[1].0));
        assert_eq!(rows[2].2, format!("__legacy_row:{}", rows[2].0));
        assert_ne!(rows[1].2, rows[2].2);
        assert!(rows.iter().all(|row| row.3 == "video" && row.4 == 0));
        assert!(index_columns(&conn, "idx_item_unique").is_empty());
    }

    #[test]
    fn v11_upgrades_to_v12_without_guessing_historical_task_links() {
        let conn = create_v11_database();

        migrate(&conn).unwrap();

        let version: String = conn
            .query_row(
                "SELECT value FROM _metadata WHERE name = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let historical: (Option<String>, String, String) = conn
            .query_row(
                "SELECT task_id, status, file_path FROM live_records WHERE room_id = 'historical-room'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(version, "12");
        assert_eq!(historical.0, None);
        assert_eq!(historical.1, "completed");
        assert_eq!(historical.2, "/tmp/old.flv");
    }

    #[test]
    fn database_open_upgrades_real_v11_file_before_creating_task_index() {
        let path = std::env::temp_dir().join(format!(
            "douyin-v11-live-migration-{}.sqlite",
            uuid::Uuid::new_v4()
        ));
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(
                "CREATE TABLE _metadata (name TEXT PRIMARY KEY, value TEXT);
                 INSERT INTO _metadata (name, value) VALUES ('schema_version', '11');
                 CREATE TABLE live_records (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    room_id TEXT, web_rid TEXT, title TEXT, nickname TEXT,
                    sec_user_id TEXT, file_path TEXT, file_size INTEGER DEFAULT 0,
                    duration_sec INTEGER DEFAULT 0,
                    status TEXT NOT NULL DEFAULT 'completed',
                    started_at INTEGER, ended_at INTEGER, cover_url TEXT
                 );
                 INSERT INTO live_records (room_id, status, started_at)
                 VALUES ('old-room', 'completed', 1);",
            )
            .unwrap();
        }

        let db = crate::database::connection::Database::open(&path).unwrap();
        let records = db.get_live_records(10, 0).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].task_id, None);
        drop(db);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn v12_rejects_duplicate_non_null_live_task_id_but_allows_null_history() {
        let conn = create_v11_database();
        migrate(&conn).unwrap();

        conn.execute(
            "INSERT INTO live_records (task_id, status, updated_at) VALUES ('task-1', 'recording', 1)",
            [],
        )
        .unwrap();
        assert!(conn
            .execute(
                "INSERT INTO live_records (task_id, status, updated_at) VALUES ('task-1', 'error', 2)",
                [],
            )
            .is_err());
        conn.execute(
            "INSERT INTO live_records (task_id, status, updated_at) VALUES (NULL, 'completed', 3)",
            [],
        )
        .unwrap();
    }

    #[test]
    fn v11_allows_multiple_media_for_one_aweme_but_rejects_duplicate_key() {
        let conn = create_v10_database();
        migrate(&conn).unwrap();

        conn.execute(
            "INSERT INTO download_task_items
             (task_id, aweme_id, media_key, media_kind, media_index, created_at)
             VALUES ('task', 'aweme', 'aweme:image:1', 'image', 1, 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO download_task_items
             (task_id, aweme_id, media_key, media_kind, media_index, created_at)
             VALUES ('task', 'aweme', 'aweme:image:2', 'image', 2, 1)",
            [],
        )
        .unwrap();
        assert!(conn
            .execute(
                "INSERT INTO download_task_items
                 (task_id, aweme_id, media_key, media_kind, media_index, created_at)
                 VALUES ('task', 'other', 'aweme:image:2', 'image', 2, 1)",
                [],
            )
            .is_err());
    }

    #[test]
    fn repeated_migrate_is_idempotent() {
        let conn = create_v10_database();
        conn.execute(
            "INSERT INTO download_task_items (task_id, aweme_id, created_at)
             VALUES ('task', NULL, 1)",
            [],
        )
        .unwrap();
        migrate(&conn).unwrap();
        let first_key: String = conn
            .query_row(
                "SELECT media_key FROM download_task_items WHERE task_id = 'task'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        migrate(&conn).unwrap();

        let second_key: String = conn
            .query_row(
                "SELECT media_key FROM download_task_items WHERE task_id = 'task'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(first_key, second_key);
        assert_eq!(index_columns(&conn, "idx_item_media_key").len(), 2);
    }
}
