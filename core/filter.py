"""响应数据过滤 — 从 API JSON 中提取关键字段"""

import json


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

    @property
    def city(self) -> str:
        return self._user.get("city", "")

    @property
    def country(self) -> str:
        return self._user.get("country", "")

    @property
    def favoriting_count(self) -> int:
        return self._user.get("favoriting_count", 0)

    @property
    def gender(self) -> int:
        return self._user.get("gender", 0)

    @property
    def is_ban(self) -> int:
        return int(get_nested(self._user, "status", "is_ban", default=0))

    @property
    def is_block(self) -> int:
        return int(get_nested(self._user, "status", "is_block", default=0))

    @property
    def is_blocked(self) -> int:
        return int(get_nested(self._user, "status", "is_blocked", default=0))

    @property
    def is_star(self) -> int:
        return int(get_nested(self._user, "status", "is_star", default=0))

    @property
    def mix_count(self) -> int:
        return self._user.get("mix_count", 0)

    @property
    def mplatform_followers_count(self) -> int:
        return self._user.get("mplatform_followers_count", 0)

    @property
    def school_name(self) -> str:
        return self._user.get("school_name", "")

    @property
    def short_id(self) -> str:
        return self._user.get("short_id", "")

    @property
    def user_age(self) -> int:
        return self._user.get("user_age", 0)

    @property
    def custom_verify(self) -> str:
        return self._user.get("custom_verify", "")

    @property
    def unique_id(self) -> str:
        return self._user.get("unique_id", "")

    @property
    def live_status(self) -> int:
        val = self._user.get("live_status", 0)
        return int(val) if val else 0

    @property
    def room_id(self) -> str:
        return str(self._user.get("room_id", ""))

    def to_dict(self) -> dict:
        return {
            "nickname": self.nickname, "uid": self.uid,
            "sec_user_id": self.sec_user_id,
            "avatar": self.avatar_url,        # 前端 UserProfile.avatar
            "avatar_url": self.avatar_url,     # Rust UserInfo.avatar_url
            "aweme_count": self.aweme_count, "follower_count": self.follower_count,
            "following_count": self.following_count, "total_favorited": self.total_favorited,
            "signature": self.signature, "ip_location": self.ip_location,
            "city": self.city, "country": self.country,
            "favoriting_count": self.favoriting_count, "gender": self.gender,
            "is_ban": self.is_ban, "is_block": self.is_block,
            "is_blocked": self.is_blocked, "is_star": self.is_star,
            "mix_count": self.mix_count, "mplatform_followers_count": self.mplatform_followers_count,
            "nickname_raw": self.nickname, "school_name": self.school_name,
            "short_id": self.short_id, "signature_raw": self.signature,
            "user_age": self.user_age, "custom_verify": self.custom_verify,
            "unique_id": self.unique_id, "live_status": self.live_status,
            "room_id": self.room_id,
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
                "id": str(c.get("collects_id", "")),
                "name": c.get("collects_name", ""),
                "count": c.get("total_number", 0),
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

    def to_list(self) -> list[dict]:
        result = []
        for m in self.music_list:
            play_url = ""
            url_list = get_nested(m, "play_url", "url_list", default=[])
            if url_list:
                play_url = url_list[0]
            cover = get_nested(m, "cover_hd", "url_list", 0, default="")
            result.append({
                "music_id": str(m.get("id", "")),
                "mid": str(m.get("mid", "")),
                "title": m.get("title", ""),
                "author": m.get("author", ""),
                "owner_nickname": m.get("owner_nickname", ""),
                "duration": m.get("duration", 0),
                "cover": cover,
                "play_url": play_url,
            })
        return result


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
        val = self._aweme.get("aweme_type", 0)
        return int(val) if val else 0

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
        val = get_nested(self._aweme, "video", "duration", default=0)
        return int(val) if val else 0

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
    def images_video(self) -> list[str]:
        """图集动图/实况 URL 列表"""
        imgs = self._aweme.get("images") or []
        result = []
        for img in imgs:
            if img:
                video_url = get_nested(img, "video", "play_addr", "url_list", 0, default="")
                result.append(video_url)
        return result

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
        val = self.statistics.get("digg_count", 0)
        return int(val) if val else 0

    @property
    def comment_count(self) -> int:
        val = self.statistics.get("comment_count", 0)
        return int(val) if val else 0

    @property
    def share_count(self) -> int:
        val = self.statistics.get("share_count", 0)
        return int(val) if val else 0

    @property
    def collect_count(self) -> int:
        val = self.statistics.get("collect_count", 0)
        return int(val) if val else 0

    @property
    def mix_id(self) -> str:
        return get_nested(self._aweme, "mix_info", "mix_id", default="")

    @property
    def mix_name(self) -> str:
        return get_nested(self._aweme, "mix_info", "mix_name", default="")

    @property
    def is_prohibited(self) -> bool:
        return bool(get_nested(self._aweme, "status", "is_prohibited", default=0))

    # === f2 对齐字段 - 作者 ===

    @property
    def author_short_id(self) -> str:
        return get_nested(self._aweme, "author", "short_id", default="")

    @property
    def author_unique_id(self) -> str:
        return get_nested(self._aweme, "author", "unique_id", default="")

    @property
    def author_avatar_url(self) -> str:
        return get_nested(self._aweme, "author", "avatar_thumb", "url_list", 0, default="")

    @property
    def author_signature(self) -> str:
        return get_nested(self._aweme, "author", "signature", default="")

    @property
    def author_follower_count(self) -> int:
        return get_nested(self._aweme, "author", "follower_count", default=0)

    @property
    def author_aweme_count(self) -> int:
        return get_nested(self._aweme, "author", "aweme_count", default=0)

    @property
    def author_following_count(self) -> int:
        return get_nested(self._aweme, "author", "following_count", default=0)

    @property
    def author_total_favorited(self) -> int:
        return get_nested(self._aweme, "author", "total_favorited", default=0)

    @property
    def author_ip_location(self) -> str:
        return get_nested(self._aweme, "author", "ip_location", default="")

    # === f2 对齐字段 - 内容 ===

    @property
    def desc_raw(self) -> str:
        return self._aweme.get("desc", "")

    @property
    def is_ads(self) -> int:
        val = self._aweme.get("is_ads", 0)
        return int(val) if val else 0

    @property
    def is_story(self) -> int:
        return 1 if self._aweme.get("story_info") else 0

    @property
    def is_top(self) -> int:
        val = self._aweme.get("is_top", 0)
        return int(val) if val else 0

    @property
    def is_long_video(self) -> int:
        val = self._aweme.get("is_long_video", 0)
        return int(val) if val else 0

    # === f2 对齐字段 - 视频 ===

    @property
    def video_bit_rate_json(self) -> str:
        return json.dumps(self.bit_rate_list, ensure_ascii=False) if self.bit_rate_list else ""

    @property
    def animated_cover(self) -> str:
        return get_nested(self._aweme, "video", "animated_cover", "url_list", 0, default="")

    @property
    def private_status(self) -> int:
        return get_nested(self._aweme, "status", "private_status", default=0)

    @property
    def is_delete(self) -> int:
        return int(get_nested(self._aweme, "status", "is_delete", default=0))

    # === f2 对齐字段 - 音乐 ===

    @property
    def music_author(self) -> str:
        return get_nested(self._aweme, "music", "author", default="")

    @property
    def music_author_raw(self) -> str:
        return get_nested(self._aweme, "music", "author_original", default="")

    @property
    def music_duration(self) -> int:
        return get_nested(self._aweme, "music", "duration", default=0)

    @property
    def music_id(self) -> str:
        return str(get_nested(self._aweme, "music", "id", default=""))

    @property
    def music_mid(self) -> str:
        return get_nested(self._aweme, "music", "mid", default="")

    @property
    def pgc_author(self) -> str:
        return get_nested(self._aweme, "music", "pgc_author", default="")

    @property
    def pgc_author_title(self) -> str:
        return get_nested(self._aweme, "music", "pgc_author_title", default="")

    @property
    def pgc_music_type(self) -> int:
        return get_nested(self._aweme, "music", "pgc_music_type", default=0)

    @property
    def music_status(self) -> int:
        return get_nested(self._aweme, "music", "status", default=0)

    @property
    def music_owner_handle(self) -> str:
        return get_nested(self._aweme, "music", "owner_handle", default="")

    @property
    def music_owner_id(self) -> str:
        return get_nested(self._aweme, "music", "owner_id", default="")

    @property
    def music_owner_nickname(self) -> str:
        return get_nested(self._aweme, "music", "owner_nickname", default="")

    @property
    def music_play_url(self) -> str:
        return get_nested(self._aweme, "music", "play_url", "url_list", 0, default="")

    @property
    def is_commerce_music(self) -> int:
        return int(get_nested(self._aweme, "music", "is_commerce_music", default=0))

    # === f2 对齐字段 - 合集 ===

    @property
    def mix_desc(self) -> str:
        return get_nested(self._aweme, "mix_info", "mix_desc", default="")

    @property
    def mix_create_time(self) -> int:
        return get_nested(self._aweme, "mix_info", "mix_create_time", default=0)

    @property
    def mix_pic_type(self) -> int:
        return get_nested(self._aweme, "mix_info", "mix_pic_type", default=0)

    @property
    def mix_type(self) -> int:
        return get_nested(self._aweme, "mix_info", "mix_type", default=0)

    @property
    def mix_share_url(self) -> str:
        return get_nested(self._aweme, "mix_info", "share_info", "share_url", default="")

    # === f2 对齐字段 - 权限 ===

    @property
    def can_comment(self) -> int:
        return int(get_nested(self._aweme, "status", "can_comment", default=1))

    @property
    def can_forward(self) -> int:
        return int(get_nested(self._aweme, "status", "can_forward", default=1))

    @property
    def can_share(self) -> int:
        return int(get_nested(self._aweme, "status", "allow_share", default=1))

    @property
    def download_setting(self) -> int:
        return get_nested(self._aweme, "status", "download_setting", default=0)

    @property
    def allow_douplus(self) -> int:
        return int(get_nested(self._aweme, "status", "allow_douplus", default=0))

    @property
    def allow_share(self) -> int:
        return int(get_nested(self._aweme, "status", "allow_share", default=1))

    # === f2 对齐字段 - 统计/标签/其他 ===

    @property
    def admire_count(self) -> int:
        return self.statistics.get("admire_count", 0)

    @property
    def hashtag_ids(self) -> str:
        extras = self._aweme.get("text_extra") or []
        ids = [str(e.get("hashtag_id", "")) for e in extras if e.get("hashtag_id")]
        return json.dumps(ids, ensure_ascii=False) if ids else ""

    @property
    def hashtag_names(self) -> str:
        extras = self._aweme.get("text_extra") or []
        names = [e.get("hashtag_name", "") for e in extras if e.get("hashtag_name")]
        return json.dumps(names, ensure_ascii=False) if names else ""

    @property
    def images_json(self) -> str:
        return json.dumps(self.images, ensure_ascii=False) if self.images else ""

    @property
    def region(self) -> str:
        return self._aweme.get("region", "")

    def to_dict(self) -> dict:
        return {
            "aweme_id": self.aweme_id, "aweme_type": self.aweme_type,
            "desc": self.desc, "author": self.author_nickname,
            "author_uid": self.author_uid, "author_sec_uid": self.author_sec_uid,
            "create_time": self.create_time, "duration": self.duration,
            "video_url": self.video_url, "cover_url": self.cover_url,
            "images": self.images, "images_video": self.images_video,
            "is_image_post": self.is_image_post,
            "music_title": self.music_title, "music_url": self.music_url,
            "digg_count": self.digg_count, "comment_count": self.comment_count,
            "share_count": self.share_count, "collect_count": self.collect_count,
            "mix_id": self.mix_id, "mix_name": self.mix_name,
            "is_prohibited": self.is_prohibited,
        }

    def to_db_dict(self) -> dict:
        """返回完整字段集，用于数据库存储"""
        return {
            "aweme_id": self.aweme_id, "aweme_type": self.aweme_type,
            "desc": self.desc, "author_nickname": self.author_nickname,
            "author_uid": self.author_uid, "author_sec_uid": self.author_sec_uid,
            "create_time": self.create_time, "duration": self.duration,
            "video_url": self.video_url, "cover_url": self.cover_url,
            "images": self.images, "images_video": self.images_video,
            "is_image_post": self.is_image_post,
            "music_title": self.music_title, "music_url": self.music_url,
            "digg_count": self.digg_count, "comment_count": self.comment_count,
            "share_count": self.share_count, "collect_count": self.collect_count,
            "mix_id": self.mix_id, "mix_name": self.mix_name,
            "is_prohibited": int(self.is_prohibited),
            "author_nickname_raw": self.author_nickname,
            "author_short_id": self.author_short_id,
            "author_unique_id": self.author_unique_id,
            "author_avatar_url": self.author_avatar_url,
            "author_signature": self.author_signature,
            "author_follower_count": self.author_follower_count,
            "author_aweme_count": self.author_aweme_count,
            "author_following_count": self.author_following_count,
            "author_total_favorited": self.author_total_favorited,
            "author_ip_location": self.author_ip_location,
            "desc_raw": self.desc_raw, "is_ads": self.is_ads,
            "is_story": self.is_story, "is_top": self.is_top,
            "is_long_video": self.is_long_video,
            "video_bit_rate": self.video_bit_rate_json,
            "animated_cover": self.animated_cover,
            "private_status": self.private_status, "is_delete": self.is_delete,
            "music_author": self.music_author, "music_author_raw": self.music_author_raw,
            "music_duration": self.music_duration, "music_id": self.music_id,
            "music_mid": self.music_mid, "pgc_author": self.pgc_author,
            "pgc_author_title": self.pgc_author_title,
            "pgc_music_type": self.pgc_music_type, "music_status": self.music_status,
            "music_owner_handle": self.music_owner_handle,
            "music_owner_id": self.music_owner_id,
            "music_owner_nickname": self.music_owner_nickname,
            "music_play_url": self.music_play_url,
            "is_commerce_music": self.is_commerce_music,
            "mix_desc": self.mix_desc, "mix_create_time": self.mix_create_time,
            "mix_pic_type": self.mix_pic_type, "mix_type": self.mix_type,
            "mix_share_url": self.mix_share_url,
            "can_comment": self.can_comment, "can_forward": self.can_forward,
            "can_share": self.can_share, "download_setting": self.download_setting,
            "allow_douplus": self.allow_douplus, "allow_share": self.allow_share,
            "admire_count": self.admire_count,
            "hashtag_ids": self.hashtag_ids, "hashtag_names": self.hashtag_names,
            "images": self.images_json, "region": self.region,
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
