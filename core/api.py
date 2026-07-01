"""向后兼容 shim — API 端点

内容已迁移到 core.crawler_engine.api。
此文件保留为向后兼容重导出。
不允许在此文件新增业务逻辑。
"""
import sys
from core.crawler_engine import api as _real

sys.modules[__name__] = _real