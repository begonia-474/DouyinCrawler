use rusqlite::{Connection, Result};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::database::models::*;
use crate::database::query_builder::{build_upsert_sql, ColKind};
use crate::lock_conn;

impl super::connection::Database {
    pub(crate) fn row_to_video(row: &rusqlite::Row) -> rusqlite::Result<VideoInfo> {
        Ok(VideoInfo {
            aweme_id: row.get("aweme_id")?,
            desc: row.get("desc")?,
            aweme_type: row.get::<_, Option<i32>>("aweme_type")?.unwrap_or(0),
            author_nickname: row.get("author_nickname")?,
            author_sec_uid: row.get("author_sec_uid")?,
            author_uid: row.get("author_uid")?,
            create_time: row.get("create_time")?,
            duration: row.get::<_, Option<i32>>("duration")?.unwrap_or(0),
            video_url: row.get("video_url")?,
            cover_url: row.get("cover_url")?,
            music_title: row.get("music_title")?,
            digg_count: row.get::<_, Option<i64>>("digg_count")?.unwrap_or(0),
            comment_count: row.get::<_, Option<i64>>("comment_count")?.unwrap_or(0),
            share_count: row.get::<_, Option<i64>>("share_count")?.unwrap_or(0),
            collect_count: row.get::<_, Option<i64>>("collect_count")?.unwrap_or(0),
            mix_id: row.get("mix_id")?,
            mix_name: row.get("mix_name")?,
            author_nickname_raw: row.get("author_nickname_raw")?,
            author_short_id: row.get("author_short_id")?,
            author_unique_id: row.get("author_unique_id")?,
            desc_raw: row.get("desc_raw")?,
            is_ads: row.get::<_, Option<i32>>("is_ads")?.unwrap_or(0),
            is_story: row.get::<_, Option<i32>>("is_story")?.unwrap_or(0),
            is_top: row.get::<_, Option<i32>>("is_top")?.unwrap_or(0),
            is_long_video: row.get::<_, Option<i32>>("is_long_video")?.unwrap_or(0),
            video_bit_rate: row.get("video_bit_rate")?,
            animated_cover: row.get("animated_cover")?,
            private_status: row.get::<_, Option<i32>>("private_status")?.unwrap_or(0),
            is_delete: row.get::<_, Option<i32>>("is_delete")?.unwrap_or(0),
            music_author: row.get("music_author")?,
            music_author_raw: row.get("music_author_raw")?,
            music_duration: row.get::<_, Option<i32>>("music_duration")?.unwrap_or(0),
            music_id: row.get("music_id")?,
            music_mid: row.get("music_mid")?,
            pgc_author: row.get("pgc_author")?,
            pgc_author_title: row.get("pgc_author_title")?,
            pgc_music_type: row.get::<_, Option<i32>>("pgc_music_type")?.unwrap_or(0),
            music_status: row.get::<_, Option<i32>>("music_status")?.unwrap_or(0),
            music_owner_handle: row.get("music_owner_handle")?,
            music_owner_id: row.get("music_owner_id")?,
            music_owner_nickname: row.get("music_owner_nickname")?,
            music_play_url: row.get("music_play_url")?,
            is_commerce_music: row.get::<_, Option<i32>>("is_commerce_music")?.unwrap_or(0),
            mix_desc: row.get("mix_desc")?,
            mix_create_time: row.get::<_, Option<i64>>("mix_create_time")?.unwrap_or(0),
            mix_pic_type: row.get::<_, Option<i32>>("mix_pic_type")?.unwrap_or(0),
            mix_type: row.get::<_, Option<i32>>("mix_type")?.unwrap_or(0),
            mix_share_url: row.get("mix_share_url")?,
            can_comment: row.get::<_, Option<i32>>("can_comment")?.unwrap_or(0),
            can_forward: row.get::<_, Option<i32>>("can_forward")?.unwrap_or(0),
            can_share: row.get::<_, Option<i32>>("can_share")?.unwrap_or(0),
            download_setting: row.get::<_, Option<i32>>("download_setting")?.unwrap_or(0),
            allow_douplus: row.get::<_, Option<i32>>("allow_douplus")?.unwrap_or(0),
            allow_share: row.get::<_, Option<i32>>("allow_share")?.unwrap_or(0),
            admire_count: row.get::<_, Option<i64>>("admire_count")?.unwrap_or(0),
            hashtag_ids: row.get("hashtag_ids")?,
            hashtag_names: row.get("hashtag_names")?,
            images: row.get("images")?,
            region: row.get("region")?,
            is_prohibited: row.get::<_, Option<i32>>("is_prohibited")?.unwrap_or(0),
            updated_at: row.get::<_, Option<i64>>("updated_at")?.unwrap_or(0),
        })
    }

    pub(crate) fn validate_sort_field(allowed: &[&str], sort_by: &Option<String>, default: &str) -> String {
        match sort_by {
            Some(s) if allowed.contains(&s.as_str()) => s.clone(),
            _ => default.to_string(),
        }
    }

    pub(crate) fn validate_sort_order(order: &Option<String>) -> String {
        match order.as_deref() {
            Some("asc") => "ASC".to_string(),
            _ => "DESC".to_string(),
        }
    }

    pub fn get_videos(
        &self,
        limit: i64,
        offset: i64,
        keyword: Option<String>,
        author_sec_uid: Option<String>,
        sort_by: Option<String>,
        sort_order: Option<String>,
        post_type: Option<String>,
    ) -> Result<Vec<VideoInfo>> {
        let conn = lock_conn!(self);

        let allowed_sorts = ["create_time", "digg_count", "comment_count", "share_count", "collect_count", "updated_at"];
        let sort_col = Self::validate_sort_field(&allowed_sorts, &sort_by, "updated_at");
        let sort_dir = Self::validate_sort_order(&sort_order);

        let mut sql = format!("SELECT {} FROM video_info", super::migration::VIDEO_COLUMNS);
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut conditions = Vec::new();

        if let Some(ref pt) = post_type {
            match pt.as_str() {
                "video" => conditions.push("aweme_type != 68"),
                "images" => conditions.push("aweme_type = 68"),
                _ => {}
            }
        }
        if let Some(ref author) = author_sec_uid {
            if !author.is_empty() {
                conditions.push("author_sec_uid = ?");
                params.push(Box::new(author.clone()));
            }
        }
        if let Some(ref kw) = keyword {
            if !kw.is_empty() {
                conditions.push("(desc LIKE ? OR author_nickname LIKE ?)");
                let pattern = format!("%{}%", kw);
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern));
            }
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }
        sql.push_str(&format!(" ORDER BY {} {} LIMIT ? OFFSET ?", sort_col, sort_dir));
        params.push(Box::new(limit));
        params.push(Box::new(offset));

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(param_refs.as_slice(), Self::row_to_video)?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    pub fn get_video_count(
        &self,
        keyword: Option<String>,
        author_sec_uid: Option<String>,
        post_type: Option<String>,
    ) -> Result<i64> {
        let conn = lock_conn!(self);
        let mut sql = String::from("SELECT COUNT(*) FROM video_info");
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut conditions = Vec::new();

        if let Some(ref pt) = post_type {
            match pt.as_str() {
                "video" => conditions.push("aweme_type != 68"),
                "images" => conditions.push("aweme_type = 68"),
                _ => {}
            }
        }
        if let Some(ref author) = author_sec_uid {
            if !author.is_empty() {
                conditions.push("author_sec_uid = ?");
                params.push(Box::new(author.clone()));
            }
        }
        if let Some(ref kw) = keyword {
            if !kw.is_empty() {
                conditions.push("(desc LIKE ? OR author_nickname LIKE ?)");
                let pattern = format!("%{}%", kw);
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern));
            }
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let count: i64 = conn.query_row(&sql, param_refs.as_slice(), |row| row.get(0))?;
        Ok(count)
    }

    pub(crate) fn save_video_inner(conn: &Connection, video: &VideoInfo) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        use ColKind::{Normal as N, Stat as S, Volatile as V};
        let cols: &[(&str, ColKind)] = &[
            ("aweme_id", N), ("desc", N), ("aweme_type", N),
            ("author_nickname", N), ("author_sec_uid", N), ("author_uid", N),
            ("create_time", N), ("duration", N),
            ("video_url", V), ("cover_url", V), ("music_title", N),
            ("digg_count", S), ("comment_count", S), ("share_count", S), ("collect_count", S),
            ("mix_id", N), ("mix_name", N), ("updated_at", N),
            ("author_nickname_raw", N), ("author_short_id", N), ("author_unique_id", N),
            ("desc_raw", N), ("is_ads", N), ("is_story", N), ("is_top", N), ("is_long_video", N),
            ("video_bit_rate", V), ("animated_cover", V), ("private_status", N), ("is_delete", N),
            ("music_author", N), ("music_author_raw", N), ("music_duration", N),
            ("music_id", N), ("music_mid", N), ("pgc_author", N), ("pgc_author_title", N),
            ("pgc_music_type", N), ("music_status", N),
            ("music_owner_handle", N), ("music_owner_id", N), ("music_owner_nickname", N),
            ("music_play_url", V), ("is_commerce_music", N),
            ("mix_desc", N), ("mix_create_time", N), ("mix_pic_type", N),
            ("mix_type", N), ("mix_share_url", N),
            ("can_comment", N), ("can_forward", N), ("can_share", N),
            ("download_setting", N), ("allow_douplus", N), ("allow_share", N),
            ("admire_count", S), ("hashtag_ids", N), ("hashtag_names", N),
            ("images", V), ("region", N), ("is_prohibited", N),
        ];

        let sql = build_upsert_sql("video_info", "aweme_id", cols);
        conn.execute(
            &sql,
            rusqlite::params![
                video.aweme_id, video.desc, video.aweme_type,
                video.author_nickname, video.author_sec_uid, video.author_uid,
                video.create_time, video.duration, video.video_url, video.cover_url,
                video.music_title, video.digg_count, video.comment_count,
                video.share_count, video.collect_count, video.mix_id, video.mix_name,
                now,
                video.author_nickname_raw, video.author_short_id, video.author_unique_id,
                video.desc_raw, video.is_ads, video.is_story, video.is_top, video.is_long_video,
                video.video_bit_rate, video.animated_cover, video.private_status, video.is_delete,
                video.music_author, video.music_author_raw, video.music_duration,
                video.music_id, video.music_mid, video.pgc_author, video.pgc_author_title,
                video.pgc_music_type, video.music_status, video.music_owner_handle,
                video.music_owner_id, video.music_owner_nickname, video.music_play_url,
                video.is_commerce_music,
                video.mix_desc, video.mix_create_time, video.mix_pic_type,
                video.mix_type, video.mix_share_url,
                video.can_comment, video.can_forward, video.can_share,
                video.download_setting, video.allow_douplus, video.allow_share,
                video.admire_count, video.hashtag_ids, video.hashtag_names,
                video.images, video.region, video.is_prohibited,
            ],
        )?;
        Ok(())
    }

    pub fn save_video(&self, video: &VideoInfo) -> Result<()> {
        let conn = lock_conn!(self);
        Self::save_video_inner(&conn, video)
    }

    /// 批量保存下载结果（单次事务：video_info + user_info）
    pub fn save_batch_results(
        &self,
        videos: &[VideoInfo],
        users: &[UserInfo],
    ) -> Result<()> {
        self.with_transaction(|tx| {
            for video in videos {
                Self::save_video_inner(tx, video)?;
            }
            for user in users {
                Self::save_user_inner(tx, user)?;
            }
            Ok(())
        })
    }

    pub fn delete_video(&self, aweme_id: &str) -> Result<()> {
        let conn = lock_conn!(self);
        conn.execute("DELETE FROM video_info WHERE aweme_id = ?1", rusqlite::params![aweme_id])?;
        Ok(())
    }

    /// 批量删除视频记录（事务保证原子性）
    pub fn delete_videos_batch(&self, aweme_ids: &[String]) -> Result<()> {
        self.with_transaction(|tx| {
            let mut stmt = tx.prepare("DELETE FROM video_info WHERE aweme_id = ?1")?;
            for id in aweme_ids {
                stmt.execute(rusqlite::params![id])?;
            }
            Ok(())
        })
    }

}
