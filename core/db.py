"""数据库操作模块（Python 侧 facade）

架构边界：
- **Rust 拥有 SQLite 连接和所有 SQL 执行**。Python 不直接打开数据库。
- 本模块是纯透传层：接收 Python 数据 → 原样通过 db_bridge 触发 Rust 写入。
- 所有数据清洗（bool→int、author_* 字段映射）由 Rust 侧统一处理：
  - `db_bridge.rs`: bool_to_int() 递归转换 JSON bool 为 int
  - `db.rs` UserInfo: #[serde(alias = "author_*")] 接受两种字段命名
- db_bridge 中的 `_save_*` 函数是 Rust 在启动时通过 PyO3 注入的闭包。
- 前端也有两条直接写入路径（不经 Python）：live_records（Tauri command）、music_collection（Tauri command）。

注意：任务生命周期函数（create_task, update_task_status, create_task_item, update_task_item_status）
已迁移到 Rust TaskApplicationService，不再通过 Python 桥接注册。

数据流：
    Python 业务逻辑 → db.py（透传）→ db_bridge.py（转发）→ Rust PyO3 closure（清洗+写入）→ db.rs（SQL）
"""

import logging
from pathlib import Path

from core import db_bridge

logger = logging.getLogger(__name__)


def save_video_info(video_data: dict) -> bool:
    """保存视频信息（纯透传，Rust 侧处理 bool→int 转换）"""
    return db_bridge.save_video_info(video_data)


def save_user_info(user_data: dict) -> bool:
    """保存用户信息（通过 Rust 桥接，支持 author_* 别名）

    支持两种数据源（Rust serde alias 自动兼容）：
    1. PostDetailFilter.to_db_dict() 的 author_* 前缀字段
    2. UserProfileFilter.to_dict() 的直接字段
    """
    sec_uid = user_data.get("sec_user_id") or user_data.get("author_sec_uid")
    if not sec_uid:
        return False
    return db_bridge.save_user_info(user_data)


def save_live_record(record: dict) -> bool:
    """[deprecated] 保存直播录制记录（通过 Rust 桥接）

    Deprecated: P2-02 后 live 记录由 Rust emit.rs 持久化。
    此函数保留仅用于向前兼容，不应在新代码中调用。

    Args:
        record: 直播记录字典，字段参见 NewLiveRecord:
            room_id, web_rid, title, nickname, sec_user_id,
            file_path, file_size, duration_sec, status, started_at, ended_at, cover_url
    """
    return db_bridge.save_live_record(record)


def save_batch_results(results: list, download_type: str = "batch") -> dict:
    """批量保存下载结果（video_info + user_info）

    Args:
        results: [{"path": str, "detail": dict}, ...]
        download_type: 下载类型（已废弃，保留参数兼容）

    Returns:
        {"saved": int, "failed": int}
    """
    saved = 0
    failed = 0
    seen_users = set()

    for item in results:
        detail = item.get("detail", {})

        # 1. 保存视频信息
        if detail.get("aweme_id"):
            save_video_info(detail)
            saved += 1
        else:
            failed += 1

        # 2. 保存用户信息（去重 + has_user 检查，避免不完整数据覆盖）
        sec_uid = detail.get("author_sec_uid")
        if sec_uid and sec_uid not in seen_users:
            seen_users.add(sec_uid)
            if not db_bridge.has_user(sec_uid):
                save_user_info(detail)

    logger.info("[db] 批量保存完成: 成功=%d, 失败=%d, 总计=%d", saved, failed, len(results))
    return {"saved": saved, "failed": failed}
