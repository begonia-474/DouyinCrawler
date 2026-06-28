"""PyO3 桥接层 — Python↔Rust 接口

可通过 `from core.bridge import X` 或保持旧路径 `from core.xxx import X`（shim 兼容）。
"""
from core.bridge.handler import DouyinHandler
from core.bridge.py_bridge import (
    parse_video, download_video, get_live_info,
    download_batch, start_download,
    get_user_profile, get_user_posts, search_videos,
    get_mix_info, get_collects_list, get_collects_video_list,
    get_following_list, get_follower_list,
    get_music_collection, download_music_batch, download_music,
    get_following_live, get_comments, get_comment_replies,
    get_tab_feed, get_follow_feed, get_friend_feed,
    get_user_likes, get_post_stats,
    start_live_record, stop_live_record, get_live_status,
)
from core.bridge.db_bridge import (
    save_download_record, save_video_info, save_user_info,
    save_live_record, has_user,
)
from core.bridge.events import emit, set_emit_func

__all__ = [
    "DouyinHandler",
    "parse_video", "download_video", "start_download",
    "download_batch", "start_live_record", "stop_live_record",
    "get_user_profile", "get_user_posts", "get_user_likes",
    "get_live_info", "get_live_status", "get_following_live",
    "get_mix_info", "get_collects_list", "get_collects_video_list",
    "get_following_list", "get_follower_list",
    "get_music_collection", "download_music_batch", "download_music",
    "get_comments", "get_comment_replies",
    "get_tab_feed", "get_follow_feed", "get_friend_feed",
    "get_post_stats", "search_videos",
    "emit", "set_emit_func",
    "save_download_record", "save_video_info", "save_user_info",
    "save_live_record", "has_user",
]
