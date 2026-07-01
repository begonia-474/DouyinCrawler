"""向后兼容 shim — 数据库桥接

内容已迁移到 core.bridge.db_bridge。
此文件保留为向后兼容重导出。
不允许在此文件新增业务逻辑。

注意：模块级变量在运行时被 Rust db_bridge.rs 通过 setattr 注入替换，
因此必须使用 sys.modules 别名而非 import * 重导出。
"""
import sys
from core.bridge import db_bridge as _real

sys.modules[__name__] = _real