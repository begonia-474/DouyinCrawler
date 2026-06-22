use rusqlite::{Connection, Result};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

const CREATE_TABLES_SQL: &str = "
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
    CREATE INDEX IF NOT EXISTS idx_download_created ON download_history(created_at);
    CREATE INDEX IF NOT EXISTS idx_download_type ON download_history(download_type);
    CREATE INDEX IF NOT EXISTS idx_download_status ON download_history(status);
    CREATE INDEX IF NOT EXISTS idx_download_author ON download_history(author_sec_uid);
    CREATE INDEX IF NOT EXISTS idx_live_started ON live_records(started_at);
";

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
             file_path, file_size, duration_sec, status, started_at, ended_at \
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
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }
}
