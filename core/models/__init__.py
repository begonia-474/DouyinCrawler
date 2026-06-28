"""数据模型子包

从 core/models.py 拆分为实体文件:
  - requests.py  — Pydantic v2 请求参数模型
  - config.py    — ServiceConfig 数据类
  - download.py  — DownloadMode 枚举
  - responses.py — Python↔Rust 桥接响应契约 + ErrorCode 枚举

所有符号通过此文件重导出，保持 `from core.models import X` 兼容。
"""

from core.models.requests import (
    BaseRequestModel, BaseLiveModel,
    UserProfile, UserPost, UserFavorite, UserCollection,
    UserCollects, UserCollectsVideo, UserMusicCollection,
    UserFollowing, UserFollower,
    PostDetail, PostComment, PostCommentReply, PostRelated,
    PostSearch, HomePostSearch, SuggestWord,
    TabFeed, FollowFeed, FriendFeed,
    UserLive, UserLive2, FollowingUserLive, UserLiveStatus,
    LiveImFetch, QueryUser, PostLocate, PostStats, UserMix,
)
from core.models.config import ServiceConfig
from core.models.download import DownloadMode
from core.models.responses import (
    ErrorCode, BridgeResponse,
    VideoParseResult, UserProfileResult, UserPostsResult,
    LiveInfoResult, MusicCollectionResult, CommentsResult,
    FollowingListResult, FollowerListResult,
    CollectsListResult, CollectsVideoListResult,
    MixInfoResult, SearchResult,
    TabFeedResult, FollowFeedResult, FriendFeedResult,
    UserLikesResult, PostStatsResult,
    DownloadResult, BatchDownloadResult, MusicBatchResult,
    LiveRecordResult, LiveStatusResult, FollowingLiveResult,
)

__all__ = [
    # 请求模型
    "BaseRequestModel", "BaseLiveModel",
    "UserProfile", "UserPost", "UserFavorite", "UserCollection",
    "UserCollects", "UserCollectsVideo", "UserMusicCollection",
    "UserFollowing", "UserFollower",
    "PostDetail", "PostComment", "PostCommentReply", "PostRelated",
    "PostSearch", "HomePostSearch", "SuggestWord",
    "TabFeed", "FollowFeed", "FriendFeed",
    "UserLive", "UserLive2", "FollowingUserLive", "UserLiveStatus",
    "LiveImFetch", "QueryUser", "PostLocate", "PostStats", "UserMix",
    # 配置
    "ServiceConfig",
    # 下载模式
    "DownloadMode",
    # 错误码
    "ErrorCode",
    # 桥接响应
    "BridgeResponse",
    "VideoParseResult", "UserProfileResult", "UserPostsResult",
    "LiveInfoResult", "MusicCollectionResult", "CommentsResult",
    "FollowingListResult", "FollowerListResult",
    "CollectsListResult", "CollectsVideoListResult",
    "MixInfoResult", "SearchResult",
    "TabFeedResult", "FollowFeedResult", "FriendFeedResult",
    "UserLikesResult", "PostStatsResult",
    "DownloadResult", "BatchDownloadResult", "MusicBatchResult",
    "LiveRecordResult", "LiveStatusResult", "FollowingLiveResult",
]
