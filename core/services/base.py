"""向后兼容 shim"""
import sys
from core.crawler_engine.services import base as _real
sys.modules[__name__] = _real
