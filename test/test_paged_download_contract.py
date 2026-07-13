"""Offline contract tests for the Python -> Rust paged post plan."""

from contextlib import asynccontextmanager
from copy import deepcopy
from pathlib import Path
from types import SimpleNamespace

import pytest
from pydantic import ValidationError

from core.bridge import py_bridge
from core.models.paged_download import PagedDownloadPlanV1
from core.services.media_plan import build_media_items_v1, ordered_aweme_ids


pytestmark = [pytest.mark.offline]


class FakeDetail:
    def __init__(
        self,
        aweme_id: str,
        *,
        image_post: bool = False,
        prohibited: bool = False,
        author_nickname: str = "tester",
    ):
        self.aweme_id = aweme_id
        self.author_nickname = author_nickname
        self.is_image_post = image_post
        self.is_prohibited = prohibited
        self.desc = f"desc-{aweme_id}"
        self.video_urls = [] if image_post else [f"https://cdn.example/{aweme_id}-1.mp4"]
        self.video_url = "" if image_post else f"https://cdn.example/{aweme_id}.mp4"
        self.images_video = (
            [f"https://cdn.example/{aweme_id}-live-1.mp4"] if image_post else []
        )
        self.images = (
            [
                f"https://cdn.example/{aweme_id}-image-1.webp",
                f"https://cdn.example/{aweme_id}-image-2.webp",
            ]
            if image_post
            else []
        )
        self.music_url = f"https://cdn.example/{aweme_id}.mp3"
        self.cover_url = f"https://cdn.example/{aweme_id}-cover.jpeg"

    def to_dict(self) -> dict:
        return {
            "aweme_id": self.aweme_id,
            "author": self.author_nickname,
            "desc": self.desc,
            "create_time": 1700000000,
        }

    def to_db_dict(self) -> dict:
        return {
            "aweme_id": self.aweme_id,
            "desc": self.desc,
            "aweme_type": 68 if self.is_image_post else 0,
            "author_nickname": self.author_nickname,
            "author_sec_uid": "MS4wLjABAAAApaged",
            "duration": 12,
            "digg_count": 3,
            "is_prohibited": int(self.is_prohibited),
        }


def _valid_item(aweme_id: str = "100") -> dict:
    return {
        "media_key": f"{aweme_id}:video:0",
        "aweme_id": aweme_id,
        "urls": [f"https://cdn.example/{aweme_id}.mp4"],
        "kind": "video",
        "output": {
            "filename": f"{aweme_id}_video",
            "suffix": ".mp4",
            "folder_name": None,
        },
        "metadata": {"aweme_id": aweme_id},
    }


def _valid_plan() -> dict:
    return {
        "success": True,
        "contract_version": 1,
        "mode": "post",
        "save_dir": "/tmp/downloads/douyin/post/tester",
        "items": [_valid_item()],
        "next_cursor": 55,
        "has_more": True,
        "page_aweme_ids": ["100"],
        "user_profile": {"sec_user_id": "sec-100", "nickname": "tester"},
    }


@pytest.mark.parametrize(
    ("has_more", "next_cursor", "items", "message"),
    [
        (True, None, [_valid_item()], "requires next_cursor"),
        (False, 55, [_valid_item()], "requires next_cursor=None"),
        (True, 55, [], "requires non-empty page_aweme_ids"),
    ],
)
def test_paged_model_enforces_cursor_protocol(has_more, next_cursor, items, message):
    data = _valid_plan()
    data.update(has_more=has_more, next_cursor=next_cursor, items=items)
    if not items:
        data["page_aweme_ids"] = []

    with pytest.raises(ValidationError, match=message):
        PagedDownloadPlanV1.model_validate(data)


@pytest.mark.parametrize(
    ("field", "value"),
    [
        ("contract_version", 2),
        ("mode", "like"),
        ("success", False),
    ],
)
def test_paged_model_rejects_unsupported_envelope_values(field, value):
    data = _valid_plan()
    data[field] = value

    with pytest.raises(ValidationError):
        PagedDownloadPlanV1.model_validate(data)


def test_paged_model_rejects_unknown_fields_and_invalid_page_identity():
    data = _valid_plan()
    data["unexpected"] = "not part of V1"
    with pytest.raises(ValidationError, match="Extra inputs are not permitted"):
        PagedDownloadPlanV1.model_validate(data)

    data = _valid_plan()
    data["page_aweme_ids"] = ["100", "100"]
    with pytest.raises(ValidationError, match="ordered and unique"):
        PagedDownloadPlanV1.model_validate(data)

    data = _valid_plan()
    data["page_aweme_ids"] = ["100", "missing-media"]
    # Issue 06: page_aweme_ids may now include IDs without media items
    # (for Rust to distinguish missing vs unavailable).
    plan = PagedDownloadPlanV1.model_validate(data)
    assert plan.page_aweme_ids == ["100", "missing-media"]
    assert len(plan.items) == 1

    data = _valid_plan()
    data["items"][0]["aweme_id"] = "other"
    data["items"][0]["media_key"] = "other:video:0"
    data["items"][0]["metadata"]["aweme_id"] = "other"
    with pytest.raises(ValidationError, match="belong to page_aweme_ids"):
        PagedDownloadPlanV1.model_validate(data)

    data = _valid_plan()
    data["items"].append(deepcopy(data["items"][0]))
    with pytest.raises(ValidationError, match="media_key must be unique"):
        PagedDownloadPlanV1.model_validate(data)


def test_paged_model_accepts_media_free_source_page_with_more_data():
    data = _valid_plan()
    data["items"] = []
    data["page_aweme_ids"] = ["seen-but-unavailable"]

    plan = PagedDownloadPlanV1.model_validate(data)

    assert plan.has_more is True
    assert plan.next_cursor == 55
    assert plan.page_aweme_ids == ["seen-but-unavailable"]
    assert plan.items == []


def test_shared_planner_preserves_f2_names_groups_and_accessories():
    video = FakeDetail("100")
    gallery = FakeDetail("200", image_post=True)
    prohibited = FakeDetail("300", prohibited=True)

    items = build_media_items_v1(
        [video, gallery, prohibited],
        naming="{aweme_id}_{desc}",
        folderize=True,
        headers={"Cookie": "sessionid=test"},
    )

    assert ordered_aweme_ids(items) == ["100", "200"]
    assert [item.media_key for item in items] == [
        "100:video:0",
        "200:live_photo:1",
        "200:image:1",
        "200:image:2",
    ]
    assert [item.output.filename for item in items] == [
        "100_desc-100_video",
        "200_desc-200_live_1",
        "200_desc-200_image_1",
        "200_desc-200_image_2",
    ]
    assert [item.output.folder_name for item in items] == [
        "100_desc-100",
        "200_desc-200",
        "200_desc-200",
        "200_desc-200",
    ]

    work_accessories = {
        item.aweme_id: [accessory.kind.value for accessory in item.accessories]
        for item in items
        if item.accessories
    }
    assert work_accessories == {
        "100": ["music", "cover", "description"],
        "200": ["music", "cover"],
    }
    assert all(not item.accessories for item in items[2:])


def _install_paged_resolver(monkeypatch, tmp_path: Path, pages: dict[int, dict]):
    calls = []

    class FakeCrawler:
        async def fetch_user_profile(self, sec_user_id):
            assert sec_user_id == "sec-user"
            return {"profile": True}

        async def fetch_user_post(self, sec_user_id, cursor, count):
            calls.append((sec_user_id, cursor, count))
            return pages[cursor]

    @asynccontextmanager
    async def make_crawler():
        yield FakeCrawler()

    class FakePostFilter:
        def __init__(self, data):
            self._data = data
            self.has_more = data["has_more"]
            self.max_cursor = data["next_cursor"]

        def get_video_list(self):
            return self._data["details"]

    class FakeProfileFilter:
        def __init__(self, _data):
            pass

        def to_dict(self):
            return {"sec_user_id": "sec-user", "nickname": "tester"}

    async def get_sec_user_id(_url):
        return "sec-user"

    handler = SimpleNamespace(
        config=SimpleNamespace(
            cookie="sessionid=test",
            naming="{aweme_id}",
            download_path=tmp_path,
            app_name="douyin",
            folderize=False,
        ),
        _user=SimpleNamespace(_make_crawler=make_crawler),
    )
    monkeypatch.setattr(py_bridge, "_get_task_manager", lambda: SimpleNamespace(handler=handler))
    monkeypatch.setattr("core.utils.SecUserIdFetcher.get_sec_user_id", get_sec_user_id)
    monkeypatch.setattr("core.crawler_engine.filter.UserPostFilter", FakePostFilter)
    monkeypatch.setattr("core.crawler_engine.filter.UserProfileFilter", FakeProfileFilter)
    return calls


def test_page_aweme_ids_preserves_all_source_ids_including_media_free(monkeypatch, tmp_path):
    """page_aweme_ids must include all source page IDs, even those with no media items."""
    prohibited = FakeDetail("prohibited", prohibited=True)
    normal = FakeDetail("normal")
    pages = {
        0: {
            "details": [prohibited, normal],
            "has_more": False,
            "next_cursor": 0,
        },
    }
    _install_paged_resolver(monkeypatch, tmp_path, pages)
    result = py_bridge.resolve_page("post", "https://douyin.com/user/sec-user", 0, 2)
    assert result["page_aweme_ids"] == ["prohibited", "normal"]
    item_ids = [item["aweme_id"] for item in result["items"]]
    assert "prohibited" not in item_ids
    assert "normal" in item_ids


def test_page_aweme_ids_preserves_order_from_source(monkeypatch, tmp_path):
    """page_aweme_ids preserves source page occurrence order."""
    pages = {
        0: {
            "details": [FakeDetail("b"), FakeDetail("a"), FakeDetail("c")],
            "has_more": False,
            "next_cursor": 0,
        },
    }
    _install_paged_resolver(monkeypatch, tmp_path, pages)
    result = py_bridge.resolve_page("post", "https://douyin.com/user/sec-user", 0, 3)
    assert result["page_aweme_ids"] == ["b", "a", "c"]


def test_resolve_post_returns_two_exact_typed_pages(monkeypatch, tmp_path):
    pages = {
        0: {
            "details": [FakeDetail("100"), FakeDetail("200", image_post=True)],
            "has_more": True,
            "next_cursor": 55,
        },
        55: {
            "details": [FakeDetail("300")],
            "has_more": False,
            "next_cursor": 0,
        },
    }
    calls = _install_paged_resolver(monkeypatch, tmp_path, pages)

    first = py_bridge.resolve_page("post", "https://douyin.com/user/sec-user", 0, 2)
    second = py_bridge.resolve_page("post", "https://douyin.com/user/sec-user", 55, 2)

    assert calls == [("sec-user", 0, 2), ("sec-user", 55, 2)]
    assert first["success"] is True
    assert first["contract_version"] == 1
    assert first["mode"] == "post"
    assert first["next_cursor"] == 55
    assert first["has_more"] is True
    assert first["page_aweme_ids"] == ["100", "200"]
    assert first["user_profile"]["sec_user_id"] == "sec-user"
    assert first["save_dir"] == str(tmp_path / "douyin" / "post" / "tester")

    assert second["next_cursor"] is None
    assert second["has_more"] is False
    assert second["page_aweme_ids"] == ["300"]
    assert second["user_profile"] is None
    assert second["save_dir"] == first["save_dir"]
    assert [item["media_key"] for item in second["items"]] == ["300:video:0"]
