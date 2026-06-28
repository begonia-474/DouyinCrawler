"""向后兼容 shim — sys.modules 别名到 core.bridge.py_bridge"""
import sys
from core.bridge import py_bridge as _real

sys.modules[__name__] = _real
