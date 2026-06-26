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
