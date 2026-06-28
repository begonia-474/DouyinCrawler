"""向后兼容 shim — sys.modules 别名到 core.crawler_engine.filter"""
import sys
from core.crawler_engine import filter as _real

sys.modules[__name__] = _real
