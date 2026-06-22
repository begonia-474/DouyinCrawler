use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

// === Upsert 列类型 ===

enum ColKind {
    /// 普通字段：冲突时直接覆盖
    Normal,
    /// 时效性字段：冲突时仅当新值非空才覆盖
    Volatile,
    /// 统计字段：冲突时取 MAX（只增不减）
    Stat,
}

const CREATE_TABLES_SQL: &str = "
    CREATE TABLE IF NOT EXISTS _metadata (
        name TEXT PRIMARY KEY,
        value TEXT
    );
    CREATE TABLE IF NOT EXISTS config (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL,
        updated_at INTEGER NOT NULL
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

// === Schema 迁移 ===

const MIGRATE_V1_USER_INFO: &[&str] = &[
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

const MIGRATE_V2_VIDEO_INFO: &[&str] = &[
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

const MIGRATE_V3_INDEXES: &[&str] = &[
    "CREATE INDEX IF NOT EXISTS idx_video_author_sec_uid ON video_info(author_sec_uid)",
    "CREATE INDEX IF NOT EXISTS idx_video_create_time ON video_info(create_time)",
    "CREATE INDEX IF NOT EXISTS idx_user_nickname ON user_info(nickname)",
];

const MIGRATE_V4_LIVE_COVER: &[&str] = &[
    "ALTER TABLE live_records ADD COLUMN cover_url TEXT",
];

const DEFAULT_CONFIG: &[(&str, &str)] = &[
    ("cookie", ""),
    ("download_path", "Download"),
    ("naming", "{create}_{desc}"),
    ("encryption", "ab"),
    ("proxy", ""),
];

pub struct Database {
    conn: Mutex<Connection>,
}

#[derive(Serialize, Clone)]
pub struct DownloadRecord {
    pub id: i64,
    pub aweme_id: Option<String>,
    pub download_type: String,
    pub title: Option<String>,
    pub author_nickname: Option<String>,
    pub author_sec_uid: Option<String>,
    pub file_path: Option<String>,
    pub file_size: i64,
    pub cover_url: Option<String>,
    pub status: String,
    pub error_msg: Option<String>,
    pub created_at: i64,
}

#[derive(Serialize, Clone)]
pub struct DownloadStats {
    pub total_count: i64,
    pub total_size: i64,
    pub by_type: Vec<TypeStat>,
    pub by_day: Vec<DayStat>,
}

#[derive(Serialize, Clone)]
pub struct TypeStat {
    pub download_type: String,
    pub cnt: i64,
    pub size: i64,
}

#[derive(Serialize, Clone)]
pub struct DayStat {
    pub day: String,
    pub cnt: i64,
}

#[derive(Serialize, Clone)]
pub struct LiveRecord {
    pub id: i64,
    pub room_id: Option<String>,
    pub web_rid: Option<String>,
    pub title: Option<String>,
    pub nickname: Option<String>,
    pub sec_user_id: Option<String>,
    pub file_path: Option<String>,
    pub file_size: i64,
    pub duration_sec: i64,
    pub status: String,
    pub started_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub cover_url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub cookie: String,
    pub download_path: String,
    pub naming: String,
    pub encryption: String,
    pub proxy: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            cookie: String::new(),
            download_path: "Download".to_string(),
            naming: "{create}_{desc}".to_string(),
            encryption: "ab".to_string(),
            proxy: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NewDownloadRecord {
    pub aweme_id: Option<String>,
    pub download_type: String,
    pub title: Option<String>,
    pub author_nickname: Option<String>,
    pub author_sec_uid: Option<String>,
    pub file_path: Option<String>,
    pub file_size: i64,
    pub cover_url: Option<String>,
    pub status: String,
    pub error_msg: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NewLiveRecord {
    pub room_id: Option<String>,
    pub web_rid: Option<String>,
    pub title: Option<String>,
    pub nickname: Option<String>,
    pub sec_user_id: Option<String>,
    pub file_path: Option<String>,
    pub file_size: i64,
    pub duration_sec: i64,
    pub status: String,
    pub started_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub cover_url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub sec_user_id: String,
    pub nickname: Option<String>,
    pub uid: Option<String>,
    pub avatar_url: Option<String>,
    pub unique_id: Option<String>,
    pub signature: Option<String>,
    pub aweme_count: i64,
    pub follower_count: i64,
    pub following_count: i64,
    pub total_favorited: i64,
    pub ip_location: Option<String>,
    pub live_status: i32,
    pub room_id: Option<String>,
    // f2 对齐字段
    #[serde(default)] pub city: Option<String>,
    #[serde(default)] pub country: Option<String>,
    #[serde(default)] pub favoriting_count: i64,
    #[serde(default)] pub gender: i32,
    #[serde(default)] pub is_ban: i32,
    #[serde(default)] pub is_block: i32,
    #[serde(default)] pub is_blocked: i32,
    #[serde(default)] pub is_star: i32,
    #[serde(default)] pub mix_count: i32,
    #[serde(default)] pub mplatform_followers_count: i64,
    #[serde(default)] pub nickname_raw: Option<String>,
    #[serde(default)] pub school_name: Option<String>,
    #[serde(default)] pub short_id: Option<String>,
    #[serde(default)] pub signature_raw: Option<String>,
    #[serde(default)] pub user_age: i32,
    #[serde(default)] pub custom_verify: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VideoInfo {
    pub aweme_id: String,
    pub desc: Option<String>,
    pub aweme_type: i32,
    pub author_nickname: Option<String>,
    pub author_sec_uid: Option<String>,
    pub author_uid: Option<String>,
    pub create_time: Option<i64>,
    pub duration: i32,
    pub video_url: Option<String>,
    pub cover_url: Option<String>,
    pub music_title: Option<String>,
    pub digg_count: i64,
    pub comment_count: i64,
    pub share_count: i64,
    pub collect_count: i64,
    pub mix_id: Option<String>,
    pub mix_name: Option<String>,
    // f2 对齐字段 - 作者
    #[serde(default)] pub author_nickname_raw: Option<String>,
    #[serde(default)] pub author_short_id: Option<String>,
    #[serde(default)] pub author_unique_id: Option<String>,
    // f2 对齐字段 - 内容
    #[serde(default)] pub desc_raw: Option<String>,
    #[serde(default)] pub is_ads: i32,
    #[serde(default)] pub is_story: i32,
    #[serde(default)] pub is_top: i32,
    #[serde(default)] pub is_long_video: i32,
    // f2 对齐字段 - 视频
    #[serde(default)] pub video_bit_rate: Option<String>,
    #[serde(default)] pub animated_cover: Option<String>,
    #[serde(default)] pub private_status: i32,
    #[serde(default)] pub is_delete: i32,
    // f2 对齐字段 - 音乐
    #[serde(default)] pub music_author: Option<String>,
    #[serde(default)] pub music_author_raw: Option<String>,
    #[serde(default)] pub music_duration: i32,
    #[serde(default)] pub music_id: Option<String>,
    #[serde(default)] pub music_mid: Option<String>,
    #[serde(default)] pub pgc_author: Option<String>,
    #[serde(default)] pub pgc_author_title: Option<String>,
    #[serde(default)] pub pgc_music_type: i32,
    #[serde(default)] pub music_status: i32,
    #[serde(default)] pub music_owner_handle: Option<String>,
    #[serde(default)] pub music_owner_id: Option<String>,
    #[serde(default)] pub music_owner_nickname: Option<String>,
    #[serde(default)] pub music_play_url: Option<String>,
    #[serde(default)] pub is_commerce_music: i32,
    // f2 对齐字段 - 合集
    #[serde(default)] pub mix_desc: Option<String>,
    #[serde(default)] pub mix_create_time: i64,
    #[serde(default)] pub mix_pic_type: i32,
    #[serde(default)] pub mix_type: i32,
    #[serde(default)] pub mix_share_url: Option<String>,
    // f2 对齐字段 - 权限
    #[serde(default)] pub can_comment: i32,
    #[serde(default)] pub can_forward: i32,
    #[serde(default)] pub can_share: i32,
    #[serde(default)] pub download_setting: i32,
    #[serde(default)] pub allow_douplus: i32,
    #[serde(default)] pub allow_share: i32,
    // f2 对齐字段 - 统计/标签/其他
    #[serde(default)] pub admire_count: i64,
    #[serde(default)] pub hashtag_ids: Option<String>,
    #[serde(default)] pub hashtag_names: Option<String>,
    #[serde(default)] pub images: Option<String>,
    #[serde(default)] pub region: Option<String>,
    #[serde(default)] pub is_prohibited: i32,
}

#[derive(Serialize, Clone)]
pub struct VideoStats {
    pub total_count: i64,
    pub total_digg: i64,
    pub total_comment: i64,
    pub total_share: i64,
    pub total_collect: i64,
    pub by_type: Vec<VideoTypeStat>,
}

#[derive(Serialize, Clone)]
pub struct VideoTypeStat {
    pub aweme_type: i32,
    pub cnt: i64,
}

#[derive(Serialize, Clone)]
pub struct UserStats {
    pub total_count: i64,
    pub total_follower: i64,
    pub total_aweme: i64,
}

#[derive(Serialize, Clone)]
pub struct MusicCollection {
    pub music_id: String,
    pub mid: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub owner_nickname: Option<String>,
    pub duration: i32,
    pub cover: Option<String>,
    pub play_url: Option<String>,
    pub file_path: Option<String>,
    pub status: String,
    pub created_at: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NewMusicCollection {
    pub music_id: String,
    pub mid: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub owner_nickname: Option<String>,
    pub duration: i32,
    pub cover: Option<String>,
    pub play_url: Option<String>,
}

impl Database {
    pub fn open(path: &PathBuf) -> Result<Self> {
        // 确保父目录存在（失败时 Connection::open 也会失败，无需包装错误）
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let conn = Connection::open(path)?;
        // 启用 WAL 模式，与 Python 侧一致
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
        // 执行 WAL checkpoint，确保读取到最新数据
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        // 自动建表（与 core/db.py 的 _create_tables 保持一致）
        conn.execute_batch(CREATE_TABLES_SQL)?;
        // Schema 迁移
        Self::migrate(&conn)?;
        // 初始化默认配置
        Self::init_default_config(&conn)?;
        // 验证数据
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM download_history",
            [],
            |row| row.get(0),
        ).unwrap_or(0);
        println!("[DB] 数据库打开成功，download_history 表有 {} 条记录", count);
        Ok(Database {
            conn: Mutex::new(conn),
        })
    }

    pub fn get_downloads(
        &self,
        limit: i64,
        offset: i64,
        status: Option<String>,
        download_type: Option<String>,
    ) -> Result<Vec<DownloadRecord>> {
        let conn = self.conn.lock().unwrap();

        // 每次查询前执行 WAL checkpoint，确保读取到最新数据
        conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE);")?;

        let mut sql = String::from(
            "SELECT id, aweme_id, download_type, title, author_nickname, author_sec_uid, \
             file_path, file_size, cover_url, status, error_msg, created_at \
             FROM download_history",
        );
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut conditions = Vec::new();

        if let Some(ref s) = status {
            conditions.push("status = ?");
            params.push(Box::new(s.clone()));
        }
        if let Some(ref t) = download_type {
            conditions.push("download_type = ?");
            params.push(Box::new(t.clone()));
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }
        sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");
        params.push(Box::new(limit));
        params.push(Box::new(offset));

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        println!("[DB] get_downloads SQL: {}", sql);
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok(DownloadRecord {
                id: row.get(0)?,
                aweme_id: row.get(1)?,
                download_type: row.get(2)?,
                title: row.get(3)?,
                author_nickname: row.get(4)?,
                author_sec_uid: row.get(5)?,
                file_path: row.get(6)?,
                file_size: row.get(7)?,
                cover_url: row.get(8)?,
                status: row.get(9)?,
                error_msg: row.get(10)?,
                created_at: row.get(11)?,
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        println!("[DB] get_downloads 返回 {} 条记录", records.len());
        Ok(records)
    }

    pub fn get_download_stats(&self) -> Result<DownloadStats> {
        let conn = self.conn.lock().unwrap();

        // 总计
        let (total_count, total_size): (i64, i64) = conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(file_size), 0) FROM download_history WHERE status = 'completed'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        // 按类型
        let mut stmt = conn.prepare(
            "SELECT download_type, COUNT(*), COALESCE(SUM(file_size), 0) \
             FROM download_history WHERE status = 'completed' GROUP BY download_type",
        )?;
        let by_type: Vec<TypeStat> = stmt
            .query_map([], |row| {
                Ok(TypeStat {
                    download_type: row.get(0)?,
                    cnt: row.get(1)?,
                    size: row.get(2)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        // 按日（最近 7 天）
        let mut stmt = conn.prepare(
            "SELECT DATE(created_at, 'unixepoch', 'localtime') as day, COUNT(*) \
             FROM download_history WHERE status = 'completed' \
             AND created_at > strftime('%s', 'now', '-7 days') \
             GROUP BY day ORDER BY day DESC",
        )?;
        let by_day: Vec<DayStat> = stmt
            .query_map([], |row| {
                Ok(DayStat {
                    day: row.get(0)?,
                    cnt: row.get(1)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(DownloadStats {
            total_count,
            total_size,
            by_type,
            by_day,
        })
    }

    pub fn get_live_records(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<LiveRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, room_id, web_rid, title, nickname, sec_user_id, \
             file_path, file_size, duration_sec, status, started_at, ended_at, cover_url \
             FROM live_records ORDER BY started_at DESC LIMIT ? OFFSET ?",
        )?;

        let rows = stmt.query_map(rusqlite::params![limit, offset], |row| {
            Ok(LiveRecord {
                id: row.get(0)?,
                room_id: row.get(1)?,
                web_rid: row.get(2)?,
                title: row.get(3)?,
                nickname: row.get(4)?,
                sec_user_id: row.get(5)?,
                file_path: row.get(6)?,
                file_size: row.get(7)?,
                duration_sec: row.get(8)?,
                status: row.get(9)?,
                started_at: row.get(10)?,
                ended_at: row.get(11)?,
                cover_url: row.get(12)?,
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    // === video_info / user_info 查询 ===

    fn row_to_video(row: &rusqlite::Row) -> rusqlite::Result<VideoInfo> {
        Ok(VideoInfo {
            aweme_id: row.get("aweme_id")?,
            desc: row.get("desc")?,
            aweme_type: row.get::<_, Option<i32>>("aweme_type")?.unwrap_or(0),
            author_nickname: row.get("author_nickname")?,
            author_sec_uid: row.get("author_sec_uid")?,
            author_uid: row.get("author_uid")?,
            create_time: row.get("create_time")?,
            duration: row.get::<_, Option<i32>>("duration")?.unwrap_or(0),
            video_url: row.get("video_url")?,
            cover_url: row.get("cover_url")?,
            music_title: row.get("music_title")?,
            digg_count: row.get::<_, Option<i64>>("digg_count")?.unwrap_or(0),
            comment_count: row.get::<_, Option<i64>>("comment_count")?.unwrap_or(0),
            share_count: row.get::<_, Option<i64>>("share_count")?.unwrap_or(0),
            collect_count: row.get::<_, Option<i64>>("collect_count")?.unwrap_or(0),
            mix_id: row.get("mix_id")?,
            mix_name: row.get("mix_name")?,
            author_nickname_raw: row.get("author_nickname_raw")?,
            author_short_id: row.get("author_short_id")?,
            author_unique_id: row.get("author_unique_id")?,
            desc_raw: row.get("desc_raw")?,
            is_ads: row.get::<_, Option<i32>>("is_ads")?.unwrap_or(0),
            is_story: row.get::<_, Option<i32>>("is_story")?.unwrap_or(0),
            is_top: row.get::<_, Option<i32>>("is_top")?.unwrap_or(0),
            is_long_video: row.get::<_, Option<i32>>("is_long_video")?.unwrap_or(0),
            video_bit_rate: row.get("video_bit_rate")?,
            animated_cover: row.get("animated_cover")?,
            private_status: row.get::<_, Option<i32>>("private_status")?.unwrap_or(0),
            is_delete: row.get::<_, Option<i32>>("is_delete")?.unwrap_or(0),
            music_author: row.get("music_author")?,
            music_author_raw: row.get("music_author_raw")?,
            music_duration: row.get::<_, Option<i32>>("music_duration")?.unwrap_or(0),
            music_id: row.get("music_id")?,
            music_mid: row.get("music_mid")?,
            pgc_author: row.get("pgc_author")?,
            pgc_author_title: row.get("pgc_author_title")?,
            pgc_music_type: row.get::<_, Option<i32>>("pgc_music_type")?.unwrap_or(0),
            music_status: row.get::<_, Option<i32>>("music_status")?.unwrap_or(0),
            music_owner_handle: row.get("music_owner_handle")?,
            music_owner_id: row.get("music_owner_id")?,
            music_owner_nickname: row.get("music_owner_nickname")?,
            music_play_url: row.get("music_play_url")?,
            is_commerce_music: row.get::<_, Option<i32>>("is_commerce_music")?.unwrap_or(0),
            mix_desc: row.get("mix_desc")?,
            mix_create_time: row.get::<_, Option<i64>>("mix_create_time")?.unwrap_or(0),
            mix_pic_type: row.get::<_, Option<i32>>("mix_pic_type")?.unwrap_or(0),
            mix_type: row.get::<_, Option<i32>>("mix_type")?.unwrap_or(0),
            mix_share_url: row.get("mix_share_url")?,
            can_comment: row.get::<_, Option<i32>>("can_comment")?.unwrap_or(0),
            can_forward: row.get::<_, Option<i32>>("can_forward")?.unwrap_or(0),
            can_share: row.get::<_, Option<i32>>("can_share")?.unwrap_or(0),
            download_setting: row.get::<_, Option<i32>>("download_setting")?.unwrap_or(0),
            allow_douplus: row.get::<_, Option<i32>>("allow_douplus")?.unwrap_or(0),
            allow_share: row.get::<_, Option<i32>>("allow_share")?.unwrap_or(0),
            admire_count: row.get::<_, Option<i64>>("admire_count")?.unwrap_or(0),
            hashtag_ids: row.get("hashtag_ids")?,
            hashtag_names: row.get("hashtag_names")?,
            images: row.get("images")?,
            region: row.get("region")?,
            is_prohibited: row.get::<_, Option<i32>>("is_prohibited")?.unwrap_or(0),
        })
    }

    fn row_to_user(row: &rusqlite::Row) -> rusqlite::Result<UserInfo> {
        Ok(UserInfo {
            sec_user_id: row.get("sec_user_id")?,
            nickname: row.get("nickname")?,
            uid: row.get("uid")?,
            avatar_url: row.get("avatar_url")?,
            unique_id: row.get("unique_id")?,
            signature: row.get("signature")?,
            aweme_count: row.get::<_, Option<i64>>("aweme_count")?.unwrap_or(0),
            follower_count: row.get::<_, Option<i64>>("follower_count")?.unwrap_or(0),
            following_count: row.get::<_, Option<i64>>("following_count")?.unwrap_or(0),
            total_favorited: row.get::<_, Option<i64>>("total_favorited")?.unwrap_or(0),
            ip_location: row.get("ip_location")?,
            live_status: row.get::<_, Option<i32>>("live_status")?.unwrap_or(0),
            room_id: row.get("room_id")?,
            city: row.get("city")?,
            country: row.get("country")?,
            favoriting_count: row.get::<_, Option<i64>>("favoriting_count")?.unwrap_or(0),
            gender: row.get::<_, Option<i32>>("gender")?.unwrap_or(0),
            is_ban: row.get::<_, Option<i32>>("is_ban")?.unwrap_or(0),
            is_block: row.get::<_, Option<i32>>("is_block")?.unwrap_or(0),
            is_blocked: row.get::<_, Option<i32>>("is_blocked")?.unwrap_or(0),
            is_star: row.get::<_, Option<i32>>("is_star")?.unwrap_or(0),
            mix_count: row.get::<_, Option<i32>>("mix_count")?.unwrap_or(0),
            mplatform_followers_count: row.get::<_, Option<i64>>("mplatform_followers_count")?.unwrap_or(0),
            nickname_raw: row.get("nickname_raw")?,
            school_name: row.get("school_name")?,
            short_id: row.get("short_id")?,
            signature_raw: row.get("signature_raw")?,
            user_age: row.get::<_, Option<i32>>("user_age")?.unwrap_or(0),
            custom_verify: row.get("custom_verify")?,
        })
    }

    fn validate_sort_field(allowed: &[&str], sort_by: &Option<String>, default: &str) -> String {
        match sort_by {
            Some(s) if allowed.contains(&s.as_str()) => s.clone(),
            _ => default.to_string(),
        }
    }

    fn validate_sort_order(order: &Option<String>) -> String {
        match order.as_deref() {
            Some("asc") => "ASC".to_string(),
            _ => "DESC".to_string(),
        }
    }

    /// 构建 upsert SQL：INSERT ... ON CONFLICT(pk) DO UPDATE SET ...
    /// - Normal: 冲突时直接覆盖
    /// - Volatile: 冲突时仅当新值非空才覆盖（时效性字段）
    /// - Stat: 冲突时取 MAX（只增不减）
    fn build_upsert_sql(
        table: &str,
        pk: &str,
        cols: &[(&str, ColKind)],
    ) -> String {
        let col_names: Vec<&str> = cols.iter().map(|(name, _)| *name).collect();
        let placeholders: Vec<String> = (1..=cols.len()).map(|i| format!("?{}", i)).collect();

        let mut set_parts = Vec::new();
        for (_i, (name, kind)) in cols.iter().enumerate() {
            match kind {
                ColKind::Normal => {
                    set_parts.push(format!("{} = excluded.{}", name, name));
                }
                ColKind::Volatile => {
                    set_parts.push(format!(
                        "{0} = CASE WHEN excluded.{0} IS NOT NULL AND excluded.{0} != '' \
                         THEN excluded.{0} ELSE {1}.{0} END",
                        name, table
                    ));
                }
                ColKind::Stat => {
                    set_parts.push(format!(
                        "{0} = CASE WHEN excluded.{0} > {1}.{0} OR {1}.{0} IS NULL \
                         THEN excluded.{0} ELSE {1}.{0} END",
                        name, table
                    ));
                }
            }
        }

        format!(
            "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT({}) DO UPDATE SET {}",
            table,
            col_names.join(", "),
            placeholders.join(", "),
            pk,
            set_parts.join(", ")
        )
    }

    pub fn get_videos(
        &self,
        limit: i64,
        offset: i64,
        keyword: Option<String>,
        author_sec_uid: Option<String>,
        sort_by: Option<String>,
        sort_order: Option<String>,
        post_type: Option<String>,
    ) -> Result<Vec<VideoInfo>> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE);")?;

        let allowed_sorts = ["create_time", "digg_count", "comment_count", "share_count", "collect_count", "updated_at"];
        let sort_col = Self::validate_sort_field(&allowed_sorts, &sort_by, "updated_at");
        let sort_dir = Self::validate_sort_order(&sort_order);

        let mut sql = String::from("SELECT * FROM video_info");
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut conditions = Vec::new();

        if let Some(ref pt) = post_type {
            match pt.as_str() {
                "video" => conditions.push("aweme_type != 68"),
                "images" => conditions.push("aweme_type = 68"),
                _ => {}
            }
        }
        if let Some(ref author) = author_sec_uid {
            if !author.is_empty() {
                conditions.push("author_sec_uid = ?");
                params.push(Box::new(author.clone()));
            }
        }
        if let Some(ref kw) = keyword {
            if !kw.is_empty() {
                conditions.push("(desc LIKE ? OR author_nickname LIKE ?)");
                let pattern = format!("%{}%", kw);
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern));
            }
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }
        sql.push_str(&format!(" ORDER BY {} {} LIMIT ? OFFSET ?", sort_col, sort_dir));
        params.push(Box::new(limit));
        params.push(Box::new(offset));

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(param_refs.as_slice(), Self::row_to_video)?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    pub fn get_video_count(
        &self,
        keyword: Option<String>,
        author_sec_uid: Option<String>,
        post_type: Option<String>,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let mut sql = String::from("SELECT COUNT(*) FROM video_info");
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut conditions = Vec::new();

        if let Some(ref pt) = post_type {
            match pt.as_str() {
                "video" => conditions.push("aweme_type != 68"),
                "images" => conditions.push("aweme_type = 68"),
                _ => {}
            }
        }
        if let Some(ref author) = author_sec_uid {
            if !author.is_empty() {
                conditions.push("author_sec_uid = ?");
                params.push(Box::new(author.clone()));
            }
        }
        if let Some(ref kw) = keyword {
            if !kw.is_empty() {
                conditions.push("(desc LIKE ? OR author_nickname LIKE ?)");
                let pattern = format!("%{}%", kw);
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern));
            }
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let count: i64 = conn.query_row(&sql, param_refs.as_slice(), |row| row.get(0))?;
        Ok(count)
    }

    pub fn get_users(
        &self,
        limit: i64,
        offset: i64,
        keyword: Option<String>,
        sort_by: Option<String>,
        sort_order: Option<String>,
    ) -> Result<Vec<UserInfo>> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE);")?;

        let allowed_sorts = ["follower_count", "aweme_count", "following_count", "total_favorited", "updated_at"];
        let sort_col = Self::validate_sort_field(&allowed_sorts, &sort_by, "updated_at");
        let sort_dir = Self::validate_sort_order(&sort_order);

        let mut sql = String::from("SELECT * FROM user_info");
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut conditions = Vec::new();

        if let Some(ref kw) = keyword {
            if !kw.is_empty() {
                conditions.push("(nickname LIKE ? OR unique_id LIKE ? OR signature LIKE ?)");
                let pattern = format!("%{}%", kw);
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern));
            }
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }
        sql.push_str(&format!(" ORDER BY {} {} LIMIT ? OFFSET ?", sort_col, sort_dir));
        params.push(Box::new(limit));
        params.push(Box::new(offset));

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(param_refs.as_slice(), Self::row_to_user)?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    pub fn get_user_count(&self, keyword: Option<String>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let mut sql = String::from("SELECT COUNT(*) FROM user_info");
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref kw) = keyword {
            if !kw.is_empty() {
                sql.push_str(" WHERE (nickname LIKE ? OR unique_id LIKE ? OR signature LIKE ?)");
                let pattern = format!("%{}%", kw);
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern));
            }
        }

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let count: i64 = conn.query_row(&sql, param_refs.as_slice(), |row| row.get(0))?;
        Ok(count)
    }

    pub fn get_user_by_sec_uid(&self, sec_user_id: &str) -> Result<Option<UserInfo>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM user_info WHERE sec_user_id = ?1")?;
        let mut rows = stmt.query_map(rusqlite::params![sec_user_id], Self::row_to_user)?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn get_videos_by_author(
        &self,
        author_sec_uid: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<VideoInfo>> {
        self.get_videos(limit, offset, None, Some(author_sec_uid.to_string()), Some("create_time".to_string()), Some("desc".to_string()), None)
    }

    pub fn get_video_stats(&self) -> Result<VideoStats> {
        let conn = self.conn.lock().unwrap();
        let (total_count, total_digg, total_comment, total_share, total_collect) = conn.query_row(
            "SELECT COALESCE(COUNT(*),0), COALESCE(SUM(digg_count),0), COALESCE(SUM(comment_count),0), \
             COALESCE(SUM(share_count),0), COALESCE(SUM(collect_count),0) FROM video_info",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )?;

        let mut stmt = conn.prepare(
            "SELECT aweme_type, COUNT(*) FROM video_info GROUP BY aweme_type ORDER BY COUNT(*) DESC",
        )?;
        let by_type: Vec<VideoTypeStat> = stmt
            .query_map([], |row| {
                Ok(VideoTypeStat {
                    aweme_type: row.get::<_, Option<i32>>(0)?.unwrap_or(0),
                    cnt: row.get(1)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(VideoStats { total_count, total_digg, total_comment, total_share, total_collect, by_type })
    }

    pub fn get_user_stats(&self) -> Result<UserStats> {
        let conn = self.conn.lock().unwrap();
        let (total_count, total_follower, total_aweme) = conn.query_row(
            "SELECT COALESCE(COUNT(*),0), COALESCE(SUM(follower_count),0), COALESCE(SUM(aweme_count),0) FROM user_info",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        Ok(UserStats { total_count, total_follower, total_aweme })
    }

    // === 配置管理 ===

    pub fn get_config(&self) -> Result<AppConfig> {
        let conn = self.conn.lock().unwrap();
        let mut config = AppConfig::default();
        let mut stmt = conn.prepare("SELECT key, value FROM config")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (key, value) = row?;
            match key.as_str() {
                "cookie" => config.cookie = value,
                "download_path" => config.download_path = value,
                "naming" => config.naming = value,
                "encryption" => config.encryption = value,
                "proxy" => config.proxy = value,
                _ => {}
            }
        }
        Ok(config)
    }

    pub fn set_config(&self, updates: &HashMap<String, String>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        for (key, value) in updates {
            conn.execute(
                "INSERT OR REPLACE INTO config (key, value, updated_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![key, value, now],
            )?;
        }
        Ok(())
    }

    fn migrate(conn: &Connection) -> Result<()> {
        let version: i64 = conn
            .query_row(
                "SELECT COALESCE(CAST(value AS INTEGER), 0) FROM _metadata WHERE name = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if version < 1 {
            println!("[DB] 迁移 v1: 扩展 user_info 表");
            for sql in MIGRATE_V1_USER_INFO {
                let _ = conn.execute(sql, []); // 忽略 "duplicate column" 错误
            }
        }
        if version < 2 {
            println!("[DB] 迁移 v2: 扩展 video_info 表");
            for sql in MIGRATE_V2_VIDEO_INFO {
                let _ = conn.execute(sql, []);
            }
        }
        if version < 3 {
            println!("[DB] 迁移 v3: 添加索引");
            for sql in MIGRATE_V3_INDEXES {
                let _ = conn.execute(sql, []);
            }
        }
        if version < 4 {
            println!("[DB] 迁移 v4: live_records 添加 cover_url");
            for sql in MIGRATE_V4_LIVE_COVER {
                let _ = conn.execute(sql, []);
            }
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        conn.execute(
            "INSERT OR REPLACE INTO _metadata (name, value) VALUES ('schema_version', '4')",
            [],
        )?;
        conn.execute(
            "INSERT OR REPLACE INTO _metadata (name, value) VALUES ('schema_updated_at', ?1)",
            rusqlite::params![now.to_string()],
        )?;
        Ok(())
    }

    fn init_default_config(conn: &Connection) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        for (key, value) in DEFAULT_CONFIG {
            conn.execute(
                "INSERT OR IGNORE INTO config (key, value, updated_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![key, value, now],
            )?;
        }
        Ok(())
    }

    // === 写入方法 ===

    pub fn save_download(&self, record: &NewDownloadRecord) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        conn.execute(
            "INSERT INTO download_history \
             (aweme_id, download_type, title, author_nickname, author_sec_uid, \
              file_path, file_size, cover_url, status, error_msg, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                record.aweme_id,
                record.download_type,
                record.title,
                record.author_nickname,
                record.author_sec_uid,
                record.file_path,
                record.file_size,
                record.cover_url,
                record.status,
                record.error_msg,
                now,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn save_live_record(&self, record: &NewLiveRecord) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        conn.execute(
            "INSERT INTO live_records \
             (room_id, web_rid, title, nickname, sec_user_id, \
              file_path, file_size, duration_sec, status, started_at, ended_at, cover_url) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                record.room_id,
                record.web_rid,
                record.title,
                record.nickname,
                record.sec_user_id,
                record.file_path,
                record.file_size,
                record.duration_sec,
                record.status,
                record.started_at.unwrap_or(now),
                record.ended_at,
                record.cover_url,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn save_user(&self, user: &UserInfo) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        use ColKind::{Normal as N, Stat as S};
        let cols: &[(&str, ColKind)] = &[
            ("sec_user_id", N), ("nickname", N), ("uid", N),
            ("avatar_url", N), ("unique_id", N), ("signature", N),
            ("aweme_count", S), ("follower_count", S), ("following_count", S),
            ("total_favorited", S), ("ip_location", N),
            ("live_status", N), ("room_id", N), ("updated_at", N),
            ("city", N), ("country", N), ("favoriting_count", S), ("gender", N),
            ("is_ban", N), ("is_block", N), ("is_blocked", N), ("is_star", N),
            ("mix_count", S), ("mplatform_followers_count", S),
            ("nickname_raw", N), ("school_name", N), ("short_id", N),
            ("signature_raw", N), ("user_age", N), ("custom_verify", N),
        ];

        let sql = Self::build_upsert_sql("user_info", "sec_user_id", cols);
        conn.execute(
            &sql,
            rusqlite::params![
                user.sec_user_id, user.nickname, user.uid, user.avatar_url,
                user.unique_id, user.signature, user.aweme_count, user.follower_count,
                user.following_count, user.total_favorited, user.ip_location,
                user.live_status, user.room_id, now,
                user.city, user.country, user.favoriting_count, user.gender,
                user.is_ban, user.is_block, user.is_blocked, user.is_star,
                user.mix_count, user.mplatform_followers_count, user.nickname_raw,
                user.school_name, user.short_id, user.signature_raw, user.user_age,
                user.custom_verify,
            ],
        )?;
        Ok(())
    }

    pub fn save_video(&self, video: &VideoInfo) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        use ColKind::{Normal as N, Volatile as V, Stat as S};
        let cols: &[(&str, ColKind)] = &[
            ("aweme_id", N), ("desc", N), ("aweme_type", N),
            ("author_nickname", N), ("author_sec_uid", N), ("author_uid", N),
            ("create_time", N), ("duration", N),
            ("video_url", V), ("cover_url", V), ("music_title", N),
            ("digg_count", S), ("comment_count", S), ("share_count", S), ("collect_count", S),
            ("mix_id", N), ("mix_name", N), ("updated_at", N),
            ("author_nickname_raw", N), ("author_short_id", N), ("author_unique_id", N),
            ("desc_raw", N), ("is_ads", N), ("is_story", N), ("is_top", N), ("is_long_video", N),
            ("video_bit_rate", V), ("animated_cover", V), ("private_status", N), ("is_delete", N),
            ("music_author", N), ("music_author_raw", N), ("music_duration", N),
            ("music_id", N), ("music_mid", N), ("pgc_author", N), ("pgc_author_title", N),
            ("pgc_music_type", N), ("music_status", N),
            ("music_owner_handle", N), ("music_owner_id", N), ("music_owner_nickname", N),
            ("music_play_url", V), ("is_commerce_music", N),
            ("mix_desc", N), ("mix_create_time", N), ("mix_pic_type", N),
            ("mix_type", N), ("mix_share_url", N),
            ("can_comment", N), ("can_forward", N), ("can_share", N),
            ("download_setting", N), ("allow_douplus", N), ("allow_share", N),
            ("admire_count", S), ("hashtag_ids", N), ("hashtag_names", N),
            ("images", V), ("region", N), ("is_prohibited", N),
        ];

        let sql = Self::build_upsert_sql("video_info", "aweme_id", cols);
        conn.execute(
            &sql,
            rusqlite::params![
                video.aweme_id, video.desc, video.aweme_type,
                video.author_nickname, video.author_sec_uid, video.author_uid,
                video.create_time, video.duration, video.video_url, video.cover_url,
                video.music_title, video.digg_count, video.comment_count,
                video.share_count, video.collect_count, video.mix_id, video.mix_name,
                now,
                video.author_nickname_raw, video.author_short_id, video.author_unique_id,
                video.desc_raw, video.is_ads, video.is_story, video.is_top, video.is_long_video,
                video.video_bit_rate, video.animated_cover, video.private_status, video.is_delete,
                video.music_author, video.music_author_raw, video.music_duration,
                video.music_id, video.music_mid, video.pgc_author, video.pgc_author_title,
                video.pgc_music_type, video.music_status, video.music_owner_handle,
                video.music_owner_id, video.music_owner_nickname, video.music_play_url,
                video.is_commerce_music,
                video.mix_desc, video.mix_create_time, video.mix_pic_type,
                video.mix_type, video.mix_share_url,
                video.can_comment, video.can_forward, video.can_share,
                video.download_setting, video.allow_douplus, video.allow_share,
                video.admire_count, video.hashtag_ids, video.hashtag_names,
                video.images, video.region, video.is_prohibited,
            ],
        )?;
        Ok(())
    }

    pub fn is_video_downloaded(&self, aweme_id: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM download_history WHERE aweme_id = ?1 AND status = 'completed'",
            rusqlite::params![aweme_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    // === 音乐收藏 ===

    pub fn save_music_collection(&self, music: &NewMusicCollection) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        conn.execute(
            "INSERT OR REPLACE INTO music_collection \
             (music_id, mid, title, author, owner_nickname, duration, cover, play_url, status, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'collected', ?9)",
            rusqlite::params![
                music.music_id,
                music.mid,
                music.title,
                music.author,
                music.owner_nickname,
                music.duration,
                music.cover,
                music.play_url,
                now,
            ],
        )?;
        Ok(())
    }

    pub fn save_music_collection_batch(&self, musics: &[NewMusicCollection]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO music_collection \
             (music_id, mid, title, author, owner_nickname, duration, cover, play_url, status, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'collected', ?9)"
        )?;

        for music in musics {
            stmt.execute(rusqlite::params![
                music.music_id,
                music.mid,
                music.title,
                music.author,
                music.owner_nickname,
                music.duration,
                music.cover,
                music.play_url,
                now,
            ])?;
        }
        Ok(())
    }

    pub fn get_music_collection(
        &self,
        limit: i64,
        offset: i64,
        keyword: Option<String>,
        status: Option<String>,
    ) -> Result<Vec<MusicCollection>> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE);")?;

        let mut sql = String::from(
            "SELECT music_id, mid, title, author, owner_nickname, duration, cover, play_url, file_path, status, created_at \
             FROM music_collection"
        );
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut conditions = Vec::new();

        if let Some(ref s) = status {
            if !s.is_empty() {
                conditions.push("status = ?");
                params.push(Box::new(s.clone()));
            }
        }

        if let Some(ref kw) = keyword {
            if !kw.is_empty() {
                conditions.push("(title LIKE ? OR author LIKE ? OR owner_nickname LIKE ?)");
                let pattern = format!("%{}%", kw);
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern));
            }
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }
        sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");
        params.push(Box::new(limit));
        params.push(Box::new(offset));

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok(MusicCollection {
                music_id: row.get(0)?,
                mid: row.get(1)?,
                title: row.get(2)?,
                author: row.get(3)?,
                owner_nickname: row.get(4)?,
                duration: row.get(5)?,
                cover: row.get(6)?,
                play_url: row.get(7)?,
                file_path: row.get(8)?,
                status: row.get(9)?,
                created_at: row.get(10)?,
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    pub fn get_music_collection_count(&self, keyword: Option<String>, status: Option<String>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let mut sql = String::from("SELECT COUNT(*) FROM music_collection");
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut conditions = Vec::new();

        if let Some(ref s) = status {
            if !s.is_empty() {
                conditions.push("status = ?");
                params.push(Box::new(s.clone()));
            }
        }

        if let Some(ref kw) = keyword {
            if !kw.is_empty() {
                conditions.push("(title LIKE ? OR author LIKE ? OR owner_nickname LIKE ?)");
                let pattern = format!("%{}%", kw);
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern));
            }
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let count: i64 = conn.query_row(&sql, param_refs.as_slice(), |row| row.get(0))?;
        Ok(count)
    }

    pub fn update_music_file_path(&self, music_id: &str, file_path: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE music_collection SET file_path = ?1, status = 'downloaded' WHERE music_id = ?2",
            rusqlite::params![file_path, music_id],
        )?;
        Ok(())
    }
}
