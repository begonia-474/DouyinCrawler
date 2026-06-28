"""合集服务 — 合集列表和下载"""

import logging

from core.filter import UserPostFilter, UserProfileFilter
from core.utils import MixIdFetcher

from .base import BaseService

logger = logging.getLogger(__name__)


class MixService(BaseService):
    """合集列表和下载"""

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

    async def handle_user_mix(self, url: str, progress_callback=None) -> dict:
        """下载合集视频"""
        mix_id = await MixIdFetcher.get_mix_id(url)
        if not mix_id:
            return {"success": False, "error": "无法从 URL 提取 mix_id"}

        user_profile = None

        async with self._make_crawler() as crawler:
            all_details = await self._paginate_and_collect(
                lambda c, n: crawler.fetch_mix_aweme(mix_id, c, n),
                skip_prohibited=False,
            )

            if all_details and all_details[0].author_sec_uid:
                try:
                    profile_data = await crawler.fetch_user_profile(all_details[0].author_sec_uid)
                    user_profile = UserProfileFilter(profile_data).to_dict()
                except Exception as e:
                    logger.warning("[handle_user_mix] 获取合集作者资料失败: %s", e)

        mix_name = all_details[0].mix_name if all_details else mix_id
        nickname = all_details[0].author_nickname if all_details else mix_id
        save_dir = self.download_path / self.app_name / "mix" / nickname
        save_dir.mkdir(parents=True, exist_ok=True)

        result = await self._batch_download(all_details, save_dir, progress_callback=progress_callback)

        return {
            **result,
            "mix_name": mix_name,
            "user_profile": user_profile,
        }
