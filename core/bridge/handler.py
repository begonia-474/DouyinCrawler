"""业务处理器 — 门面类，委托给各业务服务模块

能力分级：
- [active] 纯查询/解析方法（由 py_bridge 调用）
- 旧 execution 方法（handle_one_video, handle_user_post, handle_live_record 等）已删除；
  所有非音乐下载和直播执行已由 Rust TaskApplicationService 拥有。
"""

import logging
from pathlib import Path

from core.models import ServiceConfig
from core.services import (
    VideoService, UserService, CollectionService, MixService,
    LiveService, FeedService, ContentService, MusicService,
)

logger = logging.getLogger(__name__)


class DouyinHandler:
    """抖音业务处理器（门面类）"""

    def __init__(self, cookie: str, download_path: str = "Download",
                 naming: str = "{create}_{desc}", max_counts: int = 0,
                 page_counts: int = 20, timeout: int = 10,
                 encryption: str = "ab", proxies: dict = None,
                 app_name: str = "douyin", folderize: bool = False,
                 music: bool = False, cover: bool = False, desc: bool = False,
                 interval: str = None, max_connections: int = 5,
                 max_retries: int = 5, max_tasks: int = 10):
        if isinstance(download_path, str):
            download_path = Path(download_path)
        self.config = ServiceConfig(
            cookie=cookie,
            download_path=download_path,
            naming=naming,
            max_counts=max_counts or float("inf"),
            page_counts=page_counts,
            timeout=timeout,
            encryption=encryption,
            proxies=proxies,
            app_name=app_name,
            folderize=folderize,
            music=music,
            cover=cover,
            desc=desc,
            interval=interval,
            max_connections=max_connections,
            max_retries=max_retries,
            max_tasks=max_tasks,
        )

        self._video = VideoService(self.config)
        self._user = UserService(self.config)
        self._collection = CollectionService(self.config)
        self._mix = MixService(self.config)
        self._live = LiveService(self.config)
        self._feed = FeedService(self.config)
        self._content = ContentService(self.config)
        self._music = MusicService(self.config)

    # === 视频（仅查询） ===
    async def handle_parse_video(self, url: str) -> dict:
        return await self._video.handle_parse_video(url)

    # === 用户 ===
    async def handle_user_post_list(self, url: str, cursor: int = 0, count: int = 20) -> dict:
        return await self._user.handle_user_post_list(url, cursor, count)

    async def handle_user_like_list(self, url: str, cursor: int = 0, count: int = 20) -> dict:
        return await self._user.handle_user_like_list(url, cursor, count)

    async def handle_user_profile(self, url: str) -> dict:
        return await self._user.handle_user_profile(url)

    async def handle_user_following(self, url: str, offset: int = 0, count: int = 20) -> dict:
        return await self._user.handle_user_following(url, offset, count)

    async def handle_user_follower(self, url: str, offset: int = 0, count: int = 20) -> dict:
        return await self._user.handle_user_follower(url, offset, count)

    # === 收藏 ===
    async def handle_user_collects(self, progress_callback=None) -> dict:
        return await self._collection.handle_user_collects(progress_callback)

    async def handle_collects_video_list(self, collects_id: str, cursor: int = 0, count: int = 20) -> dict:
        return await self._collection.handle_collects_video_list(collects_id, cursor, count)

    # === 合集 ===
    async def handle_user_mix_list(self, url: str, cursor: int = 0, count: int = 20) -> dict:
        return await self._mix.handle_user_mix_list(url, cursor, count)

    # === 直播（仅查询/解析） ===
    async def handle_user_live(self, url: str, progress_callback=None) -> dict:
        return await self._live.handle_user_live(url, progress_callback)

    async def handle_following_live(self) -> dict:
        return await self._live.handle_following_live()

    # === 内容 ===
    async def handle_related(self, url: str, count: int = 20, filter_gids: str = "", progress_callback=None) -> dict:
        return await self._content.handle_related(url, count, filter_gids, progress_callback)

    async def handle_post_comment(self, url: str, cursor: int = 0, count: int = 20) -> dict:
        return await self._content.handle_post_comment(url, cursor, count)

    async def handle_post_comment_reply(self, url: str, comment_id: str, cursor: int = 0, count: int = 3) -> dict:
        return await self._content.handle_post_comment_reply(url, comment_id, cursor, count)

    async def handle_post_stats(self, url: str) -> dict:
        return await self._content.handle_post_stats(url)

    async def handle_locate_post(self, url: str, max_cursor: str, locate_item_cursor: str) -> dict:
        return await self._content.handle_locate_post(url, max_cursor, locate_item_cursor)

    # === Feed ===
    async def handle_tab_feed(self, count: int = 10) -> dict:
        return await self._feed.handle_tab_feed(count)

    async def handle_follow_feed(self, cursor: int = 0, count: int = 10) -> dict:
        return await self._feed.handle_follow_feed(cursor, count)

    async def handle_friend_feed(self, cursor: int = 0, count: int = 10) -> dict:
        return await self._feed.handle_friend_feed(cursor, count)

    async def handle_search(self, keyword: str, offset: int = 0, count: int = 10) -> dict:
        return await self._feed.handle_search(keyword, offset, count)

    # === 音乐（唯一允许的 Python 下载路径） ===
    async def handle_user_music_collection(self, cursor: int = 0, count: int = 18) -> dict:
        return await self._music.handle_user_music_collection(cursor, count)

    async def handle_download_music(self, play_url: str, title: str, author: str) -> dict:
        return await self._music.handle_download_music(play_url, title, author)
