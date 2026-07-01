"""直播录制任务管理"""

import asyncio
import uuid
import threading
from typing import Callable
from core.logger import get_logger

logger = get_logger(__name__)


class LiveRecordManager:
    """管理直播录制任务的启动、停止和状态查询"""

    def __init__(self):
        self._live_tasks: dict[str, dict] = {}

    def start_live_record(
        self,
        url: str,
        handler,
        broadcast_fn: Callable[[str, dict, str], None],
        task_id: str = None,
    ) -> str:
        """启动直播录制，返回 task_id（同步版本，使用线程后台执行）"""
        if task_id is None:
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
                    broadcast_fn(task_id, self._live_tasks[task_id], "live")

                    # 将 threading.Event 包装为 asyncio.Event
                    async_stop = asyncio.Event()
                    self._live_tasks[task_id]["_async_stop"] = async_stop
                    self._live_tasks[task_id]["_loop"] = loop

                    result = await handler.handle_live_record(url, task_id, stop_event=async_stop)
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
                logger.error("[live_record] 异常: {}", e, exc_info=True)
            finally:
                broadcast_fn(task_id, self._live_tasks[task_id], "live")
                loop.close()
                # 释放内存：录制记录已通过 Rust emit 持久化到 SQLite，前端从 DB 读历史
                self._live_tasks.pop(task_id, None)

        thread = threading.Thread(target=_run, daemon=True)
        thread.start()
        logger.info("[live_record] 直播录制已启动, task_id={}", task_id)
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
