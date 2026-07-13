"""用户服务 — 用户主页、点赞、资料、关注/粉丝"""

import logging

from core.crawler_engine.filter import UserPostFilter, UserProfileFilter
from core.utils import SecUserIdFetcher

from .base import BaseService

logger = logging.getLogger(__name__)


class UserService(BaseService):
    """用户相关业务：主页视频、点赞、资料、关注/粉丝列表"""

    # ============================================================
    # 用户主页视频 (post)
    # ============================================================

    async def handle_user_post_list(self, url: str, cursor: int = 0, count: int = 20) -> dict:
        """获取用户主页视频列表（单页，用于分页预览）"""
        sec_user_id = await SecUserIdFetcher.get_sec_user_id(url)
        if not sec_user_id:
            return {"success": False, "error": "无法从 URL 提取 sec_user_id"}

        async with self._make_crawler() as crawler:
            data = await crawler.fetch_user_post(sec_user_id, cursor, count)
            video_filter = UserPostFilter(data)
            videos = [d.to_dict() for d in video_filter.get_video_list() if not d.is_prohibited]

        return {
            "success": True,
            "videos": videos,
            "has_more": bool(video_filter.has_more),
            "next_cursor": video_filter.max_cursor,
        }
    # ============================================================
    # 用户点赞 (like)
    # ============================================================

    async def handle_user_like_list(self, url: str, cursor: int = 0, count: int = 20) -> dict:
        """获取用户点赞视频列表（单页，用于分页预览）"""
        sec_user_id = await SecUserIdFetcher.get_sec_user_id(url)
        if not sec_user_id:
            return {"success": False, "error": "无法从 URL 提取 sec_user_id"}

        async with self._make_crawler() as crawler:
            data = await crawler.fetch_user_favorite(sec_user_id, cursor, count)
            if not data or not data.get("aweme_list"):
                return {"success": True, "videos": [], "has_more": False, "next_cursor": 0}
            video_filter = UserPostFilter(data)
            videos = [d.to_dict() for d in video_filter.get_video_list()]

        return {
            "success": True,
            "videos": videos,
            "has_more": bool(video_filter.has_more),
            "next_cursor": video_filter.max_cursor,
        }
    # ============================================================
    # 用户资料 (profile)
    # ============================================================

    async def handle_user_profile(self, url: str) -> dict:
        """获取用户资料"""
        logger.info("[handle_user_profile] url=%s", url[:80])
        sec_user_id = await SecUserIdFetcher.get_sec_user_id(url)
        if not sec_user_id:
            logger.warning("[handle_user_profile] 无法提取 sec_user_id")
            return {"success": False, "error": "无法从 URL 提取 sec_user_id"}

        logger.info("[handle_user_profile] sec_user_id=%s", sec_user_id[:30])
        async with self._make_crawler() as crawler:
            data = await crawler.fetch_user_profile(sec_user_id)

        logger.info("[handle_user_profile] API status_code=%s", data.get("status_code", "N/A"))
        profile = UserProfileFilter(data)
        result = {"success": True, "profile": profile.to_dict()}
        logger.info("[handle_user_profile] 返回结果 keys=%s", list(result.keys()))
        return result

    # ============================================================
    # 关注/粉丝 (following/follower)
    # ============================================================

    async def handle_user_following(self, url: str, offset: int = 0, count: int = 20) -> dict:
        """获取关注列表"""
        sec_user_id = await SecUserIdFetcher.get_sec_user_id(url)
        if not sec_user_id:
            return {"success": False, "error": "无法从 URL 提取 sec_user_id"}

        async with self._make_crawler() as crawler:
            data = await crawler.fetch_user_following(sec_user_id, offset, count)

        from core.crawler_engine.filter import UserFollowingFilter
        f = UserFollowingFilter(data)
        return {"success": True, "followings": f.followings, "has_more": f.has_more}

    async def handle_user_follower(self, url: str, offset: int = 0, count: int = 20) -> dict:
        """获取粉丝列表"""
        sec_user_id = await SecUserIdFetcher.get_sec_user_id(url)
        if not sec_user_id:
            return {"success": False, "error": "无法从 URL 提取 sec_user_id"}

        async with self._make_crawler() as crawler:
            data = await crawler.fetch_user_follower(sec_user_id, offset, count)

        from core.crawler_engine.filter import UserFollowerFilter
        f = UserFollowerFilter(data)
        return {"success": True, "followers": f.followers, "has_more": f.has_more}
