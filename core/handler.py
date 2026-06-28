"""向后兼容 shim — sys.modules 别名到 core.bridge.handler"""
import sys
from core.bridge import handler as _real

sys.modules[__name__] = _real
