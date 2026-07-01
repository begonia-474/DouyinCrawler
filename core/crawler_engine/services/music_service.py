"""音乐服务 — 音乐收藏和下载"""

import logging

from core.crawler_engine.filter import UserMusicCollectionFilter
from core.utils import sanitize_filename

from .base import BaseService

logger = logging.getLogger(__name__)


class MusicService(BaseService):
    """音乐收藏和下载"""

    async def handle_user_music_collection(self, cursor: int = 0, count: int = 18) -> dict:
        """获取用户音乐收藏"""
        async with self._make_crawler() as crawler:
            data = await crawler.fetch_user_music_collection(cursor, count)
        music_filter = UserMusicCollectionFilter(data)
        return {
            "success": True,
            "music_list": music_filter.to_list(),
            "has_more": music_filter.has_more,
        }

    async def handle_download_music(self, play_url: str, title: str, author: str) -> dict:
        """下载单首音乐"""
        if not play_url:
            return {"success": False, "error": "音乐播放地址为空"}

        filename = sanitize_filename(f"{author} - {title}" if author else title) or "unknown"
        nickname = sanitize_filename(author) if author else "unknown"
        save_dir = self.download_path / self.app_name / "music" / nickname
        save_dir.mkdir(parents=True, exist_ok=True)

        async with self._make_downloader() as dl:
            path = await dl.download_music(play_url, save_dir, filename)

        return {"success": True, "path": str(path)}
