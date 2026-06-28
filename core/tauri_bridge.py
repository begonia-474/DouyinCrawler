"""向后兼容 shim — sys.modules 别名到 core.bridge.events"""
import sys
from core.bridge import events as _real

sys.modules[__name__] = _real
