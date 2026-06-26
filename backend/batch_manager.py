"""批量下载任务管理"""

import asyncio
import uuid
import threading
from typing import Callable
from backend.logger import get_logger

logger = get_logger(__name__)


class BatchManager:
    """管理批量下载任务的启动和状态查询"""

    def __init__(self):
        self._batch_tasks: dict[str, dict] = {}

    def start_batch_download(
        self,
        url: str,
        download_type: str,
        handler,
        broadcast_fn: Callable[[str, dict, str], None],
    ) -> str:
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
            "results": [],
        }

        def _run():
            loop = asyncio.new_event_loop()
            asyncio.set_event_loop(loop)
            logger.info("[batch_download] 线程启动 task_id=%s", task_id)
            try:
                async def _do():
                    self._batch_tasks[task_id]["status"] = "running"
                    logger.info("[batch_download] 状态改为 running, 开始广播 task_id=%s", task_id)
                    broadcast_fn(task_id, self._batch_tasks[task_id], "batch")

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
                logger.info("[batch_download] finally: 准备广播最终状态 task_id=%s, status=%s",
                           task_id, self._batch_tasks[task_id].get("status"))
                broadcast_fn(task_id, self._batch_tasks[task_id], "batch")
                loop.close()
                logger.info("[batch_download] 线程结束 task_id=%s", task_id)

        thread = threading.Thread(target=_run, daemon=True)
        thread.start()
        logger.info("[batch_download] 批量下载已启动, task_id=%s, type=%s", task_id, download_type)
        return task_id

    def get_batch_status(self) -> dict[str, dict]:
        """获取所有批量下载任务状态，以 task_id 为 key"""
        return {t["task_id"]: t for t in self._batch_tasks.values()}
