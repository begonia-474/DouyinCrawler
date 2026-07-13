"""收藏服务 — 收藏夹和收藏列表"""

import logging

from core.crawler_engine.filter import UserPostFilter, UserCollectsFilter

from .base import BaseService

logger = logging.getLogger(__name__)


class CollectionService(BaseService):
    """收藏夹和收藏列表"""

    async def handle_user_collects(self, progress_callback=None) -> dict:
        """获取收藏夹列表"""
        async with self._make_crawler() as crawler:
            data = await crawler.fetch_user_collects()
            collects_filter = UserCollectsFilter(data)
            return {"success": True, "collects": collects_filter.to_list()}

    async def handle_collects_video_list(self, collects_id: str, cursor: int = 0, count: int = 20) -> dict:
        """获取收藏夹视频列表（单页，用于分页预览）"""
        async with self._make_crawler() as crawler:
            data = await crawler.fetch_user_collects_video(collects_id, cursor, count)
            video_filter = UserPostFilter(data)
            videos = [d.to_dict() for d in video_filter.get_video_list()]

        return {
            "success": True,
            "videos": videos,
            "has_more": bool(video_filter.has_more),
            "next_cursor": video_filter.max_cursor,
        }
