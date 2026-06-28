"""收藏服务 — 收藏夹和收藏列表"""

import logging

from core.filter import UserPostFilter, UserCollectsFilter

from .base import BaseService

logger = logging.getLogger(__name__)


class CollectionService(BaseService):
    """收藏夹和收藏列表"""

    async def handle_user_collection(self, progress_callback=None) -> dict:
        """下载用户收藏视频"""
        async with self._make_crawler() as crawler:
            all_details = await self._paginate_and_collect(
                lambda c, n: crawler.fetch_user_collection(c, n),
                skip_prohibited=False,
            )

        nickname = all_details[0].author_nickname if all_details else "me"
        save_dir = self.download_path / self.app_name / "collection" / nickname
        save_dir.mkdir(parents=True, exist_ok=True)

        return await self._batch_download(all_details, save_dir, progress_callback=progress_callback)

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

    async def handle_collects_video(self, collects_id: str, progress_callback=None) -> dict:
        """下载收藏夹中的视频"""
        async with self._make_crawler() as crawler:
            all_details = await self._paginate_and_collect(
                lambda c, n: crawler.fetch_user_collects_video(collects_id, c, n),
                skip_prohibited=False,
            )

        nickname = all_details[0].author_nickname if all_details else collects_id
        save_dir = self.download_path / self.app_name / "collects" / nickname
        save_dir.mkdir(parents=True, exist_ok=True)

        return await self._batch_download(all_details, save_dir, progress_callback=progress_callback)
