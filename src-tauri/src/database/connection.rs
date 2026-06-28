use rusqlite::{Connection, Result as SqliteResult};
use parking_lot::Mutex;
use std::path::PathBuf;
use std::fs;
use log::info;

pub struct Database {
    pub(crate) conn: Mutex<Connection>,
    pub(crate) db_path: PathBuf,
}

/// 获取数据库连接锁（parking_lot 无 poisoning 概念）
#[macro_export]
macro_rules! lock_conn {
    ($self:expr) => {
        $self.conn.lock()
    };
}

impl Database {
    /// 在事务中执行闭包，失败自动回滚
    pub fn with_transaction<F, R>(&self, f: F) -> SqliteResult<R>
    where
        F: FnOnce(&rusqlite::Transaction) -> SqliteResult<R>,
    {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;
        let result = f(&tx)?;
        tx.commit()?;
        Ok(result)
    }

    /// 打开（或创建）数据库，执行 WAL 配置、建表和迁移。
    /// 对应原 `db.rs` 的 `Database::open`，CREATE_TABLES_SQL 来自 migration 模块。
    pub fn open(path: &PathBuf) -> SqliteResult<Self> {
        // 确保父目录存在
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let conn = Connection::open(path)?;
        // 启用 WAL 模式
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA busy_timeout=5000;")?;
        // 执行 WAL checkpoint
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        // 自动建表
        conn.execute_batch(super::migration::CREATE_TABLES_SQL)?;
        // Schema 迁移
        super::migration::migrate(&conn)?;
        // 验证数据
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM download_history",
            [],
            |row| row.get(0),
        ).unwrap_or(0);
        info!("[DB] 数据库打开成功，download_history 表有 {} 条记录", count);
        Ok(Database {
            conn: Mutex::new(conn),
            db_path: path.clone(),
        })
    }

    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
    }
}
