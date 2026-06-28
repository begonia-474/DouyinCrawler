"""Python↔Rust 桥接响应契约 — 跨语言类型边界

每个 py_bridge 函数对应一个 *Result dataclass。
Rust 侧有对应的 serde struct（定义在 src-tauri/src/python/handler.rs 或 error.rs）。

规则:
1. 所有 *Result 都必须继承 BridgeResponse
2. 修改 *Result 必须同步修改 Rust struct，否则 serde 反序列化失败
3. ErrorCode 枚举与 Rust error.rs 中的定义严格对齐
"""

from dataclasses import dataclass, field, asdict
from enum import Enum
from typing import Optional, Any


# ============================================================
# 全栈错误码 — 与 Rust error.rs / 前端 api-types.ts 严格对齐
# ============================================================

class ErrorCode(str, Enum):
    """跨语言错误码 — 三端统一定义，修改必须同步 Rust + Frontend"""
    # 网络层
    NETWORK_TIMEOUT = "network_timeout"
    NETWORK_ERROR = "network_error"
    RATE_LIMITED = "rate_limited"
    PROXY_ERROR = "proxy_error"
    # 认证层
    COOKIE_EXPIRED = "cookie_expired"
    COOKIE_INVALID = "cookie_invalid"
    LOGIN_REQUIRED = "login_required"
    # 业务层
    VIDEO_NOT_FOUND = "video_not_found"
    USER_NOT_FOUND = "user_not_found"
    CONTENT_DELETED = "content_deleted"
    SIGNATURE_ERROR = "signature_error"
    PARSE_ERROR = "parse_error"
    # 系统层
    DATABASE_ERROR = "database_error"
    FILE_SYSTEM_ERROR = "file_system_error"
    CONFIG_ERROR = "config_error"
    # 兜底
    UNKNOWN = "unknown"


# ============================================================
# 桥接响应基类
# ============================================================

@dataclass
class BridgeResponse:
    """所有桥接响应的基类"""
    success: bool = False
    error_code: str = "unknown"
    error: str = ""

    def to_dict(self) -> dict:
        """序列化为 Rust py_to_json_value 可消费的 dict"""
        return asdict(self)


# ============================================================
# 查询类响应
# ============================================================

@dataclass
class VideoParseResult(BridgeResponse):
    """parse_video() 返回值 — Rust: VideoParseResult"""
    detail: Optional[dict] = None
    path: Optional[str] = None
    paths: Optional[list] = None


@dataclass
class UserProfileResult(BridgeResponse):
    """get_user_profile() 返回值 — Rust: UserProfileResult"""
    profile: Optional[dict] = None


@dataclass
class UserPostsResult(BridgeResponse):
    """get_user_posts() 返回值 — Rust: UserPostsResult"""
    videos: list = field(default_factory=list)
    has_more: bool = False
    cursor: int = 0


@dataclass
class LiveInfoResult(BridgeResponse):
    """get_live_info() 返回值 — Rust: LiveInfoResult"""
    live_info: Optional[dict] = None


@dataclass
class MusicCollectionResult(BridgeResponse):
    """get_music_collection() 返回值 — Rust: MusicCollectionResult"""
    music_list: list = field(default_factory=list)
    has_more: bool = False
    cursor: int = 0


@dataclass
class CommentsResult(BridgeResponse):
    """get_comments() 返回值 — Rust: CommentsResult"""
    comments: list = field(default_factory=list)
    has_more: bool = False
    cursor: int = 0


@dataclass
class FollowingListResult(BridgeResponse):
    """get_following_list() 返回值 — Rust: FollowingListResult"""
    followings: list = field(default_factory=list)
    has_more: bool = False
    cursor: int = 0


@dataclass
class FollowerListResult(BridgeResponse):
    """get_follower_list() 返回值 — Rust: FollowerListResult"""
    followers: list = field(default_factory=list)
    has_more: bool = False
    cursor: int = 0


@dataclass
class CollectsListResult(BridgeResponse):
    """get_collects_list() 返回值 — Rust: CollectsListResult"""
    collects: list = field(default_factory=list)


@dataclass
class CollectsVideoListResult(BridgeResponse):
    """get_collects_video_list() 返回值 — Rust: CollectsVideoListResult"""
    videos: list = field(default_factory=list)
    has_more: bool = False
    cursor: int = 0


@dataclass
class MixInfoResult(BridgeResponse):
    """get_mix_info() 返回值 — Rust: MixInfoResult"""
    videos: list = field(default_factory=list)
    has_more: bool = False
    cursor: int = 0


@dataclass
class SearchResult(BridgeResponse):
    """search_videos() 返回值 — Rust: SearchResult"""
    videos: list = field(default_factory=list)
    has_more: bool = False
    cursor: int = 0


@dataclass
class TabFeedResult(BridgeResponse):
    """get_tab_feed() 返回值 — Rust: TabFeedResult"""
    videos: list = field(default_factory=list)
    has_more: bool = False
    cursor: int = 0
    next_cursor: int = 0


@dataclass
class FollowFeedResult(BridgeResponse):
    """get_follow_feed() 返回值 — Rust: FollowFeedResult"""
    videos: list = field(default_factory=list)
    has_more: bool = False
    cursor: int = 0


@dataclass
class FriendFeedResult(BridgeResponse):
    """get_friend_feed() 返回值 — Rust: FriendFeedResult"""
    videos: list = field(default_factory=list)
    has_more: bool = False
    cursor: int = 0


@dataclass
class UserLikesResult(BridgeResponse):
    """get_user_likes() 返回值 — Rust: UserLikesResult"""
    videos: list = field(default_factory=list)
    has_more: bool = False
    cursor: int = 0


@dataclass
class PostStatsResult(BridgeResponse):
    """get_post_stats() 返回值 — Rust: PostStatsResult"""
    stats: Optional[dict] = None


# ============================================================
# 下载类响应
# ============================================================

@dataclass
class DownloadResult(BridgeResponse):
    """download_video() 返回值 — Rust: PythonDownloadResult"""
    detail: Optional[dict] = None
    path: Optional[str] = None
    paths: Optional[list] = None


@dataclass
class BatchDownloadResult(BridgeResponse):
    """download_batch() 返回值 — Rust: BatchDownloadResult"""
    count: int = 0
    results: list = field(default_factory=list)


@dataclass
class MusicBatchResult(BridgeResponse):
    """download_music_batch() 返回值 — Rust: MusicBatchResult"""
    music_list: list = field(default_factory=list)
    results: list = field(default_factory=list)


# ============================================================
# 直播类响应
# ============================================================

@dataclass
class LiveRecordResult(BridgeResponse):
    """start_live_record() 返回值 — Rust: LiveRecordResult"""
    task_id: str = ""


@dataclass
class LiveStatusResult(BridgeResponse):
    """get_live_status() 返回值 — Rust: LiveStatusResult"""
    tasks: list = field(default_factory=list)


@dataclass
class FollowingLiveResult(BridgeResponse):
    """get_following_live() 返回值 — Rust: FollowingLiveResult"""
    lives: list = field(default_factory=list)
