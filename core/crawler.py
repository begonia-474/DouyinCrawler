"""向后兼容 shim — 爬虫引擎核心

内容已迁移到 core.crawler_engine.crawler。
此文件保留为向后兼容重导出。
不允许在此文件新增业务逻辑。
"""
import sys
from core.crawler_engine import crawler as _real

sys.modules[__name__] = _real