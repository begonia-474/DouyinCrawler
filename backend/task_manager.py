"""管理 DouyinHandler 实例和配置，支持持久化

职责拆分：
- TaskManager（本文件）：配置管理 + Handler 生命周期 + 事件广播（门面）
- LiveRecordManager（live_manager.py）：直播录制任务管理
- BatchManager（batch_manager.py）：批量下载任务管理
"""

import json
import uuid
import asyncio
from pathlib import Path
from backend.logger import get_logger, setup_logging
from backend.live_manager import LiveRecordManager
from backend.batch_manager import BatchManager
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
        self._batch_mgr = BatchManager()
        self._load_config()

    # ============================================================
    # 配置管理
    # ============================================================

    def _load_config(self):
        """从 config/app.json 加载配置"""
        if CONFIG_PATH.exists():
            try:
                data = json.loads(CONFIG_PATH.read_text(encoding="utf-8"))
                douyin_config = data.get("douyin", {})
                self._cookie = douyin_config.get("cookie", "")
                logger.info("[_load_config] 已加载 cookie (len=%d)", len(self._cookie))
                if '\n' in self._cookie or '\r' in self._cookie:
                    logger.warning("[_load_config] cookie 中包含换行符! \\n=%d, \\r=%d, 正在清理...",
                                   self._cookie.count('\n'), self._cookie.count('\r'))
                    self._cookie = " ".join(self._cookie.split())
                    logger.info("[_load_config] cookie 已清理换行符 (len=%d)", len(self._cookie))
                self._download_path = douyin_config.get("download_path", "Download")
                self._naming = douyin_config.get("naming", "{create}_{desc}")
                self._encryption = douyin_config.get("encryption", "ab")
                self._proxy = douyin_config.get("proxy", "")
                self._app_name = douyin_config.get("app_name", "douyin")
                self._folderize = douyin_config.get("folderize", False)
                self._music = douyin_config.get("music", False)
                self._cover = douyin_config.get("cover", False)
                self._desc = douyin_config.get("desc", False)
                self._interval = douyin_config.get("interval", None)
                self._page_counts = douyin_config.get("page_counts", 20)
                self._max_counts = douyin_config.get("max_counts", 0)
                self._timeout = douyin_config.get("timeout", 5)
                self._max_connections = douyin_config.get("max_connections", 5)
                self._max_retries = douyin_config.get("max_retries", 5)
                self._max_tasks = douyin_config.get("max_tasks", 10)
                logger.info("已加载配置: %s", CONFIG_PATH)
            except Exception as e:
                logger.error("加载配置失败: %s", e)
        else:
            logger.warning("配置文件不存在: %s", CONFIG_PATH)

    def _save_config(self):
        """保存配置到 config/app.json"""
        try:
            existing_config = {}
            if CONFIG_PATH.exists():
                existing_config = json.loads(CONFIG_PATH.read_text(encoding="utf-8"))

            existing_config["douyin"] = {
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

            CONFIG_DIR.mkdir(parents=True, exist_ok=True)
            CONFIG_PATH.write_text(
                json.dumps(existing_config, ensure_ascii=False, indent=2),
                encoding="utf-8"
            )
            logger.info("已保存配置: %s", CONFIG_PATH)
        except Exception as e:
            logger.error("保存配置失败: %s", e)

    def update_config(self, cookie: str = None, download_path: str = None,
                      naming: str = None, encryption: str = None, proxy: str = None,
                      app_name: str = None, folderize: bool = None,
                      music: bool = None, cover: bool = None, desc: bool = None,
                      interval: str = None, page_counts: int = None,
                      max_counts: int = None, timeout: int = None,
                      max_connections: int = None, max_retries: int = None,
                      max_tasks: int = None, save: bool = True):
        if cookie is not None:
            logger.info("[update_config] 收到 cookie (len=%d)", len(cookie))
            if '\n' in cookie or '\r' in cookie:
                logger.warning("[update_config] cookie 中包含换行符! \\n=%d, \\r=%d",
                               cookie.count('\n'), cookie.count('\r'))
            self._cookie = " ".join(cookie.split())
            logger.info("[update_config] cookie 已清理 (len=%d)", len(self._cookie))
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
        if save:
            self._save_config()

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
            logger.info("[handler] 创建 DouyinHandler (has_cookie=%s)", bool(self._cookie))
            logger.info("[handler] download_path=%s (原始=%s)", download_path, self._download_path)
            logger.info("[handler] encryption=%s, max_retries=%d, timeout=%d, max_connections=%d",
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
            logger.info("已创建 DouyinHandler (path=%s)", self._download_path)
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
    # 批量下载（委托给 BatchManager）
    # ============================================================

    def start_batch_download(self, url: str, download_type: str) -> str:
        """启动批量下载任务，返回 task_id"""
        return self._batch_mgr.start_batch_download(url, download_type, self.handler, self.broadcast_task_update_sync)

    def get_batch_status(self) -> dict[str, dict]:
        """获取所有批量下载任务状态"""
        return self._batch_mgr.get_batch_status()

    # ============================================================
    # 统一下载入口（mode 调度）
    # ============================================================

    def start_download(self, mode: str, url: str) -> str:
        """统一下载入口，根据 mode 分发到对应 handler，返回 task_id"""
        from core.constants import DownloadMode
        from core import db

        task_id = str(uuid.uuid4())[:8]

        # 在 DB 创建任务记录
        db.create_task(task_id, mode, url)
        logger.info("[start_download] 任务已创建 task_id=%s, mode=%s", task_id, mode)

        if mode == DownloadMode.ONE:
            self._run_single_download(task_id, url)
        elif mode in (DownloadMode.POST, DownloadMode.LIKE, DownloadMode.MIX, DownloadMode.COLLECTS):
            self._run_batch_download(task_id, mode, url)
        elif mode == DownloadMode.LIVE:
            self._run_live_record(task_id, url)
        elif mode == DownloadMode.MUSIC:
            self._run_music_download(task_id, url)
        else:
            db.update_task_status(task_id, "error", f"未知的下载模式: {mode}")
            logger.error("[start_download] 未知的下载模式: %s", mode)

        return task_id

    def _run_single_download(self, task_id: str, url: str):
        """单视频下载（后台线程）"""
        import threading

        def _run():
            try:
                result = self.handler.handle_one_video(url)
                if result.get("success"):
                    from core import db
                    detail = result.get("detail", {})
                    file_path = result.get("path") or (result.get("paths", [None])[0] if result.get("paths") else None)
                    aweme_id = detail.get("aweme_id", "")
                    db.create_task_item(task_id, aweme_id=aweme_id, title=detail.get("desc"),
                                        author_nickname=detail.get("author_nickname"))
                    file_size = 0
                    if file_path:
                        from pathlib import Path
                        try:
                            file_size = Path(file_path).stat().st_size
                        except (OSError, ValueError):
                            pass
                    db.update_task_item_status(task_id, aweme_id, "completed", file_path, file_size)
                    db.update_task_status(task_id, "completed")
                    self.broadcast_task_update_sync(task_id, {
                        "task_id": task_id, "mode": "one", "status": "completed",
                        "total": 1, "completed": 1, "skipped": 0, "failed": 0,
                    }, "batch")
                else:
                    from core import db
                    db.update_task_status(task_id, "error", result.get("error", "下载失败"))
                    self.broadcast_task_update_sync(task_id, {
                        "task_id": task_id, "mode": "one", "status": "error",
                        "error": result.get("error"),
                    }, "batch")
            except Exception as e:
                from core import db
                db.update_task_status(task_id, "error", str(e))
                logger.error("[_run_single_download] 异常: %s", e, exc_info=True)

        thread = threading.Thread(target=_run, daemon=True)
        thread.start()

    def _run_batch_download(self, task_id: str, mode: str, url: str):
        """批量下载（后台线程，委托给 BatchManager）"""
        # 映射 mode 到旧的 download_type
        mode_to_type = {"post": "user_post", "like": "user_like", "mix": "mix", "collects": "collects"}
        download_type = mode_to_type.get(mode, mode)
        self._batch_mgr.start_batch_download(url, download_type, self.handler,
                                              self.broadcast_task_update_sync, task_id)

    def _run_live_record(self, task_id: str, url: str):
        """直播录制（委托给 LiveRecordManager）"""
        self._live_mgr.start_live_record(url, self.handler, self.broadcast_task_update_sync, task_id)

    def _run_music_download(self, task_id: str, url: str):
        """音乐批量下载（后台线程）"""
        import threading

        def _run():
            from core import db
            try:
                # url 在这里实际是 sec_user_id 或 profile URL，需要先获取音乐列表
                result = asyncio.run(self.handler.handle_user_music_collection())
                if not result.get("success"):
                    db.update_task_status(task_id, "error", result.get("error", "获取音乐列表失败"))
                    return

                music_list = result.get("music_list", [])
                for music in music_list:
                    db.create_task_item(task_id, aweme_id=music.get("music_id"),
                                        title=music.get("title"), author_nickname=music.get("author"))

                db.update_task_status(task_id, "running")
                completed = 0
                for music in music_list:
                    try:
                        dl_result = asyncio.run(self.handler.handle_download_music(
                            music.get("play_url", ""), music.get("title", ""), music.get("author", "")))
                        if dl_result.get("success"):
                            file_path = dl_result.get("path", "")
                            file_size = 0
                            if file_path:
                                from pathlib import Path
                                try:
                                    file_size = Path(file_path).stat().st_size
                                except (OSError, ValueError):
                                    pass
                            db.update_task_item_status(task_id, music.get("music_id", ""), "completed",
                                                       file_path, file_size)
                            completed += 1
                        else:
                            db.update_task_item_status(task_id, music.get("music_id", ""), "failed",
                                                       error_msg=dl_result.get("error", "下载失败"))
                    except Exception as e:
                        db.update_task_item_status(task_id, music.get("music_id", ""), "failed",
                                                   error_msg=str(e))
                        logger.error("[_run_music_download] 单曲下载失败: %s", e)

                    self.broadcast_task_update_sync(task_id, {
                        "task_id": task_id, "mode": "music", "status": "running",
                        "total": len(music_list), "completed": completed,
                    }, "batch")

                db.update_task_status(task_id, "completed")
                self.broadcast_task_update_sync(task_id, {
                    "task_id": task_id, "mode": "music", "status": "completed",
                    "total": len(music_list), "completed": completed,
                }, "batch")

            except Exception as e:
                db.update_task_status(task_id, "error", str(e))
                logger.error("[_run_music_download] 异常: %s", e, exc_info=True)

        thread = threading.Thread(target=_run, daemon=True)
        thread.start()

    # ============================================================
    # 事件广播
    # ============================================================

    def broadcast_task_update_sync(self, task_id: str, task_data: dict, task_type: str = "unknown"):
        """广播任务状态更新到前端（通过 Tauri 事件系统，同步版本）"""
        try:
            import core.tauri_bridge as tb
            clean_data = {k: v for k, v in task_data.items() if not k.startswith("_")}
            emit_func = getattr(tb, '_emit_func', None)
            logger.info("[broadcast] 广播 task_id=%s, task_type=%s, status=%s, results=%d, _emit_func=%s",
                       task_id, task_type, clean_data.get("status"), len(clean_data.get("results", [])),
                       "已注册" if emit_func is not None else "未注册")
            if emit_func is None:
                logger.error("[broadcast] _emit_func 为 None，无法广播！模块属性: %s", dir(tb))
                return
            logger.info("[broadcast] 调用 emit 函数...")
            tb.emit(task_id, task_type, clean_data)
            logger.info("[broadcast] emit 调用完成")
        except Exception as e:
            logger.error("[broadcast] Tauri 事件发射失败: %s", e, exc_info=True)

    async def broadcast_task_update(self, task_id: str, task_data: dict, task_type: str = "unknown"):
        """广播任务状态更新到前端（异步版本，兼容）"""
        self.broadcast_task_update_sync(task_id, task_data, task_type)


task_manager = TaskManager()
