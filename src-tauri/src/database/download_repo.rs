//! 下载历史与直播记录仓库
//!
//! 从 db.rs 提取下载和直播记录相关的 CRUD 方法。

use rusqlite::{Connection, Result};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::database::models::*;
use crate::lock_conn;
use log::debug;

impl super::connection::Database {
    // === 下载历史查询 ===

    pub fn get_downloads(
        &self,
        limit: i64,
        offset: i64,
        status: Option<String>,
        download_type: Option<String>,
    ) -> Result<Vec<DownloadRecord>> {
        let conn = lock_conn!(self);

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

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok(DownloadRecord {
                id: row.get("id")?,
                aweme_id: row.get("aweme_id")?,
                download_type: row.get("download_type")?,
                title: row.get("title")?,
                author_nickname: row.get("author_nickname")?,
                author_sec_uid: row.get("author_sec_uid")?,
                file_path: row.get("file_path")?,
                file_size: row.get("file_size")?,
                cover_url: row.get("cover_url")?,
                status: row.get("status")?,
                error_msg: row.get("error_msg")?,
                created_at: row.get("created_at")?,
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        debug!("[DB] get_downloads 返回 {} 条记录", records.len());
        Ok(records)
    }

    pub fn get_download_stats(&self) -> Result<DownloadStats> {
        let conn = lock_conn!(self);

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

    // === 直播记录查询 ===

    pub fn get_live_records(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<LiveRecord>> {
        let conn = lock_conn!(self);
        let mut stmt = conn.prepare(
            "SELECT id, room_id, web_rid, title, nickname, sec_user_id, \
             file_path, file_size, duration_sec, status, started_at, ended_at, cover_url \
             FROM live_records ORDER BY started_at DESC LIMIT ? OFFSET ?",
        )?;

        let rows = stmt.query_map(rusqlite::params![limit, offset], |row| {
            Ok(LiveRecord {
                id: row.get("id")?,
                room_id: row.get("room_id")?,
                web_rid: row.get("web_rid")?,
                title: row.get("title")?,
                nickname: row.get("nickname")?,
                sec_user_id: row.get("sec_user_id")?,
                file_path: row.get("file_path")?,
                file_size: row.get("file_size")?,
                duration_sec: row.get("duration_sec")?,
                status: row.get("status")?,
                started_at: row.get("started_at")?,
                ended_at: row.get("ended_at")?,
                cover_url: row.get("cover_url")?,
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    pub fn get_live_records_count(&self) -> Result<i64> {
        let conn = lock_conn!(self);
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM live_records", [], |row| row.get(0))?;
        Ok(count)
    }

    // === 写入方法 ===

    pub(crate) fn save_download_inner(conn: &Connection, record: &NewDownloadRecord) -> Result<i64> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        conn.execute(
            "INSERT OR IGNORE INTO download_history \
             (aweme_id, download_type, title, author_nickname, author_sec_uid, \
              file_path, file_size, cover_url, status, error_msg, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                record.aweme_id, record.download_type, record.title,
                record.author_nickname, record.author_sec_uid, record.file_path,
                record.file_size, record.cover_url, record.status, record.error_msg, now,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn save_download(&self, record: &NewDownloadRecord) -> Result<i64> {
        let conn = lock_conn!(self);
        debug!("[DB] save_download: aweme_id={:?}, file_path={:?}", record.aweme_id, record.file_path);
        let id = Self::save_download_inner(&conn, record)?;
        debug!("[DB] save_download 成功, id={}", id);
        Ok(id)
    }

    pub fn get_download_file_path(&self, id: i64) -> Result<Option<String>> {
        let conn = lock_conn!(self);
        let mut stmt = conn.prepare("SELECT file_path FROM download_history WHERE id = ?1")?;
        let mut rows = stmt.query_map(rusqlite::params![id], |row| row.get(0))?;
        match rows.next() {
            Some(row) => row,
            None => Ok(None),
        }
    }

    pub fn delete_download(&self, id: i64) -> Result<()> {
        let conn = lock_conn!(self);
        conn.execute("DELETE FROM download_history WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }

    pub fn save_live_record(&self, record: &NewLiveRecord) -> Result<i64> {
        let conn = lock_conn!(self);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        conn.execute(
            "INSERT OR IGNORE INTO live_records \
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

    pub fn get_live_record_file_path(&self, id: i64) -> Result<Option<String>> {
        let conn = lock_conn!(self);
        let mut stmt = conn.prepare("SELECT file_path FROM live_records WHERE id = ?1")?;
        let mut rows = stmt.query_map(rusqlite::params![id], |row| row.get(0))?;
        match rows.next() {
            Some(row) => row,
            None => Ok(None),
        }
    }

    pub fn delete_live_record(&self, id: i64) -> Result<()> {
        let conn = lock_conn!(self);
        conn.execute("DELETE FROM live_records WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }
}
