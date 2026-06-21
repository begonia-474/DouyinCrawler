"""视频下载器 — 断点续传、进度回调"""

import httpx
import asyncio
import aiofiles
from pathlib import Path
from typing import Callable

# 文件名工具统一在 utils.py 维护；此处重新导出以保持向后兼容
from core.utils import sanitize_filename, format_filename  # noqa: F401


class DownloadTask:
    """单个下载任务"""

    def __init__(
        self,
        url: str,
        save_path: Path,
        filename: str,
        suffix: str = ".mp4",
        headers: dict | None = None,
    ):
        self.url = url
        self.save_path = save_path
        self.filename = filename
        self.suffix = suffix
        self.headers = headers or {}
        self.full_path = save_path / f"{filename}{suffix}"
        self.temp_path = save_path / f"{filename}{suffix}.tmp"
        self.total_size = 0
        self.downloaded = 0
        self.status = "pending"  # pending / downloading / completed / error
        self.error_msg = ""


class Downloader:
    """异步下载器，支持断点续传和进度回调"""

    CHUNK_SIZE = 8192

    def __init__(
        self,
        cookie: str,
        max_connections: int = 5,
        timeout: int = 30,
        progress_callback: Callable[[str, int, int], None] | None = None,
    ):
        """
        Args:
            cookie: 抖音 Cookie
            max_connections: 最大并发数
            timeout: 超时秒数
            progress_callback: 进度回调 fn(task_id, downloaded_bytes, total_bytes)
        """
        self.cookie = cookie
        self.progress_callback = progress_callback
        self._client = httpx.AsyncClient(
            timeout=timeout,
            follow_redirects=True,
            limits=httpx.Limits(max_connections=max_connections),
        )
        self._tasks: dict[str, DownloadTask] = {}

    async def close(self):
        await self._client.aclose()

    async def __aenter__(self):
        return self

    async def __aexit__(self, *args):
        await self.close()

    def _get_headers(self, task: DownloadTask) -> dict:
        headers = {
            "User-Agent": (
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
                "AppleWebKit/537.36 (KHTML, like Gecko) "
                "Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0"
            ),
            "Referer": "https://www.douyin.com/",
            "Cookie": self.cookie,
        }
        headers.update(task.headers)
        return headers

    async def download_video(
        self,
        video_url: str,
        save_dir: str | Path,
        filename: str,
        task_id: str | None = None,
    ) -> Path:
        """
        下载视频文件

        Args:
            video_url: 视频 URL
            save_dir: 保存目录
            filename: 文件名（不含扩展名）
            task_id: 任务 ID（用于进度回调）

        Returns:
            保存的文件路径
        """
        save_path = Path(save_dir)
        save_path.mkdir(parents=True, exist_ok=True)

        task = DownloadTask(url=video_url, save_path=save_path, filename=filename)
        tid = task_id or filename
        self._tasks[tid] = task

        headers = self._get_headers(task)

        # 检查已下载的部分
        existing_size = 0
        if task.temp_path.exists():
            existing_size = task.temp_path.stat().st_size
            headers["Range"] = f"bytes={existing_size}-"

        try:
            task.status = "downloading"
            async with self._client.stream("GET", video_url, headers=headers) as resp:
                resp.raise_for_status()

                # 获取总大小
                content_length = int(resp.headers.get("content-length", 0))
                if resp.status_code == 206:
                    task.total_size = existing_size + content_length
                    task.downloaded = existing_size
                else:
                    task.total_size = content_length
                    existing_size = 0

                # 写入文件
                mode = "ab" if existing_size > 0 and resp.status_code == 206 else "wb"
                async with aiofiles.open(task.temp_path, mode) as f:
                    async for chunk in resp.aiter_bytes(self.CHUNK_SIZE):
                        await f.write(chunk)
                        task.downloaded += len(chunk)
                        if self.progress_callback:
                            self.progress_callback(tid, task.downloaded, task.total_size)

            # 下载完成，重命名
            if task.full_path.exists():
                task.full_path.unlink()
            task.temp_path.rename(task.full_path)
            task.status = "completed"
            return task.full_path

        except Exception as e:
            task.status = "error"
            task.error_msg = str(e)
            raise

    async def download_image(self, image_url: str, save_dir: str | Path, filename: str) -> Path:
        """下载图片"""
        save_path = Path(save_dir)
        save_path.mkdir(parents=True, exist_ok=True)
        full_path = save_path / f"{filename}.webp"

        headers = {
            "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
            "Referer": "https://www.douyin.com/",
            "Cookie": self.cookie,
        }

        async with self._client.stream("GET", image_url, headers=headers) as resp:
            resp.raise_for_status()
            async with aiofiles.open(full_path, "wb") as f:
                async for chunk in resp.aiter_bytes(self.CHUNK_SIZE):
                    await f.write(chunk)

        return full_path

    async def batch_download(
        self,
        tasks: list[dict],
        max_concurrent: int = 3,
    ) -> list[Path]:
        """
        批量下载

        Args:
            tasks: [{"url": ..., "dir": ..., "filename": ...}, ...]
            max_concurrent: 最大并发数

        Returns:
            下载完成的文件路径列表
        """
        semaphore = asyncio.Semaphore(max_concurrent)
        results = []

        async def _download_one(t: dict):
            async with semaphore:
                path = await self.download_video(
                    video_url=t["url"],
                    save_dir=t["dir"],
                    filename=t["filename"],
                    task_id=t.get("task_id", t["filename"]),
                )
                results.append(path)

        await asyncio.gather(*[_download_one(t) for t in tasks])
        return results
