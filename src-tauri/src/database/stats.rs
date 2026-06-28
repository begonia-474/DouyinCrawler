//! 统计查询与健康检查
//!
//! 从 db.rs 提取，供 statistics 模块使用。

use rusqlite::Result;
use crate::database::models::{
    VideoStats, VideoTypeStat, UserStats, TrendPoint, AuthorStat, StorageStat, DbHealth,
};

impl super::connection::Database {
    pub fn get_video_stats(&self) -> Result<VideoStats> {
        let conn = crate::lock_conn!(self);
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
        let conn = crate::lock_conn!(self);
        let (total_count, total_follower, total_aweme) = conn.query_row(
            "SELECT COALESCE(COUNT(*),0), COALESCE(SUM(follower_count),0), COALESCE(SUM(aweme_count),0) FROM user_info",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        Ok(UserStats { total_count, total_follower, total_aweme })
    }

    /// 下载趋势：按日/周/月维度统计下载量
    /// range: "day"=30天, "week"=12周, "month"=12月
    pub fn get_download_trend(&self, range: &str) -> Result<Vec<TrendPoint>> {
        let conn = crate::lock_conn!(self);

        let (group_expr, label_expr, limit_clause) = match range {
            "week" => (
                "strftime('%Y-W%W', created_at, 'unixepoch', 'localtime')",
                "strftime('%Y-W%W', created_at, 'unixepoch', 'localtime')",
                "12",
            ),
            "month" => (
                "strftime('%Y-%m', created_at, 'unixepoch', 'localtime')",
                "strftime('%Y-%m', created_at, 'unixepoch', 'localtime')",
                "12",
            ),
            _ => (
                "DATE(created_at, 'unixepoch', 'localtime')",
                "DATE(created_at, 'unixepoch', 'localtime')",
                "30",
            ),
        };

        let sql = format!(
            "SELECT {} as period, COUNT(*), COALESCE(SUM(file_size), 0) \
             FROM download_history WHERE status = 'completed' \
             GROUP BY {} ORDER BY period DESC LIMIT {}",
            label_expr, group_expr, limit_clause,
        );

        let mut stmt = conn.prepare(&sql)?;
        let points = stmt
            .query_map([], |row| {
                Ok(TrendPoint {
                    day: row.get(0)?,
                    cnt: row.get(1)?,
                    size: row.get(2)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(points)
    }

    /// 下载量最多的作者 Top N
    pub fn get_top_authors(&self, limit: i64) -> Result<Vec<AuthorStat>> {
        let conn = crate::lock_conn!(self);
        let mut stmt = conn.prepare(
            "SELECT author_nickname, COUNT(*), COALESCE(SUM(file_size), 0) \
             FROM download_history \
             WHERE status = 'completed' AND author_nickname IS NOT NULL AND author_nickname != '' \
             GROUP BY author_nickname ORDER BY COUNT(*) DESC LIMIT ?1",
        )?;
        let authors = stmt
            .query_map(rusqlite::params![limit], |row| {
                Ok(AuthorStat {
                    author_nickname: row.get(0)?,
                    cnt: row.get(1)?,
                    total_size: row.get(2)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(authors)
    }

    /// 存储占用分析：按下载类型统计
    pub fn get_storage_analysis(&self) -> Result<Vec<StorageStat>> {
        let conn = crate::lock_conn!(self);
        let mut stmt = conn.prepare(
            "SELECT download_type, COUNT(*), COALESCE(SUM(file_size), 0) \
             FROM download_history WHERE status = 'completed' \
             GROUP BY download_type ORDER BY SUM(file_size) DESC",
        )?;
        let stats = stmt
            .query_map([], |row| {
                Ok(StorageStat {
                    download_type: row.get(0)?,
                    cnt: row.get(1)?,
                    total_size: row.get(2)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(stats)
    }

    /// 数据库健康检查：各表记录数 + 数据库文件大小
    pub fn db_health_check(&self) -> Result<DbHealth> {
        let conn = crate::lock_conn!(self);

        let count = |table: &str| -> Result<i64> {
            let c: i64 = conn.query_row(
                &format!("SELECT COUNT(*) FROM {}", table), [], |row| row.get(0),
            )?;
            Ok(c)
        };

        let db_size = std::fs::metadata(self.db_path())
            .map(|m| m.len() as i64)
            .unwrap_or(0);

        Ok(DbHealth {
            download_count: count("download_history")?,
            video_count: count("video_info")?,
            user_count: count("user_info")?,
            live_count: count("live_records")?,
            music_count: count("music_collection")?,
            task_count: count("download_tasks")?,
            db_size_bytes: db_size,
        })
    }
}
