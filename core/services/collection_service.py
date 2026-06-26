"""收藏服务 — 收藏夹和收藏列表"""

import asyncio
import random
import logging

from core.downloader import format_filename
from core.filter import UserPostFilter, UserCollectsFilter

from core.services.base import BaseService

logger = logging.getLogger(__name__)


class CollectionService(BaseService):
    """收藏夹和收藏列表"""

    async def handle_user_collection(self, progress_callback=None) -> dict:
        """下载用户收藏视频"""
        downloaded = 0
        cursor = 0
        all_details = []

        async with self._make_crawler() as crawler:
            while downloaded < self.max_counts:
                current_request_size = min(self.page_counts, self.max_counts - downloaded)
                data = await crawler.fetch_user_collection(cursor, current_request_size)
                video_filter = UserPostFilter(data)

                for detail in video_filter.get_video_list():
                    if downloaded >= self.max_counts:
                        break
                    all_details.append(detail)
                    downloaded += 1

                if not video_filter.has_more:
                    break
                cursor = video_filter.max_cursor
                await asyncio.sleep(self.timeout + random.uniform(-2, 2))

        nickname = all_details[0].author_nickname if all_details else "me"
        save_dir = self.download_path / self.app_name / "collection" / nickname
        save_dir.mkdir(parents=True, exist_ok=True)

        download_tasks = []
        for detail in all_details:
            if detail.video_url:
                filename = format_filename(self.naming, detail.to_dict())
                download_tasks.append({"url": detail.video_url, "dir": str(save_dir), "filename": filename, "task_id": detail.aweme_id})

        async with self._make_downloader(progress_callback) as dl:
            paths = await dl.batch_download(download_tasks)

        return {"success": True, "count": len(paths)}

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
        downloaded = 0
        cursor = 0
        all_details = []

        async with self._make_crawler() as crawler:
            while downloaded < self.max_counts:
                current_request_size = min(self.page_counts, self.max_counts - downloaded)
                data = await crawler.fetch_user_collects_video(collects_id, cursor, current_request_size)
                video_filter = UserPostFilter(data)

                for detail in video_filter.get_video_list():
                    if downloaded >= self.max_counts:
                        break
                    all_details.append(detail)
                    downloaded += 1

                if not video_filter.has_more:
                    break
                cursor = video_filter.max_cursor
                await asyncio.sleep(self.timeout + random.uniform(-2, 2))

        nickname = all_details[0].author_nickname if all_details else collects_id
        save_dir = self.download_path / self.app_name / "collects" / nickname
        save_dir.mkdir(parents=True, exist_ok=True)

        download_tasks = []
        task_details = {}
        image_tasks = []

        for detail in all_details:
            if detail.is_image_post and (detail.images or detail.images_video):
                image_tasks.append(detail)
            elif detail.video_url:
                filename = format_filename(self.naming, detail.to_dict())
                download_tasks.append({"url": detail.video_url, "dir": str(save_dir), "filename": f"{filename}_video", "task_id": detail.aweme_id})
                task_details[detail.aweme_id] = detail

        async with self._make_downloader(progress_callback) as dl:
            download_results = await dl.batch_download(download_tasks)

            for detail in image_tasks:
                filename = format_filename(self.naming, detail.to_dict())
                for i, live_url in enumerate(detail.images_video):
                    if live_url:
                        path = await dl.download_live_image(live_url, save_dir, f"{filename}_live_{i + 1}")
                        download_results.append({"task_id": detail.aweme_id, "path": path})
                for i, img_url in enumerate(detail.images):
                    if img_url:
                        path = await dl.download_image(img_url, save_dir, f"{filename}_image_{i + 1}")
                        download_results.append({"task_id": detail.aweme_id, "path": path})
                task_details[detail.aweme_id] = detail

        results = []
        for item in download_results:
            detail = task_details.get(item["task_id"])
            if detail:
                results.append({"path": str(item["path"]), "detail": detail.to_db_dict()})
            else:
                results.append({"path": str(item["path"]), "detail": {}})

        return {"success": True, "count": len(results), "results": results}
