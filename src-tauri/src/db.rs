//! 数据库模块 — 薄包装
//!
//! 原 db.rs (~1800 行) 已拆分为 `database/` 目录下的按实体组织的模块：
//!   - `connection.rs`  — Database struct, open(), with_transaction(), lock_conn! macro
//!   - `migration.rs`   — CREATE_TABLES_SQL, column constants, migration v1-v8
//!   - `models.rs`      — serde DTOs (VideoInfo, UserInfo, DownloadRecord, etc.)
//!   - `query_builder.rs` — ColKind + build_upsert_sql()
//!   - `video_repo.rs`  — video_info CRUD
//!   - `user_repo.rs`   — user_info CRUD
//!   - `download_repo.rs` — download_history + live_records CRUD
//!   - `task_repo.rs`   — download_tasks + download_task_items CRUD
//!   - `music_repo.rs`  — music_collection CRUD
//!   - `stats.rs`       — get_video_stats, get_user_stats, get_download_trend, etc.
//!
//! 此文件保留为向后兼容的 re-export 层，外部 `use crate::db::*` 继续工作。

// 核心类型
pub use crate::database::connection::Database;

// 模型类型
pub use crate::database::models::*;

// lock_conn! 宏已通过 #[macro_export] 在 crate root 可用
