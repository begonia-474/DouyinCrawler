"""pytest 全局配置与共享 fixtures

marker 约定：
- @pytest.mark.offline  — 离线测试，无需网络和真实 cookie
- @pytest.mark.integration — 需要网络和真实 cookie 的集成测试
"""

import pytest
import sys
import json
from pathlib import Path

# 确保 core/ 可导入
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))


def pytest_collection_modifyitems(config, items):
    """未标记的测试默认归为 offline；无 cookie 时跳过 integration 测试"""
    for item in items:
        if not any(item.iter_markers()):
            item.add_marker(pytest.mark.offline)


@pytest.fixture(scope="session")
def cookie():
    """从 config/app.json 加载真实 cookie，不存在则跳过 integration 测试"""
    cfg_path = Path(__file__).resolve().parent.parent / "config" / "app.json"
    if not cfg_path.exists():
        pytest.skip("config/app.json 不存在，跳过集成测试")
    try:
        with open(cfg_path, encoding="utf-8") as f:
            cfg = json.load(f)
        c = cfg.get("douyin", {}).get("cookie", "")
        if not c:
            pytest.skip("cookie 为空，跳过集成测试")
        return c
    except Exception:
        pytest.skip("无法加载 config/app.json，跳过集成测试")


@pytest.fixture(scope="session")
def download_dir():
    """下载目录路径"""
    return str(Path(__file__).resolve().parent.parent / "Download")
