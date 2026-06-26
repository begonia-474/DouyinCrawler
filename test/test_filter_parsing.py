"""离线测试：Filter 类解析 — 使用固定 JSON fixture，无需网络"""

import pytest
import json
import sys
from pathlib import Path

pytestmark = [pytest.mark.offline]

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from core.filter import (
    PostDetailFilter, UserProfileFilter, UserPostFilter,
    UserCollectsFilter, UserMusicCollectionFilter,
    UserLiveFilter, PostCommentFilter, FollowingUserLiveFilter,
)


# ============================================================
# Fixtures — 模拟 API 返回的最小 JSON
# ============================================================

@pytest.fixture
def video_detail_data():
    """模拟 fetch_post_detail 返回的最小数据"""
    return {
        "aweme_detail": {
            "aweme_id": "7650450403901017571",
            "aweme_type": 0,
            "desc": "测试视频描述",
            "create_time": 1719000000,
            "author": {
                "nickname": "测试作者",
                "uid": "1234567890",
                "sec_uid": "MS4wLjABAAAA_test_sec_uid",
                "short_id": "99999",
                "unique_id": "test_user",
                "avatar_thumb": {"url_list": ["https://example.com/avatar.jpg"]},
                "signature": "测试签名",
                "follower_count": 1000,
                "aweme_count": 50,
                "following_count": 200,
                "total_favorited": 5000,
                "ip_location": "北京",
            },
            "video": {
                "duration": 60000,
                "play_addr": {"url_list": ["https://example.com/video.mp4"]},
                "origin_cover": {"url_list": ["https://example.com/cover.jpg"]},
                "bit_rate": [
                    {"play_addr": {"url_list": ["https://example.com/video_hd.mp4"]}}
                ],
            },
            "music": {
                "title": "测试音乐",
                "author": "音乐作者",
                "play_url": {"url_list": ["https://example.com/music.mp3"]},
                "duration": 180,
                "id": "98765",
                "mid": "M123",
                "status": 1,
            },
            "statistics": {
                "digg_count": 500,
                "comment_count": 100,
                "share_count": 50,
                "collect_count": 30,
                "admire_count": 10,
            },
            "status": {
                "is_prohibited": 0,
                "can_comment": 1,
                "can_forward": 1,
                "allow_share": 1,
                "download_setting": 0,
                "allow_douplus": 0,
                "private_status": 0,
                "is_delete": 0,
            },
            "mix_info": {
                "mix_id": "MIX123",
                "mix_name": "测试合集",
                "mix_desc": "合集描述",
                "mix_create_time": 1718000000,
                "mix_pic_type": 0,
                "mix_type": 0,
                "share_info": {"share_url": "https://example.com/share"},
            },
            "text_extra": [
                {"hashtag_id": "100", "hashtag_name": "测试标签"},
                {"hashtag_id": "200", "hashtag_name": "抖音"},
            ],
            "region": "CN",
        }
    }


@pytest.fixture
def user_profile_data():
    """模拟 fetch_user_profile 返回的最小数据"""
    return {
        "user": {
            "nickname": "测试用户",
            "uid": "1234567890",
            "sec_uid": "MS4wLjABAAAA_test_sec_uid",
            "avatar_larger": {"url_list": ["https://example.com/avatar_lg.jpg"]},
            "aweme_count": 100,
            "follower_count": 5000,
            "following_count": 300,
            "total_favorited": 20000,
            "signature": "用户签名",
            "ip_location": "上海",
            "city": "上海",
            "country": "中国",
            "favoriting_count": 50,
            "gender": 1,
            "status": {
                "is_ban": 0,
                "is_block": 0,
                "is_blocked": 0,
                "is_star": 0,
            },
            "mix_count": 5,
            "mplatform_followers_count": 8000,
            "school_name": "测试大学",
            "short_id": "88888",
            "user_age": 25,
            "custom_verify": "认证用户",
            "unique_id": "testuser123",
            "live_status": 0,
            "room_id": "",
        }
    }


@pytest.fixture
def user_post_list_data():
    """模拟 fetch_user_post 返回的分页数据"""
    return {
        "aweme_list": [
            {"aweme_id": "001", "desc": "视频1", "create_time": 1719000000},
            {"aweme_id": "002", "desc": "视频2", "create_time": 1719001000},
        ],
        "max_cursor": 1719001000,
        "has_more": 1,
    }


@pytest.fixture
def user_collects_data():
    """模拟收藏夹列表数据"""
    return {
        "collects_list": [
            {"collects_id": "C001", "collects_name": "收藏夹1", "total_number": 10},
            {"collects_id": "C002", "collects_name": "收藏夹2", "total_number": 20},
        ],
        "has_more": 0,
        "cursor": 100,
    }


@pytest.fixture
def user_music_data():
    """模拟音乐收藏数据"""
    return {
        "mc_list": [
            {
                "id": "M001",
                "mid": "mid001",
                "title": "歌曲1",
                "author": "歌手1",
                "owner_nickname": " owner1",
                "duration": 200,
                "play_url": {"url_list": ["https://example.com/song1.mp3"]},
                "cover_hd": {"url_list": ["https://example.com/cover1.jpg"]},
            },
        ],
        "has_more": 1,
        "cursor": 50,
    }


@pytest.fixture
def live_data():
    """模拟直播信息数据"""
    return {
        "data": {
            "data": [
                {
                    "id_str": "ROOM123",
                    "status": 2,
                    "title": "测试直播",
                    "cover": {"url_list": ["https://example.com/live_cover.jpg"]},
                    "stats": {"user_count_str": "1.2万"},
                    "stream_url": {
                        "flv_pull_url": {"FULL_HD1": "https://example.com/live.flv"},
                        "hls_pull_url_map": {"FULL_HD1": "https://example.com/live.m3u8"},
                    },
                    "owner": {"nickname": "主播"},
                }
            ]
        }
    }


@pytest.fixture
def comment_data():
    """模拟评论数据"""
    return {
        "comments": [
            {"cid": "C001", "text": "好视频！"},
            {"cid": "C002", "text": "赞"},
        ],
        "has_more": 1,
        "cursor": 200,
    }


# ============================================================
# PostDetailFilter 测试
# ============================================================

class TestPostDetailFilter:
    """PostDetailFilter 离线测试"""

    def test_basic_fields(self, video_detail_data):
        f = PostDetailFilter(video_detail_data)
        assert f.aweme_id == "7650450403901017571"
        assert f.aweme_type == 0
        assert f.desc == "测试视频描述"
        assert f.create_time == 1719000000

    def test_author_fields(self, video_detail_data):
        f = PostDetailFilter(video_detail_data)
        assert f.author_nickname == "测试作者"
        assert f.author_uid == "1234567890"
        assert f.author_sec_uid == "MS4wLjABAAAA_test_sec_uid"
        assert f.author_short_id == "99999"
        assert f.author_unique_id == "test_user"
        assert f.author_signature == "测试签名"
        assert f.author_follower_count == 1000
        assert f.author_aweme_count == 50

    def test_video_fields(self, video_detail_data):
        f = PostDetailFilter(video_detail_data)
        assert f.duration == 60000
        assert "video_hd.mp4" in f.video_url
        assert f.cover_url == "https://example.com/cover.jpg"

    def test_music_fields(self, video_detail_data):
        f = PostDetailFilter(video_detail_data)
        assert f.music_title == "测试音乐"
        assert f.music_author == "音乐作者"
        assert f.music_id == "98765"
        assert f.music_mid == "M123"

    def test_statistics(self, video_detail_data):
        f = PostDetailFilter(video_detail_data)
        assert f.digg_count == 500
        assert f.comment_count == 100
        assert f.share_count == 50
        assert f.collect_count == 30
        assert f.admire_count == 10

    def test_mix_info(self, video_detail_data):
        f = PostDetailFilter(video_detail_data)
        assert f.mix_id == "MIX123"
        assert f.mix_name == "测试合集"
        assert f.mix_desc == "合集描述"

    def test_hashtags(self, video_detail_data):
        f = PostDetailFilter(video_detail_data)
        ids = json.loads(f.hashtag_ids)
        names = json.loads(f.hashtag_names)
        assert ids == ["100", "200"]
        assert names == ["测试标签", "抖音"]

    def test_to_dict(self, video_detail_data):
        f = PostDetailFilter(video_detail_data)
        d = f.to_dict()
        assert d["aweme_id"] == "7650450403901017571"
        assert d["author"] == "测试作者"
        assert "video_url" in d

    def test_to_db_dict_field_count(self, video_detail_data):
        """to_db_dict 应返回足够多的字段（至少 50 个）"""
        f = PostDetailFilter(video_detail_data)
        db = f.to_db_dict()
        assert len(db) >= 50, f"to_db_dict 返回 {len(db)} 个字段，预期 >= 50"

    def test_to_db_dict_has_expected_fields(self, video_detail_data):
        """to_db_dict 应包含所有关键字段"""
        f = PostDetailFilter(video_detail_data)
        db = f.to_db_dict()
        expected = [
            "aweme_id", "desc", "aweme_type", "author_nickname", "author_sec_uid",
            "create_time", "duration", "video_url", "cover_url", "music_title",
            "digg_count", "comment_count", "share_count", "collect_count",
            "mix_id", "mix_name", "author_short_id", "author_unique_id",
            "music_author", "music_id", "images", "region",
        ]
        for field in expected:
            assert field in db, f"缺少字段: {field}"

    def test_empty_data(self):
        """空数据不应崩溃"""
        f = PostDetailFilter({})
        assert f.aweme_id == ""
        assert f.author_nickname == ""
        assert f.digg_count == 0

    def test_image_post(self):
        """图集类型检测"""
        data = {"aweme_detail": {"aweme_type": 68, "aweme_id": "123"}}
        f = PostDetailFilter(data)
        assert f.is_image_post is True

    def test_images_extraction(self):
        """图集 URL 提取"""
        data = {
            "aweme_detail": {
                "aweme_id": "123",
                "images": [
                    {"url_list": ["https://example.com/img1.jpg"]},
                    {"url_list": ["https://example.com/img2.jpg"]},
                ],
            }
        }
        f = PostDetailFilter(data)
        assert len(f.images) == 2
        assert f.images[0] == "https://example.com/img1.jpg"


# ============================================================
# UserProfileFilter 测试
# ============================================================

class TestUserProfileFilter:
    """UserProfileFilter 离线测试"""

    def test_basic_fields(self, user_profile_data):
        f = UserProfileFilter(user_profile_data)
        assert f.nickname == "测试用户"
        assert f.uid == "1234567890"
        assert f.sec_user_id == "MS4wLjABAAAA_test_sec_uid"

    def test_counts(self, user_profile_data):
        f = UserProfileFilter(user_profile_data)
        assert f.aweme_count == 100
        assert f.follower_count == 5000
        assert f.following_count == 300
        assert f.total_favorited == 20000

    def test_to_dict_field_count(self, user_profile_data):
        """to_dict 应返回至少 25 个字段"""
        f = UserProfileFilter(user_profile_data)
        d = f.to_dict()
        assert len(d) >= 25, f"to_dict 返回 {len(d)} 个字段，预期 >= 25"

    def test_to_dict_has_expected_fields(self, user_profile_data):
        f = UserProfileFilter(user_profile_data)
        d = f.to_dict()
        expected = [
            "nickname", "uid", "sec_user_id", "avatar_url",
            "aweme_count", "follower_count", "following_count",
            "signature", "unique_id", "live_status",
        ]
        for field in expected:
            assert field in d, f"缺少字段: {field}"

    def test_empty_data(self):
        f = UserProfileFilter({})
        assert f.nickname == ""
        assert f.sec_user_id == ""
        assert f.follower_count == 0


# ============================================================
# 其他 Filter 测试
# ============================================================

class TestUserPostFilter:
    def test_pagination(self, user_post_list_data):
        f = UserPostFilter(user_post_list_data)
        assert len(f.aweme_list) == 2
        assert f.has_more is True
        assert f.max_cursor == 1719001000

    def test_get_video_list(self, user_post_list_data):
        f = UserPostFilter(user_post_list_data)
        videos = f.get_video_list()
        assert len(videos) == 2
        assert isinstance(videos[0], PostDetailFilter)
        assert videos[0].aweme_id == "001"


class TestUserCollectsFilter:
    def test_to_list(self, user_collects_data):
        f = UserCollectsFilter(user_collects_data)
        lst = f.to_list()
        assert len(lst) == 2
        assert lst[0]["id"] == "C001"
        assert lst[0]["name"] == "收藏夹1"
        assert lst[0]["count"] == 10


class TestUserMusicCollectionFilter:
    def test_to_list(self, user_music_data):
        f = UserMusicCollectionFilter(user_music_data)
        lst = f.to_list()
        assert len(lst) == 1
        assert lst[0]["music_id"] == "M001"
        assert lst[0]["title"] == "歌曲1"
        assert lst[0]["play_url"] == "https://example.com/song1.mp3"


class TestUserLiveFilter:
    def test_live_info(self, live_data):
        f = UserLiveFilter(live_data)
        assert f.room_id == "ROOM123"
        assert f.live_status == 2
        assert f.is_live is True
        assert f.live_title == "测试直播"
        assert f.nickname == "主播"

    def test_stream_urls(self, live_data):
        f = UserLiveFilter(live_data)
        assert "FULL_HD1" in f.flv_pull_url
        assert "FULL_HD1" in f.m3u8_pull_url


class TestPostCommentFilter:
    def test_comments(self, comment_data):
        f = PostCommentFilter(comment_data)
        assert len(f.comments) == 2
        assert f.has_more is True
        assert f.cursor == 200
