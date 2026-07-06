//! 直播记录仓库
//!
//! 从 db.rs 提取直播记录相关的 CRUD 方法。

use rusqlite::Result;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::database::models::*;
use crate::lock_conn;

impl super::connection::Database {
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

    /// 查询指定用户的所有直播记录文件路径
    pub fn get_user_live_file_paths(&self, sec_user_id: &str) -> Result<Vec<String>> {
        let conn = lock_conn!(self);
        let mut stmt = conn.prepare(
            "SELECT file_path FROM live_records WHERE sec_user_id = ?1 AND file_path IS NOT NULL AND file_path != ''"
        )?;
        let rows = stmt.query_map(rusqlite::params![sec_user_id], |row| row.get::<_, String>(0))?;
        let mut paths = Vec::new();
        for row in rows {
            paths.push(row?);
        }
        Ok(paths)
    }

    /// 查询指定用户的所有下载任务文件路径
    pub fn get_user_download_file_paths(&self, sec_user_id: &str) -> Result<Vec<String>> {
        let conn = lock_conn!(self);
        let mut stmt = conn.prepare(
            "SELECT i.file_path FROM download_task_items i \
             WHERE i.author_sec_uid = ?1 AND i.file_path IS NOT NULL AND i.file_path != ''"
        )?;
        let rows = stmt.query_map(rusqlite::params![sec_user_id], |row| row.get::<_, String>(0))?;
        let mut paths = Vec::new();
        for row in rows {
            paths.push(row?);
        }
        Ok(paths)
    }

    /// 批量删除直播记录（事务保证原子性）
    pub fn delete_live_records_batch(&self, ids: &[i64]) -> Result<()> {
        self.with_transaction(|tx| {
            let mut stmt = tx.prepare("DELETE FROM live_records WHERE id = ?1")?;
            for id in ids {
                stmt.execute(rusqlite::params![id])?;
            }
            Ok(())
        })
    }

    /// 批量查询直播记录的文件路径
    pub fn get_live_record_file_paths_batch(&self, ids: &[i64]) -> Result<Vec<String>> {
        let conn = lock_conn!(self);
        let mut stmt = conn.prepare(
            "SELECT file_path FROM live_records WHERE id = ?1 AND file_path IS NOT NULL AND file_path != ''"
        )?;
        let mut paths = Vec::new();
        for id in ids {
            let mut rows = stmt.query_map(rusqlite::params![id], |row| row.get::<_, String>(0))?;
            if let Some(row) = rows.next() {
                paths.push(row?);
            }
        }
        Ok(paths)
    }
}
