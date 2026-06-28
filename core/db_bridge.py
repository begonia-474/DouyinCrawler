"""向后兼容 shim — 通过 sys.modules 别名到 core.bridge.db_bridge

模块级变量在运行时被 Rust db_bridge.rs 通过 setattr 注入替换，
因此必须使用 sys.modules 别名而非 import * 重导出。
"""
import sys
from core.bridge import db_bridge as _real

sys.modules[__name__] = _real
