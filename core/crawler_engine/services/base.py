"""服务基类 — 共享配置和 crawler 工厂方法"""

import asyncio
import logging

from core.crawler_engine.crawler import DouyinCrawler
from core.models import ServiceConfig

logger = logging.getLogger(__name__)


class BaseService:
    """所有业务服务的基类，提供共享配置和 crawler 工厂方法"""

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
        from core.crawler_engine.filter import UserPostFilter
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
