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
    PostCommentFilter, SuggestWordFilter, FollowingUserLiveFilter,
)
from core.utils import (
    AwemeIdFetcher, SecUserIdFetcher, MixIdFetcher, WebCastIdFetcher,
    sanitize_filename,
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
    # 单视频解析 (parse)
    # ============================================================

    async def handle_parse_video(self, url: str) -> dict:
        """只解析视频信息，不下载"""
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

        return {"success": True, "detail": detail.to_db_dict()}

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

        if detail.is_image_post and (detail.images or detail.images_video):
            # 图文下载（动图 + 图片）
            paths = []
            async with self._make_downloader(progress_callback) as dl:
                # 下载动图/实况
                for i, live_url in enumerate(detail.images_video):
                    if live_url:
                        path = await dl.download_live_image(live_url, save_dir, f"{filename}_live_{i + 1}")
                        paths.append(str(path))
                # 下载静态图片
                for i, img_url in enumerate(detail.images):
                    if img_url:
                        path = await dl.download_image(img_url, save_dir, f"{filename}_image_{i + 1}")
                        paths.append(str(path))
            return {"success": True, "type": "images", "paths": paths, "detail": detail.to_db_dict()}
        else:
            # 视频下载
            if not detail.video_url:
                return {"success": False, "error": "无法获取视频下载链接"}
            async with self._make_downloader(progress_callback) as dl:
                path = await dl.download_video(detail.video_url, save_dir, f"{filename}_video")
            return {"success": True, "type": "video", "path": str(path), "detail": detail.to_db_dict()}

    # ============================================================
    # 用户主页视频 (post)
    # ============================================================

    async def handle_user_post_list(self, url: str) -> dict:
        """获取用户主页视频列表（不下载）"""
        sec_user_id = await SecUserIdFetcher.get_sec_user_id(url)
        if not sec_user_id:
            return {"success": False, "error": "无法从 URL 提取 sec_user_id"}

        downloaded = 0
        max_cursor = 0
        all_details = []

        async with self._make_crawler() as crawler:
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
                await asyncio.sleep(self.timeout)

        return {"success": True, "videos": [d.to_dict() for d in all_details]}

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
        task_details = {}
        image_tasks = []  # 图文任务单独处理

        for detail in all_details:
            if detail.is_image_post and (detail.images or detail.images_video):
                # 图文作品：下载动图和图片
                image_tasks.append(detail)
            elif detail.video_url:
                # 视频作品：下载视频
                filename = format_filename(self.naming, detail.to_dict())
                download_tasks.append({
                    "url": detail.video_url,
                    "dir": str(save_dir),
                    "filename": f"{filename}_video",
                    "task_id": detail.aweme_id,
                })
                task_details[detail.aweme_id] = detail

        # 下载视频
        async with self._make_downloader(progress_callback) as dl:
            download_results = await dl.batch_download(download_tasks)

            # 下载图文（动图 + 图片）
            for detail in image_tasks:
                filename = format_filename(self.naming, detail.to_dict())

                # 下载动图/实况
                for i, live_url in enumerate(detail.images_video):
                    if live_url:
                        path = await dl.download_live_image(live_url, save_dir, f"{filename}_live_{i + 1}")
                        download_results.append({
                            "task_id": detail.aweme_id,
                            "path": path,
                        })

                # 下载静态图片
                for i, img_url in enumerate(detail.images):
                    if img_url:
                        path = await dl.download_image(img_url, save_dir, f"{filename}_image_{i + 1}")
                        download_results.append({
                            "task_id": detail.aweme_id,
                            "path": path,
                        })

                task_details[detail.aweme_id] = detail

        # 构建返回结果
        results = []
        for item in download_results:
            detail = task_details.get(item["task_id"])
            if detail:
                results.append({
                    "path": str(item["path"]),
                    "detail": detail.to_db_dict(),
                })
            else:
                results.append({
                    "path": str(item["path"]),
                    "detail": {},
                })

        return {"success": True, "count": len(results), "results": results}

    # ============================================================
    # 用户点赞 (like)
    # ============================================================

    async def handle_user_like_list(self, url: str) -> dict:
        """获取用户点赞视频列表（不下载）"""
        sec_user_id = await SecUserIdFetcher.get_sec_user_id(url)
        if not sec_user_id:
            return {"success": False, "error": "无法从 URL 提取 sec_user_id"}

        downloaded = 0
        max_cursor = 0
        all_details = []

        async with self._make_crawler() as crawler:
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
                await asyncio.sleep(self.timeout)

        return {"success": True, "videos": [d.to_dict() for d in all_details]}

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
                await asyncio.sleep(self.timeout)

        save_dir = self.download_path / "like" / nickname
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
                # 下载动图/实况
                for i, live_url in enumerate(detail.images_video):
                    if live_url:
                        path = await dl.download_live_image(live_url, save_dir, f"{filename}_live_{i + 1}")
                        download_results.append({"task_id": detail.aweme_id, "path": path})
                # 下载静态图片
                for i, img_url in enumerate(detail.images):
                    if img_url:
                        path = await dl.download_image(img_url, save_dir, f"{filename}_image_{i + 1}")
                        download_results.append({"task_id": detail.aweme_id, "path": path})
                task_details[detail.aweme_id] = detail

        # 构建返回结果
        results = []
        for item in download_results:
            detail = task_details.get(item["task_id"])
            if detail:
                results.append({"path": str(item["path"]), "detail": detail.to_db_dict()})
            else:
                results.append({"path": str(item["path"]), "detail": {}})

        return {"success": True, "count": len(results), "results": results}

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

    async def handle_collects_video_list(self, collects_id: str) -> dict:
        """获取收藏夹视频列表（不下载）"""
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

        return {"success": True, "videos": [d.to_dict() for d in all_details]}

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
                # 下载动图/实况
                for i, live_url in enumerate(detail.images_video):
                    if live_url:
                        path = await dl.download_live_image(live_url, save_dir, f"{filename}_live_{i + 1}")
                        download_results.append({"task_id": detail.aweme_id, "path": path})
                # 下载静态图片
                for i, img_url in enumerate(detail.images):
                    if img_url:
                        path = await dl.download_image(img_url, save_dir, f"{filename}_image_{i + 1}")
                        download_results.append({"task_id": detail.aweme_id, "path": path})
                task_details[detail.aweme_id] = detail

        # 构建返回结果
        results = []
        for item in download_results:
            detail = task_details.get(item["task_id"])
            if detail:
                results.append({"path": str(item["path"]), "detail": detail.to_db_dict()})
            else:
                results.append({"path": str(item["path"]), "detail": {}})

        return {"success": True, "count": len(results), "results": results}

    # ============================================================
    # 合集 (mix)
    # ============================================================

    async def handle_user_mix_list(self, url: str) -> dict:
        """获取合集视频列表（不下载）"""
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
        return {"success": True, "videos": [d.to_dict() for d in all_details], "detail": {"desc": mix_name}}

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
                # 下载动图/实况
                for i, live_url in enumerate(detail.images_video):
                    if live_url:
                        path = await dl.download_live_image(live_url, save_dir, f"{filename}_live_{i + 1}")
                        download_results.append({"task_id": detail.aweme_id, "path": path})
                # 下载静态图片
                for i, img_url in enumerate(detail.images):
                    if img_url:
                        path = await dl.download_image(img_url, save_dir, f"{filename}_image_{i + 1}")
                        download_results.append({"task_id": detail.aweme_id, "path": path})
                task_details[detail.aweme_id] = detail

        # 构建返回结果
        results = []
        for item in download_results:
            detail = task_details.get(item["task_id"])
            if detail:
                results.append({"path": str(item["path"]), "detail": detail.to_db_dict()})
            else:
                results.append({"path": str(item["path"]), "detail": {}})

        return {"success": True, "count": len(results), "results": results, "mix_name": mix_name}

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

        # 转换为前端期望的格式
        flv_dict = live_filter.flv_pull_url or {}
        m3u8_dict = live_filter.m3u8_pull_url or {}

        return {
            "success": True,
            "title": live_filter.live_title,
            "nickname": live_filter.nickname,
            "is_live": live_filter.is_live,
            "user_count": live_filter.user_count,
            "room_id": live_filter.room_id,
            "cover": live_filter.cover_url,
            "flv_urls": list(flv_dict.values()),
            "m3u8_urls": list(m3u8_dict.values()),
        }

    async def handle_live_record(self, url: str, task_id: str, progress_callback=None, stop_event=None) -> dict:
        """录制直播流"""
        webcast_id = await WebCastIdFetcher.get_webcast_id(url)
        if not webcast_id:
            return {"success": False, "error": "无法从 URL 提取直播 ID"}

        async with self._make_crawler() as crawler:
            data = await crawler.fetch_live_info(web_rid=webcast_id)

        live_filter = UserLiveFilter(data)
        if not live_filter.is_live:
            return {"success": False, "error": f"未在直播中 (status={live_filter.live_status})"}

        m3u8_urls = live_filter.m3u8_pull_url
        if not m3u8_urls:
            return {"success": False, "error": "未获取到 m3u8 拉流地址"}

        # 选择最高画质
        m3u8_url = m3u8_urls.get("FULL_HD1") or m3u8_urls.get("HD1") or next(iter(m3u8_urls.values()))
        if not m3u8_url:
            return {"success": False, "error": "拉流地址为空"}

        # 构建保存路径（对齐 f2 命名：{create}_{desc}_live.flv）
        nickname = sanitize_filename(live_filter.nickname or "unknown")
        save_dir = self.download_path / nickname
        create_str = time.strftime("%Y-%m-%d_%H-%M-%S")
        title = sanitize_filename(live_filter.live_title or "live")
        filename = f"{create_str}_{title}_live.flv"
        full_path = save_dir / filename

        started_at = int(time.time())
        async with self._make_downloader(progress_callback) as dl:
            if stop_event:
                dl._stop_event = stop_event
            await dl.download_m3u8_stream(task_id, m3u8_url, full_path)
        ended_at = int(time.time())

        file_size = full_path.stat().st_size if full_path.exists() else 0

        return {
            "success": True,
            "file": str(full_path),
            "room_id": live_filter.room_id,
            "web_rid": webcast_id,
            "title": live_filter.live_title,
            "nickname": live_filter.nickname,
            "file_size": file_size,
            "duration_sec": ended_at - started_at,
            "started_at": started_at,
            "ended_at": ended_at,
            "cover_url": live_filter.cover_url,
        }

    async def handle_following_live(self) -> dict:
        """获取关注用户中正在直播的列表"""
        try:
            async with self._make_crawler() as crawler:
                data = await crawler.fetch_following_user_live()
        except Exception as e:
            return {"success": False, "error": f"网络请求失败: {e}"}

        if data.get("status_code", -1) != 0:
            return {"success": False, "error": f"API 错误: {data.get('data', {}).get('message', 'unknown')}"}

        live_filter = FollowingUserLiveFilter(data)
        rooms = live_filter.live_rooms

        live_list = []
        for item in rooms:
            room = item.get("room", {})
            owner = room.get("owner", {})
            cover = room.get("cover", {})

            live_list.append({
                "web_rid": item.get("web_rid", ""),
                "room_id": room.get("id_str", ""),
                "title": room.get("title", ""),
                "nickname": owner.get("nickname", ""),
                "avatar": (owner.get("avatar_thumb", {}).get("url_list", [""]))[0],
                "cover": (cover.get("url_list", [""]))[0],
                "user_count": room.get("user_count", 0),
                "tag_name": item.get("tag_name", ""),
            })

        return {"success": True, "count": len(live_list), "lives": live_list}

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

        from core.filter import PostStatsFilter
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
            "music_list": music_filter.to_list(),
            "has_more": music_filter.has_more,
        }

    async def handle_download_music(self, play_url: str, title: str, author: str) -> dict:
        """下载单首音乐"""
        if not play_url:
            return {"success": False, "error": "音乐播放地址为空"}

        filename = sanitize_filename(f"{author} - {title}" if author else title)
        save_dir = self.download_path / "music"
        save_dir.mkdir(parents=True, exist_ok=True)

        async with self._make_downloader() as dl:
            path = await dl.download_music(play_url, save_dir, filename)

        return {"success": True, "path": str(path)}

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
