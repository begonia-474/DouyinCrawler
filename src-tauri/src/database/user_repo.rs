use rusqlite::{Connection, Result};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::database::models::*;
use crate::database::query_builder::{build_upsert_sql, ColKind};
use crate::lock_conn;

impl super::connection::Database {
    pub(crate) fn row_to_user(row: &rusqlite::Row) -> rusqlite::Result<UserInfo> {
        Ok(UserInfo {
            sec_user_id: row.get("sec_user_id")?,
            nickname: row.get("nickname")?,
            uid: row.get("uid")?,
            avatar_url: row.get("avatar_url")?,
            unique_id: row.get("unique_id")?,
            signature: row.get("signature")?,
            aweme_count: row.get::<_, Option<i64>>("aweme_count")?.unwrap_or(0),
            follower_count: row.get::<_, Option<i64>>("follower_count")?.unwrap_or(0),
            following_count: row.get::<_, Option<i64>>("following_count")?.unwrap_or(0),
            total_favorited: row.get::<_, Option<i64>>("total_favorited")?.unwrap_or(0),
            ip_location: row.get("ip_location")?,
            live_status: row.get::<_, Option<i32>>("live_status")?.unwrap_or(0),
            room_id: row.get("room_id")?,
            city: row.get("city")?,
            country: row.get("country")?,
            favoriting_count: row.get::<_, Option<i64>>("favoriting_count")?.unwrap_or(0),
            gender: row.get::<_, Option<i32>>("gender")?.unwrap_or(0),
            is_ban: row.get::<_, Option<i32>>("is_ban")?.unwrap_or(0),
            is_block: row.get::<_, Option<i32>>("is_block")?.unwrap_or(0),
            is_blocked: row.get::<_, Option<i32>>("is_blocked")?.unwrap_or(0),
            is_star: row.get::<_, Option<i32>>("is_star")?.unwrap_or(0),
            mix_count: row.get::<_, Option<i32>>("mix_count")?.unwrap_or(0),
            mplatform_followers_count: row.get::<_, Option<i64>>("mplatform_followers_count")?.unwrap_or(0),
            nickname_raw: row.get("nickname_raw")?,
            school_name: row.get("school_name")?,
            short_id: row.get("short_id")?,
            signature_raw: row.get("signature_raw")?,
            user_age: row.get::<_, Option<i32>>("user_age")?.unwrap_or(0),
            custom_verify: row.get("custom_verify")?,
            updated_at: row.get::<_, Option<i64>>("updated_at")?.unwrap_or(0),
        })
    }

    pub fn get_users(
        &self,
        limit: i64,
        offset: i64,
        keyword: Option<String>,
        sort_by: Option<String>,
        sort_order: Option<String>,
    ) -> Result<Vec<UserInfo>> {
        let conn = lock_conn!(self);

        let allowed_sorts = ["follower_count", "aweme_count", "following_count", "total_favorited", "updated_at"];
        let sort_col = Self::validate_sort_field(&allowed_sorts, &sort_by, "updated_at");
        let sort_dir = Self::validate_sort_order(&sort_order);

        let mut sql = format!("SELECT {} FROM user_info", super::migration::USER_COLUMNS);
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut conditions = Vec::new();

        if let Some(ref kw) = keyword {
            if !kw.is_empty() {
                conditions.push("(nickname LIKE ? OR unique_id LIKE ? OR signature LIKE ?)");
                let pattern = format!("%{}%", kw);
                params.push(Box::new(pattern.clone()));
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
        let rows = stmt.query_map(param_refs.as_slice(), Self::row_to_user)?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    pub fn get_user_count(&self, keyword: Option<String>) -> Result<i64> {
        let conn = lock_conn!(self);
        let mut sql = String::from("SELECT COUNT(*) FROM user_info");
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref kw) = keyword {
            if !kw.is_empty() {
                sql.push_str(" WHERE (nickname LIKE ? OR unique_id LIKE ? OR signature LIKE ?)");
                let pattern = format!("%{}%", kw);
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern));
            }
        }

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let count: i64 = conn.query_row(&sql, param_refs.as_slice(), |row| row.get(0))?;
        Ok(count)
    }

    pub fn get_user_by_sec_uid(&self, sec_user_id: &str) -> Result<Option<UserInfo>> {
        let conn = lock_conn!(self);
        let mut stmt = conn.prepare(&format!(
            "SELECT {} FROM user_info WHERE sec_user_id = ?1",
            super::migration::USER_COLUMNS
        ))?;
        let mut rows = stmt.query_map(rusqlite::params![sec_user_id], Self::row_to_user)?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub(crate) fn save_user_inner(conn: &Connection, user: &UserInfo) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        use ColKind::{Normal as N, Stat as S};
        let cols: &[(&str, ColKind)] = &[
            ("sec_user_id", N), ("nickname", N), ("uid", N),
            ("avatar_url", N), ("unique_id", N), ("signature", N),
            ("aweme_count", S), ("follower_count", S), ("following_count", S),
            ("total_favorited", S), ("ip_location", N),
            ("live_status", N), ("room_id", N), ("updated_at", N),
            ("city", N), ("country", N), ("favoriting_count", S), ("gender", N),
            ("is_ban", N), ("is_block", N), ("is_blocked", N), ("is_star", N),
            ("mix_count", S), ("mplatform_followers_count", S),
            ("nickname_raw", N), ("school_name", N), ("short_id", N),
            ("signature_raw", N), ("user_age", N), ("custom_verify", N),
        ];

        let sql = build_upsert_sql("user_info", "sec_user_id", cols);
        conn.execute(
            &sql,
            rusqlite::params![
                user.sec_user_id, user.nickname, user.uid, user.avatar_url,
                user.unique_id, user.signature, user.aweme_count, user.follower_count,
                user.following_count, user.total_favorited, user.ip_location,
                user.live_status, user.room_id, now,
                user.city, user.country, user.favoriting_count, user.gender,
                user.is_ban, user.is_block, user.is_blocked, user.is_star,
                user.mix_count, user.mplatform_followers_count, user.nickname_raw,
                user.school_name, user.short_id, user.signature_raw, user.user_age,
                user.custom_verify,
            ],
        )?;
        Ok(())
    }

    pub fn save_user(&self, user: &UserInfo) -> Result<()> {
        let conn = lock_conn!(self);
        Self::save_user_inner(&conn, user)
    }

    pub fn delete_user(&self, sec_user_id: &str) -> Result<()> {
        let conn = lock_conn!(self);
        conn.execute("DELETE FROM user_info WHERE sec_user_id = ?1", rusqlite::params![sec_user_id])?;
        Ok(())
    }
}
