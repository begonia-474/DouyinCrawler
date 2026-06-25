"""管理 DouyinHandler 实例和配置，支持持久化"""

import json
import asyncio
import uuid
import threading
from pathlib import Path
from backend.logger import get_logger, setup_logging
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
        self._live_tasks: dict[str, dict] = {}
        self._batch_tasks: dict[str, dict] = {}  # 批量下载任务
        self._load_config()

    def _load_config(self):
        """从 config/app.json 加载配置"""
        if CONFIG_PATH.exists():
            try:
                data = json.loads(CONFIG_PATH.read_text(encoding="utf-8"))
                # 读取 douyin 配置（默认平台）
                douyin_config = data.get("douyin", {})
                self._cookie = douyin_config.get("cookie", "")
                logger.info("[_load_config] cookie 原始长度=%d, 前60字符=%s", len(self._cookie), repr(self._cookie[:60]))
                # 检查并清理 cookie 中的换行符
                if '\n' in self._cookie or '\r' in self._cookie:
                    logger.warning("[_load_config] cookie 中包含换行符! \\n=%d, \\r=%d, 正在清理...",
                                   self._cookie.count('\n'), self._cookie.count('\r'))
                    self._cookie = " ".join(self._cookie.split())
                    logger.info("[_load_config] 清理后 cookie 长度=%d, 前60字符=%s", len(self._cookie), repr(self._cookie[:60]))
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
            # 读取现有配置
            existing_config = {}
            if CONFIG_PATH.exists():
                existing_config = json.loads(CONFIG_PATH.read_text(encoding="utf-8"))

            # 更新 douyin 配置
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

            # 确保目录存在
            CONFIG_DIR.mkdir(parents=True, exist_ok=True)

            # 保存配置
            CONFIG_PATH.write_text(
                json.dumps(existing_config, ensure_ascii=False, indent=2),
                encoding="utf-8"
            )
            logger.info("已保存配置: %s", CONFIG_PATH)
        except Exception as e:
            logger.error("保存配置失败: %s", e)

    @property
    def handler(self) -> DouyinHandler:
        if self._handler is None:
            proxies = None
            if self._proxy:
                proxies = {"http://": self._proxy, "https://": self._proxy}
            # 将相对路径解析为基于项目根目录的绝对路径
            download_path = Path(self._download_path)
            if not download_path.is_absolute():
                download_path = PROJECT_ROOT / download_path
            logger.info("[handler] 创建 DouyinHandler, cookie 长度=%d, 前40字符=%s", len(self._cookie), repr(self._cookie[:40]))
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
            logger.info("已创建 DouyinHandler (cookie=%s..., path=%s)", self._cookie[:20], self._download_path)
        return self._handler

    def update_config(self, cookie: str = None, download_path: str = None,
                      naming: str = None, encryption: str = None, proxy: str = None,
                      app_name: str = None, folderize: bool = None,
                      music: bool = None, cover: bool = None, desc: bool = None,
                      interval: str = None, page_counts: int = None,
                      max_counts: int = None, timeout: int = None,
                      max_connections: int = None, max_retries: int = None,
                      max_tasks: int = None):
        if cookie is not None:
            # 清理换行符，合并为空格分隔的单行格式
            logger.info("[update_config] 收到 cookie, 原始长度=%d, 前60字符=%s", len(cookie), repr(cookie[:60]))
            if '\n' in cookie or '\r' in cookie:
                logger.warning("[update_config] cookie 中包含换行符! \\n=%d, \\r=%d",
                               cookie.count('\n'), cookie.count('\r'))
            self._cookie = " ".join(cookie.split())
            logger.info("[update_config] 清理后 cookie 长度=%d, 前60字符=%s", len(self._cookie), repr(self._cookie[:60]))
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
    # 直播录制任务管理
    # ============================================================

    def start_live_record(self, url: str) -> str:
        """启动直播录制，返回 task_id（同步版本，使用线程后台执行）"""
        task_id = str(uuid.uuid4())[:8]
        stop_event = threading.Event()
        self._live_tasks[task_id] = {
            "task_id": task_id,
            "url": url,
            "status": "starting",
            "file": "",
            "error": "",
            "_stop_event": stop_event,
        }

        def _run():
            loop = asyncio.new_event_loop()
            asyncio.set_event_loop(loop)
            try:
                async def _do():
                    self._live_tasks[task_id]["status"] = "recording"
                    self.broadcast_task_update_sync(task_id, self._live_tasks[task_id], "live")

                    # 将 threading.Event 包装为 asyncio.Event
                    async_stop = asyncio.Event()
                    # 保存 loop 引用，供 stop_live_record 线程安全地唤醒
                    self._live_tasks[task_id]["_async_stop"] = async_stop
                    self._live_tasks[task_id]["_loop"] = loop

                    result = await self.handler.handle_live_record(url, task_id, stop_event=async_stop)
                    if result.get("success"):
                        self._live_tasks[task_id].update({
                            "status": "completed",
                            "file": result.get("file", ""),
                            "room_id": result.get("room_id", ""),
                            "web_rid": result.get("web_rid", ""),
                            "title": result.get("title", ""),
                            "nickname": result.get("nickname", ""),
                            "file_size": result.get("file_size", 0),
                            "duration_sec": result.get("duration_sec", 0),
                            "started_at": result.get("started_at", 0),
                            "ended_at": result.get("ended_at", 0),
                            "cover_url": result.get("cover_url", ""),
                        })
                    else:
                        self._live_tasks[task_id]["status"] = "error"
                        self._live_tasks[task_id]["error"] = result.get("error", "未知错误")

                loop.run_until_complete(_do())
            except Exception as e:
                self._live_tasks[task_id]["status"] = "error"
                self._live_tasks[task_id]["error"] = str(e)
                logger.error("[live_record] 异常: %s", e, exc_info=True)
            finally:
                # 广播最终状态
                self.broadcast_task_update_sync(task_id, self._live_tasks[task_id], "live")
                loop.close()

        thread = threading.Thread(target=_run, daemon=True)
        thread.start()
        logger.info("[live_record] 直播录制已启动, task_id=%s", task_id)
        return task_id

    def stop_live_record(self, task_id: str) -> bool:
        """停止直播录制"""
        if task_id not in self._live_tasks:
            return False
        stop_event = self._live_tasks[task_id].get("_stop_event")
        if stop_event:
            stop_event.set()
        # 线程安全地唤醒 asyncio.Event（无轮询）
        async_stop = self._live_tasks[task_id].get("_async_stop")
        loop = self._live_tasks[task_id].get("_loop")
        if async_stop and loop:
            loop.call_soon_threadsafe(async_stop.set)
        self._live_tasks[task_id]["status"] = "stopping"
        return True

    def get_live_status(self) -> dict[str, dict]:
        """获取所有录制任务状态（排除内部字段），以 task_id 为 key"""
        return {
            t["task_id"]: {k: v for k, v in t.items() if not k.startswith("_")}
            for t in self._live_tasks.values()
        }

    # ============================================================
    # 批量下载任务管理
    # ============================================================

    def start_batch_download(self, url: str, download_type: str) -> str:
        """启动批量下载任务，返回 task_id（同步版本，使用线程后台执行）"""
        task_id = str(uuid.uuid4())[:8]
        self._batch_tasks[task_id] = {
            "task_id": task_id,
            "type": download_type,
            "url": url,
            "status": "starting",
            "total": 0,
            "completed": 0,
            "failed": 0,
            "current_item": "",
            "error": "",
            "results": [],  # 存储下载结果，供前端保存到数据库
        }

        def _run():
            loop = asyncio.new_event_loop()
            asyncio.set_event_loop(loop)
            logger.info("[batch_download] 线程启动 task_id=%s", task_id)
            try:
                async def _do():
                    self._batch_tasks[task_id]["status"] = "running"
                    logger.info("[batch_download] 状态改为 running, 开始广播 task_id=%s", task_id)
                    self.broadcast_task_update_sync(task_id, self._batch_tasks[task_id], "batch")

                    # 根据类型调用对应的下载方法
                    handler = self.handler
                    logger.info("[batch_download] 开始下载 task_id=%s, type=%s, url=%s", task_id, download_type, url)
                    if download_type == "user_post":
                        result = await handler.handle_user_post(url)
                    elif download_type == "user_like":
                        result = await handler.handle_user_like(url)
                    elif download_type == "mix":
                        result = await handler.handle_user_mix(url)
                    elif download_type == "collects":
                        result = await handler.handle_collects_video(url)
                    else:
                        result = {"success": False, "error": f"未知的下载类型: {download_type}"}

                    logger.info("[batch_download] handler 返回 task_id=%s, success=%s, count=%s, results=%d",
                               task_id, result.get("success"), result.get("count"), len(result.get("results", [])))
                    if result.get("success"):
                        count = result.get("count", 0)
                        results = result.get("results", [])

                        # 直接写入数据库（不经过前端）
                        try:
                            from core.db import save_batch_results
                            db_result = save_batch_results(results, download_type)
                            logger.info("[batch_download] 数据库写入完成: %s", db_result)
                        except Exception as e:
                            logger.error("[batch_download] 数据库写入失败: %s", e, exc_info=True)

                        self._batch_tasks[task_id].update({
                            "status": "completed",
                            "total": count,
                            "completed": count,
                        })
                        logger.info("[batch_download] 任务状态已更新为 completed task_id=%s", task_id)
                    else:
                        self._batch_tasks[task_id]["status"] = "error"
                        self._batch_tasks[task_id]["error"] = result.get("error", "未知错误")
                        logger.error("[batch_download] 下载失败 task_id=%s, error=%s", task_id, result.get("error"))

                loop.run_until_complete(_do())
            except Exception as e:
                self._batch_tasks[task_id]["status"] = "error"
                self._batch_tasks[task_id]["error"] = str(e)
                logger.error("[batch_download] 线程异常: %s", e, exc_info=True)
            finally:
                # 广播最终状态
                logger.info("[batch_download] finally: 准备广播最终状态 task_id=%s, status=%s",
                           task_id, self._batch_tasks[task_id].get("status"))
                self.broadcast_task_update_sync(task_id, self._batch_tasks[task_id], "batch")
                loop.close()
                logger.info("[batch_download] 线程结束 task_id=%s", task_id)

        thread = threading.Thread(target=_run, daemon=True)
        thread.start()
        logger.info("[batch_download] 批量下载已启动, task_id=%s, type=%s", task_id, download_type)
        return task_id

    def get_batch_status(self) -> dict[str, dict]:
        """获取所有批量下载任务状态，以 task_id 为 key"""
        return {t["task_id"]: t for t in self._batch_tasks.values()}

    def broadcast_task_update_sync(self, task_id: str, task_data: dict, task_type: str = "unknown"):
        """广播任务状态更新到前端（通过 Tauri 事件系统，同步版本）"""
        try:
            import core.tauri_bridge as tb
            import sys
            # 过滤内部字段（以 _ 开头的）
            clean_data = {k: v for k, v in task_data.items() if not k.startswith("_")}
            # 直接从模块获取最新值（避免导入时的值拷贝）
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

    # 保留异步版本以兼容
    async def broadcast_task_update(self, task_id: str, task_data: dict, task_type: str = "unknown"):
        """广播任务状态更新到前端（通过 Tauri 事件系统）"""
        self.broadcast_task_update_sync(task_id, task_data, task_type)


task_manager = TaskManager()
