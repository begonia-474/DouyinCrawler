"""
DouyinCrawler Python Core — 公有 API

所有符号通过此文件重导出，确保:
1. 外部 import 路径稳定 (from core import X)
2. 内部包重组对调用者透明
"""

# ── 爬虫引擎 (crawler_engine/) ──
from core.crawler_engine.crawler import DouyinCrawler
from core.crawler_engine.filter import (
    JSONModel,
    PostDetailFilter,
    UserProfileFilter,
    UserPostFilter,
    UserLiveFilter,
    UserLive2Filter,
    UserLiveStatusFilter,
    FollowingUserLiveFilter,
    UserCollectsFilter,
    UserMusicCollectionFilter,
    UserFollowingFilter,
    UserFollowerFilter,
    PostCommentFilter,
    PostRelatedFilter,
    HomePostSearchFilter,
    SuggestWordFilter,
    QueryUserFilter,
    PostStatsFilter,
    LiveImFetchFilter,
)
from core.crawler_engine.api import DouyinAPIEndpoints
from core.crawler_engine.signature.abogus import ABogus
from core.crawler_engine.signature.xbogus import XBogus
from core.crawler_engine.signature.fingerprint import BrowserFingerprintGenerator
from core.crawler_engine.tokens.token_manager import TokenManager

# ── 服务层 (crawler_engine/services/) ──
from core.crawler_engine.services.base import BaseService, run_concurrent
from core.crawler_engine.services.video_service import VideoService
from core.crawler_engine.services.user_service import UserService
from core.crawler_engine.services.collection_service import CollectionService
from core.crawler_engine.services.mix_service import MixService
from core.crawler_engine.services.live_service import LiveService
from core.crawler_engine.services.feed_service import FeedService
from core.crawler_engine.services.content_service import ContentService
from core.crawler_engine.services.music_service import MusicService

# ── 下载引擎 (download/) ──
from core.download.downloader import Downloader, format_filename

# ── 桥接层 (bridge/) ──
from core.bridge.handler import DouyinHandler
from core.bridge.db_bridge import (
    save_download_record,
    save_video_info,
    save_user_info,
    save_live_record,
    has_user,
)
from core.bridge.events import emit, set_emit_func
from core.bridge.py_bridge import (
    parse_video,
    download_video,
    get_live_info,
    download_batch,
    start_download,
    get_user_profile,
    get_user_posts,
    search_videos,
    get_mix_info,
    get_collects_list,
    get_collects_video_list,
    get_following_list,
    get_follower_list,
    get_music_collection,
    download_music_batch,
    download_music,
    get_following_live,
    get_comments,
    get_comment_replies,
    get_tab_feed,
    get_follow_feed,
    get_friend_feed,
    get_user_likes,
    get_post_stats,
    start_live_record,
    stop_live_record,
    get_live_status,
)

# ── 数据模型 ──
from core.models import ServiceConfig, BaseRequestModel, BaseLiveModel

# ── 任务管理 (task/) ──
from core.task.task_manager import TaskManager, task_manager
from core.task.live_manager import LiveRecordManager

# ── 工具函数 (utils/) ──
from core.utils import (
    extract_valid_urls,
    AwemeIdFetcher, SecUserIdFetcher, MixIdFetcher, WebCastIdFetcher,
    detect_url_type,
    replaceT, sanitize_filename,
    timestamp_2_str, interval_2_timestamp, filter_by_date_interval,
    get_segments_from_m3u8, get_content_length, get_chunk_size,
)
