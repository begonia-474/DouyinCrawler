"""管理 DouyinHandler 实例和配置，支持持久化"""

import json
import asyncio
import uuid
from pathlib import Path
from backend.logger import get_logger
from core.handler import DouyinHandler

logger = get_logger(__name__)

CONFIG_PATH = Path(__file__).parent / "config.json"


class TaskManager:
    def __init__(self):
        self._cookie: str = ""
        self._download_path: str = "Download"
        self._naming: str = "{create}_{desc}"
        self._encryption: str = "ab"
        self._proxy: str = ""
        self._handler: DouyinHandler | None = None
        self._live_tasks: dict[str, dict] = {}
        self._load_config()

    def _load_config(self):
        if CONFIG_PATH.exists():
            try:
                data = json.loads(CONFIG_PATH.read_text(encoding="utf-8"))
                self._cookie = data.get("cookie", "")
                self._download_path = data.get("download_path", "Download")
                self._naming = data.get("naming", "{create}_{desc}")
                self._encryption = data.get("encryption", "ab")
                self._proxy = data.get("proxy", "")
                logger.info("已加载配置: %s", CONFIG_PATH)
            except Exception as e:
                logger.error("加载配置失败: %s", e)

    def _save_config(self):
        try:
            CONFIG_PATH.write_text(json.dumps({
                "cookie": self._cookie,
                "download_path": self._download_path,
                "naming": self._naming,
                "encryption": self._encryption,
                "proxy": self._proxy,
            }, ensure_ascii=False, indent=2), encoding="utf-8")
            logger.info("已保存配置: %s", CONFIG_PATH)
        except Exception as e:
            logger.error("保存配置失败: %s", e)

    @property
    def handler(self) -> DouyinHandler:
        if self._handler is None:
            proxies = None
            if self._proxy:
                proxies = {"http://": self._proxy, "https://": self._proxy}
            self._handler = DouyinHandler(
                cookie=self._cookie,
                download_path=self._download_path,
                naming=self._naming,
                encryption=self._encryption,
                proxies=proxies,
            )
            logger.info("已创建 DouyinHandler (cookie=%s..., path=%s)", self._cookie[:20], self._download_path)
        return self._handler

    def update_config(self, cookie: str = None, download_path: str = None,
                      naming: str = None, encryption: str = None, proxy: str = None):
        if cookie is not None:
            # 清理换行符，合并为空格分隔的单行格式
            self._cookie = " ".join(cookie.split())
        if download_path is not None:
            self._download_path = download_path
        if naming is not None:
            self._naming = naming
        if encryption is not None:
            self._encryption = encryption
        if proxy is not None:
            self._proxy = proxy
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
        }

    # ============================================================
    # 直播录制任务管理
    # ============================================================

    async def start_live_record(self, url: str) -> str:
        """启动直播录制，返回 task_id"""
        task_id = str(uuid.uuid4())[:8]
        stop_event = asyncio.Event()
        self._live_tasks[task_id] = {
            "task_id": task_id,
            "url": url,
            "status": "starting",
            "file": "",
            "error": "",
            "_stop_event": stop_event,
        }

        async def _run():
            try:
                self._live_tasks[task_id]["status"] = "recording"
                result = await self.handler.handle_live_record(url, task_id, stop_event=stop_event)
                if result.get("success"):
                    self._live_tasks[task_id].update({
                        "status": "completed",
                        "file": result.get("file", ""),
                        "room_id": result.get("room_id", ""),
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
            except Exception as e:
                self._live_tasks[task_id]["status"] = "error"
                self._live_tasks[task_id]["error"] = str(e)

        asyncio.create_task(_run())
        return task_id

    async def stop_live_record(self, task_id: str) -> bool:
        """停止直播录制"""
        if task_id not in self._live_tasks:
            return False
        stop_event = self._live_tasks[task_id].get("_stop_event")
        if stop_event:
            stop_event.set()
        self._live_tasks[task_id]["status"] = "stopping"
        return True

    def get_live_status(self) -> dict[str, dict]:
        """获取所有录制任务状态（排除内部字段），以 task_id 为 key"""
        return {
            t["task_id"]: {k: v for k, v in t.items() if not k.startswith("_")}
            for t in self._live_tasks.values()
        }


task_manager = TaskManager()
