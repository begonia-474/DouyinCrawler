"""Offline contract tests for Python -> Rust paged download plans."""

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
        ("mode", "unknown"),
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


# ============================================================
# Mode-specific adapter tests: like, mix, collects
# ============================================================


def _install_like_resolver(monkeypatch, tmp_path: Path, pages: dict[int, dict]):
    calls = []

    class FakeCrawler:
        async def fetch_user_profile(self, sec_user_id):
            assert sec_user_id == "sec-user-like"
            return {"profile": True}

        async def fetch_user_favorite(self, sec_user_id, cursor, count):
            calls.append(("favorite", sec_user_id, cursor, count))
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
            return {"sec_user_id": "sec-user-like", "nickname": "liked-owner"}

    async def get_sec_user_id(_url):
        return "sec-user-like"

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


def _install_mix_resolver(monkeypatch, tmp_path: Path, pages: dict[int, dict]):
    calls = []

    class FakeCrawler:
        async def fetch_mix_aweme(self, mix_id, cursor, count):
            calls.append(("mix", mix_id, cursor, count))
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

    async def get_mix_id(_url):
        return "mix-123"

    handler = SimpleNamespace(
        config=SimpleNamespace(
            cookie="sessionid=test",
            naming="{aweme_id}",
            download_path=tmp_path,
            app_name="douyin",
            folderize=False,
        ),
        _user=SimpleNamespace(_make_crawler=make_crawler),
        _mix=SimpleNamespace(_make_crawler=make_crawler),
    )
    monkeypatch.setattr(py_bridge, "_get_task_manager", lambda: SimpleNamespace(handler=handler))
    monkeypatch.setattr("core.utils.MixIdFetcher.get_mix_id", get_mix_id)
    monkeypatch.setattr("core.crawler_engine.filter.UserPostFilter", FakePostFilter)
    return calls


def _install_collects_resolver(monkeypatch, tmp_path: Path, pages: dict[int, dict]):
    calls = []

    class FakeCrawler:
        async def fetch_user_collects_video(self, collects_id, cursor, count):
            calls.append(("collects", collects_id, cursor, count))
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

    handler = SimpleNamespace(
        config=SimpleNamespace(
            cookie="sessionid=test",
            naming="{aweme_id}",
            download_path=tmp_path,
            app_name="douyin",
            folderize=False,
        ),
        _user=SimpleNamespace(_make_crawler=make_crawler),
        _collection=SimpleNamespace(_make_crawler=make_crawler),
    )
    monkeypatch.setattr(py_bridge, "_get_task_manager", lambda: SimpleNamespace(handler=handler))
    monkeypatch.setattr("core.crawler_engine.filter.UserPostFilter", FakePostFilter)
    return calls


def test_resolve_like_returns_typed_plan(monkeypatch, tmp_path):
    pages = {
        0: {
            "details": [FakeDetail("like-1"), FakeDetail("like-2")],
            "has_more": True,
            "next_cursor": 99,
        },
        99: {
            "details": [FakeDetail("like-3")],
            "has_more": False,
            "next_cursor": 0,
        },
    }
    calls = _install_like_resolver(monkeypatch, tmp_path, pages)

    first = py_bridge.resolve_page("like", "https://douyin.com/user/sec-like", 0, 2)
    second = py_bridge.resolve_page("like", "https://douyin.com/user/sec-like", 99, 2)

    assert calls == [("favorite", "sec-user-like", 0, 2), ("favorite", "sec-user-like", 99, 2)]
    assert first["success"] is True
    assert first["contract_version"] == 1
    assert first["mode"] == "like"
    assert first["next_cursor"] == 99
    assert first["has_more"] is True
    assert first["page_aweme_ids"] == ["like-1", "like-2"]
    assert first["save_dir"] == str(tmp_path / "douyin" / "like" / "liked-owner")
    assert first["user_profile"]["nickname"] == "liked-owner"

    assert second["next_cursor"] is None
    assert second["has_more"] is False
    assert second["page_aweme_ids"] == ["like-3"]
    assert second["user_profile"] is None
    assert [item["media_key"] for item in second["items"]] == ["like-3:video:0"]


def test_resolve_like_uses_target_profile_for_stable_cross_page_directory(
    monkeypatch, tmp_path
):
    pages = {
        0: {
            "details": [FakeDetail("like-a", author_nickname="video-author-a")],
            "has_more": True,
            "next_cursor": 99,
        },
        99: {
            "details": [FakeDetail("like-b", author_nickname="video-author-b")],
            "has_more": False,
            "next_cursor": 0,
        },
    }
    _install_like_resolver(monkeypatch, tmp_path, pages)

    first = py_bridge.resolve_page("like", "https://douyin.com/user/sec-like", 0, 1)
    second = py_bridge.resolve_page("like", "https://douyin.com/user/sec-like", 99, 1)

    expected = str(tmp_path / "douyin" / "like" / "liked-owner")
    assert first["save_dir"] == expected
    assert second["save_dir"] == expected
    assert first["user_profile"]["nickname"] == "liked-owner"
    assert second["user_profile"] is None


def test_resolve_mix_returns_typed_plan(monkeypatch, tmp_path):
    pages = {
        0: {
            "details": [FakeDetail("mix-1")],
            "has_more": False,
            "next_cursor": 0,
        },
    }
    calls = _install_mix_resolver(monkeypatch, tmp_path, pages)

    result = py_bridge.resolve_page("mix", "https://douyin.com/mix/abc", 0, 2)

    assert calls == [("mix", "mix-123", 0, 2)]
    assert result["success"] is True
    assert result["contract_version"] == 1
    assert result["mode"] == "mix"
    assert result["next_cursor"] is None
    assert result["has_more"] is False
    assert result["page_aweme_ids"] == ["mix-1"]
    assert result["user_profile"] is None
    assert result["save_dir"] == str(tmp_path / "douyin" / "mix" / "tester")
    assert [item["media_key"] for item in result["items"]] == ["mix-1:video:0"]


def test_resolve_collects_returns_typed_plan(monkeypatch, tmp_path):
    pages = {
        0: {
            "details": [FakeDetail("col-1"), FakeDetail("col-2", image_post=True)],
            "has_more": False,
            "next_cursor": 0,
        },
    }
    calls = _install_collects_resolver(monkeypatch, tmp_path, pages)

    result = py_bridge.resolve_page("collects", "collects-abc-123", 0, 2)

    assert calls == [("collects", "collects-abc-123", 0, 2)]
    assert result["success"] is True
    assert result["contract_version"] == 1
    assert result["mode"] == "collects"
    assert result["next_cursor"] is None
    assert result["has_more"] is False
    assert result["page_aweme_ids"] == ["col-1", "col-2"]
    assert result["user_profile"] is None
    assert result["save_dir"] == str(
        tmp_path / "douyin" / "collects" / "collects-abc-123"
    )
    media_keys = [item["media_key"] for item in result["items"]]
    assert media_keys == ["col-1:video:0", "col-2:live_photo:1", "col-2:image:1", "col-2:image:2"]


def test_resolve_collects_uses_collection_id_for_stable_cross_page_directory(
    monkeypatch, tmp_path
):
    pages = {
        0: {
            "details": [FakeDetail("col-a", author_nickname="video-author-a")],
            "has_more": True,
            "next_cursor": 20,
        },
        20: {
            "details": [FakeDetail("col-b", author_nickname="video-author-b")],
            "has_more": False,
            "next_cursor": 0,
        },
    }
    _install_collects_resolver(monkeypatch, tmp_path, pages)

    first = py_bridge.resolve_page("collects", "collects-abc-123", 0, 1)
    second = py_bridge.resolve_page("collects", "collects-abc-123", 20, 1)

    expected = str(tmp_path / "douyin" / "collects" / "collects-abc-123")
    assert first["save_dir"] == expected
    assert second["save_dir"] == expected


def test_resolve_like_mix_collects_have_no_items_built_from_details(monkeypatch, tmp_path):
    """Verify that like/mix/collects do not use _build_items_from_details (already removed)."""
    pages = {
        0: {
            "details": [FakeDetail("vid-1"), FakeDetail("vid-2")],
            "has_more": False,
            "next_cursor": 0,
        },
    }
    _install_like_resolver(monkeypatch, tmp_path, pages)
    result = py_bridge.resolve_page("like", "https://douyin.com/user/sec-like", 0, 2)
    assert result["contract_version"] == 1
    assert all(item["media_key"] for item in result["items"])
    assert all(item["kind"] in ("video", "image", "live_photo") for item in result["items"])


def test_resolve_like_page_aweme_ids_includes_all_source_ids(monkeypatch, tmp_path):
    prohibited = FakeDetail("bad", prohibited=True)
    normal = FakeDetail("good")
    pages = {
        0: {
            "details": [prohibited, normal],
            "has_more": False,
            "next_cursor": 0,
        },
    }
    _install_like_resolver(monkeypatch, tmp_path, pages)
    result = py_bridge.resolve_page("like", "https://douyin.com/user/sec-like", 0, 2)
    assert result["page_aweme_ids"] == ["bad", "good"]
    item_ids = [item["aweme_id"] for item in result["items"]]
    assert "bad" not in item_ids
    assert "good" in item_ids


def test_resolve_mix_has_more_next_cursor_contract(monkeypatch, tmp_path):
    """mix mode: has_more=True requires next_cursor; has_more=False requires next_cursor=None."""
    pages = {
        0: {
            "details": [FakeDetail("mix-page1")],
            "has_more": True,
            "next_cursor": 42,
        },
    }
    _install_mix_resolver(monkeypatch, tmp_path, pages)
    first = py_bridge.resolve_page("mix", "https://douyin.com/mix/abc", 0, 2)
    assert first["has_more"] is True
    assert first["next_cursor"] == 42

    pages2 = {
        42: {
            "details": [FakeDetail("mix-last")],
            "has_more": False,
            "next_cursor": 0,
        },
    }
    _install_mix_resolver(monkeypatch, tmp_path, pages2)
    last = py_bridge.resolve_page("mix", "https://douyin.com/mix/abc", 42, 2)
    assert last["has_more"] is False
    assert last["next_cursor"] is None


def _install_fast_path(monkeypatch, tmp_path):
    """Install handler with _video._make_crawler for single-aweme fast path tests."""

    def _make_detail_data(aweme_id):
        return {
            "aweme_detail": {
                "aweme_id": aweme_id,
                "desc": f"desc-{aweme_id}",
                "create_time": 1700000000,
                "aweme_type": 0,
                "author": {
                    "nickname": "tester",
                    "sec_uid": "MS4w.secret",
                    "uid": "12345",
                },
                "video": {
                    "bit_rate": [{
                        "play_addr": {
                            "url_list": [f"https://cdn/{aweme_id}.mp4"]
                        }
                    }],
                    "origin_cover": {"url_list": []},
                },
                "images": None,
            }
        }

    handler = SimpleNamespace(
        config=SimpleNamespace(
            cookie="sessionid=test",
            naming="{aweme_id}",
            download_path=tmp_path,
            app_name="douyin",
            folderize=False,
        ),
        _video=SimpleNamespace(),
    )

    class FakeCrawler:
        async def fetch_post_detail(self, aweme_id):
            return _make_detail_data(aweme_id)

    @asynccontextmanager
    async def make_crawler():
        yield FakeCrawler()

    handler._video._make_crawler = make_crawler
    monkeypatch.setattr(py_bridge, "_get_task_manager", lambda: SimpleNamespace(handler=handler))
    return handler


def test_resolve_like_single_aweme_fast_path_typed(monkeypatch, tmp_path):
    """Single aweme_id fast path returns typed plan for like mode."""
    _install_fast_path(monkeypatch, tmp_path)
    result = py_bridge.resolve_page("like", "https://douyin.com/user/sec-like", 0, 2, aweme_ids=["fast-like"])
    assert result["success"] is True
    assert result["contract_version"] == 1
    assert result["mode"] == "like"
    assert result["has_more"] is False
    assert result["page_aweme_ids"] == ["fast-like"]
    items = result["items"]
    assert len(items) == 1
    assert items[0]["media_key"] == "fast-like:video:0"


def test_resolve_post_single_aweme_fast_path_typed(monkeypatch, tmp_path):
    """Single aweme_id fast path returns typed plan for post mode."""
    _install_fast_path(monkeypatch, tmp_path)
    result = py_bridge.resolve_page("post", "https://douyin.com/user/sec-user", 0, 2, aweme_ids=["fast-post"])
    assert result["success"] is True
    assert result["contract_version"] == 1
    assert result["mode"] == "post"
    assert result["has_more"] is False
    assert result["page_aweme_ids"] == ["fast-post"]
    items = result["items"]
    assert len(items) == 1
    assert items[0]["media_key"] == "fast-post:video:0"


def test_resolve_mix_single_aweme_fast_path_typed(monkeypatch, tmp_path):
    """Single aweme_id fast path returns typed plan for mix mode."""
    _install_fast_path(monkeypatch, tmp_path)
    result = py_bridge.resolve_page("mix", "https://douyin.com/mix/abc", 0, 2, aweme_ids=["fast-mix"])
    assert result["success"] is True
    assert result["contract_version"] == 1
    assert result["mode"] == "mix"
    assert result["has_more"] is False
    assert result["page_aweme_ids"] == ["fast-mix"]
    items = result["items"]
    assert len(items) == 1
    assert items[0]["media_key"] == "fast-mix:video:0"


def test_resolve_collects_single_aweme_fast_path_typed(monkeypatch, tmp_path):
    """Single aweme_id fast path returns typed plan for collects mode."""
    _install_fast_path(monkeypatch, tmp_path)
    result = py_bridge.resolve_page("collects", "collects-id", 0, 2, aweme_ids=["fast-col"])
    assert result["success"] is True
    assert result["contract_version"] == 1
    assert result["mode"] == "collects"
    assert result["has_more"] is False
    assert result["page_aweme_ids"] == ["fast-col"]
    items = result["items"]
    assert len(items) == 1
    assert items[0]["media_key"] == "fast-col:video:0"
