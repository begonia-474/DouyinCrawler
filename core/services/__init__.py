"""服务模块 — handler.py 按业务域拆分后独立服务"""

from core.services.video_service import VideoService
from core.services.user_service import UserService
from core.services.collection_service import CollectionService
from core.services.mix_service import MixService
from core.services.live_service import LiveService
from core.services.feed_service import FeedService
from core.services.content_service import ContentService
from core.services.music_service import MusicService

__all__ = [
    "VideoService", "UserService", "CollectionService", "MixService",
    "LiveService", "FeedService", "ContentService", "MusicService",
]
