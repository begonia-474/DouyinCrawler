"""请求参数模型"""

from pydantic import BaseModel


class BaseRequestModel(BaseModel):
    """基础请求参数 — 浏览器指纹模拟"""
    device_platform: str = "webapp"
    aid: str = "6383"
    channel: str = "channel_pc_web"
    pc_client_type: int = 1
    publish_video_strategy_type: int = 2
    pc_libra_divert: str = "Windows"
    version_code: str = "290100"
    version_name: str = "29.1.0"
    cookie_enabled: str = "true"
    screen_width: int = 1920
    screen_height: int = 1080
    browser_language: str = "zh-CN"
    browser_platform: str = "Win32"
    browser_name: str = "Edge"
    browser_version: str = "130.0.0.0"
    browser_online: str = "true"
    engine_name: str = "Blink"
    engine_version: str = "130.0.0.0"
    os_name: str = "Windows"
    os_version: str = "10"
    cpu_core_num: int = 12
    device_memory: int = 8
    platform: str = "PC"
    downlink: int = 10
    effective_type: str = "4g"
    round_trip_time: int = 100
    msToken: str = ""


class BaseLiveModel(BaseModel):
    """直播请求基础参数"""
    aid: str = "6383"
    app_name: str = "douyin_web"
    live_id: int = 1
    device_platform: str = "web"
    language: str = "zh-CN"
    cookie_enabled: str = "true"
    screen_width: int = 1920
    screen_height: int = 1080
    browser_language: str = "zh-CN"
    browser_platform: str = "Win32"
    browser_name: str = "Edge"
    browser_version: str = "130.0.0.0"
    enter_source: str = ""
    is_need_double_stream: str = "false"
    insert_task_id: str = ""
    live_reason: str = ""


# === 用户相关 ===

class UserProfile(BaseRequestModel):
    sec_user_id: str


class UserPost(BaseRequestModel):
    sec_user_id: str
    max_cursor: int = 0
    count: int = 18
    locate_query: bool = False
    show_live_replay_strategy: int = 1
    need_time_list: int = 1
    time_list_query: int = 0
    cut_version: int = 1


class UserFavorite(BaseRequestModel):
    sec_user_id: str
    max_cursor: int = 0
    count: int = 18


class UserCollection(BaseRequestModel):
    cursor: int = 0
    count: int = 18


class UserCollects(BaseRequestModel):
    cursor: int = 0
    count: int = 18


class UserCollectsVideo(BaseRequestModel):
    collects_id: str
    cursor: int = 0
    count: int = 18


class UserMusicCollection(BaseRequestModel):
    cursor: int = 0
    count: int = 18


class UserFollowing(BaseRequestModel):
    user_id: str = ""
    sec_user_id: str = ""
    offset: int = 0
    count: int = 20
    source_type: int = 4


class UserFollower(BaseRequestModel):
    user_id: str = ""
    sec_user_id: str = ""
    offset: int = 0
    count: int = 20
    source_type: int = 4


# === 视频相关 ===

class PostDetail(BaseRequestModel):
    aweme_id: str


class PostComment(BaseRequestModel):
    aweme_id: str
    cursor: int = 0
    count: int = 20
    item_type: int = 0


class PostCommentReply(BaseRequestModel):
    item_id: str
    comment_id: str
    cursor: int = 0
    count: int = 20


class PostRelated(BaseRequestModel):
    aweme_id: str
    count: int = 20
    filterGids: str = ""


class PostStats(BaseRequestModel):
    aweme_type: int = 0
    item_id: str = ""
    play_delta: int = 1
    source: int = 0


class UserMix(BaseRequestModel):
    mix_id: str
    cursor: int = 0
    count: int = 20


# === 搜索 ===

class PostSearch(BaseRequestModel):
    keyword: str
    search_channel: str = "aweme_general"
    search_source: str = "normal_search"
    query_correct_type: int = 1
    is_filter_search: int = 0
    offset: int = 0
    count: int = 10
    need_filter_settings: int = 0
    list_type: str = "single"
    search_id: str = ""


class HomePostSearch(BaseRequestModel):
    keyword: str
    from_user: int = 0
    offset: int = 0
    count: int = 10


class SuggestWord(BaseRequestModel):
    query: str
    count: int = 10


# === 信息流 ===

class TabFeed(BaseRequestModel):
    count: int = 10
    tag_id: str = ""
    refresh_index: int = 1


class FollowFeed(BaseRequestModel):
    cursor: int = 0
    level: int = 1
    count: int = 10


class FriendFeed(BaseRequestModel):
    cursor: int = 0
    level: int = 1
    pull_type: int = 1


# === 直播 ===

class UserLive(BaseLiveModel):
    web_rid: str = ""
    room_id_str: str = ""


class UserLive2(BaseLiveModel):
    room_id: str = ""


class FollowingUserLive(BaseRequestModel):
    scene: str = "aweme_pc_follow_top"


class UserLiveStatus(BaseRequestModel):
    user_ids: str = ""
    distribution_scenes: str = "1"


class LiveImFetch(BaseModel):
    app_name: str = "douyin_web"
    version_code: str = "180800"
    device_platform: str = "web"
    aid: int = 6383
    live_id: int = 1
    room_id: str = ""
    user_unique_id: str = ""
    identity: str = "audience"


# === 其他 ===

class QueryUser(BaseRequestModel):
    pass


class PostLocate(BaseRequestModel):
    """定位作品请求参数"""
    sec_user_id: str
    max_cursor: str = ""  # last max_cursor
    locate_item_id: str = ""  # aweme_id
    locate_item_cursor: str = ""
    locate_query: str = "true"
    count: int = 10
    publish_video_strategy_type: int = 2
