use rusqlite::Result;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::database::models::*;
use crate::lock_conn;

// ============================================================
// TaskRepository trait — support unit testing with mock impls
// ============================================================

/// 任务仓库抽象
///
/// 定义 TaskApplicationService 对 task 实体的核心 CRUD 操作。
/// 实现此 trait 的类型可用于替代 Database 进行单元测试（纯 Rust，无 PyO3 依赖）。
///
/// # Example
///
/// ```ignore
/// struct MockTaskRepo { ... }
/// impl TaskRepository for MockTaskRepo { ... }
/// let service = TaskApplicationService::new(&mock_repo, adapter);
/// ```
#[allow(dead_code)] // TODO P2-01: may become used after download entry unification
pub trait TaskRepository {
    fn create_task(&self, task: &NewDownloadTask) -> Result<()>;
    fn update_task_status(&self, task_id: &str, status: &str, error_msg: Option<&str>) -> Result<()>;
    fn update_task_counts(&self, task_id: &str) -> Result<()>;
    fn create_task_item(&self, item: &NewTaskItem) -> Result<()>;
    fn update_task_item_status(
        &self,
        task_id: &str,
        aweme_id: &str,
        status: &str,
        file_path: Option<&str>,
        file_size: i64,
        error_msg: Option<&str>,
    ) -> Result<()>;
    fn complete_single_download(
        &self,
        task_id: &str,
        item: &NewTaskItem,
        file_path: Option<&str>,
        file_size: i64,
        video_info: Option<&VideoInfo>,
        user_info: Option<&UserInfo>,
    ) -> Result<()>;
}

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

    /// 原子操作：完成单视频下载的全部持久化（单次事务）
    ///
    /// F2.1: 替代 service.rs 中多个独立 DB 写入，保证原子性。
    /// 事务覆盖：
    /// 1. 创建任务子项 (download_task_items INSERT)
    /// 2. 更新任务子项状态为 completed
    /// 3. 保存视频信息 (video_info)
    /// 4. 保存用户信息 (user_info) — 仅当用户不存在时
    /// 5. 更新任务计数 (download_tasks)
    /// 6. 更新任务状态为 completed (download_tasks)
    ///
    /// 任何一步失败，整个事务回滚。
    pub fn complete_single_download(
        &self,
        task_id: &str,
        item: &NewTaskItem,
        file_path: Option<&str>,
        file_size: i64,
        video_info: Option<&VideoInfo>,
        user_info: Option<&UserInfo>,
    ) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.with_transaction(|tx| {
            // 1. 创建任务子项
            tx.execute(
                "INSERT INTO download_task_items (task_id, aweme_id, title, author_nickname, author_sec_uid, cover_url, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![item.task_id, item.aweme_id, item.title, item.author_nickname, item.author_sec_uid, item.cover_url, now],
            )?;

            // 2. 更新任务子项状态为 completed
            let aweme_id = item.aweme_id.as_deref().unwrap_or("unknown");
            tx.execute(
                "UPDATE download_task_items SET status = 'completed', file_path = ?1, file_size = ?2 \
                 WHERE task_id = ?3 AND aweme_id = ?4",
                rusqlite::params![file_path, file_size, task_id, aweme_id],
            )?;

            // 3. 保存视频信息
            if let Some(video) = video_info {
                Self::save_video_inner(tx, video)?;
            }

            // 4. 保存用户信息（仅当用户不存在时）
            if let Some(user) = user_info {
                let sec_uid = &user.sec_user_id;
                if !sec_uid.is_empty() {
                    let exists: bool = tx.query_row(
                        "SELECT COUNT(*) > 0 FROM user_info WHERE sec_user_id = ?1",
                        rusqlite::params![sec_uid],
                        |row| row.get(0),
                    )?;
                    if !exists {
                        Self::save_user_inner(tx, user)?;
                    }
                }
            }

            // 5. 更新任务计数
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

            // 6. 更新任务状态为 completed
            tx.execute(
                "UPDATE download_tasks SET status = 'completed', updated_at = ?1 WHERE id = ?2",
                rusqlite::params![now, task_id],
            )?;

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

// ============================================================
// TaskRepository trait implementation for Database
// ============================================================

impl TaskRepository for super::connection::Database {
    fn create_task(&self, task: &NewDownloadTask) -> Result<()> {
        self.create_task(task)
    }

    fn update_task_status(&self, task_id: &str, status: &str, error_msg: Option<&str>) -> Result<()> {
        self.update_task_status(task_id, status, error_msg)
    }

    fn update_task_counts(&self, task_id: &str) -> Result<()> {
        self.update_task_counts(task_id)
    }

    fn create_task_item(&self, item: &NewTaskItem) -> Result<()> {
        self.create_task_item(item)
    }

    fn update_task_item_status(
        &self,
        task_id: &str,
        aweme_id: &str,
        status: &str,
        file_path: Option<&str>,
        file_size: i64,
        error_msg: Option<&str>,
    ) -> Result<()> {
        self.update_task_item_status(task_id, aweme_id, status, file_path, file_size, error_msg)
    }

    fn complete_single_download(
        &self,
        task_id: &str,
        item: &NewTaskItem,
        file_path: Option<&str>,
        file_size: i64,
        video_info: Option<&VideoInfo>,
        user_info: Option<&UserInfo>,
    ) -> Result<()> {
        self.complete_single_download(task_id, item, file_path, file_size, video_info, user_info)
    }
}
