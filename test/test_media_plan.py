"""Offline golden tests for shared Python media planner (issue 07)."""

import sys
from pathlib import Path
from types import SimpleNamespace

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from core.models.single_download import MediaAccessoryKindV1, MediaDownloadItemV1, MediaKindV1
from core.services.media_plan import build_media_items_v1, ordered_aweme_ids


pytestmark = [pytest.mark.offline]


class FakeDetail:
    def __init__(self, aweme_id: str, **kwargs):
        self.aweme_id = aweme_id
        self.author_nickname = kwargs.get("author_nickname", "tester")
        self.is_image_post = kwargs.get("is_image_post", False)
        self.is_prohibited = kwargs.get("is_prohibited", False)
        self.desc = kwargs.get("desc", f"desc-{aweme_id}")
        self.video_urls = kwargs.get("video_urls", [])
        self.video_url = kwargs.get("video_url", "")
        self.images_video = kwargs.get("images_video", [])
        self.images = kwargs.get("images", [])
        self.music_url = kwargs.get("music_url", f"https://cdn.example/{aweme_id}.mp3")
        self.cover_url = kwargs.get("cover_url", f"https://cdn.example/{aweme_id}-cover.jpeg")

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
            "aweme_type": 68 if self.is_image_post else 0,
            "author_nickname": self.author_nickname,
            "author_sec_uid": "MS4wLjABAAAAgolden",
            "duration": 12,
            "digg_count": 3,
        }


# ============================================================
# Video golden
# ============================================================

def test_video_single_item():
    detail = FakeDetail("100", video_urls=["https://cdn.example/100.mp4"], video_url="https://cdn.example/100.mp4")
    items = build_media_items_v1([detail], naming="{aweme_id}", folderize=False, headers={"Cookie": "test"})
    assert len(items) == 1
    item = items[0]
    assert item.media_key == "100:video:0"
    assert item.kind == MediaKindV1.VIDEO
    assert item.urls == ["https://cdn.example/100.mp4"]
    assert item.output.filename == "100_video"
    assert item.output.suffix == ".mp4"
    assert item.output.folder_name is None
    assert [a.kind for a in item.accessories] == [
        MediaAccessoryKindV1.MUSIC,
        MediaAccessoryKindV1.COVER,
        MediaAccessoryKindV1.DESCRIPTION,
    ]
    assert item.metadata.aweme_id == "100"


# ============================================================
# Static images golden
# ============================================================

def test_gallery_pure_images_compact_indices():
    detail = FakeDetail(
        "200", is_image_post=True,
        images_video=[],
        images=["https://cdn.example/200-1.webp", "", "https://cdn.example/200-3.webp"],
    )
    items = build_media_items_v1([detail], naming="{aweme_id}", folderize=False, headers={})
    assert len(items) == 2
    assert [item.media_key for item in items] == ["200:image:1", "200:image:2"]
    assert [item.urls[0] for item in items] == [
        "https://cdn.example/200-1.webp",
        "https://cdn.example/200-3.webp",
    ]
    # compact: empty URL at index 2 is skipped, no gap
    assert items[0].output.filename == "200_image_1"
    assert items[1].output.filename == "200_image_2"


# ============================================================
# Live photo + image hybrid golden
# ============================================================

def test_gallery_live_photos_before_images():
    detail = FakeDetail(
        "300", is_image_post=True,
        images_video=["https://cdn.example/300-live-1.mp4", "https://cdn.example/300-live-2.mp4"],
        images=["https://cdn.example/300-img-1.webp", "https://cdn.example/300-img-2.webp", "https://cdn.example/300-img-3.webp"],
    )
    items = build_media_items_v1([detail], naming="{aweme_id}", folderize=False, headers={})
    assert [item.kind for item in items] == [
        MediaKindV1.LIVE_PHOTO,
        MediaKindV1.LIVE_PHOTO,
        MediaKindV1.IMAGE,
        MediaKindV1.IMAGE,
        MediaKindV1.IMAGE,
    ]
    assert [item.media_key for item in items] == [
        "300:live_photo:1",
        "300:live_photo:2",
        "300:image:1",
        "300:image:2",
        "300:image:3",
    ]
    assert [item.output.filename for item in items] == [
        "300_live_1",
        "300_live_2",
        "300_image_1",
        "300_image_2",
        "300_image_3",
    ]
    assert [item.output.suffix for item in items] == [
        ".mp4",
        ".mp4",
        ".webp",
        ".webp",
        ".webp",
    ]


# ============================================================
# Accessories: gallery first item only, no description
# ============================================================

def test_gallery_accessories_only_on_first_item():
    detail = FakeDetail(
        "400", is_image_post=True,
        images_video=["https://cdn.example/400-live.mp4"],
        images=["https://cdn.example/400-img.webp"],
    )
    items = build_media_items_v1([detail], naming="{aweme_id}", folderize=False, headers={})
    assert len(items) == 2
    first_acc = {a.kind for a in items[0].accessories}
    assert first_acc == {MediaAccessoryKindV1.MUSIC, MediaAccessoryKindV1.COVER}
    assert len(items[1].accessories) == 0


def test_video_has_all_accessories():
    detail = FakeDetail("500", video_urls=["https://cdn.example/500.mp4"])
    items = build_media_items_v1([detail], naming="{aweme_id}", folderize=False, headers={})
    assert len(items) == 1
    acc_kinds = {a.kind for a in items[0].accessories}
    assert acc_kinds == {
        MediaAccessoryKindV1.MUSIC,
        MediaAccessoryKindV1.COVER,
        MediaAccessoryKindV1.DESCRIPTION,
    }


# ============================================================
# One/paged equivalence
# ============================================================

def test_shared_planner_is_deterministic_for_same_detail():
    detail = FakeDetail(
        "600", is_image_post=True,
        images_video=["https://cdn.example/600-live.mp4"],
        images=["https://cdn.example/600-img-1.webp", "https://cdn.example/600-img-2.webp"],
    )
    items_one = build_media_items_v1(
        [detail], naming="{aweme_id}_{desc}", folderize=False, headers={"Cookie": "x"}
    )
    items_paged = build_media_items_v1(
        [detail], naming="{aweme_id}_{desc}", folderize=False, headers={"Cookie": "x"}
    )
    assert items_one == items_paged


def test_one_and_post_adapters_use_identical_typed_media_items(monkeypatch, tmp_path):
    from core.bridge import py_bridge
    from core.crawler_engine import filter as filter_module
    from core.utils import AwemeIdFetcher

    detail = FakeDetail(
        "adapter-600",
        is_image_post=True,
        images_video=["https://cdn.example/adapter-live.mp4"],
        images=["https://cdn.example/adapter-image.webp"],
    )

    class FakeCrawler:
        async def __aenter__(self):
            return self

        async def __aexit__(self, _exc_type, _exc, _tb):
            return False

        async def fetch_post_detail(self, _aweme_id):
            return {"aweme_id": detail.aweme_id}

    class FakeVideoService:
        async def handle_parse_video(self, _url):
            return {"success": True}

        def _make_crawler(self):
            return FakeCrawler()

    async def fake_get_aweme_id(_url):
        return detail.aweme_id

    config = SimpleNamespace(
        cookie="cookie",
        naming="{aweme_id}_{desc}",
        download_path=tmp_path,
        app_name="douyin",
        folderize=False,
        music=True,
        cover=True,
        desc=True,
    )
    handler = SimpleNamespace(config=config, _video=FakeVideoService())
    monkeypatch.setattr(
        py_bridge, "_get_task_manager", lambda: SimpleNamespace(handler=handler)
    )
    monkeypatch.setattr(AwemeIdFetcher, "get_aweme_id", staticmethod(fake_get_aweme_id))
    monkeypatch.setattr(filter_module, "PostDetailFilter", lambda _data: detail)

    one_plan = py_bridge.resolve_urls("one", "https://example.test/video")
    post_plan = py_bridge.resolve_page(
        "post",
        "https://example.test/user",
        cursor=0,
        count=20,
        aweme_ids=[detail.aweme_id],
    )

    assert one_plan["success"] is True
    assert post_plan["success"] is True
    assert one_plan["items"] == post_plan["items"]


# ============================================================
# Unavailable: no medium yields empty list
# ============================================================

def test_prohibited_work_skipped():
    detail = FakeDetail("700", is_prohibited=True)
    items = build_media_items_v1([detail], naming="{aweme_id}", folderize=False, headers={})
    assert items == []


def test_gallery_with_all_empty_urls_yields_no_items():
    detail = FakeDetail(
        "800", is_image_post=True,
        images_video=[""],
        images=["", None],
    )
    items = build_media_items_v1([detail], naming="{aweme_id}", folderize=False, headers={})
    assert items == []


def test_video_without_urls_yields_no_items():
    detail = FakeDetail("900", video_urls=[], video_url="")
    items = build_media_items_v1([detail], naming="{aweme_id}", folderize=False, headers={})
    assert items == []


# ============================================================
# Ordered aweme_ids
# ============================================================

def test_ordered_aweme_ids_skips_empty():
    items = []
    assert ordered_aweme_ids(items) == []

    items = build_media_items_v1(
        [
            FakeDetail("a", video_urls=["https://a.mp4"]),
            FakeDetail("c", is_image_post=True, images_video=["https://c-live.mp4"], images=["https://c-img.webp"]),
        ],
        naming="{aweme_id}", folderize=False, headers={},
    )
    assert ordered_aweme_ids(items) == ["a", "c"]


# ============================================================
# URL fallback: always non-empty list
# ============================================================

def test_video_urls_is_list_not_scalar():
    detail = FakeDetail("1000", video_urls=["https://primary.mp4", "https://fallback.mp4"])
    items = build_media_items_v1([detail], naming="{aweme_id}", folderize=False, headers={})
    assert len(items) == 1
    assert items[0].urls == ["https://primary.mp4", "https://fallback.mp4"]
    assert isinstance(items[0].urls, list)
