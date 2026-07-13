from pathlib import Path
from types import SimpleNamespace

import pytest

from core.bridge import py_bridge
from core.utils import WebCastIdFetcher


class _FakeLiveHandler:
    def __init__(self, download_path: Path):
        self.config = SimpleNamespace(
            download_path=download_path,
            app_name="douyin",
            naming="{create}_{desc}",
            cookie="session=live-test",
        )

    async def handle_user_live(self, url: str) -> dict:
        assert url == "https://live.douyin.com/123456"
        return {
            "success": True,
            "web_rid": "123456",
            "room_id": "987654321",
            "title": "测试直播",
            "nickname": "测试主播",
            "sec_user_id": "sec-live-user",
            "user_id": "live-user",
            "cover_url": "https://example.com/cover.jpeg",
            "user_count": "100",
            "is_live": True,
            "flv_pull_url": {"FULL_HD1": "https://example.com/live.flv"},
            "m3u8_pull_url": {
                "HD1": "https://example.com/hd.m3u8",
                "FULL_HD1": "https://example.com/full-hd.m3u8",
            },
        }


def test_resolve_live_returns_f2_recording_contract(monkeypatch, tmp_path):
    manager = SimpleNamespace(handler=_FakeLiveHandler(tmp_path))
    monkeypatch.setattr(py_bridge, "_get_task_manager", lambda: manager)

    result = py_bridge.resolve_live("https://live.douyin.com/123456")

    assert result["success"] is True
    assert result["m3u8_url"] == "https://example.com/full-hd.m3u8"
    assert result["filename"].endswith("_live")
    assert "测试直播" in result["filename"]
    assert result["suffix"] == ".flv"
    assert Path(result["save_dir"]) == tmp_path / "douyin" / "live" / "测试主播"
    assert result["headers"]["Cookie"] == "session=live-test"
    assert result["room_id"] == "987654321"
    assert result["sec_user_id"] == "sec-live-user"


def test_get_live_info_matches_rust_live_info_contract(monkeypatch, tmp_path):
    manager = SimpleNamespace(handler=_FakeLiveHandler(tmp_path))
    monkeypatch.setattr(py_bridge, "_get_task_manager", lambda: manager)

    result = py_bridge.get_live_info("https://live.douyin.com/123456")

    assert result["success"] is True
    assert result["live_info"]["title"] == "测试直播"
    assert result["live_info"]["nickname"] == "测试主播"
    assert result["live_info"]["is_live"] is True
    assert result["live_info"]["flv_urls"] == ["https://example.com/live.flv"]
    assert result["live_info"]["m3u8_urls"] == [
        "https://example.com/hd.m3u8",
        "https://example.com/full-hd.m3u8",
    ]


@pytest.mark.asyncio
async def test_webcast_id_supports_web_and_app_reflow_links():
    assert await WebCastIdFetcher.get_webcast_id(
        "https://live.douyin.com/775841227732"
    ) == "775841227732"
    assert await WebCastIdFetcher.get_webcast_id(
        "https://webcast.amemv.com/douyin/webcast/reflow/7318296342189919011"
    ) == "7318296342189919011"
