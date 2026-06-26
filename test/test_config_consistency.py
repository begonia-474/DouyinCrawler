"""离线测试：Rust/Python 配置默认值一致性

验证 core.config.DEFAULTS 与 Rust AppConfig::default() 的字段名和默认值一致。
"""

import pytest
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from core.config import DEFAULTS

pytestmark = [pytest.mark.offline]

# Rust AppConfig::default() 的值（来自 src-tauri/src/config.rs:45-66）
RUST_DEFAULTS = {
    "cookie": "",
    "download_path": "Download",
    "naming": "{create}_{desc}",
    "encryption": "ab",
    "proxy": "",
    "app_name": "douyin",
    "folderize": False,
    "music": False,
    "cover": False,
    "desc": False,
    "interval": None,
    "page_counts": 20,
    "max_counts": 0,
    "timeout": 5,
    "max_connections": 5,
    "max_retries": 5,
    "max_tasks": 10,
}


class TestConfigConsistency:
    """Python DEFAULTS 与 Rust AppConfig 默认值一致性"""

    def test_keys_match(self):
        """Python 和 Rust 的配置键集合应完全一致"""
        py_keys = set(DEFAULTS.keys())
        rust_keys = set(RUST_DEFAULTS.keys())
        missing = rust_keys - py_keys
        extra = py_keys - rust_keys
        assert not missing, f"Python DEFAULTS 缺少 Rust 字段: {missing}"
        assert not extra, f"Python DEFAULTS 多出 Rust 没有的字段: {extra}"

    def test_values_match(self):
        """每个配置键的默认值应一致"""
        for key in RUST_DEFAULTS:
            assert key in DEFAULTS, f"Python DEFAULTS 缺少字段: {key}"
            py_val = DEFAULTS[key]
            rust_val = RUST_DEFAULTS[key]
            assert py_val == rust_val, (
                f"字段 '{key}' 默认值不一致: Python={py_val!r}, Rust={rust_val!r}"
            )

    def test_field_count(self):
        """配置字段总数应为 17"""
        assert len(DEFAULTS) == 17, f"Python DEFAULTS 有 {len(DEFAULTS)} 个字段，预期 17"
        assert len(RUST_DEFAULTS) == 17, f"RUST_DEFAULTS 有 {len(RUST_DEFAULTS)} 个字段，预期 17"
