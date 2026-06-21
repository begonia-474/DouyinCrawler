"""数据库层 — SQLite 存储用户信息和下载记录"""

import json
import aiosqlite
from pathlib import Path

DB_PATH = Path("data/douyin.db")


class Database:
    """异步 SQLite 数据库"""

    def __init__(self, db_path: str | Path = DB_PATH):
        self._path = Path(db_path)
        self._path.parent.mkdir(parents=True, exist_ok=True)
        self._db: aiosqlite.Connection | None = None

    async def connect(self):
        self._db = await aiosqlite.connect(str(self._path))
        await self._create_tables()

    async def close(self):
        if self._db:
            await self._db.close()

    async def __aenter__(self):
        await self.connect()
        return self

    async def __aexit__(self, *args):
        await self.close()

    async def _create_tables(self):
        await self._db.executescript("""
            CREATE TABLE IF NOT EXISTS user_info (
                sec_user_id TEXT PRIMARY KEY,
                nickname TEXT,
                uid TEXT,
                avatar_url TEXT,
                aweme_count INTEGER DEFAULT 0,
                follower_count INTEGER DEFAULT 0,
                following_count INTEGER DEFAULT 0,
                total_favorited INTEGER DEFAULT 0,
                signature TEXT,
                ip_location TEXT
            );

            CREATE TABLE IF NOT EXISTS video_info (
                aweme_id TEXT PRIMARY KEY,
                aweme_type INTEGER DEFAULT 0,
                desc TEXT,
                author_nickname TEXT,
                author_uid TEXT,
                author_sec_uid TEXT,
                create_time INTEGER,
                duration INTEGER DEFAULT 0,
                video_url TEXT,
                cover_url TEXT,
                music_title TEXT,
                digg_count INTEGER DEFAULT 0,
                comment_count INTEGER DEFAULT 0,
                share_count INTEGER DEFAULT 0,
                collect_count INTEGER DEFAULT 0,
                mix_id TEXT,
                mix_name TEXT
            );

            CREATE TABLE IF NOT EXISTS download_history (
                aweme_id TEXT PRIMARY KEY,
                file_path TEXT,
                file_size INTEGER DEFAULT 0,
                download_time INTEGER
            );
        """)
        await self._db.commit()

    # === 用户信息 ===

    async def save_user(self, **kwargs):
        """保存或更新用户信息"""
        sec_user_id = kwargs.get("sec_user_id")
        if not sec_user_id:
            return
        await self._db.execute("""
            INSERT OR REPLACE INTO user_info
            (sec_user_id, nickname, uid, avatar_url, aweme_count,
             follower_count, following_count, total_favorited, signature, ip_location)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """, (
            sec_user_id, kwargs.get("nickname"), kwargs.get("uid"),
            kwargs.get("avatar_url"), kwargs.get("aweme_count", 0),
            kwargs.get("follower_count", 0), kwargs.get("following_count", 0),
            kwargs.get("total_favorited", 0), kwargs.get("signature"),
            kwargs.get("ip_location"),
        ))
        await self._db.commit()

    async def get_user(self, sec_user_id: str) -> dict | None:
        """获取用户信息"""
        async with self._db.execute(
            "SELECT * FROM user_info WHERE sec_user_id = ?", (sec_user_id,)
        ) as cursor:
            row = await cursor.fetchone()
            if row:
                columns = [d[0] for d in cursor.description]
                return dict(zip(columns, row))
        return None

    # === 视频信息 ===

    async def save_video(self, **kwargs):
        """保存视频信息"""
        aweme_id = kwargs.get("aweme_id")
        if not aweme_id:
            return
        await self._db.execute("""
            INSERT OR REPLACE INTO video_info
            (aweme_id, aweme_type, desc, author_nickname, author_uid, author_sec_uid,
             create_time, duration, video_url, cover_url, music_title,
             digg_count, comment_count, share_count, collect_count, mix_id, mix_name)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """, (
            aweme_id, kwargs.get("aweme_type", 0), kwargs.get("desc"),
            kwargs.get("author"), kwargs.get("author_uid"), kwargs.get("author_sec_uid"),
            kwargs.get("create_time"), kwargs.get("duration", 0),
            kwargs.get("video_url"), kwargs.get("cover_url"), kwargs.get("music_title"),
            kwargs.get("digg_count", 0), kwargs.get("comment_count", 0),
            kwargs.get("share_count", 0), kwargs.get("collect_count", 0),
            kwargs.get("mix_id"), kwargs.get("mix_name"),
        ))
        await self._db.commit()

    async def get_video(self, aweme_id: str) -> dict | None:
        async with self._db.execute(
            "SELECT * FROM video_info WHERE aweme_id = ?", (aweme_id,)
        ) as cursor:
            row = await cursor.fetchone()
            if row:
                columns = [d[0] for d in cursor.description]
                return dict(zip(columns, row))
        return None

    async def is_video_downloaded(self, aweme_id: str) -> bool:
        """检查视频是否已下载"""
        async with self._db.execute(
            "SELECT 1 FROM download_history WHERE aweme_id = ?", (aweme_id,)
        ) as cursor:
            return await cursor.fetchone() is not None

    # === 下载记录 ===

    async def save_download(self, aweme_id: str, file_path: str, file_size: int = 0):
        """保存下载记录"""
        import time
        await self._db.execute("""
            INSERT OR REPLACE INTO download_history
            (aweme_id, file_path, file_size, download_time)
            VALUES (?, ?, ?, ?)
        """, (aweme_id, file_path, file_size, int(time.time())))
        await self._db.commit()

    async def get_download_count(self) -> int:
        """获取总下载数"""
        async with self._db.execute("SELECT COUNT(*) FROM download_history") as cursor:
            row = await cursor.fetchone()
            return row[0] if row else 0
