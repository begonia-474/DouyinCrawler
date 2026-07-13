//! 直播记录仓库
//!
//! 从 db.rs 提取直播记录相关的 CRUD 方法。

use rusqlite::Result;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::database::models::*;
use crate::lock_conn;

impl super::connection::Database {
    // === 直播记录查询 ===

    pub fn get_live_records(&self, limit: i64, offset: i64) -> Result<Vec<LiveRecord>> {
        let conn = lock_conn!(self);
        let mut stmt = conn.prepare(
            "SELECT id, task_id, room_id, web_rid, title, nickname, sec_user_id, \
             file_path, file_size, duration_sec, status, error_msg, started_at, ended_at, cover_url, updated_at \
             FROM live_records ORDER BY started_at DESC LIMIT ? OFFSET ?",
        )?;

        let rows = stmt.query_map(rusqlite::params![limit, offset], |row| {
            Ok(LiveRecord {
                id: row.get("id")?,
                task_id: row.get("task_id")?,
                room_id: row.get("room_id")?,
                web_rid: row.get("web_rid")?,
                title: row.get("title")?,
                nickname: row.get("nickname")?,
                sec_user_id: row.get("sec_user_id")?,
                file_path: row.get("file_path")?,
                file_size: row.get("file_size")?,
                duration_sec: row.get("duration_sec")?,
                status: row.get("status")?,
                error_msg: row.get("error_msg")?,
                started_at: row.get("started_at")?,
                ended_at: row.get("ended_at")?,
                cover_url: row.get("cover_url")?,
                updated_at: row.get("updated_at")?,
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
        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM live_records", [], |row| row.get(0))?;
        Ok(count)
    }

    // === 写入方法 ===

    pub fn save_live_record(&self, record: &NewLiveRecord) -> Result<i64> {
        if record.task_id.is_some() {
            return Err(rusqlite::Error::InvalidParameterName(
                "task_id is reserved for create_recording_live_record".to_string(),
            ));
        }
        let conn = lock_conn!(self);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        conn.execute(
            "INSERT OR IGNORE INTO live_records \
             (task_id, room_id, web_rid, title, nickname, sec_user_id, \
              file_path, file_size, duration_sec, status, error_msg, started_at, ended_at, cover_url, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            rusqlite::params![
                record.task_id,
                record.room_id,
                record.web_rid,
                record.title,
                record.nickname,
                record.sec_user_id,
                record.file_path,
                record.file_size,
                record.duration_sec,
                record.status,
                record.error_msg,
                record.started_at.unwrap_or(now),
                record.ended_at,
                record.cover_url,
                record.updated_at.unwrap_or(now),
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// 在 recorder 启动前，把 task 和唯一 live row 一次性推进到 recording。
    pub fn create_recording_live_record(&self, record: &RecordingLiveRecord) -> Result<i64> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        self.with_transaction(|tx| {
            let current_status: String = tx.query_row(
                "SELECT status FROM download_tasks WHERE id = ?1 AND mode = 'live'",
                rusqlite::params![record.task_id],
                |row| row.get(0),
            )?;
            if !matches!(current_status.as_str(), "starting" | "running" | "stopping") {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            }
            let active_status = if current_status == "stopping" {
                "stopping"
            } else {
                "recording"
            };
            let updated = tx.execute(
                "UPDATE download_tasks
                 SET title = ?1, author_nickname = ?2, status = ?3, error_msg = NULL, updated_at = ?4
                 WHERE id = ?5 AND mode = 'live' AND status = ?6",
                rusqlite::params![
                    record.title,
                    record.nickname,
                    active_status,
                    now,
                    record.task_id,
                    current_status,
                ],
            )?;
            if updated != 1 {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            }

            tx.execute(
                "INSERT INTO live_records
                 (task_id, room_id, web_rid, title, nickname, sec_user_id, cover_url,
                  file_path, file_size, duration_sec, status, error_msg, started_at, ended_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, 0, ?9, NULL, ?10, NULL, ?11)",
                rusqlite::params![
                    record.task_id,
                    record.room_id,
                    record.web_rid,
                    record.title,
                    record.nickname,
                    record.sec_user_id,
                    record.cover_url,
                    record.file_path,
                    active_status,
                    record.started_at,
                    now,
                ],
            )?;
            Ok(tx.last_insert_rowid())
        })
    }

    /// 只持久化非终态 stop 请求；最终状态仍由 recorder/orchestrator 提交。
    pub fn request_live_stop(&self, task_id: &str) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        self.with_transaction(|tx| {
            let updated = tx.execute(
                "UPDATE download_tasks SET status = 'stopping', updated_at = ?1
                 WHERE id = ?2 AND mode = 'live'
                   AND status IN ('starting', 'running', 'recording')",
                rusqlite::params![now, task_id],
            )?;
            if updated != 1 {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            }
            tx.execute(
                "UPDATE live_records SET status = 'stopping', updated_at = ?1
                 WHERE task_id = ?2 AND status = 'recording'",
                rusqlite::params![now, task_id],
            )?;
            Ok(())
        })
    }

    /// 同一事务提交 task 和它唯一关联的 live row 终态。
    pub fn commit_live_terminal(&self, outcome: &LiveTerminalCommit) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let status = outcome.status.as_str();
        self.with_transaction(|tx| {
            let task_updated = tx.execute(
                "UPDATE download_tasks SET status = ?1, error_msg = ?2, updated_at = ?3
                 WHERE id = ?4 AND mode = 'live'
                   AND status IN ('recording', 'stopping')",
                rusqlite::params![status, outcome.error_msg, now, outcome.task_id],
            )?;
            if task_updated != 1 {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            }

            let live_updated = tx.execute(
                "UPDATE live_records
                 SET status = ?1, error_msg = ?2, file_path = ?3, file_size = ?4,
                     duration_sec = ?5, started_at = ?6, ended_at = ?7, updated_at = ?8
                 WHERE task_id = ?9 AND status IN ('recording', 'stopping')",
                rusqlite::params![
                    status,
                    outcome.error_msg,
                    outcome.file_path,
                    outcome.file_size,
                    outcome.duration_sec,
                    outcome.started_at,
                    outcome.ended_at,
                    now,
                    outcome.task_id,
                ],
            )?;
            if live_updated != 1 {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            }
            Ok(())
        })
    }

    #[allow(dead_code)] // public lifecycle/recovery tests query through this repository seam
    pub fn get_live_record_by_task_id(&self, task_id: &str) -> Result<Option<LiveRecord>> {
        let conn = lock_conn!(self);
        let mut stmt = conn.prepare(
            "SELECT id, task_id, room_id, web_rid, title, nickname, sec_user_id,
                    file_path, file_size, duration_sec, status, error_msg,
                    started_at, ended_at, cover_url, updated_at
             FROM live_records WHERE task_id = ?1",
        )?;
        let mut rows = stmt.query_map(rusqlite::params![task_id], |row| {
            Ok(LiveRecord {
                id: row.get("id")?,
                task_id: row.get("task_id")?,
                room_id: row.get("room_id")?,
                web_rid: row.get("web_rid")?,
                title: row.get("title")?,
                nickname: row.get("nickname")?,
                sec_user_id: row.get("sec_user_id")?,
                file_path: row.get("file_path")?,
                file_size: row.get("file_size")?,
                duration_sec: row.get("duration_sec")?,
                status: row.get("status")?,
                error_msg: row.get("error_msg")?,
                started_at: row.get("started_at")?,
                ended_at: row.get("ended_at")?,
                cover_url: row.get("cover_url")?,
                updated_at: row.get("updated_at")?,
            })
        })?;
        match rows.next() {
            Some(row) => row.map(Some),
            None => Ok(None),
        }
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
        conn.execute(
            "DELETE FROM live_records WHERE id = ?1",
            rusqlite::params![id],
        )?;
        Ok(())
    }

    /// 查询指定用户的所有直播记录文件路径
    pub fn get_user_live_file_paths(&self, sec_user_id: &str) -> Result<Vec<String>> {
        let conn = lock_conn!(self);
        let mut stmt = conn.prepare(
            "SELECT file_path FROM live_records WHERE sec_user_id = ?1 AND file_path IS NOT NULL AND file_path != ''"
        )?;
        let rows = stmt.query_map(rusqlite::params![sec_user_id], |row| {
            row.get::<_, String>(0)
        })?;
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
             WHERE i.author_sec_uid = ?1 AND i.file_path IS NOT NULL AND i.file_path != ''",
        )?;
        let rows = stmt.query_map(rusqlite::params![sec_user_id], |row| {
            row.get::<_, String>(0)
        })?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::connection::Database;
    use crate::database::models::{LiveTerminalStatus, NewDownloadTask};

    fn test_database() -> (Database, std::path::PathBuf) {
        let path =
            std::env::temp_dir().join(format!("douyin-live-repo-{}.sqlite", uuid::Uuid::new_v4()));
        (Database::open(&path).unwrap(), path)
    }

    fn create_starting_live_task(db: &Database, task_id: &str) {
        db.create_task(&NewDownloadTask {
            id: task_id.to_string(),
            mode: "live".to_string(),
            url: "https://live.douyin.com/1".to_string(),
            title: None,
            author_nickname: None,
        })
        .unwrap();
        db.update_task_status(task_id, "starting", None).unwrap();
    }

    fn recording(task_id: &str) -> RecordingLiveRecord {
        RecordingLiveRecord {
            task_id: task_id.to_string(),
            room_id: "room-1".to_string(),
            web_rid: "web-1".to_string(),
            title: "live title".to_string(),
            nickname: "anchor".to_string(),
            sec_user_id: "sec-1".to_string(),
            cover_url: "https://example.com/cover.jpg".to_string(),
            file_path: "/tmp/record_live.flv".to_string(),
            started_at: 100,
        }
    }

    #[test]
    fn create_recording_live_record_links_one_row_and_task_transactionally() {
        let (db, path) = test_database();
        create_starting_live_task(&db, "live-1");

        let record_id = db
            .create_recording_live_record(&recording("live-1"))
            .unwrap();

        let tasks = db.get_tasks(10, 0, None, Some("live".to_string())).unwrap();
        assert_eq!(tasks[0].status, "recording");
        assert_eq!(tasks[0].title.as_deref(), Some("live title"));
        let live = db.get_live_record_by_task_id("live-1").unwrap().unwrap();
        assert_eq!(live.id, record_id);
        assert_eq!(live.task_id.as_deref(), Some("live-1"));
        assert_eq!(live.status, "recording");
        assert_eq!(live.file_path.as_deref(), Some("/tmp/record_live.flv"));

        drop(db);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn terminal_commit_updates_same_task_and_live_row() {
        let (db, path) = test_database();
        create_starting_live_task(&db, "live-2");
        let record_id = db
            .create_recording_live_record(&recording("live-2"))
            .unwrap();

        db.commit_live_terminal(&LiveTerminalCommit {
            task_id: "live-2".to_string(),
            status: LiveTerminalStatus::Completed,
            file_path: "/tmp/record_live.flv".to_string(),
            file_size: 6,
            duration_sec: 8,
            started_at: 100,
            ended_at: 108,
            error_msg: None,
        })
        .unwrap();

        let task = db
            .get_tasks(10, 0, None, Some("live".to_string()))
            .unwrap()
            .pop()
            .unwrap();
        let live = db.get_live_record_by_task_id("live-2").unwrap().unwrap();
        assert_eq!(task.status, "completed");
        assert_eq!(live.id, record_id);
        assert_eq!(live.status, "completed");
        assert_eq!(live.file_size, 6);
        assert_eq!(live.duration_sec, 8);

        drop(db);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn error_terminal_commit_updates_both_rows_with_context() {
        let (db, path) = test_database();
        create_starting_live_task(&db, "live-error");
        db.create_recording_live_record(&recording("live-error"))
            .unwrap();

        db.commit_live_terminal(&LiveTerminalCommit {
            task_id: "live-error".to_string(),
            status: LiveTerminalStatus::Error,
            file_path: "/tmp/record_live.flv".to_string(),
            file_size: 3,
            duration_sec: 1,
            started_at: 100,
            ended_at: 101,
            error_msg: Some("segment retry exhausted".to_string()),
        })
        .unwrap();

        let task = db.get_task_by_id("live-error").unwrap().unwrap();
        let live = db
            .get_live_record_by_task_id("live-error")
            .unwrap()
            .unwrap();
        assert_eq!(task.status, "error");
        assert_eq!(live.status, "error");
        assert_eq!(live.file_size, 3);
        assert!(live.error_msg.unwrap().contains("segment retry"));

        drop(db);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn create_recording_insert_failure_rolls_back_task_update() {
        let (db, path) = test_database();
        create_starting_live_task(&db, "live-rollback-create");
        {
            let conn = lock_conn!(db);
            conn.execute(
                "INSERT INTO live_records (task_id, status, updated_at) VALUES (?1, 'recording', 1)",
                ["live-rollback-create"],
            )
            .unwrap();
        }

        assert!(db
            .create_recording_live_record(&recording("live-rollback-create"))
            .is_err());
        let task = db.get_task_by_id("live-rollback-create").unwrap().unwrap();
        assert_eq!(task.status, "starting");
        assert!(task.title.is_none());

        drop(db);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn terminal_live_update_failure_rolls_back_task_terminal_state() {
        let (db, path) = test_database();
        create_starting_live_task(&db, "live-rollback-terminal");
        db.create_recording_live_record(&recording("live-rollback-terminal"))
            .unwrap();
        db.with_transaction(|tx| {
            tx.execute_batch(
                "CREATE TRIGGER fail_live_terminal BEFORE UPDATE OF status ON live_records
                 WHEN NEW.status = 'completed'
                 BEGIN SELECT RAISE(ABORT, 'terminal rollback'); END;",
            )?;
            Ok(())
        })
        .unwrap();

        assert!(db
            .commit_live_terminal(&LiveTerminalCommit {
                task_id: "live-rollback-terminal".to_string(),
                status: LiveTerminalStatus::Completed,
                file_path: "/tmp/record_live.flv".to_string(),
                file_size: 6,
                duration_sec: 2,
                started_at: 100,
                ended_at: 102,
                error_msg: None,
            })
            .is_err());
        let task = db
            .get_task_by_id("live-rollback-terminal")
            .unwrap()
            .unwrap();
        let live = db
            .get_live_record_by_task_id("live-rollback-terminal")
            .unwrap()
            .unwrap();
        assert_eq!(task.status, "recording");
        assert_eq!(live.status, "recording");

        drop(db);
        let _ = std::fs::remove_file(path);
    }
}
