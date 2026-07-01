"""向后兼容 shim — Rust 可调用函数桥接

内容已迁移到 core.bridge.py_bridge。
此文件保留为向后兼容重导出。
不允许在此文件新增业务逻辑。
"""
import sys
from core.bridge import py_bridge as _real

sys.modules[__name__] = _real