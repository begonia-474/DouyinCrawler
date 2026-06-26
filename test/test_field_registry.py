"""离线测试：DB schema 字段注册表

验证 Python Filter 的 to_db_dict()/to_dict() 输出字段
与 Rust VideoInfo/UserInfo 结构体字段一致。
"""

import pytest
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

pytestmark = [pytest.mark.offline]


# ============================================================
# 字段注册表 — 与 db.rs VideoInfo / UserInfo 结构体同步
# ============================================================

# video_info 表：61 个字段（含 aweme_id 主键 + updated_at）
VIDEO_FIELDS = [
    # 基础字段 (17)
    "aweme_id", "desc", "aweme_type",
    "author_nickname", "author_sec_uid", "author_uid",
    "create_time", "duration", "video_url", "cover_url", "music_title",
    "digg_count", "comment_count", "share_count", "collect_count",
    "mix_id", "mix_name",
    # f2 对齐 - 作者 (3)
    "author_nickname_raw", "author_short_id", "author_unique_id",
    # f2 对齐 - 内容 (5)
    "desc_raw", "is_ads", "is_story", "is_top", "is_long_video",
    # f2 对齐 - 视频 (4)
    "video_bit_rate", "animated_cover", "private_status", "is_delete",
    # f2 对齐 - 音乐 (15)
    "music_author", "music_author_raw", "music_duration", "music_id", "music_mid",
    "pgc_author", "pgc_author_title", "pgc_music_type", "music_status",
    "music_owner_handle", "music_owner_id", "music_owner_nickname",
    "music_play_url", "is_commerce_music",
    # f2 对齐 - 合集 (5)
    "mix_desc", "mix_create_time", "mix_pic_type", "mix_type", "mix_share_url",
    # f2 对齐 - 权限 (6)
    "can_comment", "can_forward", "can_share", "download_setting",
    "allow_douplus", "allow_share",
    # f2 对齐 - 统计/标签/其他 (6)
    "admire_count", "hashtag_ids", "hashtag_names", "images", "region", "is_prohibited",
    # 系统字段 (1)
    "updated_at",
]

# user_info 表：30 个字段（含 sec_user_id 主键 + updated_at）
USER_FIELDS = [
    # 基础字段 (13)
    "sec_user_id", "nickname", "uid", "avatar_url", "unique_id", "signature",
    "aweme_count", "follower_count", "following_count", "total_favorited",
    "ip_location", "live_status", "room_id",
    # f2 对齐 (17)
    "city", "country", "favoriting_count", "gender",
    "is_ban", "is_block", "is_blocked", "is_star",
    "mix_count", "mplatform_followers_count", "nickname_raw", "school_name",
    "short_id", "signature_raw", "user_age", "custom_verify",
    "updated_at",
]


class TestFieldRegistry:
    """字段注册表完整性验证"""

    def test_video_field_count(self):
        """video_info 应有 61 个字段"""
        assert len(VIDEO_FIELDS) == 61, f"预期 61 个字段，实际 {len(VIDEO_FIELDS)} 个"

    def test_user_field_count(self):
        """user_info 应有 30 个字段"""
        assert len(USER_FIELDS) == 30, f"预期 30 个字段，实际 {len(USER_FIELDS)} 个"


class TestTauriTypesConsistency:
    """验证 tauri-types.ts 与 Rust struct 字段数一致"""

    def _count_interface_fields(self, ts_content: str, interface_name: str) -> int:
        """统计 TS 接口中的字段数"""
        import re
        pattern = re.compile(
            rf"export interface {re.escape(interface_name)} \{{([^}}]+)\}}",
            re.DOTALL,
        )
        match = pattern.search(ts_content)
        if not match:
            return -1
        body = match.group(1)
        # 统计 "field: type;" 行数
        return len(re.findall(r"^\s+\w+\s*:", body, re.MULTILINE))

    def test_tauri_types_video_info_field_count(self):
        """tauri-types.ts VideoInfo 应有 61 个字段"""
        ts_path = Path(__file__).resolve().parent.parent / "src" / "lib" / "tauri-types.ts"
        if not ts_path.exists():
            pytest.skip("tauri-types.ts 不存在")
        content = ts_path.read_text(encoding="utf-8")
        count = self._count_interface_fields(content, "VideoInfo")
        assert count == 61, f"VideoInfo 字段数 {count}，预期 61"

    def test_tauri_types_user_info_field_count(self):
        """tauri-types.ts UserInfo 应有 30 个字段"""
        ts_path = Path(__file__).resolve().parent.parent / "src" / "lib" / "tauri-types.ts"
        if not ts_path.exists():
            pytest.skip("tauri-types.ts 不存在")
        content = ts_path.read_text(encoding="utf-8")
        count = self._count_interface_fields(content, "UserInfo")
        assert count == 30, f"UserInfo 字段数 {count}，预期 30"

    def test_tauri_types_video_fields_match_registry(self):
        """tauri-types.ts VideoInfo 字段名应与注册表一致"""
        ts_path = Path(__file__).resolve().parent.parent / "src" / "lib" / "tauri-types.ts"
        if not ts_path.exists():
            pytest.skip("tauri-types.ts 不存在")
        import re
        content = ts_path.read_text(encoding="utf-8")
        pattern = re.compile(r"export interface VideoInfo \{([^}]+)\}", re.DOTALL)
        match = pattern.search(content)
        assert match, "找不到 VideoInfo 接口"
        body = match.group(1)
        ts_fields = re.findall(r"^\s+(\w+)\s*:", body, re.MULTILINE)
        missing = [f for f in VIDEO_FIELDS if f not in ts_fields]
        extra = [f for f in ts_fields if f not in VIDEO_FIELDS]
        assert not missing, f"tauri-types.ts VideoInfo 缺少字段: {missing}"
        assert not extra, f"tauri-types.ts VideoInfo 多出字段: {extra}"

    def test_tauri_types_user_fields_match_registry(self):
        """tauri-types.ts UserInfo 字段名应与注册表一致"""
        ts_path = Path(__file__).resolve().parent.parent / "src" / "lib" / "tauri-types.ts"
        if not ts_path.exists():
            pytest.skip("tauri-types.ts 不存在")
        import re
        content = ts_path.read_text(encoding="utf-8")
        pattern = re.compile(r"export interface UserInfo \{([^}]+)\}", re.DOTALL)
        match = pattern.search(content)
        assert match, "找不到 UserInfo 接口"
        body = match.group(1)
        ts_fields = re.findall(r"^\s+(\w+)\s*:", body, re.MULTILINE)
        missing = [f for f in USER_FIELDS if f not in ts_fields]
        extra = [f for f in ts_fields if f not in USER_FIELDS]
        assert not missing, f"tauri-types.ts UserInfo 缺少字段: {missing}"
        assert not extra, f"tauri-types.ts UserInfo 多出字段: {extra}"

    def test_video_fields_unique(self):
        """video_info 字段名应唯一"""
        assert len(VIDEO_FIELDS) == len(set(VIDEO_FIELDS)), "video_info 字段有重复"

    def test_user_fields_unique(self):
        """user_info 字段名应唯一"""
        assert len(USER_FIELDS) == len(set(USER_FIELDS)), "user_info 字段有重复"

    def test_video_primary_key_present(self):
        """video_info 应包含 aweme_id 主键"""
        assert "aweme_id" in VIDEO_FIELDS

    def test_user_primary_key_present(self):
        """user_info 应包含 sec_user_id 主键"""
        assert "sec_user_id" in USER_FIELDS

    def test_video_updated_at_present(self):
        """video_info 应包含 updated_at 系统字段"""
        assert "updated_at" in VIDEO_FIELDS

    def test_user_updated_at_present(self):
        """user_info 应包含 updated_at 系统字段"""
        assert "updated_at" in USER_FIELDS


class TestFilterOutputFields:
    """验证 Filter 类输出字段与注册表一致"""

    def test_post_detail_filter_to_db_dict_keys(self):
        """PostDetailFilter.to_db_dict() 的键应覆盖所有 VIDEO_FIELDS（updated_at 除外）"""
        from core.filter import PostDetailFilter

        # 最小 mock 数据
        data = {
            "aweme_id": "123",
            "desc": "test",
            "aweme_type": 0,
            "create_time": 1719000000,
            "duration": 10000,
            "author": {
                "nickname": "author",
                "sec_uid": "sec123",
                "uid": "uid123",
            },
            "video": {"play_addr": {"url_list": ["http://example.com/v.mp4"]}},
            "music": {"title": "song"},
            "statistics": {
                "digg_count": 10, "comment_count": 5,
                "share_count": 2, "collect_count": 1,
            },
        }
        f = PostDetailFilter(data)
        db_dict = f.to_db_dict()

        # updated_at 是 Rust 侧系统字段，不由 Python Filter 输出
        expected = [k for k in VIDEO_FIELDS if k != "updated_at"]
        missing = [k for k in expected if k not in db_dict]
        assert not missing, f"to_db_dict() 缺少字段: {missing}"

    def test_user_profile_filter_to_dict_keys(self):
        """UserProfileFilter.to_dict() 的键应覆盖所有 USER_FIELDS（updated_at 除外）"""
        from core.filter import UserProfileFilter

        data = {
            "user": {
                "sec_uid": "sec123",
                "nickname": "test_user",
                "uid": "uid123",
                "avatar_larger": {"url_list": ["http://example.com/avatar.jpg"]},
                "unique_id": "test_id",
                "signature": "hello",
                "aweme_count": 100,
                "follower_count": 500,
                "following_count": 50,
                "total_favorited": 1000,
                "ip_location": "北京",
            }
        }
        f = UserProfileFilter(data)
        udict = f.to_dict()

        # updated_at 是 Rust 侧系统字段，不由 Python Filter 输出
        expected = [k for k in USER_FIELDS if k != "updated_at"]
        missing = [k for k in expected if k not in udict]
        assert not missing, f"to_dict() 缺少字段: {missing}"
