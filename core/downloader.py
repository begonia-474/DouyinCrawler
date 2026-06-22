"""视频下载器 — 断点续传、进度回调、M3U8 直播流录制"""

import httpx
import asyncio
import aiofiles
from pathlib import Path
from typing import Callable

# 文件名工具统一在 utils.py 维护；此处重新导出以保持向后兼容
from core.utils import sanitize_filename, format_filename  # noqa: F401
from core.utils import get_segments_from_m3u8, get_content_length, get_chunk_size


# 已下载 TS 分片记录上限，超过后清空以释放内存（允许少量重复下载）
MAX_SEGMENT_COUNT = 1000


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
        self._stop_event = asyncio.Event()

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

    async def download_live_image(self, video_url: str, save_dir: str | Path, filename: str) -> Path:
        """下载动图/实况（保存为 mp4）"""
        save_path = Path(save_dir)
        save_path.mkdir(parents=True, exist_ok=True)
        full_path = save_path / f"{filename}.mp4"

        headers = {
            "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
            "Referer": "https://www.douyin.com/",
            "Cookie": self.cookie,
        }

        async with self._client.stream("GET", video_url, headers=headers) as resp:
            resp.raise_for_status()
            async with aiofiles.open(full_path, "wb") as f:
                async for chunk in resp.aiter_bytes(self.CHUNK_SIZE):
                    await f.write(chunk)

        return full_path

    async def download_music(
        self,
        music_url: str,
        save_dir: str | Path,
        filename: str,
        task_id: str | None = None,
    ) -> Path:
        """下载音乐文件"""
        save_path = Path(save_dir)
        save_path.mkdir(parents=True, exist_ok=True)

        task = DownloadTask(url=music_url, save_path=save_path, filename=filename, suffix=".mp3")
        tid = task_id or filename
        self._tasks[tid] = task

        headers = self._get_headers(task)

        try:
            task.status = "downloading"
            async with self._client.stream("GET", music_url, headers=headers) as resp:
                resp.raise_for_status()
                task.total_size = int(resp.headers.get("content-length", 0))

                async with aiofiles.open(task.full_path, "wb") as f:
                    async for chunk in resp.aiter_bytes(self.CHUNK_SIZE):
                        await f.write(chunk)
                        task.downloaded += len(chunk)
                        if self.progress_callback:
                            self.progress_callback(tid, task.downloaded, task.total_size)

            task.status = "completed"
            return task.full_path

        except Exception as e:
            task.status = "error"
            task.error_msg = str(e)
            raise

    async def batch_download(
        self,
        tasks: list[dict],
        max_concurrent: int = 3,
    ) -> list[dict]:
        """
        批量下载

        Args:
            tasks: [{"url": ..., "dir": ..., "filename": ..., "task_id": ...}, ...]
            max_concurrent: 最大并发数

        Returns:
            [{"task_id": ..., "path": ...}, ...]
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
                results.append({
                    "task_id": t.get("task_id", t["filename"]),
                    "path": path,
                })

        await asyncio.gather(*[_download_one(t) for t in tasks])
        return results

    # ============================================================
    # M3U8 直播流录制
    # ============================================================

    def stop_stream(self):
        """停止当前录制流"""
        self._stop_event.set()

    def reset_stop(self):
        """重置停止信号（开始新录制前调用）"""
        self._stop_event.clear()

    async def download_m3u8_stream(
        self,
        task_id: str,
        url: str,
        full_path: Path,
    ) -> Path:
        """
        录制 M3U8 直播流，TS 分片追加写入 .flv 文件

        Args:
            task_id: 任务 ID
            url: m3u8 拉流地址
            full_path: 保存文件路径

        Returns:
            保存的文件路径
        """
        full_path = Path(full_path)
        full_path.parent.mkdir(parents=True, exist_ok=True)

        headers = {
            "User-Agent": self._client.headers.get("User-Agent", "Mozilla/5.0"),
            "Referer": "https://www.douyin.com/",
            "Cookie": self.cookie,
        }

        total_downloaded = 0
        default_chunks = 409600
        downloaded_segments: set = set()

        task = DownloadTask(url=url, save_path=full_path.parent, filename=full_path.stem, suffix="")
        task.full_path = full_path
        task.status = "recording"
        self._tasks[task_id] = task

        while not self._stop_event.is_set():
            try:
                segments = await get_segments_from_m3u8(url)
                if not segments:
                    task.status = "completed"
                    return full_path

                async with aiofiles.open(full_path, "ab") as file:
                    for segment in segments:
                        if self._stop_event.is_set():
                            break

                        if segment.absolute_uri in downloaded_segments:
                            continue

                        ts_url = segment.absolute_uri
                        ts_content_length = await get_content_length(ts_url, headers)
                        if ts_content_length == 0:
                            ts_content_length = default_chunks

                        try:
                            req = self._client.build_request("GET", ts_url, headers=headers)
                            resp = await self._client.send(req, stream=True)

                            async for chunk in resp.aiter_bytes(get_chunk_size(ts_content_length)):
                                if self._stop_event.is_set():
                                    break
                                await file.write(chunk)
                                total_downloaded += len(chunk)
                                task.downloaded = total_downloaded
                                if self.progress_callback:
                                    self.progress_callback(task_id, total_downloaded, 0)

                            downloaded_segments.add(segment.absolute_uri)

                        except httpx.ReadTimeout:
                            continue
                        except httpx.RemoteProtocolError:
                            continue
                        finally:
                            try:
                                await resp.aclose()
                            except Exception:
                                pass

                    if len(downloaded_segments) > MAX_SEGMENT_COUNT:
                        downloaded_segments = set()

                # 等待最后一个分片的时长，避免过快请求
                if segments:
                    await asyncio.sleep(segments[-1].duration)

            except httpx.HTTPStatusError as e:
                if e.response.status_code in (404, 504):
                    task.status = "completed"
                    return full_path
                continue
            except Exception:
                task.status = "error"
                return full_path

        task.status = "stopped"
        return full_path
