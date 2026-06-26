"""Feed 服务 — 推荐、关注、好友 Feed + 搜索"""

import logging

from core.filter import UserPostFilter

from core.services.base import BaseService

logger = logging.getLogger(__name__)


class FeedService(BaseService):
    """信息流和搜索"""

    async def handle_tab_feed(self, count: int = 10) -> dict:
        """获取首页推荐"""
        async with self._make_crawler() as crawler:
            data = await crawler.fetch_tab_feed(count)
        video_filter = UserPostFilter(data)
        return {"success": True, "videos": video_filter.to_list()}

    async def handle_follow_feed(self, cursor: int = 0, count: int = 10) -> dict:
        """获取关注 feed"""
        async with self._make_crawler() as crawler:
            data = await crawler.fetch_follow_feed(cursor, count)
        video_filter = UserPostFilter(data)
        return {"success": True, "videos": video_filter.to_list()}

    async def handle_friend_feed(self, cursor: int = 0, count: int = 10) -> dict:
        """获取好友 feed"""
        async with self._make_crawler() as crawler:
            data = await crawler.fetch_friend_feed(cursor, count)
        video_filter = UserPostFilter(data)
        return {"success": True, "videos": video_filter.to_list()}

    async def handle_search(self, keyword: str, offset: int = 0, count: int = 10) -> dict:
        """搜索视频"""
        async with self._make_crawler() as crawler:
            data = await crawler.fetch_post_search(keyword, offset, count)

        video_filter = UserPostFilter(data)
        return {"success": True, "count": len(video_filter.aweme_list), "videos": video_filter.to_list()}
