"""管理 DouyinHandler 实例和配置，支持持久化

职责拆分：
- TaskManager（本文件）：配置管理 + Handler 生命周期 + 事件广播（门面）
- LiveRecordManager（live_manager.py）：直播录制任务管理

注：批量下载（BatchManager）已迁移到 Rust TaskApplicationService，
    batch_manager.py 保留为废弃代码。
"""

import json
import uuid
import asyncio
from pathlib import Path
from core.logger import get_logger, setup_logging
from core.task.live_manager import LiveRecordManager
from core.handler import DouyinHandler

# 确保日志系统初始化（PyO3 路径不会经过 server.py）
setup_logging()

logger = get_logger(__name__)

# 项目根目录（backend 的父目录）
PROJECT_ROOT = Path(__file__).parent.parent
CONFIG_DIR = PROJECT_ROOT / "config"
CONFIG_PATH = CONFIG_DIR / "app.json"


class TaskManager:
    def __init__(self):
        self._cookie: str = ""
        self._download_path: str = "Download"
        self._naming: str = "{create}_{desc}"
        self._encryption: str = "ab"
        self._proxy: str = ""
        self._app_name: str = "douyin"
        self._folderize: bool = False
        self._music: bool = False
        self._cover: bool = False
        self._desc: bool = False
        self._interval: str = None
        self._page_counts: int = 20
        self._max_counts: int = 0
        self._timeout: int = 5
        self._max_connections: int = 5
        self._max_retries: int = 5
        self._max_tasks: int = 10
        self._handler: DouyinHandler | None = None
        self._live_mgr = LiveRecordManager()
        self._load_config()

    # ============================================================
    # 配置管理
    # ============================================================

    def _load_config(self):
        """Phase 6: 不再直接读取 config/app.json，由 Rust init_config() 推送配置。"""
        logger.info("[_load_config] 跳过文件读取，等待 Rust 推送配置")

    def _save_config(self):
        """Phase 6: 已废弃，由 Rust ConfigManager 管理配置文件。保留方法签名避免调用方报错。"""
        logger.debug("[_save_config] 已废弃，配置由 Rust 管理")
        return

    def update_config(self, cookie: str = None, download_path: str = None,
                      naming: str = None, encryption: str = None, proxy: str = None,
                      app_name: str = None, folderize: bool = None,
                      music: bool = None, cover: bool = None, desc: bool = None,
                      interval: str = None, page_counts: int = None,
                      max_counts: int = None, timeout: int = None,
                      max_connections: int = None, max_retries: int = None,
                      max_tasks: int = None, save: bool = True):
        if cookie is not None:
            logger.info("[update_config] 收到 cookie (len={})", len(cookie))
            if '\n' in cookie or '\r' in cookie:
                logger.warning("[update_config] cookie 中包含换行符! \\n={}, \\r={}",
                               cookie.count('\n'), cookie.count('\r'))
            self._cookie = " ".join(cookie.split())
            logger.info("[update_config] cookie 已清理 (len={})", len(self._cookie))
        if download_path is not None:
            self._download_path = download_path
        if naming is not None:
            self._naming = naming
        if encryption is not None:
            self._encryption = encryption
        if proxy is not None:
            self._proxy = proxy
        if app_name is not None:
            self._app_name = app_name
        if folderize is not None:
            self._folderize = folderize
        if music is not None:
            self._music = music
        if cover is not None:
            self._cover = cover
        if desc is not None:
            self._desc = desc
        if interval is not None:
            self._interval = interval
        if page_counts is not None:
            self._page_counts = page_counts
        if max_counts is not None:
            self._max_counts = max_counts
        if timeout is not None:
            self._timeout = timeout
        if max_connections is not None:
            self._max_connections = max_connections
        if max_retries is not None:
            self._max_retries = max_retries
        if max_tasks is not None:
            self._max_tasks = max_tasks
        self._handler = None  # 重建 handler
        # Phase 6: 不再由 Python 写入 config/app.json，由 Rust ConfigManager 管理

    @property
    def is_configured(self) -> bool:
        return bool(self._cookie)

    @property
    def config_summary(self) -> dict:
        return {
            "has_cookie": bool(self._cookie),
            "download_path": self._download_path,
            "naming": self._naming,
            "encryption": self._encryption,
            "has_proxy": bool(self._proxy),
            "app_name": self._app_name,
            "folderize": self._folderize,
            "music": self._music,
            "cover": self._cover,
            "desc": self._desc,
            "page_counts": self._page_counts,
            "max_counts": self._max_counts,
            "timeout": self._timeout,
            "max_connections": self._max_connections,
            "max_retries": self._max_retries,
            "max_tasks": self._max_tasks,
        }

    def get_config_dict(self) -> dict:
        """获取完整配置字典（供 Rust 调用）"""
        return {
            "cookie": self._cookie,
            "download_path": self._download_path,
            "naming": self._naming,
            "encryption": self._encryption,
            "proxy": self._proxy,
            "app_name": self._app_name,
            "folderize": self._folderize,
            "music": self._music,
            "cover": self._cover,
            "desc": self._desc,
            "interval": self._interval,
            "page_counts": self._page_counts,
            "max_counts": self._max_counts,
            "timeout": self._timeout,
            "max_connections": self._max_connections,
            "max_retries": self._max_retries,
            "max_tasks": self._max_tasks,
        }

    # ============================================================
    # Handler 生命周期
    # ============================================================

    @property
    def handler(self) -> DouyinHandler:
        if self._handler is None:
            proxies = None
            if self._proxy:
                proxies = {"http://": self._proxy, "https://": self._proxy}
            download_path = Path(self._download_path)
            if not download_path.is_absolute():
                download_path = PROJECT_ROOT / download_path
            logger.info("[handler] 创建 DouyinHandler (has_cookie={})", bool(self._cookie))
            logger.info("[handler] download_path={} (原始={})", download_path, self._download_path)
            logger.info("[handler] encryption={}, max_retries={}, timeout={}, max_connections={}",
                        self._encryption, self._max_retries, self._timeout, self._max_connections)
            self._handler = DouyinHandler(
                cookie=self._cookie,
                download_path=str(download_path),
                naming=self._naming,
                encryption=self._encryption,
                proxies=proxies,
                app_name=self._app_name,
                folderize=self._folderize,
                music=self._music,
                cover=self._cover,
                desc=self._desc,
                interval=self._interval,
                page_counts=self._page_counts,
                max_counts=self._max_counts,
                timeout=self._timeout,
                max_connections=self._max_connections,
                max_retries=self._max_retries,
                max_tasks=self._max_tasks,
            )
            logger.info("已创建 DouyinHandler (path={})", self._download_path)
        return self._handler

    # ============================================================
    # 直播录制（委托给 LiveRecordManager）
    # ============================================================

    def start_live_record(self, url: str) -> str:
        """启动直播录制，返回 task_id"""
        return self._live_mgr.start_live_record(url, self.handler, self.broadcast_task_update_sync)

    def stop_live_record(self, task_id: str) -> bool:
        """停止直播录制"""
        return self._live_mgr.stop_live_record(task_id)

    def get_live_status(self) -> dict[str, dict]:
        """获取所有录制任务状态"""
        return self._live_mgr.get_live_status()

    # ============================================================
    # 统一下载入口（mode 调度）
    # ============================================================

    def start_download(self, mode: str, url: str) -> str:
        """统一下载入口（Phase 7: 仅 live 模式仍走此路径，其他模式已迁移到 Rust）

        one/music/post/like/mix/collects 已迁移到 Rust TaskApplicationService，
        此函数仅保留 live 模式的兜底调用。
        """
        from core.models import DownloadMode

        task_id = str(uuid.uuid4())[:8]
        logger.info("[start_download] task_id={}, mode={}", task_id, mode)

        if mode == DownloadMode.LIVE:
            self._run_live_record(task_id, url)
        else:
            logger.error("[start_download] mode={} 已迁移到 Rust TaskApplicationService，不应走此路径", mode)
            return ""

        return task_id

    def _run_live_record(self, task_id: str, url: str):
        """直播录制（委托给 LiveRecordManager）"""
        self._live_mgr.start_live_record(url, self.handler, self.broadcast_task_update_sync, task_id)

    # ============================================================
    # 事件广播
    # ============================================================

    # 终态状态集合（对齐 Rust TaskStatus）
    _TERMINAL_STATUSES = {"completed", "error", "cancelled"}

    # 每个 task_id 的已发射次数（用于派生 event_type: started/progress/finished）
    _task_event_counts: dict[str, int] = {}

    @classmethod
    def _derive_event_type(cls, task_id: str, status: str) -> str:
        """派生 event_type（对齐 Rust TaskEventType: started | progress | finished）"""
        count = cls._task_event_counts.get(task_id, 0)
        cls._task_event_counts[task_id] = count + 1

        if count == 0:
            return "started"
        if status in cls._TERMINAL_STATUSES:
            # 终态 → finished，清理计数以避免内存泄漏
            cls._task_event_counts.pop(task_id, None)
            return "finished"
        return "progress"

    def broadcast_task_update_sync(self, task_id: str, task_data: dict, task_type: str = "unknown"):
        """广播任务状态更新到前端（通过 Tauri 事件系统，同步版本）

        发射格式对齐 Rust TaskEvent:
            { event_type, task_id, mode, url, ...patch_fields }

        event_type 派生规则:
            - 首次发射 → "started"
            - 中间发射 → "progress"
            - 终态 status → "finished"
        """
        try:
            import core.tauri_bridge as tb
            clean_data = {k: v for k, v in task_data.items() if not k.startswith("_")}

            # 派生 event_type（对齐 Rust TaskEventType）
            status = clean_data.get("status", "")
            event_type = self._derive_event_type(task_id, status)

            # 构建对齐 Rust TaskEvent 的 payload
            # mode 从 task_type 映射（batch 事件映射到具体类型，live 事件为 "live"）
            mode = clean_data.get("mode") or task_type
            if task_type == "batch" and "mode" not in clean_data:
                # batch 事件的 type 字段映射到 mode（如 user_post → post）
                raw_type = clean_data.get("type", "")
                if raw_type:
                    mode = raw_type

            # 注入 event_type / mode / url（对齐 TaskEvent 顶层字段）
            clean_data["event_type"] = event_type
            if "mode" not in clean_data:
                clean_data["mode"] = mode
            if "url" in task_data and "url" not in clean_data:
                clean_data["url"] = task_data["url"]

            emit_func = getattr(tb, '_emit_func', None)
            logger.info("[broadcast] 广播 task_id={}, task_type={}, event_type={}, status={}, results={}, _emit_func={}",
                       task_id, task_type, event_type, status, len(clean_data.get("results", [])),
                       "已注册" if emit_func is not None else "未注册")
            if emit_func is None:
                logger.error("[broadcast] _emit_func 为 None，无法广播！模块属性: {}", dir(tb))
                return
            logger.info("[broadcast] 调用 emit 函数...")
            tb.emit(task_id, task_type, clean_data)
            logger.info("[broadcast] emit 调用完成")
        except Exception as e:
            logger.error("[broadcast] Tauri 事件发射失败: {}", e, exc_info=True)

    async def broadcast_task_update(self, task_id: str, task_data: dict, task_type: str = "unknown"):
        """广播任务状态更新到前端（异步版本，兼容）"""
        self.broadcast_task_update_sync(task_id, task_data, task_type)


task_manager = TaskManager()
