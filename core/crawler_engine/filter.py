"""响应数据过滤 — 从 API JSON 中提取关键字段

基于 JSONModel (JSONPath) 实现，移植自 f2 项目的 filter 架构。
每个 Filter 类包装原始 API 响应，通过 @property 懒加载提取字段。

保留 get_nested() 供非 Filter 场景使用。
"""

import json

from jsonpath_ng import parse
from core.utils import replaceT


# ============================================================
# JSONModel 基类（移植自 f2/utils/json_filter.py）
# ============================================================

class JSONModel:
    """JSONPath 驱动的数据提取基类。

    子类通过 @property + _get_attr_value("$.path.xxx") 声明式提取字段。
    JSONPath 表达式自动缓存，避免重复解析。
    """

    def __init__(self, data):
        self._data = data
        self._cache = {}

    def _parse_expression(self, jsonpath_expr: str):
        if jsonpath_expr not in self._cache:
            self._cache[jsonpath_expr] = parse(jsonpath_expr)
        return self._cache[jsonpath_expr]

    def _get_attr_value(self, jsonpath_expr: str):
        """根据 JSONPath 获取单一属性值。无匹配返回 None。"""
        expr = self._parse_expression(jsonpath_expr)
        matches = expr.find(self._data)
        if not matches:
            return None
        return matches[0].value if len(matches) == 1 else [m.value for m in matches]

    def _get_list_attr_value(self, jsonpath_expr: str, as_json: bool = False):
        """获取列表属性值，缺失字段补 None 保证列表对齐。"""
        if "[*]" in jsonpath_expr:
            idx = jsonpath_expr.find("[*]")
            parent_str = jsonpath_expr[:idx + 3]
            child_str = jsonpath_expr[idx + 3:]
        else:
            parent_str = jsonpath_expr
            child_str = ""

        parent_expr = self._parse_expression(parent_str)
        parent_matches = parent_expr.find(self._data)

        values = []
        if child_str:
            child_expr = self._parse_expression(f"$.{child_str.lstrip('.')}")
            for match in parent_matches:
                child_matches = child_expr.find(match.value)
                values.append(child_matches[0].value if child_matches else None)
        else:
            values = [m.value for m in parent_matches]

        return json.dumps(values, ensure_ascii=False) if as_json else values


# ============================================================
# 兼容工具（非 Filter 场景仍可用）
# ============================================================

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
# 辅助：安全取值（None → 默认值）
# ============================================================

def _str(val, default="") -> str:
    return str(val) if val is not None else default


def _int(val, default=0) -> int:
    if val is None:
        return default
    try:
        return int(val)
    except (ValueError, TypeError):
        return default


def _bool_int(val, default=0) -> int:
    """布尔/数字 → int (0/1)"""
    if val is None:
        return default
    return int(bool(val))


# ============================================================
# 用户相关
# ============================================================

class UserProfileFilter(JSONModel):
    """用户资料过滤器"""

    @property
    def nickname(self) -> str:
        return replaceT(_str(self._get_attr_value("$.user.nickname")))

    @property
    def nickname_raw(self) -> str:
        return _str(self._get_attr_value("$.user.nickname"))

    @property
    def uid(self) -> str:
        return _str(self._get_attr_value("$.user.uid"))

    @property
    def sec_user_id(self) -> str:
        return _str(self._get_attr_value("$.user.sec_uid"))

    @property
    def avatar_url(self) -> str:
        return _str(self._get_attr_value("$.user.avatar_larger.url_list[0]"))

    @property
    def aweme_count(self) -> int:
        return _int(self._get_attr_value("$.user.aweme_count"))

    @property
    def follower_count(self) -> int:
        return _int(self._get_attr_value("$.user.follower_count"))

    @property
    def following_count(self) -> int:
        return _int(self._get_attr_value("$.user.following_count"))

    @property
    def total_favorited(self) -> int:
        return _int(self._get_attr_value("$.user.total_favorited"))

    @property
    def signature(self) -> str:
        return replaceT(_str(self._get_attr_value("$.user.signature")))

    @property
    def signature_raw(self) -> str:
        return _str(self._get_attr_value("$.user.signature"))

    @property
    def ip_location(self) -> str:
        return _str(self._get_attr_value("$.user.ip_location"))

    @property
    def city(self) -> str:
        return _str(self._get_attr_value("$.user.city"))

    @property
    def country(self) -> str:
        return _str(self._get_attr_value("$.user.country"))

    @property
    def favoriting_count(self) -> int:
        return _int(self._get_attr_value("$.user.favoriting_count"))

    @property
    def gender(self) -> int:
        return _int(self._get_attr_value("$.user.gender"))

    @property
    def is_ban(self) -> int:
        return _bool_int(self._get_attr_value("$.user.is_ban"))

    @property
    def is_block(self) -> int:
        return _bool_int(self._get_attr_value("$.user.is_block"))

    @property
    def is_blocked(self) -> int:
        return _bool_int(self._get_attr_value("$.user.is_blocked"))

    @property
    def is_star(self) -> int:
        return _bool_int(self._get_attr_value("$.user.is_star"))

    @property
    def mix_count(self) -> int:
        return _int(self._get_attr_value("$.user.mix_count"))

    @property
    def mplatform_followers_count(self) -> int:
        return _int(self._get_attr_value("$.user.mplatform_followers_count"))

    @property
    def school_name(self) -> str:
        return _str(self._get_attr_value("$.user.school_name"))

    @property
    def short_id(self) -> str:
        return _str(self._get_attr_value("$.user.short_id"))

    @property
    def user_age(self) -> int:
        return _int(self._get_attr_value("$.user.user_age"))

    @property
    def custom_verify(self) -> str:
        return _str(self._get_attr_value("$.user.custom_verify"))

    @property
    def unique_id(self) -> str:
        return _str(self._get_attr_value("$.user.unique_id"))

    @property
    def live_status(self) -> int:
        return _int(self._get_attr_value("$.user.live_status"))

    @property
    def room_id(self) -> str:
        return _str(self._get_attr_value("$.user.room_id"))

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
            "nickname_raw": self.nickname_raw, "school_name": self.school_name,
            "short_id": self.short_id, "signature_raw": self.signature_raw,
            "user_age": self.user_age, "custom_verify": self.custom_verify,
            "unique_id": self.unique_id, "live_status": self.live_status,
            "room_id": self.room_id,
        }


# ============================================================
# 视频列表相关
# ============================================================

class UserPostFilter(JSONModel):
    """用户视频列表过滤器"""

    @property
    def has_more(self) -> bool:
        return bool(self._get_attr_value("$.has_more"))

    @property
    def max_cursor(self) -> int:
        return _int(self._get_attr_value("$.max_cursor"))

    @property
    def aweme_list(self) -> list:
        return self._data.get("aweme_list", [])

    def get_video_list(self) -> list:
        """将列表数据包装为 PostDetailFilter 实例列表"""
        result = []
        for aweme in self.aweme_list:
            result.append(PostDetailFilter({"aweme_detail": aweme}))
        return result

    def to_list(self) -> list[dict]:
        return [v.to_dict() for v in self.get_video_list()]


class UserCollectsFilter(JSONModel):
    """收藏夹列表过滤器"""

    @property
    def has_more(self) -> bool:
        return bool(self._get_attr_value("$.has_more"))

    @property
    def cursor(self) -> int:
        return _int(self._get_attr_value("$.cursor"))

    @property
    def collects_list(self) -> list:
        return self._data.get("collects_list", [])

    def to_list(self) -> list[dict]:
        result = []
        for c in self.collects_list:
            result.append({
                "id": str(c.get("collects_id", "")),
                "name": c.get("collects_name", ""),
                "count": c.get("total_number", 0),
            })
        return result


class UserMusicCollectionFilter(JSONModel):
    """音乐收藏过滤器"""

    @property
    def has_more(self) -> bool:
        return bool(self._get_attr_value("$.has_more"))

    @property
    def cursor(self) -> int:
        return _int(self._get_attr_value("$.cursor"))

    @property
    def music_list(self) -> list:
        return self._data.get("mc_list", [])

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


class UserFollowingFilter(JSONModel):
    """关注列表过滤器"""

    @property
    def has_more(self) -> bool:
        return bool(self._get_attr_value("$.has_more"))

    @property
    def offset(self) -> int:
        return _int(self._get_attr_value("$.offset"))

    @property
    def followings(self) -> list:
        return self._data.get("followings", [])


class UserFollowerFilter(JSONModel):
    """粉丝列表过滤器"""

    @property
    def has_more(self) -> bool:
        return bool(self._get_attr_value("$.has_more"))

    @property
    def offset(self) -> int:
        return _int(self._get_attr_value("$.offset"))

    @property
    def followers(self) -> list:
        return self._data.get("followers", [])


# ============================================================
# 视频详情
# ============================================================

class PostDetailFilter(JSONModel):
    """视频详情过滤器"""

    # === 基础 ===

    @property
    def aweme_id(self) -> str:
        return _str(self._get_attr_value("$.aweme_detail.aweme_id"))

    @property
    def aweme_type(self) -> int:
        return _int(self._get_attr_value("$.aweme_detail.aweme_type"))

    @property
    def desc(self) -> str:
        return replaceT(_str(self._get_attr_value("$.aweme_detail.desc")))

    @property
    def desc_raw(self) -> str:
        return _str(self._get_attr_value("$.aweme_detail.desc"))

    @property
    def create_time(self) -> int:
        return _int(self._get_attr_value("$.aweme_detail.create_time"))

    @property
    def duration(self) -> int:
        return _int(self._get_attr_value("$.aweme_detail.duration"))

    @property
    def region(self) -> str:
        return _str(self._get_attr_value("$.aweme_detail.region"))

    # === 作者 ===

    @property
    def author_nickname(self) -> str:
        return replaceT(_str(self._get_attr_value("$.aweme_detail.author.nickname")))

    @property
    def author_nickname_raw(self) -> str:
        return _str(self._get_attr_value("$.aweme_detail.author.nickname"))

    @property
    def author_uid(self) -> str:
        return _str(self._get_attr_value("$.aweme_detail.author.uid"))

    @property
    def author_sec_uid(self) -> str:
        return _str(self._get_attr_value("$.aweme_detail.author.sec_uid"))

    @property
    def author_short_id(self) -> str:
        return _str(self._get_attr_value("$.aweme_detail.author.short_id"))

    @property
    def author_unique_id(self) -> str:
        return _str(self._get_attr_value("$.aweme_detail.author.unique_id"))

    @property
    def author_avatar_url(self) -> str:
        return _str(self._get_attr_value("$.aweme_detail.author.avatar_thumb.url_list[0]"))

    @property
    def author_signature(self) -> str:
        return _str(self._get_attr_value("$.aweme_detail.author.signature"))

    @property
    def author_follower_count(self) -> int:
        return _int(self._get_attr_value("$.aweme_detail.author.follower_count"))

    @property
    def author_aweme_count(self) -> int:
        return _int(self._get_attr_value("$.aweme_detail.author.aweme_count"))

    @property
    def author_following_count(self) -> int:
        return _int(self._get_attr_value("$.aweme_detail.author.following_count"))

    @property
    def author_total_favorited(self) -> int:
        return _int(self._get_attr_value("$.aweme_detail.author.total_favorited"))

    @property
    def author_ip_location(self) -> str:
        return _str(self._get_attr_value("$.aweme_detail.author.ip_location"))

    # === 视频 URL（保留手写逻辑：去水印过滤） ===

    @property
    def bit_rate_list(self) -> list[dict]:
        val = self._get_attr_value("$.aweme_detail.video.bit_rate")
        return val if isinstance(val, list) else []

    @property
    def video_url(self) -> str:
        """无水印视频 URL（优先最高码率，兼容旧调用）"""
        urls = self.video_urls
        return urls[0] if urls else ""

    @property
    def video_urls(self) -> list[str]:
        """所有可用的无水印视频 URL（CDN 降级用，对齐 f2）"""
        bit_rate = self.bit_rate_list
        if bit_rate:
            url_list = get_nested(bit_rate[0], "play_addr", "url_list", default=[])
            urls = [u for u in url_list if u and "playwm" not in u]
            if urls:
                return urls
            if url_list:
                return list(url_list)
        url_list = self._get_attr_value("$.aweme_detail.video.play_addr.url_list")
        if isinstance(url_list, list):
            urls = [u for u in url_list if u and "playwm" not in u]
            if urls:
                return urls
            if url_list:
                return list(url_list)
        return []

    @property
    def cover_url(self) -> str:
        return _str(self._get_attr_value("$.aweme_detail.video.origin_cover.url_list[0]"))

    @property
    def animated_cover(self) -> str:
        return _str(self._get_attr_value("$.aweme_detail.video.animated_cover.url_list[0]"))

    @property
    def video_bit_rate_json(self) -> str:
        return json.dumps(self.bit_rate_list, ensure_ascii=False) if self.bit_rate_list else ""

    # === 图集（保留手写逻辑：条件过滤） ===

    @property
    def images(self) -> list[str]:
        """图集静态图 URL 列表"""
        imgs = self._get_attr_value("$.aweme_detail.images")
        if not isinstance(imgs, list):
            return []
        return [get_nested(img, "url_list", 0, default="") for img in imgs if img]

    @property
    def images_video(self) -> list[str]:
        """图集动图/实况 URL 列表"""
        imgs = self._get_attr_value("$.aweme_detail.images")
        if not isinstance(imgs, list):
            return []
        result = []
        for img in imgs:
            if img:
                vu = get_nested(img, "video", "play_addr", "url_list", 0, default="")
                result.append(vu)
        return result

    @property
    def is_image_post(self) -> bool:
        return self.aweme_type == 68

    @property
    def images_json(self) -> str:
        return json.dumps(self.images, ensure_ascii=False) if self.images else ""

    # === 音乐 ===

    @property
    def music_title(self) -> str:
        return replaceT(_str(self._get_attr_value("$.aweme_detail.music.title")))

    @property
    def music_url(self) -> str:
        return _str(self._get_attr_value("$.aweme_detail.music.play_url.url_list[0]"))

    @property
    def music_author(self) -> str:
        return replaceT(_str(self._get_attr_value("$.aweme_detail.music.author")))

    @property
    def music_author_raw(self) -> str:
        return _str(self._get_attr_value("$.aweme_detail.music.author"))

    @property
    def music_duration(self) -> int:
        return _int(self._get_attr_value("$.aweme_detail.music.duration"))

    @property
    def music_id(self) -> str:
        return _str(self._get_attr_value("$.aweme_detail.music.id"))

    @property
    def music_mid(self) -> str:
        return _str(self._get_attr_value("$.aweme_detail.music.mid"))

    @property
    def pgc_author(self) -> str:
        return replaceT(_str(self._get_attr_value(
            "$.aweme_detail.music.matched_pgc_sound.pgc_author"
        )))

    @property
    def pgc_author_title(self) -> str:
        return replaceT(_str(self._get_attr_value(
            "$.aweme_detail.music.matched_pgc_sound.pgc_author_title"
        )))

    @property
    def pgc_music_type(self) -> int:
        return _int(self._get_attr_value(
            "$.aweme_detail.music.matched_pgc_sound.pgc_music_type"
        ))

    @property
    def music_status(self) -> int:
        return _int(self._get_attr_value("$.aweme_detail.music.status"))

    @property
    def music_owner_handle(self) -> str:
        return _str(self._get_attr_value("$.aweme_detail.music.owner_handle"))

    @property
    def music_owner_id(self) -> str:
        return _str(self._get_attr_value("$.aweme_detail.music.owner_id"))

    @property
    def music_owner_nickname(self) -> str:
        return replaceT(_str(self._get_attr_value("$.aweme_detail.music.owner_nickname")))

    @property
    def music_play_url(self) -> str:
        return _str(self._get_attr_value("$.aweme_detail.music.play_url.url_list[0]"))

    @property
    def is_commerce_music(self) -> int:
        return _bool_int(self._get_attr_value("$.aweme_detail.music.is_commerce_music"))

    # === 统计 ===

    @property
    def statistics(self) -> dict:
        val = self._get_attr_value("$.aweme_detail.statistics")
        return val if isinstance(val, dict) else {}

    @property
    def digg_count(self) -> int:
        return _int(self.statistics.get("digg_count"))

    @property
    def comment_count(self) -> int:
        return _int(self.statistics.get("comment_count"))

    @property
    def share_count(self) -> int:
        return _int(self.statistics.get("share_count"))

    @property
    def collect_count(self) -> int:
        return _int(self.statistics.get("collect_count"))

    @property
    def admire_count(self) -> int:
        return _int(self.statistics.get("admire_count"))

    # === 合集 ===

    @property
    def mix_id(self) -> str:
        return _str(self._get_attr_value("$.aweme_detail.mix_info.mix_id"))

    @property
    def mix_name(self) -> str:
        return _str(self._get_attr_value("$.aweme_detail.mix_info.mix_name"))

    @property
    def mix_desc(self) -> str:
        return replaceT(_str(self._get_attr_value("$.aweme_detail.mix_info.mix_desc")))

    @property
    def mix_create_time(self) -> int:
        return _int(self._get_attr_value("$.aweme_detail.mix_info.mix_create_time"))

    @property
    def mix_pic_type(self) -> int:
        return _int(self._get_attr_value("$.aweme_detail.mix_info.mix_pic_type"))

    @property
    def mix_type(self) -> int:
        return _int(self._get_attr_value("$.aweme_detail.mix_info.mix_type"))

    @property
    def mix_share_url(self) -> str:
        return _str(self._get_attr_value("$.aweme_detail.mix_info.mix_share_url"))

    # === 状态/权限 ===

    @property
    def is_prohibited(self) -> bool:
        return bool(_int(self._get_attr_value("$.aweme_detail.status.is_prohibited")))

    @property
    def is_ads(self) -> int:
        return _bool_int(self._get_attr_value("$.aweme_detail.is_ads"))

    @property
    def is_story(self) -> int:
        return _bool_int(self._get_attr_value("$.aweme_detail.is_story"))

    @property
    def is_top(self) -> int:
        return _bool_int(self._get_attr_value("$.aweme_detail.is_top"))

    @property
    def is_long_video(self) -> int:
        return _bool_int(self._get_attr_value("$.aweme_detail.is_long_video"))

    @property
    def private_status(self) -> int:
        return _int(self._get_attr_value("$.aweme_detail.status.private_status"))

    @property
    def is_delete(self) -> int:
        return _bool_int(self._get_attr_value("$.aweme_detail.status.is_delete"))

    @property
    def can_comment(self) -> int:
        return _bool_int(self._get_attr_value("$.aweme_detail.aweme_control.can_comment"), 1)

    @property
    def can_forward(self) -> int:
        return _bool_int(self._get_attr_value("$.aweme_detail.aweme_control.can_forward"), 1)

    @property
    def can_share(self) -> int:
        return _bool_int(self._get_attr_value("$.aweme_detail.aweme_control.can_share"), 1)

    @property
    def download_setting(self) -> int:
        return _int(self._get_attr_value("$.aweme_detail.status.download_setting"))

    @property
    def allow_douplus(self) -> int:
        return _bool_int(self._get_attr_value("$.aweme_detail.status.allow_douplus"))

    @property
    def allow_share(self) -> int:
        return _bool_int(self._get_attr_value("$.aweme_detail.status.allow_share"), 1)

    # === 标签 ===

    @property
    def hashtag_ids(self) -> str:
        extras = self._get_attr_value("$.aweme_detail.text_extra")
        if not isinstance(extras, list):
            return ""
        ids = [str(e.get("hashtag_id", "")) for e in extras if e.get("hashtag_id")]
        return json.dumps(ids, ensure_ascii=False) if ids else ""

    @property
    def hashtag_names(self) -> str:
        extras = self._get_attr_value("$.aweme_detail.text_extra")
        if not isinstance(extras, list):
            return ""
        names = [e.get("hashtag_name", "") for e in extras if e.get("hashtag_name")]
        return json.dumps(names, ensure_ascii=False) if names else ""

    # === 输出 ===

    def to_dict(self) -> dict:
        """前端展示 + 文件命名用（含时效 URL）"""
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
        """入库用（剔除时效 CDN URL，保留稳定字段）"""
        return {
            "aweme_id": self.aweme_id, "aweme_type": self.aweme_type,
            "desc": self.desc, "author_nickname": self.author_nickname,
            "author_uid": self.author_uid, "author_sec_uid": self.author_sec_uid,
            "create_time": self.create_time, "duration": self.duration,
            "video_url": self.video_url, "cover_url": self.cover_url,
            "video_bit_rate": self.video_bit_rate_json,
            "animated_cover": self.animated_cover,
            "is_image_post": self.is_image_post,
            "music_title": self.music_title,
            "digg_count": self.digg_count, "comment_count": self.comment_count,
            "share_count": self.share_count, "collect_count": self.collect_count,
            "mix_id": self.mix_id, "mix_name": self.mix_name,
            "is_prohibited": int(self.is_prohibited),
            "author_nickname_raw": self.author_nickname_raw,
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


# ============================================================
# 评论
# ============================================================

class PostCommentFilter(JSONModel):
    """评论列表过滤器"""

    @property
    def has_more(self) -> bool:
        return bool(self._get_attr_value("$.has_more"))

    @property
    def cursor(self) -> int:
        return _int(self._get_attr_value("$.cursor"))

    @property
    def comments(self) -> list:
        return self._data.get("comments", [])


class PostRelatedFilter(UserPostFilter):
    """相关推荐过滤器"""
    pass


class HomePostSearchFilter(UserPostFilter):
    """主页搜索过滤器"""
    pass


class SuggestWordFilter(JSONModel):
    """搜索建议词过滤器"""

    @property
    def words(self) -> list[str]:
        val = self._get_list_attr_value("$.data[0].words[*].word")
        return val if isinstance(val, list) else []


# ============================================================
# 直播相关
# ============================================================

class UserLiveFilter(JSONModel):
    """直播信息过滤器（web_rid 接口）"""

    @property
    def room_id(self) -> str:
        return _str(self._get_attr_value("$.data.data[0].id_str"))

    @property
    def live_status(self) -> int:
        return _int(self._get_attr_value("$.data.data[0].status"))

    @property
    def live_title(self) -> str:
        return replaceT(_str(self._get_attr_value("$.data.data[0].title")))

    @property
    def cover_url(self) -> str:
        return _str(self._get_attr_value("$.data.data[0].cover.url_list[0]"))

    @property
    def user_count(self) -> str:
        return _str(self._get_attr_value("$.data.data[0].stats.user_count_str"), "0")

    @property
    def flv_pull_url(self) -> dict:
        val = self._get_attr_value("$.data.data[0].stream_url.flv_pull_url")
        return val if isinstance(val, dict) else {}

    @property
    def m3u8_pull_url(self) -> dict:
        val = self._get_attr_value("$.data.data[0].stream_url.hls_pull_url_map")
        return val if isinstance(val, dict) else {}

    @property
    def nickname(self) -> str:
        return replaceT(_str(self._get_attr_value("$.data.data[0].owner.nickname")))

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


class UserLive2Filter(JSONModel):
    """直播信息过滤器（room_id 接口）"""

    @property
    def room_id(self) -> str:
        return _str(self._get_attr_value("$.data.room.id"))

    @property
    def live_status(self) -> int:
        return _int(self._get_attr_value("$.data.room.status"))

    @property
    def live_title(self) -> str:
        return replaceT(_str(self._get_attr_value("$.data.room.title")))

    @property
    def flv_pull_url(self) -> dict:
        val = self._get_attr_value("$.data.room.stream_url.flv_pull_url")
        return val if isinstance(val, dict) else {}

    @property
    def m3u8_pull_url(self) -> dict:
        val = self._get_attr_value("$.data.room.stream_url.hls_pull_url_map")
        return val if isinstance(val, dict) else {}

    @property
    def nickname(self) -> str:
        return replaceT(_str(self._get_attr_value("$.data.room.owner.nickname")))

    @property
    def is_live(self) -> bool:
        return self.live_status == 2


class UserLiveStatusFilter(JSONModel):
    """用户直播状态过滤器"""

    @property
    def live_status(self) -> int:
        return _int(self._get_attr_value("$.data[0].user_live[0].live_status"))

    @property
    def room_id(self) -> str:
        return _str(self._get_attr_value("$.data[0].user_live[0].room_id"))


class FollowingUserLiveFilter(JSONModel):
    """关注用户直播过滤器"""

    @property
    def live_rooms(self) -> list:
        val = self._get_attr_value("$.data.data")
        return val if isinstance(val, list) else []


class LiveImFetchFilter(JSONModel):
    """直播弹幕初始化过滤器"""

    @property
    def cursor(self) -> str:
        return _str(self._get_attr_value("$.extra.cursor"))

    @property
    def internal_ext(self) -> str:
        return _str(self._get_attr_value("$.internal_ext"))


class QueryUserFilter(JSONModel):
    """用户查询结果过滤器"""

    @property
    def status_code(self) -> int:
        return _int(self._get_attr_value("$.status_code"))

    @property
    def status_msg(self) -> str:
        return _str(self._get_attr_value("$.status_msg"))

    @property
    def browser_name(self) -> str:
        return _str(self._get_attr_value("$.browser_name"))

    @property
    def create_time(self) -> str:
        return _str(self._get_attr_value("$.create_time"))

    @property
    def firebase_instance_id(self) -> str:
        return _str(self._get_attr_value("$.firebase_instance_id"))

    @property
    def user_unique_id(self) -> str:
        return _str(self._get_attr_value("$.id"))

    @property
    def last_time(self) -> str:
        return _str(self._get_attr_value("$.last_time"))

    @property
    def user_agent(self) -> str:
        return _str(self._get_attr_value("$.user_agent"))

    @property
    def user_uid(self) -> str:
        return _str(self._get_attr_value("$.user_uid"))

    @property
    def user_uid_type(self) -> int:
        return _int(self._get_attr_value("$.user_uid_type"))

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


class PostStatsFilter(JSONModel):
    """作品统计过滤器"""

    @property
    def status_code(self) -> int:
        return _int(self._get_attr_value("$.status_code"), -1)

    @property
    def status_msg(self) -> str:
        return _str(self._get_attr_value("$.status_msg"))

    def to_dict(self) -> dict:
        return {
            "status_code": self.status_code,
            "status_msg": self.status_msg,
        }
