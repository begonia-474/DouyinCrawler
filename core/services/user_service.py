"""用户服务 — 用户主页、点赞、资料、关注/粉丝"""

import asyncio
import random
import logging

from core.downloader import format_filename
from core.filter import UserPostFilter, UserProfileFilter
from core import db
from core.utils import (
    SecUserIdFetcher, filter_by_date_interval,
)

from core.services.base import BaseService, run_concurrent

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

    async def handle_user_post(self, url: str, progress_callback=None) -> dict:
        """下载用户主页视频"""
        sec_user_id = await SecUserIdFetcher.get_sec_user_id(url)
        if not sec_user_id:
            return {"success": False, "error": "无法从 URL 提取 sec_user_id"}

        downloaded = 0
        max_cursor = 0
        all_details = []

        async with self._make_crawler() as crawler:
            profile_data = await crawler.fetch_user_profile(sec_user_id)
            profile = UserProfileFilter(profile_data)
            nickname = profile.nickname or "unknown"
            db.save_user_info(profile.to_dict())

            while downloaded < self.max_counts:
                current_request_size = min(self.page_counts, self.max_counts - downloaded)
                data = await crawler.fetch_user_post(sec_user_id, max_cursor, current_request_size)
                video_filter = UserPostFilter(data)

                for detail in video_filter.get_video_list():
                    if downloaded >= self.max_counts:
                        break
                    if detail.is_prohibited:
                        continue
                    all_details.append(detail)
                    downloaded += 1

                if not video_filter.has_more:
                    break
                max_cursor = video_filter.max_cursor
                await asyncio.sleep(self.timeout + random.uniform(-2, 2))

        if self.interval and self.interval != "all":
            all_details = filter_by_date_interval(all_details, self.interval, "create_time")

        user_dir = self.download_path / self.app_name / "post" / nickname
        user_dir.mkdir(parents=True, exist_ok=True)

        return await self._batch_download(all_details, user_dir)

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

    async def handle_user_like(self, url: str, progress_callback=None) -> dict:
        """下载用户点赞视频"""
        sec_user_id = await SecUserIdFetcher.get_sec_user_id(url)
        if not sec_user_id:
            return {"success": False, "error": "无法从 URL 提取 sec_user_id"}

        downloaded = 0
        max_cursor = 0
        all_details = []

        async with self._make_crawler() as crawler:
            profile_data = await crawler.fetch_user_profile(sec_user_id)
            profile = UserProfileFilter(profile_data)
            nickname = profile.nickname or "unknown"
            db.save_user_info(profile.to_dict())

            while downloaded < self.max_counts:
                current_request_size = min(self.page_counts, self.max_counts - downloaded)
                data = await crawler.fetch_user_favorite(sec_user_id, max_cursor, current_request_size)
                if not data or not data.get("aweme_list"):
                    break
                video_filter = UserPostFilter(data)

                for detail in video_filter.get_video_list():
                    if downloaded >= self.max_counts:
                        break
                    all_details.append(detail)
                    downloaded += 1

                if not video_filter.has_more:
                    break
                max_cursor = video_filter.max_cursor
                await asyncio.sleep(self.timeout + random.uniform(-2, 2))

        if self.interval and self.interval != "all":
            all_details = filter_by_date_interval(all_details, self.interval, "create_time")

        save_dir = self.download_path / self.app_name / "like" / nickname
        save_dir.mkdir(parents=True, exist_ok=True)

        return await self._batch_download(all_details, save_dir)

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

        from core.filter import UserFollowingFilter
        f = UserFollowingFilter(data)
        return {"success": True, "followings": f.followings, "has_more": f.has_more}

    async def handle_user_follower(self, url: str, offset: int = 0, count: int = 20) -> dict:
        """获取粉丝列表"""
        sec_user_id = await SecUserIdFetcher.get_sec_user_id(url)
        if not sec_user_id:
            return {"success": False, "error": "无法从 URL 提取 sec_user_id"}

        async with self._make_crawler() as crawler:
            data = await crawler.fetch_user_follower(sec_user_id, offset, count)

        from core.filter import UserFollowerFilter
        f = UserFollowerFilter(data)
        return {"success": True, "followers": f.followers, "has_more": f.has_more}

    # ============================================================
    # 共享批量下载逻辑
    # ============================================================

    async def _batch_download(self, all_details: list, save_dir) -> dict:
        """批量下载视频+图文+附属文件（post/like/collects 共用）"""
        from core.downloader import format_filename

        download_tasks = []
        task_details = {}
        image_tasks = []

        for detail in all_details:
            filename = format_filename(self.naming, detail.to_dict())
            dir_path = save_dir / filename if self.folderize else save_dir
            dir_path.mkdir(parents=True, exist_ok=True)

            if detail.is_image_post and (detail.images or detail.images_video):
                image_tasks.append((detail, dir_path, filename))
            elif detail.video_url:
                download_tasks.append({
                    "url": detail.video_url,
                    "dir": str(dir_path),
                    "filename": f"{filename}_video",
                    "task_id": detail.aweme_id,
                })
                task_details[detail.aweme_id] = detail

        async with self._make_downloader() as dl:
            download_results = await dl.batch_download(download_tasks)

            # 附属文件并发下载
            accessory_tasks = []
            for detail in all_details:
                filename = format_filename(self.naming, detail.to_dict())
                dir_path = save_dir / filename if self.folderize else save_dir
                if self.music and detail.music_url:
                    accessory_tasks.append(dl.download_music(detail.music_url, dir_path, filename))
                if self.cover and detail.cover_url:
                    accessory_tasks.append(dl.download_cover(detail.cover_url, dir_path, filename))
                if self.desc and detail.desc:
                    accessory_tasks.append(dl.download_desc(detail.desc, dir_path, filename))
            if accessory_tasks:
                await run_concurrent(accessory_tasks)

            for detail, dir_path, filename in image_tasks:
                for i, live_url in enumerate(detail.images_video):
                    if live_url:
                        path = await dl.download_live_image(live_url, dir_path, f"{filename}_live_{i + 1}")
                        download_results.append({"task_id": detail.aweme_id, "path": path})
                for i, img_url in enumerate(detail.images):
                    if img_url:
                        path = await dl.download_image(img_url, dir_path, f"{filename}_image_{i + 1}")
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
