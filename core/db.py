"""数据库操作模块（Python 侧 facade）

架构边界：
- **Rust 拥有 SQLite 连接和所有 SQL 执行**。Python 不直接打开数据库。
- 本模块是纯透传层：接收 Python 数据 → 原样通过 db_bridge 触发 Rust 写入。
- 所有数据清洗（bool→int、author_* 字段映射）由 Rust 侧统一处理：
  - `db_bridge.rs`: bool_to_int() 递归转换 JSON bool 为 int
  - `db.rs` UserInfo: #[serde(alias = "author_*")] 接受两种字段命名
- db_bridge 中的 `_save_*` 函数是 Rust 在启动时通过 PyO3 注入的闭包。
- 前端也有两条直接写入路径（不经 Python）：live_records（Tauri command）、music_collection（Tauri command）。

数据流：
    Python 业务逻辑 → db.py（透传）→ db_bridge.py（转发）→ Rust PyO3 closure（清洗+写入）→ db.rs（SQL）
"""

import logging
from pathlib import Path

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
    """保存直播录制记录（通过 Rust 桥接）

    Args:
        record: 直播记录字典，字段参见 NewLiveRecord:
            room_id, web_rid, title, nickname, sec_user_id,
            file_path, file_size, duration_sec, status, started_at, ended_at, cover_url
    """
    return db_bridge.save_live_record(record)


# ============================================================
# 下载任务管理
# ============================================================

def create_task(task_id: str, mode: str, url: str, title: str = None) -> bool:
    """创建下载任务记录（通过 Rust 桥接）"""
    return db_bridge.create_task(task_id, mode, url, title)


def update_task_status(task_id: str, status: str, error_msg: str = None) -> bool:
    """更新任务状态"""
    return db_bridge.update_task_status(task_id, status, error_msg)


def create_task_item(task_id: str, aweme_id: str = None, title: str = None,
                     author_nickname: str = None, cover_url: str = None) -> bool:
    """创建任务子项"""
    return db_bridge.create_task_item(task_id, aweme_id, title, author_nickname, cover_url)


def update_task_item_status(task_id: str, aweme_id: str, status: str,
                            file_path: str = None, file_size: int = 0,
                            error_msg: str = None) -> bool:
    """更新子项状态（completed / skipped / failed）"""
    return db_bridge.update_task_item_status(
        task_id, aweme_id, status, file_path, file_size, error_msg
    )


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
    seen_users = set()  # 问题3修复：避免同一用户重复 upsert

    for item in results:
        detail = item.get("detail", {})
        file_path = item.get("path")

        # 问题2修复：从文件获取实际大小
        file_size = 0
        if file_path:
            try:
                file_size = Path(file_path).stat().st_size
            except (OSError, ValueError):
                pass

        # 1. 保存下载记录
        if save_download_record(
            aweme_id=detail.get("aweme_id"),
            download_type=download_type,
            title=detail.get("desc"),
            author_nickname=detail.get("author_nickname"),
            author_sec_uid=detail.get("author_sec_uid"),
            file_path=file_path,
            file_size=file_size,
            cover_url=detail.get("cover_url"),
            status="completed",
        ):
            saved += 1
        else:
            failed += 1

        # 2. 保存视频信息
        if detail.get("aweme_id"):
            save_video_info(detail)

        # 3. 保存用户信息（问题1+3修复：去重 + has_user 检查，避免不完整数据覆盖）
        sec_uid = detail.get("author_sec_uid")
        if sec_uid and sec_uid not in seen_users:
            seen_users.add(sec_uid)
            if not db_bridge.has_user(sec_uid):
                save_user_info(detail)

    logger.info("[db] 批量保存完成: 成功=%d, 失败=%d, 总计=%d", saved, failed, len(results))
    return {"saved": saved, "failed": failed}
