"""数据库桥接模块 — Python→Rust DB 写入的 stubs

此模块的 `_save_*` 和 `_has_user` 函数在应用启动时由 Rust 侧的
`src-tauri/src/python/db_bridge.rs::register_db_bridge()` 通过 PyO3 `setattr` 注入。

调用链：
    Python 业务代码 → core.db（facade）→ 本模块（stubs）→ Rust PyO3 closure → core::db::Database

Python 侧**不直接执行任何 SQL**。所有数据库操作最终由 Rust 的 rusqlite 执行。
"""

import logging

logger = logging.getLogger(__name__)

# 由 Rust db_bridge.rs 的 register_db_bridge() 注入
_save_download_record = None
_save_video_info = None
_save_user_info = None
_save_live_record = None
_has_user = None

# 任务管理（同样由 Rust 注入）
_create_task = None
_update_task_status = None
_create_task_item = None
_update_task_item_status = None


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


def save_live_record(data: dict) -> bool:
    """保存直播录制记录（通过 Rust）"""
    if _save_live_record is None:
        logger.error("[db_bridge] _save_live_record 未注册")
        return False
    try:
        _save_live_record(data)
        return True
    except Exception as e:
        logger.error("[db_bridge] save_live_record 失败: %s", e)
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


# ============================================================
# 任务管理桥接
# ============================================================

def create_task(task_id: str, mode: str, url: str, title: str = None) -> bool:
    """在数据库创建下载任务记录"""
    if _create_task is None:
        logger.error("[db_bridge] _create_task 未注册")
        return False
    try:
        _create_task({"id": task_id, "mode": mode, "url": url, "title": title})
        return True
    except Exception as e:
        logger.error("[db_bridge] create_task 失败: %s", e)
        return False


def update_task_status(task_id: str, status: str, error_msg: str = None) -> bool:
    """更新任务状态"""
    if _update_task_status is None:
        logger.error("[db_bridge] _update_task_status 未注册")
        return False
    try:
        _update_task_status(task_id, status, error_msg)
        return True
    except Exception as e:
        logger.error("[db_bridge] update_task_status 失败: %s", e)
        return False


def create_task_item(task_id: str, aweme_id: str = None, title: str = None,
                     author_nickname: str = None, cover_url: str = None) -> bool:
    """创建任务子项"""
    if _create_task_item is None:
        logger.error("[db_bridge] _create_task_item 未注册")
        return False
    try:
        _create_task_item({
            "task_id": task_id, "aweme_id": aweme_id, "title": title,
            "author_nickname": author_nickname, "cover_url": cover_url,
        })
        return True
    except Exception as e:
        logger.error("[db_bridge] create_task_item 失败: %s", e)
        return False


def update_task_item_status(task_id: str, aweme_id: str, status: str,
                            file_path: str = None, file_size: int = 0,
                            error_msg: str = None) -> bool:
    """更新子项状态（completed / skipped / failed）"""
    if _update_task_item_status is None:
        logger.error("[db_bridge] _update_task_item_status 未注册")
        return False
    try:
        _update_task_item_status(task_id, aweme_id, status, file_path, file_size, error_msg)
        return True
    except Exception as e:
        logger.error("[db_bridge] update_task_item_status 失败: %s", e)
        return False
