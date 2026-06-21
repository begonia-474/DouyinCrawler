"""业务处理器 — 协调爬虫、过滤器、下载器"""

import asyncio
import time
from pathlib import Path

from core.crawler import DouyinCrawler
from core.downloader import Downloader, format_filename
from core.filter import (
    PostDetailFilter, UserPostFilter, UserProfileFilter,
    UserCollectsFilter, UserMusicCollectionFilter,
    UserLiveFilter, UserLive2Filter, UserLiveStatusFilter,
    PostCommentFilter, SuggestWordFilter,
)
from core.utils import (
    AwemeIdFetcher, SecUserIdFetcher, MixIdFetcher, WebCastIdFetcher,
)


class DouyinHandler:
    """抖音业务处理器"""

    def __init__(self, cookie: str, download_path: str = "Download",
                 naming: str = "{create}_{desc}", max_counts: int = 0,
                 page_counts: int = 20, timeout: int = 5,
                 encryption: str = "ab", proxies: dict = None):
        self.cookie = cookie
        self.download_path = Path(download_path)
        self.naming = naming
        self.max_counts = max_counts or float("inf")
        self.page_counts = page_counts
        self.timeout = timeout
        self.encryption = encryption
        self.proxies = proxies

    def _make_crawler(self) -> DouyinCrawler:
        return DouyinCrawler(self.cookie, self.proxies, self.encryption)

    def _make_downloader(self, progress_callback=None) -> Downloader:
        return Downloader(self.cookie, progress_callback=progress_callback)

    # ============================================================
    # 单视频下载 (one)
    # ============================================================

    async def handle_one_video(self, url: str, progress_callback=None) -> dict:
        """下载单个视频"""
        aweme_id = await AwemeIdFetcher.get_aweme_id(url)
        if not aweme_id:
            return {"success": False, "error": "无法从 URL 提取 aweme_id"}

        async with self._make_crawler() as crawler:
            data = await crawler.fetch_post_detail(aweme_id)

        if data.get("status_code", -1) != 0:
            return {"success": False, "error": f"API 错误: {data.get('status_msg', 'unknown')}"}

        detail = PostDetailFilter(data)
        if detail.is_prohibited:
            return {"success": False, "error": "视频侵权不可用"}

        # 下载视频
        filename = format_filename(self.naming, detail.to_dict())
        save_dir = self.download_path / "one" / detail.author_nickname
        save_dir.mkdir(parents=True, exist_ok=True)

        if detail.is_image_post and detail.images:
            # 图集下载
            paths = []
            async with self._make_downloader(progress_callback) as dl:
                for i, img_url in enumerate(detail.images):
                    if img_url:
                        path = await dl.download_image(img_url, save_dir, f"{filename}_{i}")
                        paths.append(str(path))
            return {"success": True, "type": "images", "paths": paths, "detail": detail.to_dict()}
        else:
            # 视频下载
            if not detail.video_url:
                return {"success": False, "error": "无法获取视频下载链接"}
            async with self._make_downloader(progress_callback) as dl:
                path = await dl.download_video(detail.video_url, save_dir, filename)
            return {"success": True, "type": "video", "path": str(path), "detail": detail.to_dict()}

    # ============================================================
    # 用户主页视频 (post)
    # ============================================================

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

            while downloaded < self.max_counts:
                # 动态请求量，和 f2 一致
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

                # 避免请求过于频繁，和 f2 一致
                await asyncio.sleep(self.timeout)

        # 批量下载
        save_dir = self.download_path / "post" / nickname
        save_dir.mkdir(parents=True, exist_ok=True)

        download_tasks = []
        for detail in all_details:
            if detail.video_url:
                filename = format_filename(self.naming, detail.to_dict())
                download_tasks.append({
                    "url": detail.video_url,
                    "dir": str(save_dir),
                    "filename": filename,
                    "task_id": detail.aweme_id,
                })

        async with self._make_downloader(progress_callback) as dl:
            paths = await dl.batch_download(download_tasks)

        return {"success": True, "count": len(paths), "paths": [str(p) for p in paths]}

    # ============================================================
    # 用户点赞 (like)
    # ============================================================

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

            while downloaded < self.max_counts:
                current_request_size = min(self.page_counts, self.max_counts - downloaded)
                data = await crawler.fetch_user_favorite(sec_user_id, max_cursor, current_request_size)
                video_filter = UserPostFilter(data)

                for detail in video_filter.get_video_list():
                    if downloaded >= self.max_counts:
                        break
                    all_details.append(detail)
                    downloaded += 1

                if not video_filter.has_more:
                    break
                max_cursor = video_filter.max_cursor
                await asyncio.sleep(self.timeout)

        save_dir = self.download_path / "like" / nickname
        save_dir.mkdir(parents=True, exist_ok=True)

        download_tasks = []
        for detail in all_details:
            if detail.video_url:
                filename = format_filename(self.naming, detail.to_dict())
                download_tasks.append({"url": detail.video_url, "dir": str(save_dir), "filename": filename, "task_id": detail.aweme_id})

        async with self._make_downloader(progress_callback) as dl:
            paths = await dl.batch_download(download_tasks)

        return {"success": True, "count": len(paths)}

    # ============================================================
    # 用户收藏 (collection)
    # ============================================================

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
                await asyncio.sleep(self.timeout)

        save_dir = self.download_path / "collection"
        save_dir.mkdir(parents=True, exist_ok=True)

        download_tasks = []
        for detail in all_details:
            if detail.video_url:
                filename = format_filename(self.naming, detail.to_dict())
                download_tasks.append({"url": detail.video_url, "dir": str(save_dir), "filename": filename, "task_id": detail.aweme_id})

        async with self._make_downloader(progress_callback) as dl:
            paths = await dl.batch_download(download_tasks)

        return {"success": True, "count": len(paths)}

    # ============================================================
    # 收藏夹 (collects)
    # ============================================================

    async def handle_user_collects(self, progress_callback=None) -> dict:
        """获取收藏夹列表"""
        async with self._make_crawler() as crawler:
            data = await crawler.fetch_user_collects()
            collects_filter = UserCollectsFilter(data)
            return {"success": True, "collects": collects_filter.to_list()}

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
                await asyncio.sleep(self.timeout)

        save_dir = self.download_path / "collects" / collects_id
        save_dir.mkdir(parents=True, exist_ok=True)

        download_tasks = []
        for detail in all_details:
            if detail.video_url:
                filename = format_filename(self.naming, detail.to_dict())
                download_tasks.append({"url": detail.video_url, "dir": str(save_dir), "filename": filename, "task_id": detail.aweme_id})

        async with self._make_downloader(progress_callback) as dl:
            paths = await dl.batch_download(download_tasks)

        return {"success": True, "count": len(paths)}

    # ============================================================
    # 合集 (mix)
    # ============================================================

    async def handle_user_mix(self, url: str, progress_callback=None) -> dict:
        """下载合集视频"""
        mix_id = await MixIdFetcher.get_mix_id(url)
        if not mix_id:
            return {"success": False, "error": "无法从 URL 提取 mix_id"}

        downloaded = 0
        cursor = 0
        all_details = []

        async with self._make_crawler() as crawler:
            while downloaded < self.max_counts:
                current_request_size = min(self.page_counts, self.max_counts - downloaded)
                data = await crawler.fetch_mix_aweme(mix_id, cursor, current_request_size)
                video_filter = UserPostFilter(data)

                for detail in video_filter.get_video_list():
                    if downloaded >= self.max_counts:
                        break
                    all_details.append(detail)
                    downloaded += 1

                if not video_filter.has_more:
                    break
                cursor = video_filter.max_cursor
                await asyncio.sleep(self.timeout)

        mix_name = all_details[0].mix_name if all_details else mix_id
        save_dir = self.download_path / "mix" / mix_name
        save_dir.mkdir(parents=True, exist_ok=True)

        download_tasks = []
        for detail in all_details:
            if detail.video_url:
                filename = format_filename(self.naming, detail.to_dict())
                download_tasks.append({"url": detail.video_url, "dir": str(save_dir), "filename": filename, "task_id": detail.aweme_id})

        async with self._make_downloader(progress_callback) as dl:
            paths = await dl.batch_download(download_tasks)

        return {"success": True, "count": len(paths), "mix_name": mix_name}

    # ============================================================
    # 直播 (live)
    # ============================================================

    async def handle_user_live(self, url: str, progress_callback=None) -> dict:
        """获取直播信息 / 录制直播"""
        webcast_id = await WebCastIdFetcher.get_webcast_id(url)
        if not webcast_id:
            return {"success": False, "error": "无法从 URL 提取直播 ID"}

        async with self._make_crawler() as crawler:
            data = await crawler.fetch_live_info(web_rid=webcast_id)

        live_filter = UserLiveFilter(data)
        if not live_filter.is_live:
            return {"success": False, "error": f"未在直播中 (status={live_filter.live_status})"}

        return {
            "success": True,
            "live_info": live_filter.to_dict(),
            "m3u8_urls": live_filter.m3u8_pull_url,
            "flv_urls": live_filter.flv_pull_url,
        }

    # ============================================================
    # 相关推荐 (related)
    # ============================================================

    async def handle_related(self, url: str, progress_callback=None) -> dict:
        """获取相关推荐视频"""
        aweme_id = await AwemeIdFetcher.get_aweme_id(url)
        if not aweme_id:
            return {"success": False, "error": "无法从 URL 提取 aweme_id"}

        async with self._make_crawler() as crawler:
            data = await crawler.fetch_post_related(aweme_id)

        video_filter = UserPostFilter(data)
        videos = video_filter.get_video_list()
        return {"success": True, "count": len(videos), "videos": [v.to_dict() for v in videos]}

    # ============================================================
    # 评论 (comment)
    # ============================================================

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

    # ============================================================
    # 搜索 (search)
    # ============================================================

    async def handle_search(self, keyword: str, offset: int = 0, count: int = 10) -> dict:
        """搜索视频"""
        async with self._make_crawler() as crawler:
            data = await crawler.fetch_post_search(keyword, offset, count)

        video_filter = UserPostFilter(data)
        return {"success": True, "count": len(video_filter.aweme_list), "videos": video_filter.to_list()}

    # ============================================================
    # Feed (feed/friend)
    # ============================================================

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

    # ============================================================
    # 音乐收藏 (music)
    # ============================================================

    async def handle_user_music_collection(self, cursor: int = 0, count: int = 18) -> dict:
        """获取用户音乐收藏"""
        async with self._make_crawler() as crawler:
            data = await crawler.fetch_user_music_collection(cursor, count)
        music_filter = UserMusicCollectionFilter(data)
        return {
            "success": True,
            "music_list": music_filter.music_list,
            "has_more": music_filter.has_more,
        }

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
    # 用户资料 (profile)
    # ============================================================

    async def handle_user_profile(self, url: str) -> dict:
        """获取用户资料"""
        sec_user_id = await SecUserIdFetcher.get_sec_user_id(url)
        if not sec_user_id:
            return {"success": False, "error": "无法从 URL 提取 sec_user_id"}

        async with self._make_crawler() as crawler:
            data = await crawler.fetch_user_profile(sec_user_id)

        profile = UserProfileFilter(data)
        return {"success": True, "profile": profile.to_dict()}
