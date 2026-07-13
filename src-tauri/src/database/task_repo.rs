use rusqlite::Result;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::database::models::*;
use crate::lock_conn;

impl super::connection::Database {
    pub fn create_task(&self, task: &NewDownloadTask) -> Result<()> {
        let conn = lock_conn!(self);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        conn.execute(
            "INSERT OR IGNORE INTO download_tasks \
             (id, mode, url, title, author_nickname, status, total, completed, skipped, failed, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, 'running', 0, 0, 0, 0, ?6, ?6)",
            rusqlite::params![task.id, task.mode, task.url, task.title, task.author_nickname, now],
        )?;
        Ok(())
    }

    pub fn update_task_status(&self, task_id: &str, status: &str, error_msg: Option<&str>) -> Result<()> {
        let conn = lock_conn!(self);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        conn.execute(
            "UPDATE download_tasks SET status = ?1, error_msg = ?2, updated_at = ?3 WHERE id = ?4",
            rusqlite::params![status, error_msg, now, task_id],
        )?;
        Ok(())
    }

    pub fn update_task_metadata(
        &self,
        task_id: &str,
        title: Option<&str>,
        author_nickname: Option<&str>,
    ) -> Result<()> {
        let conn = lock_conn!(self);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        conn.execute(
            "UPDATE download_tasks SET title = ?1, author_nickname = ?2, updated_at = ?3 WHERE id = ?4",
            rusqlite::params![title, author_nickname, now, task_id],
        )?;
        Ok(())
    }

    pub fn update_task_counts(&self, task_id: &str) -> Result<()> {
        let conn = lock_conn!(self);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        conn.execute(
            "UPDATE download_tasks SET \
             completed = (SELECT COUNT(*) FROM download_task_items WHERE task_id = ?1 AND status = 'completed'), \
             skipped = (SELECT COUNT(*) FROM download_task_items WHERE task_id = ?1 AND status = 'skipped'), \
             failed = (SELECT COUNT(*) FROM download_task_items WHERE task_id = ?1 AND status = 'failed'), \
             total = (SELECT COUNT(*) FROM download_task_items WHERE task_id = ?1), \
             updated_at = ?2 \
             WHERE id = ?1",
            rusqlite::params![task_id, now],
        )?;
        Ok(())
    }

    pub fn get_tasks(
        &self,
        limit: i64,
        offset: i64,
        status: Option<String>,
        mode: Option<String>,
    ) -> Result<Vec<DownloadTask>> {
        let conn = lock_conn!(self);
        let mut sql = String::from(
            "SELECT id, mode, url, title, author_nickname, status, total, completed, skipped, failed, error_msg, created_at, updated_at \
             FROM download_tasks"
        );
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut conditions = Vec::new();

        if let Some(ref s) = status {
            if !s.is_empty() {
                conditions.push("status = ?");
                params.push(Box::new(s.clone()));
            }
        }
        if let Some(ref m) = mode {
            if !m.is_empty() {
                conditions.push("mode = ?");
                params.push(Box::new(m.clone()));
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
            Ok(DownloadTask {
                id: row.get("id")?,
                mode: row.get("mode")?,
                url: row.get("url")?,
                title: row.get("title")?,
                author_nickname: row.get("author_nickname")?,
                status: row.get("status")?,
                total: row.get("total")?,
                completed: row.get("completed")?,
                skipped: row.get("skipped")?,
                failed: row.get("failed")?,
                error_msg: row.get("error_msg")?,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
            })
        })?;

        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(row?);
        }
        Ok(tasks)
    }

    pub fn get_task_by_id(&self, task_id: &str) -> Result<Option<DownloadTask>> {
        let conn = lock_conn!(self);
        let mut stmt = conn.prepare(
            "SELECT id, mode, url, title, author_nickname, status, total, completed, skipped, failed, error_msg, created_at, updated_at \
             FROM download_tasks WHERE id = ?1"
        )?;
        let mut rows = stmt.query_map(rusqlite::params![task_id], |row| {
            Ok(DownloadTask {
                id: row.get("id")?,
                mode: row.get("mode")?,
                url: row.get("url")?,
                title: row.get("title")?,
                author_nickname: row.get("author_nickname")?,
                status: row.get("status")?,
                total: row.get("total")?,
                completed: row.get("completed")?,
                skipped: row.get("skipped")?,
                failed: row.get("failed")?,
                error_msg: row.get("error_msg")?,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn delete_task(&self, task_id: &str) -> Result<()> {
        self.with_transaction(|tx| {
            tx.execute("DELETE FROM download_task_items WHERE task_id = ?1", rusqlite::params![task_id])?;
            tx.execute("DELETE FROM download_tasks WHERE id = ?1", rusqlite::params![task_id])?;
            Ok(())
        })
    }

    // === 下载任务子项 (download_task_items) ===

    pub fn create_task_item(&self, item: &NewTaskItem) -> Result<()> {
        let conn = lock_conn!(self);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        conn.execute(
            "INSERT OR IGNORE INTO download_task_items \
             (task_id, aweme_id, title, author_nickname, author_sec_uid, cover_url, status, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending', ?7)",
            rusqlite::params![
                item.task_id, item.aweme_id, item.title, item.author_nickname, item.author_sec_uid, item.cover_url, now,
            ],
        )?;
        Ok(())
    }

    pub fn update_task_item_status(
        &self,
        task_id: &str,
        aweme_id: &str,
        status: &str,
        file_path: Option<&str>,
        file_size: i64,
        error_msg: Option<&str>,
    ) -> Result<()> {
        let conn = lock_conn!(self);
        conn.execute(
            "UPDATE download_task_items SET status = ?1, file_path = COALESCE(?2, file_path), \
             file_size = CASE WHEN ?3 > 0 THEN ?3 ELSE file_size END, \
             error_msg = ?4 \
             WHERE task_id = ?5 AND aweme_id = ?6",
            rusqlite::params![status, file_path, file_size, error_msg, task_id, aweme_id],
        )?;
        Ok(())
    }

    /// 原子操作：更新任务子项状态 + 重新计算任务计数（单次事务）
    pub fn update_task_item_and_counts(
        &self,
        task_id: &str,
        aweme_id: &str,
        status: &str,
        file_path: Option<&str>,
        file_size: i64,
        error_msg: Option<&str>,
    ) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        self.with_transaction(|tx| {
            tx.execute(
                "UPDATE download_task_items SET status = ?1, file_path = COALESCE(?2, file_path), \
                 file_size = CASE WHEN ?3 > 0 THEN ?3 ELSE file_size END, \
                 error_msg = ?4 \
                 WHERE task_id = ?5 AND aweme_id = ?6",
                rusqlite::params![status, file_path, file_size, error_msg, task_id, aweme_id],
            )?;
            tx.execute(
                "UPDATE download_tasks SET \
                 completed = (SELECT COUNT(*) FROM download_task_items WHERE task_id = ?1 AND status = 'completed'), \
                 skipped = (SELECT COUNT(*) FROM download_task_items WHERE task_id = ?1 AND status = 'skipped'), \
                 failed = (SELECT COUNT(*) FROM download_task_items WHERE task_id = ?1 AND status = 'failed'), \
                 total = (SELECT COUNT(*) FROM download_task_items WHERE task_id = ?1), \
                 updated_at = ?2 \
                 WHERE id = ?1",
                rusqlite::params![task_id, now],
            )?;
            Ok(())
        })
    }

    /// 原子提交一个媒体项的最终结果、元数据和任务计数。
    ///
    /// 整个任务的 completed/error 终态由应用层在本事务成功后单独提交，
    /// 防止结果事务回滚时仍向前端宣告任务完成。
    pub fn commit_media_item_result(&self, result: &MediaItemResult<'_>) -> Result<()> {
        let aweme_id = result.item.aweme_id.as_deref().ok_or_else(|| {
            rusqlite::Error::InvalidParameterName("media item result requires aweme_id".to_string())
        })?;
        if result
            .video_info
            .is_some_and(|video| video.aweme_id != aweme_id)
        {
            return Err(rusqlite::Error::InvalidParameterName(
                "media item video_info.aweme_id must match item aweme_id".to_string(),
            ));
        }

        let (status, file_path, file_size, error_msg) = match result.outcome {
            MediaItemOutcome::Completed {
                file_path,
                file_size,
            } => ("completed", Some(file_path), file_size, None),
            MediaItemOutcome::Skipped {
                file_path,
                file_size,
            } => ("skipped", Some(file_path), file_size, None),
            MediaItemOutcome::Failed { error_msg } => ("failed", None, 0, Some(error_msg)),
        };
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.with_transaction(|tx| {
            tx.execute(
                "INSERT INTO download_task_items \
                 (task_id, aweme_id, title, author_nickname, author_sec_uid, cover_url, \
                  file_path, file_size, status, error_msg, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11) \
                 ON CONFLICT(task_id, aweme_id) DO UPDATE SET \
                  title = excluded.title, author_nickname = excluded.author_nickname, \
                  author_sec_uid = excluded.author_sec_uid, cover_url = excluded.cover_url, \
                  file_path = excluded.file_path, file_size = excluded.file_size, \
                  status = excluded.status, error_msg = excluded.error_msg",
                rusqlite::params![
                    result.item.task_id,
                    aweme_id,
                    result.item.title,
                    result.item.author_nickname,
                    result.item.author_sec_uid,
                    result.item.cover_url,
                    file_path,
                    file_size,
                    status,
                    error_msg,
                    now,
                ],
            )?;

            if let Some(video) = result.video_info {
                Self::save_video_inner(tx, video)?;
            }

            if let Some(user) = result.user_info {
                if !user.sec_user_id.trim().is_empty() {
                    Self::save_user_inner(tx, user)?;
                }
            }

            let updated_tasks = tx.execute(
                "UPDATE download_tasks SET \
                 completed = (SELECT COUNT(*) FROM download_task_items WHERE task_id = ?1 AND status = 'completed'), \
                 skipped = (SELECT COUNT(*) FROM download_task_items WHERE task_id = ?1 AND status = 'skipped'), \
                 failed = (SELECT COUNT(*) FROM download_task_items WHERE task_id = ?1 AND status = 'failed'), \
                 total = (SELECT COUNT(*) FROM download_task_items WHERE task_id = ?1), \
                 updated_at = ?2 \
                 WHERE id = ?1",
                rusqlite::params![result.item.task_id, now],
            )?;
            if updated_tasks != 1 {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            }

            Ok(())
        })
    }

    pub fn get_task_items(&self, task_id: &str, status: Option<String>) -> Result<Vec<TaskItem>> {
        let conn = lock_conn!(self);
        let mut sql = String::from(
            "SELECT id, task_id, aweme_id, title, author_nickname, author_sec_uid, cover_url, file_path, file_size, status, error_msg, created_at \
             FROM download_task_items WHERE task_id = ?1"
        );
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        params.push(Box::new(task_id.to_string()));

        if let Some(ref s) = status {
            if !s.is_empty() {
                sql.push_str(" AND status = ?");
                params.push(Box::new(s.clone()));
            }
        }
        sql.push_str(" ORDER BY id ASC");

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok(TaskItem {
                id: row.get("id")?,
                task_id: row.get("task_id")?,
                aweme_id: row.get("aweme_id")?,
                title: row.get("title")?,
                author_nickname: row.get("author_nickname")?,
                author_sec_uid: row.get("author_sec_uid")?,
                cover_url: row.get("cover_url")?,
                file_path: row.get("file_path")?,
                file_size: row.get("file_size")?,
                status: row.get("status")?,
                error_msg: row.get("error_msg")?,
                created_at: row.get("created_at")?,
            })
        })?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    pub fn get_task_item_counts(&self, task_id: &str) -> Result<TaskItemCounts> {
        let conn = lock_conn!(self);
        let counts = conn.query_row(
            "SELECT \
             COUNT(*), \
             COALESCE(SUM(CASE WHEN status = 'completed' THEN 1 ELSE 0 END), 0), \
             COALESCE(SUM(CASE WHEN status = 'skipped' THEN 1 ELSE 0 END), 0), \
             COALESCE(SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END), 0), \
             COALESCE(SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END), 0) \
             FROM download_task_items WHERE task_id = ?1",
            rusqlite::params![task_id],
            |row| Ok(TaskItemCounts {
                total: row.get(0)?,
                completed: row.get(1)?,
                skipped: row.get(2)?,
                failed: row.get(3)?,
                pending: row.get(4)?,
            }),
        )?;
        Ok(counts)
    }

    /// 根据 aweme_id 查询下载文件路径
    pub fn get_task_item_file_path(&self, aweme_id: &str) -> Result<Option<String>> {
        let conn = lock_conn!(self);
        let mut stmt = conn.prepare(
            "SELECT file_path FROM download_task_items WHERE aweme_id = ?1 AND file_path IS NOT NULL AND file_path != '' LIMIT 1"
        )?;
        let mut rows = stmt.query_map(rusqlite::params![aweme_id], |row| row.get::<_, String>(0))?;
        match rows.next() {
            Some(row) => row.map(Some),
            None => Ok(None),
        }
    }

    pub fn get_task_detail(&self, task_id: &str) -> Result<Option<DownloadTaskDetail>> {
        let task = self.get_task_by_id(task_id)?;
        match task {
            Some(t) => {
                let items = self.get_task_items(task_id, None)?;
                Ok(Some(DownloadTaskDetail { task: t, items }))
            }
            None => Ok(None),
        }
    }
}
