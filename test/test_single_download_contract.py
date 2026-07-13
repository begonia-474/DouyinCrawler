"""Offline contract tests for the Python -> Rust single-download plan."""

from contextlib import asynccontextmanager
from pathlib import Path
from types import SimpleNamespace

import pytest
from pydantic import ValidationError

from core.bridge import py_bridge
from core.models.single_download import SingleDownloadPlanV1


pytestmark = [pytest.mark.offline]


class FakeDetail:
    aweme_id = "7650450403901017571"
    author_nickname = "tester"
    video_urls = ["https://cdn.example/video-1.mp4", "https://cdn.example/video-2.mp4"]
    video_url = "https://cdn.example/video.mp4"
    images = []
    images_video = []
    is_image_post = False
    is_prohibited = False
    music_url = "https://cdn.example/music.mp3"
    cover_url = "https://cdn.example/cover.jpeg"
    desc = "contract test"

    def to_dict(self):
        return {
            "aweme_id": self.aweme_id,
            "author": self.author_nickname,
            "desc": self.desc,
            "create_time": 1700000000,
        }

    def to_db_dict(self):
        return {
            "aweme_id": self.aweme_id,
            "desc": self.desc,
            "aweme_type": 0,
            "author_nickname": self.author_nickname,
            "author_sec_uid": "MS4wLjABAAAAcontract",
            "duration": 12,
            "digg_count": 3,
            "author_avatar_url": "ignored-by-rust-video-info",
        }


def _install_single_resolver(monkeypatch, tmp_path: Path, detail: FakeDetail, *, folderize=False):
    async def parse_video(_url):
        return {"success": True}

    async def fetch_post_detail(_aweme_id):
        return {"aweme_detail": {}}

    @asynccontextmanager
    async def make_crawler():
        yield SimpleNamespace(fetch_post_detail=fetch_post_detail)

    handler = SimpleNamespace(
        config=SimpleNamespace(
            cookie="sessionid=test",
            naming="{aweme_id}",
            download_path=tmp_path,
            app_name="douyin",
            folderize=folderize,
            music=True,
            cover=True,
            desc=True,
        ),
        _video=SimpleNamespace(
            handle_parse_video=parse_video,
            _make_crawler=make_crawler,
        ),
    )

    async def get_aweme_id(_url):
        return detail.aweme_id

    monkeypatch.setattr(py_bridge, "_get_task_manager", lambda: SimpleNamespace(handler=handler))
    monkeypatch.setattr(
        "core.utils.AwemeIdFetcher.get_aweme_id",
        get_aweme_id,
    )
    monkeypatch.setattr("core.crawler_engine.filter.PostDetailFilter", lambda _data: detail)


def _valid_contract_data() -> dict:
    return {
        "save_dir": "/tmp/downloads",
        "total": 1,
        "items": [
            {
                "media_key": "7650450403901017571:video:0",
                "aweme_id": "7650450403901017571",
                "urls": ["https://cdn.example/video.mp4"],
                "kind": "video",
                "output": {
                    "filename": "7650450403901017571_video",
                    "suffix": ".mp4",
                    "folder_name": None,
                },
                "metadata": {"aweme_id": "7650450403901017571"},
            }
        ],
    }


def test_resolve_one_returns_versioned_typed_contract(monkeypatch, tmp_path):
    detail = FakeDetail()
    _install_single_resolver(monkeypatch, tmp_path, detail)

    result = py_bridge.resolve_urls("one", "https://www.douyin.com/video/7650450403901017571")

    assert result["success"] is True
    assert result["contract_version"] == 1
    assert result["mode"] == "one"
    assert result["save_dir"] == str(tmp_path / "douyin" / "one" / "tester")
    assert result["total"] == 1

    item = result["items"][0]
    assert item["urls"] == detail.video_urls
    assert item["kind"] == "video"
    assert item["media_key"] == f"{detail.aweme_id}:video:0"
    assert item["output"] == {
        "filename": f"{detail.aweme_id}_video",
        "suffix": ".mp4",
        "folder_name": None,
    }
    assert item["metadata"]["aweme_id"] == detail.aweme_id
    assert "author_avatar_url" not in item["metadata"]
    assert {accessory["kind"] for accessory in item["accessories"]} == {
        "music",
        "cover",
        "description",
    }
    assert "download_url" not in item
    assert "detail" not in item


def test_resolve_one_preserves_live_photo_before_image_order(monkeypatch, tmp_path):
    detail = FakeDetail()
    detail.is_image_post = True
    detail.video_urls = []
    detail.video_url = None
    detail.images_video = ["https://cdn.example/live-1.mp4"]
    detail.images = ["https://cdn.example/image-1.webp", "https://cdn.example/image-2.webp"]
    _install_single_resolver(monkeypatch, tmp_path, detail, folderize=True)

    result = py_bridge.resolve_urls("one", "https://www.douyin.com/note/7650450403901017571")

    assert [item["kind"] for item in result["items"]] == [
        "live_photo",
        "image",
        "image",
    ]
    assert [item["output"]["filename"] for item in result["items"]] == [
        f"{detail.aweme_id}_live_1",
        f"{detail.aweme_id}_image_1",
        f"{detail.aweme_id}_image_2",
    ]
    assert result["save_dir"] == str(
        tmp_path / "douyin" / "one" / "tester" / detail.aweme_id
    )
    assert all(item["output"]["folder_name"] is None for item in result["items"])
    assert {accessory["kind"] for accessory in result["items"][0]["accessories"]} == {
        "music",
        "cover",
    }


def test_single_download_model_rejects_inconsistent_semantics():
    data = _valid_contract_data()
    data["total"] = 2

    with pytest.raises(ValidationError, match="total must equal"):
        SingleDownloadPlanV1.model_validate(data)

    data = _valid_contract_data()
    data["items"][0]["metadata"]["aweme_id"] = "other"

    with pytest.raises(ValidationError, match="metadata.aweme_id must match"):
        SingleDownloadPlanV1.model_validate(data)


def test_single_download_model_rejects_invalid_accessory_payload():
    data = _valid_contract_data()
    data["items"][0]["accessories"] = [
        {
            "kind": "music",
            "output": {
                "filename": "music",
                "suffix": ".mp3",
                "folder_name": None,
            },
        }
    ]

    with pytest.raises(ValidationError, match="require url"):
        SingleDownloadPlanV1.model_validate(data)


def test_single_download_model_rejects_unknown_contract_fields():
    data = _valid_contract_data()
    data["unexpected"] = "not part of V1"

    with pytest.raises(ValidationError, match="Extra inputs are not permitted"):
        SingleDownloadPlanV1.model_validate(data)
