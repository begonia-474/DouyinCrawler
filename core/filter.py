"""响应数据过滤 — 从 API JSON 中提取关键字段"""


def get_nested(data: dict, *keys, default=None):
    """安全地从嵌套字典中取值"""
    current = data
    for key in keys:
        if isinstance(current, dict):
            current = current.get(key)
        elif isinstance(current, list) and isinstance(key, int) and key < len(current):
            current = current[key]
        else:
            return default
        if current is None:
            return default
    return current


# ============================================================
# 用户相关
# ============================================================

class UserProfileFilter:
    """用户资料过滤器"""

    def __init__(self, data: dict):
        self._user = get_nested(data, "user", default={})

    @property
    def nickname(self) -> str:
        return self._user.get("nickname", "")

    @property
    def uid(self) -> str:
        return str(self._user.get("uid", ""))

    @property
    def sec_user_id(self) -> str:
        return self._user.get("sec_uid", "")

    @property
    def avatar_url(self) -> str:
        return get_nested(self._user, "avatar_larger", "url_list", 0, default="")

    @property
    def aweme_count(self) -> int:
        return self._user.get("aweme_count", 0)

    @property
    def follower_count(self) -> int:
        return self._user.get("follower_count", 0)

    @property
    def following_count(self) -> int:
        return self._user.get("following_count", 0)

    @property
    def total_favorited(self) -> int:
        return self._user.get("total_favorited", 0)

    @property
    def signature(self) -> str:
        return self._user.get("signature", "")

    @property
    def ip_location(self) -> str:
        return self._user.get("ip_location", "")

    def to_dict(self) -> dict:
        return {
            "nickname": self.nickname, "uid": self.uid,
            "sec_user_id": self.sec_user_id, "avatar_url": self.avatar_url,
            "aweme_count": self.aweme_count, "follower_count": self.follower_count,
            "following_count": self.following_count, "total_favorited": self.total_favorited,
            "signature": self.signature, "ip_location": self.ip_location,
        }


class UserPostFilter:
    """用户视频列表过滤器"""

    def __init__(self, data: dict):
        self._data = data

    @property
    def aweme_list(self) -> list[dict]:
        return self._data.get("aweme_list", [])

    @property
    def max_cursor(self) -> int:
        return self._data.get("max_cursor", 0)

    @property
    def has_more(self) -> bool:
        return bool(self._data.get("has_more", 0))

    def get_video_list(self) -> list["PostDetailFilter"]:
        result = []
        for aweme in self.aweme_list:
            result.append(PostDetailFilter({"aweme_detail": aweme}))
        return result

    def to_list(self) -> list[dict]:
        return [v.to_dict() for v in self.get_video_list()]


class UserCollectsFilter:
    """收藏夹列表过滤器"""

    def __init__(self, data: dict):
        self._data = data

    @property
    def collects_list(self) -> list[dict]:
        return self._data.get("collects_list", [])

    @property
    def has_more(self) -> bool:
        return bool(self._data.get("has_more", 0))

    @property
    def cursor(self) -> int:
        return self._data.get("cursor", 0)

    def to_list(self) -> list[dict]:
        result = []
        for c in self.collects_list:
            result.append({
                "collects_id": str(c.get("id", "")),
                "name": c.get("name", ""),
                "count": c.get("collects_count", 0),
            })
        return result


class UserMusicCollectionFilter:
    """音乐收藏过滤器"""

    def __init__(self, data: dict):
        self._data = data

    @property
    def music_list(self) -> list[dict]:
        return self._data.get("mc_list", [])

    @property
    def has_more(self) -> bool:
        return bool(self._data.get("has_more", 0))

    @property
    def cursor(self) -> int:
        return self._data.get("cursor", 0)


class UserFollowingFilter:
    """关注列表过滤器"""

    def __init__(self, data: dict):
        self._data = data

    @property
    def followings(self) -> list[dict]:
        return self._data.get("followings", [])

    @property
    def has_more(self) -> bool:
        return bool(self._data.get("has_more", 0))

    @property
    def offset(self) -> int:
        return self._data.get("offset", 0)


class UserFollowerFilter:
    """粉丝列表过滤器"""

    def __init__(self, data: dict):
        self._data = data

    @property
    def followers(self) -> list[dict]:
        return self._data.get("followers", [])

    @property
    def has_more(self) -> bool:
        return bool(self._data.get("has_more", 0))

    @property
    def offset(self) -> int:
        return self._data.get("offset", 0)


# ============================================================
# 视频相关
# ============================================================

class PostDetailFilter:
    """视频详情过滤器"""

    def __init__(self, data: dict):
        self._data = data
        self._aweme = get_nested(data, "aweme_detail", default=data)

    @property
    def aweme_id(self) -> str:
        return str(self._aweme.get("aweme_id", ""))

    @property
    def aweme_type(self) -> int:
        return self._aweme.get("aweme_type", 0)

    @property
    def desc(self) -> str:
        return self._aweme.get("desc", "")

    @property
    def author_nickname(self) -> str:
        return get_nested(self._aweme, "author", "nickname", default="")

    @property
    def author_uid(self) -> str:
        return get_nested(self._aweme, "author", "uid", default="")

    @property
    def author_sec_uid(self) -> str:
        return get_nested(self._aweme, "author", "sec_uid", default="")

    @property
    def create_time(self) -> int:
        return self._aweme.get("create_time", 0)

    @property
    def duration(self) -> int:
        return get_nested(self._aweme, "video", "duration", default=0)

    @property
    def bit_rate_list(self) -> list[dict]:
        return get_nested(self._aweme, "video", "bit_rate", default=[])

    @property
    def video_url(self) -> str:
        """无水印视频 URL（优先最高码率）"""
        bit_rate = self.bit_rate_list
        if bit_rate:
            url_list = get_nested(bit_rate[0], "play_addr", "url_list", default=[])
            for url in url_list:
                if url and "playwm" not in url:
                    return url
            if url_list:
                return url_list[0]
        url_list = get_nested(self._aweme, "video", "play_addr", "url_list", default=[])
        for url in url_list:
            if url and "playwm" not in url:
                return url
        return url_list[0] if url_list else ""

    @property
    def cover_url(self) -> str:
        return get_nested(self._aweme, "video", "origin_cover", "url_list", 0, default="")

    @property
    def images(self) -> list[str]:
        """图集 URL 列表"""
        imgs = self._aweme.get("images") or []
        return [get_nested(img, "url_list", 0, default="") for img in imgs if img]

    @property
    def is_image_post(self) -> bool:
        return self.aweme_type == 68

    @property
    def music_title(self) -> str:
        return get_nested(self._aweme, "music", "title", default="")

    @property
    def music_url(self) -> str:
        return get_nested(self._aweme, "music", "play_url", "url_list", 0, default="")

    @property
    def statistics(self) -> dict:
        return self._aweme.get("statistics") or {}

    @property
    def digg_count(self) -> int:
        return self.statistics.get("digg_count", 0)

    @property
    def comment_count(self) -> int:
        return self.statistics.get("comment_count", 0)

    @property
    def share_count(self) -> int:
        return self.statistics.get("share_count", 0)

    @property
    def collect_count(self) -> int:
        return self.statistics.get("collect_count", 0)

    @property
    def mix_id(self) -> str:
        return get_nested(self._aweme, "mix_info", "mix_id", default="")

    @property
    def mix_name(self) -> str:
        return get_nested(self._aweme, "mix_info", "mix_name", default="")

    @property
    def is_prohibited(self) -> bool:
        return bool(get_nested(self._aweme, "status", "is_prohibited", default=0))

    def to_dict(self) -> dict:
        return {
            "aweme_id": self.aweme_id, "aweme_type": self.aweme_type,
            "desc": self.desc, "author": self.author_nickname,
            "author_uid": self.author_uid, "author_sec_uid": self.author_sec_uid,
            "create_time": self.create_time, "duration": self.duration,
            "video_url": self.video_url, "cover_url": self.cover_url,
            "images": self.images, "is_image_post": self.is_image_post,
            "music_title": self.music_title, "music_url": self.music_url,
            "digg_count": self.digg_count, "comment_count": self.comment_count,
            "share_count": self.share_count, "collect_count": self.collect_count,
            "mix_id": self.mix_id, "mix_name": self.mix_name,
            "is_prohibited": self.is_prohibited,
        }


class PostCommentFilter:
    """评论列表过滤器"""

    def __init__(self, data: dict):
        self._data = data

    @property
    def comments(self) -> list[dict]:
        return self._data.get("comments", [])

    @property
    def has_more(self) -> bool:
        return bool(self._data.get("has_more", 0))

    @property
    def cursor(self) -> int:
        return self._data.get("cursor", 0)


class PostRelatedFilter(UserPostFilter):
    """相关推荐过滤器"""
    pass


class HomePostSearchFilter(UserPostFilter):
    """主页搜索过滤器"""
    pass


class SuggestWordFilter:
    """搜索建议词过滤器"""

    def __init__(self, data: dict):
        self._data = data

    @property
    def words(self) -> list[str]:
        word_list = get_nested(self._data, "data", 0, "words", default=[])
        return [w.get("word", "") for w in word_list]


# ============================================================
# 直播相关
# ============================================================

class UserLiveFilter:
    """直播信息过滤器"""

    def __init__(self, data: dict):
        self._data = data
        self._room = get_nested(data, "data", "data", 0, default={})

    @property
    def room_id(self) -> str:
        return str(self._room.get("id_str", ""))

    @property
    def live_status(self) -> int:
        return self._room.get("status", 0)

    @property
    def live_title(self) -> str:
        return self._room.get("title", "")

    @property
    def cover_url(self) -> str:
        return get_nested(self._room, "cover", "url_list", 0, default="")

    @property
    def user_count(self) -> str:
        return get_nested(self._room, "stats", "user_count_str", default="0")

    @property
    def flv_pull_url(self) -> dict:
        return get_nested(self._room, "stream_url", "flv_pull_url", default={})

    @property
    def m3u8_pull_url(self) -> dict:
        return get_nested(self._room, "stream_url", "hls_pull_url_map", default={})

    @property
    def nickname(self) -> str:
        return get_nested(self._room, "owner", "nickname", default="")

    @property
    def is_live(self) -> bool:
        return self.live_status == 2

    def to_dict(self) -> dict:
        return {
            "room_id": self.room_id, "live_status": self.live_status,
            "live_title": self.live_title, "cover_url": self.cover_url,
            "user_count": self.user_count, "nickname": self.nickname,
            "flv_pull_url": self.flv_pull_url, "m3u8_pull_url": self.m3u8_pull_url,
            "is_live": self.is_live,
        }


class UserLive2Filter:
    """直播信息过滤器（room_id 接口）"""

    def __init__(self, data: dict):
        self._room = get_nested(data, "data", "room", default={})

    @property
    def room_id(self) -> str:
        return str(self._room.get("id", ""))

    @property
    def live_status(self) -> int:
        return self._room.get("status", 0)

    @property
    def live_title(self) -> str:
        return self._room.get("title", "")

    @property
    def flv_pull_url(self) -> dict:
        return get_nested(self._room, "stream_url", "flv_pull_url", default={})

    @property
    def m3u8_pull_url(self) -> dict:
        return get_nested(self._room, "stream_url", "hls_pull_url_map", default={})

    @property
    def nickname(self) -> str:
        return get_nested(self._room, "owner", "nickname", default="")

    @property
    def is_live(self) -> bool:
        return self.live_status == 2


class UserLiveStatusFilter:
    """用户直播状态过滤器"""

    def __init__(self, data: dict):
        self._data = data

    @property
    def live_status(self) -> int:
        return get_nested(self._data, "data", 0, "user_live", 0, "status", default=0)

    @property
    def room_id(self) -> str:
        return str(get_nested(self._data, "data", 0, "user_live", 0, "room_id", default=""))


class FollowingUserLiveFilter:
    """关注用户直播过滤器"""

    def __init__(self, data: dict):
        self._data = data

    @property
    def live_rooms(self) -> list[dict]:
        return get_nested(self._data, "data", "data", default=[])


class LiveImFetchFilter:
    """直播弹幕初始化过滤器"""

    def __init__(self, data: dict):
        self._data = data

    @property
    def cursor(self) -> str:
        return str(get_nested(self._data, "data", 0, "cursor", default=""))

    @property
    def internal_ext(self) -> str:
        return get_nested(self._data, "data", 0, "internal_ext", default="")


class QueryUserFilter:
    """用户查询结果过滤器"""

    def __init__(self, data: dict):
        self._data = data

    @property
    def status_code(self) -> int:
        return self._data.get("status_code")

    @property
    def status_msg(self) -> str:
        return self._data.get("status_msg", "")

    @property
    def browser_name(self) -> str:
        return self._data.get("browser_name", "")

    @property
    def create_time(self) -> str:
        return str(self._data.get("create_time", ""))

    @property
    def firebase_instance_id(self) -> str:
        return self._data.get("firebase_instance_id", "")

    @property
    def user_unique_id(self) -> str:
        return str(self._data.get("id", ""))

    @property
    def last_time(self) -> str:
        return str(self._data.get("last_time", ""))

    @property
    def user_agent(self) -> str:
        return self._data.get("user_agent", "")

    @property
    def user_uid(self) -> str:
        return str(self._data.get("user_uid", ""))

    @property
    def user_uid_type(self) -> int:
        return self._data.get("user_uid_type", 0)

    def to_dict(self) -> dict:
        return {
            "status_code": self.status_code,
            "status_msg": self.status_msg,
            "browser_name": self.browser_name,
            "create_time": self.create_time,
            "firebase_instance_id": self.firebase_instance_id,
            "user_unique_id": self.user_unique_id,
            "last_time": self.last_time,
            "user_agent": self.user_agent,
            "user_uid": self.user_uid,
            "user_uid_type": self.user_uid_type,
        }


class PostStatsFilter:
    """作品统计过滤器"""

    def __init__(self, data: dict):
        self._data = data

    @property
    def status_code(self) -> int:
        return self._data.get("status_code", -1)

    @property
    def status_msg(self) -> str:
        return self._data.get("status_msg", "")

    def to_dict(self) -> dict:
        return {
            "status_code": self.status_code,
            "status_msg": self.status_msg,
        }
