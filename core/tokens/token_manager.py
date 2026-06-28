"""向后兼容 shim"""
import sys
from core.crawler_engine.tokens import token_manager as _real

sys.modules[__name__] = _real
