"""工具函数包 — 按领域拆分为 url / text / time / m3u8

所有符号通过此文件重导出，保持 `from core.utils import X` 兼容。
"""

# ── URL 解析与 ID 提取 ──
from core.utils.url import (
    extract_valid_urls,
    AwemeIdFetcher,
    SecUserIdFetcher,
    MixIdFetcher,
    WebCastIdFetcher,
    detect_url_type,
)

# ── 文本清洗与文件名格式化 ──
from core.utils.text import (
    replaceT,
    sanitize_filename,
    format_filename,
)

# ── 时间戳与日期过滤 ──
from core.utils.time import (
    timestamp_2_str,
    interval_2_timestamp,
    filter_by_date_interval,
)

# ── M3U8 直播流 ──
from core.utils.m3u8 import (
    get_segments_from_m3u8,
    get_content_length,
    get_chunk_size,
)

__all__ = [
    # url
    "extract_valid_urls",
    "AwemeIdFetcher",
    "SecUserIdFetcher",
    "MixIdFetcher",
    "WebCastIdFetcher",
    "detect_url_type",
    # text
    "replaceT",
    "sanitize_filename",
    "format_filename",
    # time
    "timestamp_2_str",
    "interval_2_timestamp",
    "filter_by_date_interval",
    # m3u8
    "get_segments_from_m3u8",
    "get_content_length",
    "get_chunk_size",
]
