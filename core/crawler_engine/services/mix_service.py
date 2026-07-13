"""合集服务 — 合集列表"""

import logging

from core.crawler_engine.filter import UserPostFilter
from core.utils import MixIdFetcher

from .base import BaseService

logger = logging.getLogger(__name__)


class MixService(BaseService):
    """合集列表"""

    async def handle_user_mix_list(self, url: str, cursor: int = 0, count: int = 20) -> dict:
        """获取合集视频列表（单页，用于分页预览）"""
        mix_id = await MixIdFetcher.get_mix_id(url)
        if not mix_id:
            return {"success": False, "error": "无法从 URL 提取 mix_id"}

        async with self._make_crawler() as crawler:
            data = await crawler.fetch_mix_aweme(mix_id, cursor, count)
            video_filter = UserPostFilter(data)
            videos = [d.to_dict() for d in video_filter.get_video_list()]

        mix_name = videos[0].get("mix_name", mix_id) if videos else mix_id
        return {
            "success": True,
            "videos": videos,
            "detail": {"desc": mix_name},
            "has_more": bool(video_filter.has_more),
            "next_cursor": video_filter.max_cursor,
        }
