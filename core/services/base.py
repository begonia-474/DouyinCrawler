"""服务基类 — 共享配置和工厂方法"""

import asyncio
import logging
from pathlib import Path

from core.crawler import DouyinCrawler
from core.downloader import Downloader

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

    def __init__(self, *, cookie: str, download_path: Path, naming: str,
                 max_counts: float, page_counts: int, timeout: int,
                 encryption: str, proxies: dict, app_name: str,
                 folderize: bool, music: bool, cover: bool, desc: bool,
                 interval: str, max_connections: int, max_retries: int,
                 max_tasks: int):
        self.cookie = cookie
        self.download_path = download_path
        self.naming = naming
        self.max_counts = max_counts
        self.page_counts = page_counts
        self.timeout = timeout
        self.encryption = encryption
        self.proxies = proxies
        self.app_name = app_name
        self.folderize = folderize
        self.music = music
        self.cover = cover
        self.desc = desc
        self.interval = interval
        self.max_connections = max_connections
        self.max_retries = max_retries
        self.max_tasks = max_tasks

    def _make_crawler(self) -> DouyinCrawler:
        return DouyinCrawler(self.cookie, self.proxies, self.encryption, self.max_retries)

    def _make_downloader(self, progress_callback=None) -> Downloader:
        return Downloader(self.cookie, max_connections=self.max_connections,
                          timeout=self.timeout, progress_callback=progress_callback)
