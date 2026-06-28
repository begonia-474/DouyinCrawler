use rusqlite::Result;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::database::models::{MusicCollection, NewMusicCollection};
use crate::lock_conn;

impl super::connection::Database {
    pub fn save_music_collection(&self, music: &NewMusicCollection) -> Result<()> {
        let conn = lock_conn!(self);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        conn.execute(
            "INSERT OR REPLACE INTO music_collection \
             (music_id, mid, title, author, owner_nickname, duration, cover, play_url, status, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'collected', ?9)",
            rusqlite::params![
                music.music_id,
                music.mid,
                music.title,
                music.author,
                music.owner_nickname,
                music.duration,
                music.cover,
                music.play_url,
                now,
            ],
        )?;
        Ok(())
    }

    pub fn save_music_collection_batch(&self, musics: &[NewMusicCollection]) -> Result<()> {
        let conn = lock_conn!(self);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO music_collection \
             (music_id, mid, title, author, owner_nickname, duration, cover, play_url, status, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'collected', ?9)"
        )?;

        for music in musics {
            stmt.execute(rusqlite::params![
                music.music_id,
                music.mid,
                music.title,
                music.author,
                music.owner_nickname,
                music.duration,
                music.cover,
                music.play_url,
                now,
            ])?;
        }
        Ok(())
    }

    pub fn get_music_collection(
        &self,
        limit: i64,
        offset: i64,
        keyword: Option<String>,
        status: Option<String>,
    ) -> Result<Vec<MusicCollection>> {
        let conn = lock_conn!(self);

        let mut sql = String::from(
            "SELECT music_id, mid, title, author, owner_nickname, duration, cover, play_url, file_path, status, created_at \
             FROM music_collection"
        );
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut conditions = Vec::new();

        if let Some(ref s) = status {
            if !s.is_empty() {
                conditions.push("status = ?");
                params.push(Box::new(s.clone()));
            }
        }

        if let Some(ref kw) = keyword {
            if !kw.is_empty() {
                conditions.push("(title LIKE ? OR author LIKE ? OR owner_nickname LIKE ?)");
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
        sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");
        params.push(Box::new(limit));
        params.push(Box::new(offset));

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok(MusicCollection {
                music_id: row.get("music_id")?,
                mid: row.get("mid")?,
                title: row.get("title")?,
                author: row.get("author")?,
                owner_nickname: row.get("owner_nickname")?,
                duration: row.get("duration")?,
                cover: row.get("cover")?,
                play_url: row.get("play_url")?,
                file_path: row.get("file_path")?,
                status: row.get("status")?,
                created_at: row.get("created_at")?,
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    pub fn get_music_collection_count(&self, keyword: Option<String>, status: Option<String>) -> Result<i64> {
        let conn = lock_conn!(self);
        let mut sql = String::from("SELECT COUNT(*) FROM music_collection");
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut conditions = Vec::new();

        if let Some(ref s) = status {
            if !s.is_empty() {
                conditions.push("status = ?");
                params.push(Box::new(s.clone()));
            }
        }

        if let Some(ref kw) = keyword {
            if !kw.is_empty() {
                conditions.push("(title LIKE ? OR author LIKE ? OR owner_nickname LIKE ?)");
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

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let count: i64 = conn.query_row(&sql, param_refs.as_slice(), |row| row.get(0))?;
        Ok(count)
    }

    pub fn update_music_file_path(&self, music_id: &str, file_path: &str) -> Result<()> {
        let conn = lock_conn!(self);
        conn.execute(
            "UPDATE music_collection SET file_path = ?1, status = 'downloaded' WHERE music_id = ?2",
            rusqlite::params![file_path, music_id],
        )?;
        Ok(())
    }

    pub fn get_music_file_path(&self, music_id: &str) -> Result<Option<String>> {
        let conn = lock_conn!(self);
        let mut stmt = conn.prepare("SELECT file_path FROM music_collection WHERE music_id = ?1")?;
        let mut rows = stmt.query_map(rusqlite::params![music_id], |row| row.get(0))?;
        match rows.next() {
            Some(row) => row,
            None => Ok(None),
        }
    }

    pub fn delete_music_collection(&self, music_id: &str) -> Result<()> {
        let conn = lock_conn!(self);
        conn.execute("DELETE FROM music_collection WHERE music_id = ?1", rusqlite::params![music_id])?;
        Ok(())
    }
}
