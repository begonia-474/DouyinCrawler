"""向后兼容 shim — 业务 facade

内容已迁移到 core.bridge.handler。
此文件保留为向后兼容重导出。
不允许在此文件新增业务逻辑。
"""
import sys
from core.bridge import handler as _real

sys.modules[__name__] = _real