//! 数据库模块
//!
//! 拆分自 db.rs，按实体组织为 repository 模式目录。
//! db.rs 现为薄包装，重导出所有公有 API 保持向后兼容。

pub mod connection;
pub mod migration;
pub mod models;
pub mod query_builder;
pub mod download_repo;
pub mod video_repo;
pub mod user_repo;
pub mod music_repo;
pub mod task_repo;
pub mod stats;
