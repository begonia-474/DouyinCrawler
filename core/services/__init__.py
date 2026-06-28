"""向后兼容 shim — 重导出到 core.crawler_engine.services"""
from core.crawler_engine.services import (
    VideoService, UserService, CollectionService, MixService,
    LiveService, FeedService, ContentService, MusicService,
)

__all__ = [
    "VideoService", "UserService", "CollectionService", "MixService",
    "LiveService", "FeedService", "ContentService", "MusicService",
]
