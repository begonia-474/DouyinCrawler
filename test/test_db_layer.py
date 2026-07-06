"""离线测试：db.py 和 db_bridge.py 逻辑

db.py 是纯透传层，测试其参数组装和调用链路。
db_bridge.py 的 stubs 未注册时应安全返回 False。

注意：任务生命周期函数（create_task, update_task_status, create_task_item, update_task_item_status）
已迁移到 Rust TaskApplicationService，相关测试已移除。
"""

import pytest
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

import core.db as db
import core.db_bridge as db_bridge

pytestmark = [pytest.mark.offline]


# ============================================================
# db_bridge — stubs 未注册时的安全行为
# ============================================================

class TestDbBridgeUnregistered:
    """测试 db_bridge 在 Rust 未注入 stub 时的行为"""

    def test_save_video_info_returns_false_when_unregistered(self):
        """_save_video_info 为 None 时应返回 False"""
        original = db_bridge._save_video_info
        db_bridge._save_video_info = None
        try:
            result = db_bridge.save_video_info({"aweme_id": "123"})
            assert result is False
        finally:
            db_bridge._save_video_info = original

    def test_save_user_info_returns_false_when_unregistered(self):
        """_save_user_info 为 None 时应返回 False"""
        original = db_bridge._save_user_info
        db_bridge._save_user_info = None
        try:
            result = db_bridge.save_user_info({"sec_user_id": "abc"})
            assert result is False
        finally:
            db_bridge._save_user_info = original

    def test_has_user_returns_false_when_unregistered(self):
        """_has_user 为 None 时应返回 False"""
        original = db_bridge._has_user
        db_bridge._has_user = None
        try:
            result = db_bridge.has_user("abc")
            assert result is False
        finally:
            db_bridge._has_user = original


# ============================================================
# db_bridge — mock stub 调用行为
# ============================================================

class TestDbBridgeMocked:
    """测试 db_bridge 在 stub 注册后正确转发调用"""

    def test_save_video_info_calls_stub(self):
        calls = []
        original = db_bridge._save_video_info
        db_bridge._save_video_info = lambda data: calls.append(data)
        try:
            result = db_bridge.save_video_info({"aweme_id": "v1", "desc": "test"})
            assert result is True
            assert len(calls) == 1
            assert calls[0]["aweme_id"] == "v1"
        finally:
            db_bridge._save_video_info = original

    def test_save_user_info_calls_stub(self):
        calls = []
        original = db_bridge._save_user_info
        db_bridge._save_user_info = lambda data: calls.append(data)
        try:
            result = db_bridge.save_user_info({"sec_user_id": "u1", "nickname": "test"})
            assert result is True
            assert len(calls) == 1
            assert calls[0]["sec_user_id"] == "u1"
        finally:
            db_bridge._save_user_info = original

    def test_has_user_calls_stub(self):
        original = db_bridge._has_user
        db_bridge._has_user = lambda uid: uid == "exists"
        try:
            assert db_bridge.has_user("exists") is True
            assert db_bridge.has_user("missing") is False
        finally:
            db_bridge._has_user = original

    def test_stub_exception_returns_false(self):
        """stub 抛异常时应捕获并返回 False"""
        original = db_bridge._save_video_info
        db_bridge._save_video_info = lambda data: (_ for _ in ()).throw(RuntimeError("boom"))
        try:
            result = db_bridge.save_video_info({"aweme_id": "x"})
            assert result is False
        finally:
            db_bridge._save_video_info = original


# ============================================================
# db.py — 参数组装
# ============================================================

class TestDbFacade:
    """测试 db.py 的参数组装逻辑"""

    def test_save_video_info_passthrough(self):
        """save_video_info 应原样传递字典"""
        captured = {}
        original_fn = db_bridge._save_video_info
        db_bridge._save_video_info = lambda data: captured.update(data)
        try:
            video_data = {"aweme_id": "v1", "desc": "test", "digg_count": 100}
            result = db.save_video_info(video_data)
            assert result is True
            assert captured == video_data
        finally:
            db_bridge._save_video_info = original_fn

    def test_save_user_info_passthrough(self):
        """save_user_info 应原样传递字典"""
        captured = {}
        original_fn = db_bridge._save_user_info
        db_bridge._save_user_info = lambda data: captured.update(data)
        try:
            user_data = {"sec_user_id": "u1", "nickname": "test", "follower_count": 500}
            result = db.save_user_info(user_data)
            assert result is True
            assert captured == user_data
        finally:
            db_bridge._save_user_info = original_fn

    def test_save_batch_results_counts(self):
        """save_batch_results 应正确计数成功/失败"""
        call_count = {"video": 0, "user": 0}

        def mock_video(data):
            call_count["video"] += 1
            return True

        def mock_user(data):
            call_count["user"] += 1
            return True

        orig_vi = db_bridge._save_video_info
        orig_ui = db_bridge._save_user_info
        orig_hu = db_bridge._has_user
        db_bridge._save_video_info = mock_video
        db_bridge._save_user_info = mock_user
        db_bridge._has_user = lambda uid: False
        try:
            results = [
                {"path": "/a.mp4", "detail": {"aweme_id": "1", "desc": "v1", "author_sec_uid": "u1"}},
                {"path": "/b.mp4", "detail": {"aweme_id": "2", "desc": "v2", "author_sec_uid": "u2"}},
                {"path": "/c.mp4", "detail": {"aweme_id": "3", "desc": "v3"}},  # 无 author_sec_uid
            ]
            stats = db.save_batch_results(results, download_type="video")
            assert stats["saved"] == 3
            assert stats["failed"] == 0
            assert call_count["video"] == 3
            assert call_count["user"] == 2  # 第3条无 author_sec_uid
        finally:
            db_bridge._save_video_info = orig_vi
            db_bridge._save_user_info = orig_ui
            db_bridge._has_user = orig_hu
