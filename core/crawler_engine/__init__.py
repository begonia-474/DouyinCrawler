"""⛔ RED LINE — 爬虫引擎核心，内部逻辑不可修改

可通过 `from core.crawler_engine import X` 或保持旧路径 `from core.xxx import X`（shim 兼容）。
"""
from core.crawler_engine.crawler import DouyinCrawler
from core.crawler_engine.filter import PostDetailFilter, UserProfileFilter, UserPostFilter, JSONModel
from core.crawler_engine.api import DouyinAPIEndpoints

__all__ = [
    "DouyinCrawler",
    "PostDetailFilter", "UserProfileFilter", "UserPostFilter", "JSONModel",
    "DouyinAPIEndpoints",
]
