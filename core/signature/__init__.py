"""向后兼容 shim — sys.modules 别名到 core.crawler_engine.signature"""
import sys
from core.crawler_engine import signature as _real

sys.modules[__name__] = _real
