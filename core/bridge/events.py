"""
Tauri 事件桥接模块
将 Python 的任务状态推送到前端（通过 Rust Tauri 事件系统）

_emit_func 由 Rust 端在初始化时注入，签名为:
    _emit_func(task_id: str, task_type: str, data: dict) -> None
"""

import logging
import sys

logger = logging.getLogger(__name__)

# 由 Rust emit.rs 的 register_app_handle() 注入
_emit_func = None


def set_emit_func(func):
    """设置 emit 函数（由 Rust 调用）"""
    global _emit_func
    _emit_func = func
    logger.info("[tauri_bridge] _emit_func 已设置: %s", type(func).__name__)


def emit(task_id: str, task_type: str, data: dict):
    """发射任务状态事件到前端"""
    global _emit_func
    logger.info("[tauri_bridge.emit] 被调用 task_id=%s, task_type=%s, status=%s", task_id, task_type, data.get("status"))

    if _emit_func is None:
        # 尝试从模块属性重新获取（兼容 Rust setattr 方式）
        current_module = sys.modules.get(__name__)
        if current_module and hasattr(current_module, '_emit_func'):
            _emit_func = getattr(current_module, '_emit_func')
            logger.info("[tauri_bridge.emit] 从模块属性获取 _emit_func: %s", type(_emit_func).__name__)

    if _emit_func is None:
        logger.warning("[tauri_bridge] _emit_func 未注册，跳过事件 task_id=%s", task_id)
        return
    try:
        logger.info("[tauri_bridge] 调用 _emit_func 发射事件 task_id=%s", task_id)
        _emit_func(task_id, task_type, data)
        logger.info("[tauri_bridge] 事件发射成功 task_id=%s", task_id)
    except Exception as e:
        logger.error("[tauri_bridge] 事件发射失败: %s", e, exc_info=True)
