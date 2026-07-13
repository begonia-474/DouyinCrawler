"""测试 ParsingContext — 轻量配置和 lazy handler 容器"""

from pathlib import Path

import pytest

from core.bridge.parsing_context import ParsingContext, context


pytestmark = [pytest.mark.offline]


class TestParsingContext:
    """ParsingContext 基本生命周期"""

    def test_default_config(self):
        ctx = ParsingContext()
        assert ctx._cookie == ""
        assert ctx._download_path == "Download"
        assert ctx._naming == "{create}_{desc}"
        assert ctx._encryption == "ab"
        assert ctx._app_name == "douyin"
        assert ctx._proxy == ""
        assert ctx._folderize is False
        assert ctx._music is False
        assert ctx._cover is False
        assert ctx._desc is False
        assert ctx._page_counts == 20
        assert ctx._max_counts == 0
        assert ctx._timeout == 5
        assert ctx._max_connections == 5
        assert ctx._max_retries == 5
        assert ctx._max_tasks == 10

    def test_update_config_sets_fields(self):
        ctx = ParsingContext()
        ctx.update_config(cookie="test-cookie", download_path="/tmp/dl", page_counts=50)
        assert ctx._cookie == "test-cookie"
        assert ctx._download_path == "/tmp/dl"
        assert ctx._page_counts == 50

    def test_update_config_normalizes_cookie_whitespace(self):
        ctx = ParsingContext()

        ctx.update_config(cookie="sessionid=one;\n  sid_guard=two\t passport=three")

        assert ctx._cookie == "sessionid=one; sid_guard=two passport=three"

    def test_update_config_invalidates_handler(self):
        ctx = ParsingContext()
        ctx._handler = "fake"  # simulate existing handler
        ctx.update_config(cookie="new-cookie")
        assert ctx._handler is None

    def test_handler_is_lazy(self):
        ctx = ParsingContext()
        # handler not created until accessed
        assert ctx._handler is None

    def test_reset_clears_everything(self):
        ctx = ParsingContext()
        ctx.update_config(cookie="some-cookie", download_path="/tmp/dl")
        ctx.reset()
        assert ctx._cookie == ""
        assert ctx._download_path == "Download"

    def test_reset_clears_handler(self):
        ctx = ParsingContext()
        ctx._handler = "stale"
        ctx.reset()
        assert ctx._handler is None

    def test_unknown_field_ignored(self):
        ctx = ParsingContext()
        ctx.update_config(nonexistent_field="value")
        # should not raise, just ignore

    def test_context_singleton_is_parsing_context(self):
        from core.bridge.parsing_context import context
        assert isinstance(context, ParsingContext)

    def test_get_context_from_py_bridge(self):
        from core.bridge.py_bridge import _get_context
        ctx = _get_context()
        assert isinstance(ctx, ParsingContext)

    def test_py_bridge_parse_video_no_crash_without_handler(self, monkeypatch):
        """ParsingContext should not create handler with empty config (will fail on handler)"""
        from core.bridge.py_bridge import parse_video
        result = parse_video("https://example.com/test")
        assert result["success"] is False
        assert "error" in result

    def test_py_bridge_get_context_returns_singleton(self):
        from core.bridge.py_bridge import _get_context
        ctx1 = _get_context()
        ctx2 = _get_context()
        assert ctx1 is ctx2
