"""数据库桥接模块 — Python→Rust DB 写入的 stubs

此模块的 `_save_*` 和 `_has_user` 函数在应用启动时由 Rust 侧的
`src-tauri/src/python/db_bridge.rs::register_db_bridge()` 通过 PyO3 `setattr` 注入。

调用链：
    Python 业务代码 → core.db（facade）→ 本模块（stubs）→ Rust PyO3 closure → core::db::Database

Python 侧**不直接执行任何 SQL**。所有数据库操作最终由 Rust 的 rusqlite 执行。

注意：任务生命周期函数（_create_task, _update_task_status, _create_task_item, _update_task_item_status）
已迁移到 Rust TaskApplicationService，不再通过 Python 桥接注册。
"""

import logging

logger = logging.getLogger(__name__)

# 由 Rust db_bridge.rs 的 register_db_bridge() 注入。
# 直播写入口自 Issue 09 起不再注入；字段仅保留给 Issue 10 做兼容清理。
_save_video_info = None
_save_user_info = None
_save_live_record = None
_has_user = None


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
    """[legacy disabled] 直播记录由 Rust TaskApplicationService 独占。"""
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
