"""向后兼容 shim"""
import sys
from core.crawler_engine.signature import fingerprint as _real

sys.modules[__name__] = _real
