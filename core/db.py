"""数据库层 — SQLite 存储用户信息、视频信息、下载记录和直播录制记录"""

import time
import aiosqlite
from pathlib import Path

DB_PATH = Path("data/douyin.db")
SCHEMA_VERSION = 1


class Database:
    """异步 SQLite 数据库（WAL 模式，支持 Python 写 + Rust 读并发）"""

    def __init__(self, db_path: str | Path = DB_PATH):
        self._path = Path(db_path)
        self._path.parent.mkdir(parents=True, exist_ok=True)
        self._db: aiosqlite.Connection | None = None

    async def connect(self):
        self._db = await aiosqlite.connect(str(self._path))
        await self._db.execute("PRAGMA journal_mode=WAL")
        await self._db.execute("PRAGMA synchronous=NORMAL")
        await self._create_tables()

    async def close(self):
        if self._db:
            await self._db.close()

    async def __aenter__(self):
        await self.connect()
        return self

    async def __aexit__(self, *args):
        await self.close()

    @property
    def path(self) -> Path:
        return self._path

    async def _create_tables(self):
        await self._db.executescript("""
            CREATE TABLE IF NOT EXISTS _metadata (
                name TEXT PRIMARY KEY,
                value TEXT
            );

            CREATE TABLE IF NOT EXISTS user_info (
                sec_user_id TEXT PRIMARY KEY,
                nickname TEXT,
                uid TEXT,
                avatar_url TEXT,
                unique_id TEXT,
                signature TEXT,
                aweme_count INTEGER DEFAULT 0,
                follower_count INTEGER DEFAULT 0,
                following_count INTEGER DEFAULT 0,
                total_favorited INTEGER DEFAULT 0,
                ip_location TEXT,
                live_status INTEGER DEFAULT 0,
                room_id TEXT,
                updated_at INTEGER DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS video_info (
                aweme_id TEXT PRIMARY KEY,
                desc TEXT,
                aweme_type INTEGER DEFAULT 0,
                author_nickname TEXT,
                author_sec_uid TEXT,
                author_uid TEXT,
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
                mix_name TEXT,
                updated_at INTEGER DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS download_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                aweme_id TEXT,
                download_type TEXT NOT NULL DEFAULT 'video',
                title TEXT,
                author_nickname TEXT,
                author_sec_uid TEXT,
                file_path TEXT,
                file_size INTEGER DEFAULT 0,
                cover_url TEXT,
                status TEXT NOT NULL DEFAULT 'completed',
                error_msg TEXT,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS live_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                room_id TEXT,
                web_rid TEXT,
                title TEXT,
                nickname TEXT,
                sec_user_id TEXT,
                file_path TEXT,
                file_size INTEGER DEFAULT 0,
                duration_sec INTEGER DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'completed',
                started_at INTEGER,
                ended_at INTEGER
            );

            CREATE INDEX IF NOT EXISTS idx_download_created ON download_history(created_at);
            CREATE INDEX IF NOT EXISTS idx_download_type ON download_history(download_type);
            CREATE INDEX IF NOT EXISTS idx_download_status ON download_history(status);
            CREATE INDEX IF NOT EXISTS idx_download_author ON download_history(author_sec_uid);
            CREATE INDEX IF NOT EXISTS idx_live_started ON live_records(started_at);
        """)
        await self._db.commit()

        # 写入 schema 版本
        existing = await self._fetch_one(
            "SELECT value FROM _metadata WHERE name = 'schema_version'"
        )
        if not existing:
            await self._db.execute(
                "INSERT INTO _metadata (name, value) VALUES ('schema_version', ?)",
                (str(SCHEMA_VERSION),),
            )
            await self._db.commit()

    # === 通用查询方法 ===

    async def _fetch_one(self, sql: str, params: tuple = ()) -> dict | None:
        async with self._db.execute(sql, params) as cursor:
            row = await cursor.fetchone()
            if row:
                columns = [d[0] for d in cursor.description]
                return dict(zip(columns, row))
        return None

    async def _fetch_all(self, sql: str, params: tuple = ()) -> list[dict]:
        async with self._db.execute(sql, params) as cursor:
            rows = await cursor.fetchall()
            if rows:
                columns = [d[0] for d in cursor.description]
                return [dict(zip(columns, row)) for row in rows]
        return []

    # === 用户信息 ===

    async def save_user(self, **kwargs):
        """保存或更新用户信息（upsert）"""
        sec_user_id = kwargs.get("sec_user_id")
        if not sec_user_id:
            return
        await self._db.execute("""
            INSERT OR REPLACE INTO user_info
            (sec_user_id, nickname, uid, avatar_url, unique_id, signature,
             aweme_count, follower_count, following_count, total_favorited,
             ip_location, live_status, room_id, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """, (
            sec_user_id,
            kwargs.get("nickname"),
            kwargs.get("uid"),
            kwargs.get("avatar_url"),
            kwargs.get("unique_id"),
            kwargs.get("signature"),
            kwargs.get("aweme_count", 0),
            kwargs.get("follower_count", 0),
            kwargs.get("following_count", 0),
            kwargs.get("total_favorited", 0),
            kwargs.get("ip_location"),
            kwargs.get("live_status", 0),
            kwargs.get("room_id"),
            int(time.time()),
        ))
        await self._db.commit()
        await self._db.execute("PRAGMA wal_checkpoint(TRUNCATE)")

    async def get_user(self, sec_user_id: str) -> dict | None:
        """获取用户信息"""
        return await self._fetch_one(
            "SELECT * FROM user_info WHERE sec_user_id = ?", (sec_user_id,)
        )

    # === 视频信息 ===

    async def save_video(self, **kwargs):
        """保存视频信息（upsert）"""
        aweme_id = kwargs.get("aweme_id")
        if not aweme_id:
            return
        await self._db.execute("""
            INSERT OR REPLACE INTO video_info
            (aweme_id, desc, aweme_type, author_nickname, author_sec_uid, author_uid,
             create_time, duration, video_url, cover_url, music_title,
             digg_count, comment_count, share_count, collect_count,
             mix_id, mix_name, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """, (
            aweme_id,
            kwargs.get("desc"),
            kwargs.get("aweme_type", 0),
            kwargs.get("author_nickname"),
            kwargs.get("author_sec_uid"),
            kwargs.get("author_uid"),
            kwargs.get("create_time"),
            kwargs.get("duration", 0),
            kwargs.get("video_url"),
            kwargs.get("cover_url"),
            kwargs.get("music_title"),
            kwargs.get("digg_count", 0),
            kwargs.get("comment_count", 0),
            kwargs.get("share_count", 0),
            kwargs.get("collect_count", 0),
            kwargs.get("mix_id"),
            kwargs.get("mix_name"),
            int(time.time()),
        ))
        await self._db.commit()
        await self._db.execute("PRAGMA wal_checkpoint(TRUNCATE)")

    async def get_video(self, aweme_id: str) -> dict | None:
        """获取视频信息"""
        return await self._fetch_one(
            "SELECT * FROM video_info WHERE aweme_id = ?", (aweme_id,)
        )

    async def is_video_downloaded(self, aweme_id: str) -> bool:
        """检查视频是否已下载"""
        row = await self._fetch_one(
            "SELECT 1 FROM download_history WHERE aweme_id = ? AND status = 'completed'",
            (aweme_id,),
        )
        return row is not None

    # === 下载记录 ===

    async def save_download(
        self,
        *,
        aweme_id: str | None = None,
        download_type: str = "video",
        title: str | None = None,
        author_nickname: str | None = None,
        author_sec_uid: str | None = None,
        file_path: str | None = None,
        file_size: int = 0,
        cover_url: str | None = None,
        status: str = "completed",
        error_msg: str | None = None,
    ):
        """保存下载记录"""
        await self._db.execute("""
            INSERT INTO download_history
            (aweme_id, download_type, title, author_nickname, author_sec_uid,
             file_path, file_size, cover_url, status, error_msg, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """, (
            aweme_id, download_type, title, author_nickname, author_sec_uid,
            file_path, file_size, cover_url, status, error_msg, int(time.time()),
        ))
        await self._db.commit()
        # WAL checkpoint，确保 Rust 侧能读取到最新数据
        await self._db.execute("PRAGMA wal_checkpoint(TRUNCATE)")

    async def get_download_count(self) -> int:
        """获取总下载数"""
        row = await self._fetch_one(
            "SELECT COUNT(*) as cnt FROM download_history WHERE status = 'completed'"
        )
        return row["cnt"] if row else 0

    async def get_download_total_size(self) -> int:
        """获取总下载大小（字节）"""
        row = await self._fetch_one(
            "SELECT COALESCE(SUM(file_size), 0) as total FROM download_history WHERE status = 'completed'"
        )
        return row["total"] if row else 0

    async def get_downloads(
        self,
        *,
        limit: int = 20,
        offset: int = 0,
        status: str | None = None,
        download_type: str | None = None,
        author_sec_uid: str | None = None,
    ) -> list[dict]:
        """获取下载记录（分页 + 筛选）"""
        conditions = []
        params = []

        if status:
            conditions.append("status = ?")
            params.append(status)
        if download_type:
            conditions.append("download_type = ?")
            params.append(download_type)
        if author_sec_uid:
            conditions.append("author_sec_uid = ?")
            params.append(author_sec_uid)

        where = f"WHERE {' AND '.join(conditions)}" if conditions else ""
        sql = f"""
            SELECT * FROM download_history {where}
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
        """
        params.extend([limit, offset])
        return await self._fetch_all(sql, tuple(params))

    async def get_download_stats(self) -> dict:
        """获取下载统计"""
        total = await self._fetch_one(
            "SELECT COUNT(*) as cnt, COALESCE(SUM(file_size), 0) as size FROM download_history WHERE status = 'completed'"
        )
        by_type = await self._fetch_all(
            "SELECT download_type, COUNT(*) as cnt, COALESCE(SUM(file_size), 0) as size FROM download_history WHERE status = 'completed' GROUP BY download_type"
        )
        by_day = await self._fetch_all("""
            SELECT DATE(created_at, 'unixepoch', 'localtime') as day, COUNT(*) as cnt
            FROM download_history WHERE status = 'completed'
            AND created_at > ?
            GROUP BY day ORDER BY day DESC
        """, (int(time.time()) - 7 * 86400,))

        return {
            "total_count": total["cnt"] if total else 0,
            "total_size": total["size"] if total else 0,
            "by_type": by_type,
            "by_day": by_day,
        }

    # === 直播录制记录 ===

    async def save_live_record(
        self,
        *,
        room_id: str | None = None,
        web_rid: str | None = None,
        title: str | None = None,
        nickname: str | None = None,
        sec_user_id: str | None = None,
        file_path: str | None = None,
        file_size: int = 0,
        duration_sec: int = 0,
        status: str = "completed",
        started_at: int | None = None,
        ended_at: int | None = None,
    ):
        """保存直播录制记录"""
        await self._db.execute("""
            INSERT INTO live_records
            (room_id, web_rid, title, nickname, sec_user_id,
             file_path, file_size, duration_sec, status, started_at, ended_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """, (
            room_id, web_rid, title, nickname, sec_user_id,
            file_path, file_size, duration_sec, status,
            started_at or int(time.time()), ended_at,
        ))
        await self._db.commit()
        await self._db.execute("PRAGMA wal_checkpoint(TRUNCATE)")

    async def get_live_records(
        self,
        *,
        limit: int = 20,
        offset: int = 0,
        status: str | None = None,
    ) -> list[dict]:
        """获取直播录制记录（分页）"""
        if status:
            sql = "SELECT * FROM live_records WHERE status = ? ORDER BY started_at DESC LIMIT ? OFFSET ?"
            return await self._fetch_all(sql, (status, limit, offset))
        else:
            sql = "SELECT * FROM live_records ORDER BY started_at DESC LIMIT ? OFFSET ?"
            return await self._fetch_all(sql, (limit, offset))

    async def get_live_record_count(self) -> int:
        """获取直播录制记录总数"""
        row = await self._fetch_one(
            "SELECT COUNT(*) as cnt FROM live_records WHERE status = 'completed'"
        )
        return row["cnt"] if row else 0
