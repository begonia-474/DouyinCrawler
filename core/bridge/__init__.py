"""PyO3 桥接层 — Python↔Rust 接口

可通过 `from core.bridge import X` 或保持旧路径 `from core.xxx import X`（shim 兼容）。
所有非音乐 execution 导出已删除；Rust 拥有下载和直播生命周期。
"""
from core.bridge.handler import DouyinHandler
from core.bridge.py_bridge import (
    parse_video, get_live_info, resolve_live,
    resolve_single, resolve_music_urls, resolve_page,
    get_user_profile, get_user_posts, search_videos,
    get_mix_info, get_collects_list, get_collects_video_list,
    get_following_list, get_follower_list,
    get_music_collection, download_music_batch, download_music,
    get_following_live, get_comments, get_comment_replies,
    get_tab_feed, get_follow_feed, get_friend_feed,
    get_user_likes, get_post_stats,
)
from core.bridge.db_bridge import (
    save_video_info, save_user_info,
    has_user,
)
from core.bridge.parsing_context import ParsingContext, context

__all__ = [
    "DouyinHandler",
    "ParsingContext", "context",
    "parse_video", "resolve_live", "resolve_single", "resolve_music_urls", "resolve_page",
    "get_user_profile", "get_user_posts", "get_user_likes",
    "get_live_info", "get_following_live",
    "get_mix_info", "get_collects_list", "get_collects_video_list",
    "get_following_list", "get_follower_list",
    "get_music_collection", "download_music_batch", "download_music",
    "get_comments", "get_comment_replies",
    "get_tab_feed", "get_follow_feed", "get_friend_feed",
    "get_post_stats", "search_videos",
    "save_video_info", "save_user_info",
    "has_user",
]
