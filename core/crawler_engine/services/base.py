"""服务基类 — 共享配置和工厂方法"""

import asyncio
import logging

from core.crawler import DouyinCrawler
from core.downloader import Downloader
from core.models import ServiceConfig

logger = logging.getLogger(__name__)


async def run_concurrent(tasks: list, limit: int = 3) -> list:
    """并发运行协程列表，限制最大并发数。返回结果列表（顺序对应输入）。"""
    sem = asyncio.Semaphore(limit)

    async def _wrapper(coro):
        async with sem:
            return await coro

    return await asyncio.gather(*[_wrapper(t) for t in tasks])


class BaseService:
    """所有业务服务的基类，提供共享配置和 crawler/downloader 工厂方法"""

    def __init__(self, config: ServiceConfig):
        self.config = config
        # 展开为实例属性，保持 self.xxx 访问模式兼容
        self.cookie = config.cookie
        self.download_path = config.download_path
        self.naming = config.naming
        self.max_counts = config.max_counts
        self.page_counts = config.page_counts
        self.timeout = config.timeout
        self.encryption = config.encryption
        self.proxies = config.proxies
        self.app_name = config.app_name
        self.folderize = config.folderize
        self.music = config.music
        self.cover = config.cover
        self.desc = config.desc
        self.interval = config.interval
        self.max_connections = config.max_connections
        self.max_retries = config.max_retries
        self.max_tasks = config.max_tasks

    def _make_crawler(self) -> DouyinCrawler:
        return DouyinCrawler(self.cookie, self.proxies, self.encryption, self.max_retries,
                             max_connections=self.max_connections, timeout=self.timeout)

    def _make_downloader(self, progress_callback=None) -> Downloader:
        return Downloader(self.cookie, max_connections=self.max_connections,
                          max_concurrent=self.max_tasks, max_retries=self.max_retries,
                          timeout=self.timeout, progress_callback=progress_callback)

    # ============================================================
    # 共享分页收集器（消除 user/collection/mix 五处分页循环重复）
    # ============================================================

    async def _paginate_and_collect(self, fetch_fn, *,
                                     skip_prohibited: bool = True,
                                     cursor: int = 0) -> list:
        """通用分页收集器 — 循环拉取直到达到 max_counts 或无更多数据

        消除 user_service (post/like)、collection_service (collection/collects_video)、
        mix_service 五处相同的 while-downloaded 分页循环。

        Args:
            fetch_fn: async callable(cursor, count) -> raw API response dict
            skip_prohibited: 是否跳过 is_prohibited 作品（用户主页跳过，点赞不跳过）
            cursor: 初始游标

        Returns:
            PostDetail 对象列表
        """
        from core.filter import UserPostFilter
        import random

        downloaded = 0
        all_details = []

        while downloaded < self.max_counts:
            current_request_size = min(self.page_counts, self.max_counts - downloaded)
            data = await fetch_fn(cursor, current_request_size)

            if not data:
                break

            video_filter = UserPostFilter(data)

            for detail in video_filter.get_video_list():
                if downloaded >= self.max_counts:
                    break
                if skip_prohibited and getattr(detail, 'is_prohibited', False):
                    continue
                all_details.append(detail)
                downloaded += 1

            if not video_filter.has_more:
                break
            cursor = video_filter.max_cursor
            await asyncio.sleep(self.timeout + random.uniform(-2, 2))

        return all_details

    # ============================================================
    # 共享下载管线（消除 user/collection/mix 三处重复的 60+ 行）
    # ============================================================

    async def _batch_download(self, all_details: list, save_dir,
                              progress_callback=None,
                              download_accessories: bool = False) -> dict:
        """通用批量下载管线：视频+图文+可选附属文件

        Args:
            all_details: PostDetail 对象列表
            save_dir: 保存目录 Path
            progress_callback: 进度回调
            download_accessories: 是否下载附属文件 (music/cover/desc)

        Returns:
            {"success": bool, "count": int, "results": [{"path": str, "detail": dict}]}
        """
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
            elif detail.video_urls:
                download_tasks.append({
                    "url": detail.video_urls,
                    "dir": str(dir_path),
                    "filename": f"{filename}_video",
                    "task_id": detail.aweme_id,
                })
                task_details[detail.aweme_id] = detail
            elif getattr(detail, 'video_url', None):
                filename = format_filename(self.naming, detail.to_dict())
                download_tasks.append({
                    "url": detail.video_url,
                    "dir": str(save_dir),
                    "filename": f"{filename}_video",
                    "task_id": detail.aweme_id,
                })
                task_details[detail.aweme_id] = detail

        async with self._make_downloader(progress_callback) as dl:
            download_results = await dl.batch_download(download_tasks)

            # 附属文件并发下载
            if download_accessories:
                accessory_tasks = []
                for detail in all_details:
                    filename = format_filename(self.naming, detail.to_dict())
                    dir_path = save_dir / filename if self.folderize else save_dir
                    if self.music and getattr(detail, 'music_url', None):
                        accessory_tasks.append(dl.download_music(detail.music_url, dir_path, filename))
                    if self.cover and getattr(detail, 'cover_url', None):
                        accessory_tasks.append(dl.download_cover(detail.cover_url, dir_path, filename))
                    if self.desc and getattr(detail, 'desc', None):
                        accessory_tasks.append(dl.download_desc(detail.desc, dir_path, filename))
                if accessory_tasks:
                    await run_concurrent(accessory_tasks)

            # 图片下载
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
