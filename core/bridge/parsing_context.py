"""Parsing runtime/context — 轻量配置和 lazy handler 容器

替代旧的 TaskManager，只拥有：
- 当前 crawler 配置（支持热更新）
- 懒加载 DouyinHandler
- 解析/查询调用所需生命周期

没有后台线程、任务注册、取消信号、事件发射器、数据库写入或媒体下载器。
"""

import logging
from pathlib import Path
from typing import Optional

from core.bridge.handler import DouyinHandler
from core.logger import setup_logging

setup_logging()

logger = logging.getLogger(__name__)

PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent


class ParsingContext:
    """解析运行时上下文 — 纯查询/解析生命周期"""

    def __init__(self):
        self._cookie: str = ""
        self._download_path: str = "Download"
        self._naming: str = "{create}_{desc}"
        self._encryption: str = "ab"
        self._proxy: str = ""
        self._app_name: str = "douyin"
        self._folderize: bool = False
        self._music: bool = False
        self._cover: bool = False
        self._desc: bool = False
        self._interval: Optional[str] = None
        self._page_counts: int = 20
        self._max_counts: int = 0
        self._timeout: int = 5
        self._max_connections: int = 5
        self._max_retries: int = 5
        self._max_tasks: int = 10
        self._handler: Optional[DouyinHandler] = None

    def update_config(self, **kwargs):
        """更新配置并使 handler 失效（下次访问重建）

        Args:
            **kwargs: 配置字段键值对，与 __init__ 参数保持一致
        """
        for key, value in kwargs.items():
            private_key = f"_{key}"
            if hasattr(self, private_key):
                # Cookie 空白/换行规范化（保持旧 TaskManager 行为）
                if key == "cookie" and value is not None:
                    if '\n' in value or '\r' in value:
                        logger.warning("[parsing_context] cookie 中包含换行符")
                    value = " ".join(value.split())
                setattr(self, private_key, value)
        self._handler = None

    def reset(self):
        """重置为默认值（测试隔离用）"""
        self.__init__()

    @property
    def handler(self) -> DouyinHandler:
        """获取或懒加载 DouyinHandler"""
        if self._handler is None:
            proxies = None
            if self._proxy:
                proxies = {"http://": self._proxy, "https://": self._proxy}
            download_path = Path(self._download_path)
            if not download_path.is_absolute():
                download_path = PROJECT_ROOT / download_path
            logger.info("[parsing_context] 创建 DouyinHandler (has_cookie=%s)", bool(self._cookie))
            self._handler = DouyinHandler(
                cookie=self._cookie,
                download_path=str(download_path),
                naming=self._naming,
                encryption=self._encryption,
                proxies=proxies,
                app_name=self._app_name,
                folderize=self._folderize,
                music=self._music,
                cover=self._cover,
                desc=self._desc,
                interval=self._interval,
                page_counts=self._page_counts,
                max_counts=self._max_counts,
                timeout=self._timeout,
                max_connections=self._max_connections,
                max_retries=self._max_retries,
                max_tasks=self._max_tasks,
            )
        return self._handler


context = ParsingContext()
