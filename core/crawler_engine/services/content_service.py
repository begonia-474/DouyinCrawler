"""内容服务 — 相关推荐、评论、统计、定位"""

import logging

from core.crawler_engine.filter import UserPostFilter, PostCommentFilter, PostRelatedFilter
from core.utils import AwemeIdFetcher, SecUserIdFetcher

from .base import BaseService

logger = logging.getLogger(__name__)


class ContentService(BaseService):
    """内容相关：相关推荐、评论、统计、定位"""

    async def handle_related(self, url: str, count: int = 20, filter_gids: str = "", progress_callback=None) -> dict:
        """获取相关推荐视频（单页，前端控制分页）

        Args:
            url: 视频 URL
            count: 每页数量（默认 20）
            filter_gids: 已看过的 aweme_id 逗号列表（首页需包含当前 aweme_id 以排除自身）
            progress_callback: 进度回调（未使用，保留接口兼容）
        """
        aweme_id = await AwemeIdFetcher.get_aweme_id(url)
        if not aweme_id:
            return {"success": False, "error": "无法从 URL 提取 aweme_id"}

        # 首次请求：用当前 aweme_id 作为 filterGids 排除自身
        if not filter_gids:
            filter_gids = f"{aweme_id},"

        logger.info("[handle_related] 获取相关推荐: aweme_id=%s, filterGids=%s", aweme_id, filter_gids[:80])

        async with self._make_crawler() as crawler:
            data = await crawler.fetch_post_related(aweme_id, count=count, filter_gids=filter_gids)

        video_filter = PostRelatedFilter(data)
        videos = video_filter.get_video_list()

        # 本页返回的 aweme_id 列表，前端用于下次请求的 filterGids
        page_ids = [v.aweme_id for v in videos if v.aweme_id]

        logger.info("[handle_related] 获取 %d 条, has_more=%s", len(videos), video_filter.has_more)
        return {
            "success": True,
            "count": len(videos),
            "has_more": video_filter.has_more,
            "filter_gids": ",".join(page_ids),
            "videos": [v.to_dict() for v in videos],
        }

    async def handle_post_comment(self, url: str, cursor: int = 0, count: int = 20) -> dict:
        """获取视频评论"""
        aweme_id = await AwemeIdFetcher.get_aweme_id(url)
        if not aweme_id:
            return {"success": False, "error": "无法从 URL 提取 aweme_id"}

        async with self._make_crawler() as crawler:
            data = await crawler.fetch_post_comment(aweme_id, cursor, count)

        comment_filter = PostCommentFilter(data)
        return {
            "success": True,
            "comments": comment_filter.comments,
            "has_more": comment_filter.has_more,
            "cursor": comment_filter.cursor,
        }

    async def handle_post_comment_reply(self, url: str, comment_id: str, cursor: int = 0, count: int = 3) -> dict:
        """获取评论回复"""
        aweme_id = await AwemeIdFetcher.get_aweme_id(url)
        if not aweme_id:
            return {"success": False, "error": "无法从 URL 提取 aweme_id"}

        async with self._make_crawler() as crawler:
            data = await crawler.fetch_post_comment_reply(aweme_id, comment_id, cursor, count)

        comment_filter = PostCommentFilter(data)
        return {
            "success": True,
            "comments": comment_filter.comments,
            "has_more": comment_filter.has_more,
            "cursor": comment_filter.cursor,
        }

    async def handle_post_stats(self, url: str) -> dict:
        """获取作品统计"""
        aweme_id = await AwemeIdFetcher.get_aweme_id(url)
        if not aweme_id:
            return {"success": False, "error": "无法从 URL 提取 aweme_id"}

        async with self._make_crawler() as crawler:
            data = await crawler.fetch_post_stats(aweme_id)

        from core.crawler_engine.filter import PostStatsFilter
        stats_filter = PostStatsFilter(data)
        return {
            "success": True,
            "stats": stats_filter.to_dict(),
        }

    async def handle_locate_post(self, url: str, max_cursor: str, locate_item_cursor: str) -> dict:
        """定位作品 — 用于跳页定位"""
        sec_user_id = await SecUserIdFetcher.get_sec_user_id(url)
        if not sec_user_id:
            return {"success": False, "error": "无法从 URL 提取 sec_user_id"}

        from core.models import PostLocate
        params = PostLocate(
            sec_user_id=sec_user_id,
            max_cursor=max_cursor,
            locate_item_cursor=locate_item_cursor,
        ).model_dump()

        async with self._make_crawler() as crawler:
            data = await crawler.fetch_locate_post(params)

        video_filter = UserPostFilter(data)
        return {
            "success": True,
            "videos": video_filter.to_list(),
        }
