"""离线测试：URL 解析与工具函数 — 无需网络"""

import pytest
import sys
from pathlib import Path

pytestmark = [pytest.mark.offline]

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from core.utils import (
    extract_valid_urls, sanitize_filename, format_filename,
    filter_by_date_interval, interval_2_timestamp, detect_url_type,
)


# ============================================================
# extract_valid_urls
# ============================================================

class TestExtractValidUrls:
    def test_plain_url(self):
        assert extract_valid_urls("https://www.douyin.com/video/123") == "https://www.douyin.com/video/123"

    def test_share_text(self):
        text = "0.53 复制打开抖音 https://v.douyin.com/ixYxBfoo/ 复制此链接"
        assert extract_valid_urls(text) == "https://v.douyin.com/ixYxBfoo/"

    def test_trailing_punctuation(self):
        assert extract_valid_urls("https://v.douyin.com/xxx/。") == "https://v.douyin.com/xxx/"
        assert extract_valid_urls("https://v.douyin.com/xxx/！") == "https://v.douyin.com/xxx/"
        assert extract_valid_urls("https://v.douyin.com/xxx/)") == "https://v.douyin.com/xxx/"

    def test_no_url(self):
        assert extract_valid_urls("没有链接的文本") == "没有链接的文本"

    def test_multiple_urls(self):
        text = "https://first.com https://second.com"
        result = extract_valid_urls(text)
        assert result == "https://first.com"


# ============================================================
# detect_url_type
# ============================================================

class TestDetectUrlType:
    def test_video_url(self):
        assert detect_url_type("https://www.douyin.com/video/123") == "one"

    def test_note_url(self):
        assert detect_url_type("https://www.douyin.com/note/123") == "one"

    def test_user_url(self):
        assert detect_url_type("https://www.douyin.com/user/abc123") == "post"

    def test_mix_url(self):
        assert detect_url_type("https://www.douyin.com/collection/123") == "mix"

    def test_live_url(self):
        assert detect_url_type("https://live.douyin.com/12345") == "live"


# ============================================================
# sanitize_filename
# ============================================================

class TestSanitizeFilename:
    def test_normal(self):
        assert sanitize_filename("正常文件名") == "正常文件名"

    def test_special_chars(self):
        assert sanitize_filename('file:name*test?"<>|') == "file_name_test_____"

    def test_max_length(self):
        result = sanitize_filename("a" * 300, max_len=80)
        assert len(result) == 80

    def test_byte_truncation(self):
        """中文按字节截断，不产生乱码"""
        result = sanitize_filename("中" * 10, max_len=20)
        assert result == "中" * 6

    def test_empty(self):
        """空字符串返回空串，不强行补 untitled"""
        assert sanitize_filename("") == ""
        assert sanitize_filename("...") == ""

    def test_emoji_filtered(self):
        assert sanitize_filename("视频🔥标题") == "视频标题"
        assert sanitize_filename("🎮game") == "game"

    def test_newlines(self):
        assert sanitize_filename("file\nname\r") == "file_name_"


# ============================================================
# format_filename
# ============================================================

class TestFormatFilename:
    def test_basic_template(self):
        data = {
            "create_time": 1719000000,
            "desc": "测试视频",
            "author": "作者名",
            "aweme_id": "123456",
            "author_uid": "789",
        }
        result = format_filename("{create}_{desc}", data)
        assert "测试视频" in result
        assert "_" in result

    def test_all_variables(self):
        data = {
            "create_time": 1719000000,
            "desc": "desc",
            "author": "author",
            "aweme_id": "aid",
            "author_uid": "uid",
        }
        result = format_filename("{create}_{desc}_{nickname}_{aweme_id}_{uid}", data)
        assert "desc" in result
        assert "author" in result
        assert "aid" in result
        assert "uid" in result

    def test_caption_alias(self):
        """caption 应与 desc 相同"""
        data = {"create_time": 0, "desc": "test_desc", "author": "", "aweme_id": "", "author_uid": ""}
        result = format_filename("{caption}", data)
        assert result == "test_desc"

    def test_empty_data(self):
        result = format_filename("{desc}_{aweme_id}", {})
        assert "untitled" not in result

    def test_default_template_empty_desc(self):
        """默认模板 + 空 desc：时间戳后无多余下划线"""
        data = {"create_time": 1719000000, "desc": "", "author": "", "aweme_id": "123", "author_uid": ""}
        result = format_filename("{create}_{desc}", data)
        assert "untitled" not in result
        assert not result.endswith('_')


# ============================================================
# filter_by_date_interval
# ============================================================

class TestFilterByDateInterval:
    def test_empty_interval(self):
        items = [{"create_time": 1719000000}]
        assert filter_by_date_interval(items, "") == items
        assert filter_by_date_interval(items, "all") == items

    def test_dict_items(self):
        items = [
            {"create_time": 1719000000, "name": "in"},   # 2024-06-22
            {"create_time": 1609459200, "name": "out"},  # 2021-01-01
        ]
        result = filter_by_date_interval(items, "2024-01-01|2024-12-31")
        assert len(result) == 1
        assert result[0]["name"] == "in"

    def test_object_items(self):
        """兼容对象属性访问"""
        class Item:
            def __init__(self, ct):
                self.create_time = ct
        items = [Item(1719000000), Item(1609459200)]
        result = filter_by_date_interval(items, "2024-01-01|2024-12-31")
        assert len(result) == 1

    def test_millisecond_timestamps(self):
        """毫秒时间戳也应正确过滤"""
        items = [{"create_time": 1719000000000}]  # 毫秒
        result = filter_by_date_interval(items, "2024-01-01|2024-12-31")
        assert len(result) == 1

    def test_invalid_interval(self):
        items = [{"create_time": 1719000000}]
        result = filter_by_date_interval(items, "invalid")
        assert result == items

    def test_custom_field(self):
        items = [{"publish_time": 1719000000}]
        result = filter_by_date_interval(items, "2024-01-01|2024-12-31", field="publish_time")
        assert len(result) == 1


# ============================================================
# interval_2_timestamp
# ============================================================

class TestInterval2Timestamp:
    def test_start(self):
        ts = interval_2_timestamp("2024-06-22|2024-06-23", "start")
        assert ts > 0
        # 应该是 2024-06-22 00:00:00 的毫秒时间戳

    def test_end(self):
        ts = interval_2_timestamp("2024-06-22|2024-06-23", "end")
        assert ts > 0
        # end 应该比 start 大（包含全天）

    def test_invalid(self):
        assert interval_2_timestamp("invalid", "start") == 0
        assert interval_2_timestamp("a|b|c", "start") == 0
