"""数据库操作模块

通过 Rust 桥接写入数据库，确保数据格式一致。
"""

import logging
from core import db_bridge

logger = logging.getLogger(__name__)


def save_download_record(
    aweme_id: str = None,
    download_type: str = "video",
    title: str = None,
    author_nickname: str = None,
    author_sec_uid: str = None,
    file_path: str = None,
    file_size: int = 0,
    cover_url: str = None,
    status: str = "completed",
    error_msg: str = None,
) -> bool:
    """保存下载记录（通过 Rust 桥接）"""
    return db_bridge.save_download_record({
        "aweme_id": aweme_id,
        "download_type": download_type,
        "title": title,
        "author_nickname": author_nickname,
        "author_sec_uid": author_sec_uid,
        "file_path": file_path,
        "file_size": file_size,
        "cover_url": cover_url,
        "status": status,
        "error_msg": error_msg,
    })


def save_video_info(video_data: dict) -> bool:
    """保存视频信息（通过 Rust 桥接）

    直接透传 to_db_dict() 的全部字段，只做类型修正。
    Rust 端 VideoInfo 所有 f2 字段都有 #[serde(default)]，缺失字段自动填 0/NULL。
    """
    # 需要转 int 的字段（Python bool → Rust i32 会报错）
    INT_FIELDS = {
        "aweme_type", "duration", "digg_count", "comment_count", "share_count",
        "collect_count", "is_ads", "is_story", "is_top", "is_long_video",
        "private_status", "is_delete", "music_duration", "pgc_music_type",
        "music_status", "is_commerce_music", "mix_pic_type", "mix_type",
        "mix_create_time", "can_comment", "can_forward", "can_share",
        "download_setting", "allow_douplus", "allow_share", "admire_count",
        "is_prohibited", "create_time",
    }
    clean_data = {}
    for k, v in video_data.items():
        if k in INT_FIELDS:
            clean_data[k] = int(v) if v else 0
        else:
            clean_data[k] = v
    return db_bridge.save_video_info(clean_data)


def save_user_info(user_data: dict) -> bool:
    """保存用户信息（通过 Rust 桥接）

    支持两种数据源：
    1. PostDetailFilter.to_db_dict() 的 author_* 前缀字段
    2. UserProfileFilter.to_dict() 的直接字段
    Rust 端 UserInfo 所有 f2 字段都有 #[serde(default)]，缺失字段自动填 0/NULL。
    """
    # author_* → 直接字段名映射
    AUTHOR_MAP = {
        "author_sec_uid": "sec_user_id",
        "author_nickname": "nickname",
        "author_uid": "uid",
        "author_avatar_url": "avatar_url",
        "author_unique_id": "unique_id",
        "author_signature": "signature",
        "author_ip_location": "ip_location",
        "author_aweme_count": "aweme_count",
        "author_follower_count": "follower_count",
        "author_following_count": "following_count",
        "author_total_favorited": "total_favorited",
    }
    INT_FIELDS = {
        "aweme_count", "follower_count", "following_count", "total_favorited",
        "live_status", "favoriting_count", "gender", "is_ban", "is_block",
        "is_blocked", "is_star", "mix_count", "mplatform_followers_count",
        "user_age",
    }

    clean_data = {}
    seen = set()
    for k, v in user_data.items():
        # author_* 前缀映射
        if k in AUTHOR_MAP:
            mapped = AUTHOR_MAP[k]
            clean_data[mapped] = v
            seen.add(mapped)
        else:
            clean_data[k] = v
            seen.add(k)

    # 确保 sec_user_id 存在
    if not clean_data.get("sec_user_id"):
        return False

    # 数值字段转 int
    for k in INT_FIELDS:
        if k in clean_data:
            clean_data[k] = int(clean_data[k]) if clean_data[k] else 0

    return db_bridge.save_user_info(clean_data)


def save_batch_results(results: list, download_type: str = "batch") -> dict:
    """批量保存下载结果

    Args:
        results: [{"path": str, "detail": dict}, ...]
        download_type: 下载类型

    Returns:
        {"saved": int, "failed": int}
    """
    saved = 0
    failed = 0

    for item in results:
        detail = item.get("detail", {})
        file_path = item.get("path")

        # 1. 保存下载记录
        if save_download_record(
            aweme_id=detail.get("aweme_id"),
            download_type=download_type,
            title=detail.get("desc"),
            author_nickname=detail.get("author_nickname"),
            author_sec_uid=detail.get("author_sec_uid"),
            file_path=file_path,
            cover_url=detail.get("cover_url"),
            status="completed",
        ):
            saved += 1
        else:
            failed += 1

        # 2. 保存视频信息
        if detail.get("aweme_id"):
            save_video_info(detail)

        # 3. 保存用户信息
        if detail.get("author_sec_uid") is not None:
            save_user_info(detail)

    logger.info("[db] 批量保存完成: 成功=%d, 失败=%d, 总计=%d", saved, failed, len(results))
    return {"saved": saved, "failed": failed}
