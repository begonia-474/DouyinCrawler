"""
数据库桥接模块
供 Python 通过 Rust 的 PyO3 方法写入数据库

_save_download_record, _save_video_info, _save_user_info
由 Rust 端在初始化时注入
"""

import logging

logger = logging.getLogger(__name__)

# 由 Rust db_bridge.rs 的 register_db_bridge() 注入
_save_download_record = None
_save_video_info = None
_save_user_info = None
_has_user = None


def save_download_record(data: dict) -> bool:
    """保存下载记录（通过 Rust）"""
    if _save_download_record is None:
        logger.error("[db_bridge] _save_download_record 未注册")
        return False
    try:
        _save_download_record(data)
        return True
    except Exception as e:
        logger.error("[db_bridge] save_download_record 失败: %s", e)
        return False


def save_video_info(data: dict) -> bool:
    """保存视频信息（通过 Rust）"""
    if _save_video_info is None:
        logger.error("[db_bridge] _save_video_info 未注册")
        return False
    try:
        _save_video_info(data)
        return True
    except Exception as e:
        logger.error("[db_bridge] save_video_info 失败: %s", e)
        return False


def save_user_info(data: dict) -> bool:
    """保存用户信息（通过 Rust）"""
    if _save_user_info is None:
        logger.error("[db_bridge] _save_user_info 未注册")
        return False
    try:
        _save_user_info(data)
        return True
    except Exception as e:
        logger.error("[db_bridge] save_user_info 失败: %s", e)
        return False


def has_user(sec_user_id: str) -> bool:
    """查询用户是否已存在于数据库"""
    if _has_user is None:
        logger.error("[db_bridge] _has_user 未注册")
        return False
    try:
        return _has_user(sec_user_id)
    except Exception as e:
        logger.error("[db_bridge] has_user 失败: %s", e)
        return False
