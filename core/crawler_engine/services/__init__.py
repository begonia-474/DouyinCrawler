"""服务模块 — handler.py 按业务域拆分后独立服务"""

from .video_service import VideoService
from .user_service import UserService
from .collection_service import CollectionService
from .mix_service import MixService
from .live_service import LiveService
from .feed_service import FeedService
from .content_service import ContentService
from .music_service import MusicService

__all__ = [
    "VideoService", "UserService", "CollectionService", "MixService",
    "LiveService", "FeedService", "ContentService", "MusicService",
]
