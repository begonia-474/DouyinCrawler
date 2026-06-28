"""视频下载器 — 断点续传、进度回调、M3U8 直播流录制"""

import httpx
import asyncio
import aiofiles
import random
import logging
import shutil
from pathlib import Path
from typing import Callable, Union

logger = logging.getLogger(__name__)

# 文件名工具统一在 utils.py 维护；此处重新导出以保持向后兼容
from core.utils import sanitize_filename, format_filename  # noqa: F401
from core.utils import get_segments_from_m3u8, get_content_length, get_chunk_size


# 已下载 TS 分片记录上限，超过后淘汰最早记录以释放内存
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
        max_concurrent: int = 5,
        max_retries: int = 3,
        timeout: int = 30,
        progress_callback: Callable[[str, int, int], None] | None = None,
    ):
        """
        Args:
            cookie: 抖音 Cookie
            max_connections: 最大 TCP 连接数
            max_concurrent: 最大并发下载任务数（Semaphore）
            max_retries: 默认重试次数
            timeout: 超时秒数
            progress_callback: 进度回调 fn(task_id, downloaded_bytes, total_bytes)
        """
        self.cookie = cookie
        self.progress_callback = progress_callback
        self._max_retries = max_retries
        self._client = httpx.AsyncClient(
            timeout=timeout,
            follow_redirects=True,
            limits=httpx.Limits(max_connections=max_connections),
        )
        self._semaphore = asyncio.Semaphore(max_concurrent)
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

    @staticmethod
    def _safe_rename(src: Path, dst: Path) -> None:
        """Windows 兼容的文件重命名（对齐 f2）"""
        if dst.exists():
            dst.unlink()
        try:
            src.rename(dst)
        except (PermissionError, OSError):
            shutil.move(str(src), str(dst))

    async def _download_stream(
        self,
        url: str,
        task: DownloadTask,
        *,
        task_id: str | None = None,
        max_retries: int = 3,
        validate_size: bool = False,
        use_jitter: bool = False,
    ) -> Path:
        """
        核心下载方法：断点续传 + 流式写入 + 重试 backoff。

        公开方法（download_video 等）构造 DownloadTask 后调用此方法。
        """
        tid = task_id or task.filename
        self._tasks[tid] = task

        last_error = None
        for attempt in range(max_retries):
            headers = self._get_headers(task)
            existing_size = 0
            if task.temp_path.exists():
                existing_size = task.temp_path.stat().st_size
                headers["Range"] = f"bytes={existing_size}-"

            try:
                task.status = "downloading"
                async with self._client.stream("GET", url, headers=headers) as resp:
                    resp.raise_for_status()
                    content_length = int(resp.headers.get("content-length", 0))
                    if resp.status_code == 206:
                        task.total_size = existing_size + content_length
                        task.downloaded = existing_size
                    else:
                        task.total_size = content_length
                        existing_size = 0

                    chunk_size = get_chunk_size(task.total_size) if task.total_size > 0 else self.CHUNK_SIZE
                    mode = "ab" if existing_size > 0 and resp.status_code == 206 else "wb"
                    async with aiofiles.open(task.temp_path, mode) as f:
                        async for chunk in resp.aiter_bytes(chunk_size):
                            await f.write(chunk)
                            task.downloaded += len(chunk)
                            if self.progress_callback:
                                self.progress_callback(tid, task.downloaded, task.total_size)

                # 文件大小校验
                if validate_size and task.total_size > 0:
                    actual_size = task.temp_path.stat().st_size
                    if actual_size != task.total_size:
                        logger.warning("[download] 文件大小不匹配: 期望 %d, 实际 %d, URL: %s", task.total_size, actual_size, url)
                        continue

                self._safe_rename(task.temp_path, task.full_path)
                task.status = "completed"
                return task.full_path

            except (httpx.RemoteProtocolError, httpx.ReadTimeout, httpx.ConnectTimeout) as e:
                last_error = e
                jitter = random.uniform(0, 1) if use_jitter else 0
                wait = (attempt + 1) * 2 + jitter
                logger.warning("下载异常 (%s)，等待 %.1f 秒后重试 (%d/%d)", type(e).__name__, wait, attempt + 1, max_retries)
                await asyncio.sleep(wait)
            except httpx.HTTPStatusError as e:
                if e.response.status_code in (429, 500, 502, 503):
                    last_error = e
                    jitter = random.uniform(0, 2) if use_jitter else 0
                    wait = (attempt + 1) * 3 + jitter
                    logger.warning("下载 HTTP 错误 (%d)，等待 %.1f 秒后重试 (%d/%d)", e.response.status_code, wait, attempt + 1, max_retries)
                    await asyncio.sleep(wait)
                    continue
                task.status = "error"
                task.error_msg = str(e)
                raise
            except Exception as e:
                last_error = e
                jitter = random.uniform(0, 1) if use_jitter else 0
                wait = (attempt + 1) * 2 + jitter
                logger.warning("下载未知异常 (%s)，等待 %.1f 秒后重试 (%d/%d)", type(e).__name__, wait, attempt + 1, max_retries)
                await asyncio.sleep(wait)

        task.status = "error"
        task.error_msg = f"下载失败，已重试 {max_retries} 次: {last_error}"
        raise last_error

    async def download_video(
        self,
        video_url: Union[str, list[str]],
        save_dir: str | Path,
        filename: str,
        task_id: str | None = None,
        max_retries: int | None = None,
    ) -> Path:
        """
        下载视频文件（多 URL CDN 降级 + 断点续传 + 大小校验）

        Args:
            video_url: 视频 URL 或 URL 列表（CDN 降级用）
            save_dir: 保存目录
            filename: 文件名（不含扩展名）
            task_id: 任务 ID（用于进度回调）
            max_retries: 每个 URL 的最大重试次数（默认用配置值）

        Returns:
            保存的文件路径
        """
        max_retries = max_retries if max_retries is not None else self._max_retries
        async with self._semaphore:
            save_path = Path(save_dir)
            save_path.mkdir(parents=True, exist_ok=True)

            urls = [video_url] if isinstance(video_url, str) else video_url
            if not urls:
                raise ValueError("video_url 不能为空")

            task = DownloadTask(url=urls[0], save_path=save_path, filename=filename)

            if task.full_path.exists():
                logger.info("[download] 文件已存在，跳过: %s", task.full_path.name)
                task.status = "completed"
                return task.full_path

            # CDN 降级：逐个 URL 尝试
            last_error = None
            for link in urls:
                task.url = link
                try:
                    return await self._download_stream(
                        link, task, task_id=task_id,
                        max_retries=max_retries, validate_size=True, use_jitter=True,
                    )
                except Exception as e:
                    last_error = e
                    logger.warning("[download] URL 失败，尝试下一个: %s", link)

            raise last_error

    async def download_image(
        self,
        image_url: str,
        save_dir: str | Path,
        filename: str,
        max_retries: int | None = None,
    ) -> Path:
        """下载图片（断点续传 + 重试）"""
        max_retries = max_retries if max_retries is not None else self._max_retries
        async with self._semaphore:
            save_path = Path(save_dir)
            save_path.mkdir(parents=True, exist_ok=True)
            task = DownloadTask(url=image_url, save_path=save_path, filename=filename, suffix=".webp")
            if task.full_path.exists():
                return task.full_path
            return await self._download_stream(image_url, task, max_retries=max_retries)

    async def download_live_image(
        self,
        video_url: str,
        save_dir: str | Path,
        filename: str,
        max_retries: int | None = None,
    ) -> Path:
        """下载动图/实况（断点续传 + 重试）"""
        max_retries = max_retries if max_retries is not None else self._max_retries
        async with self._semaphore:
            save_path = Path(save_dir)
            save_path.mkdir(parents=True, exist_ok=True)
            task = DownloadTask(url=video_url, save_path=save_path, filename=filename, suffix=".mp4")
            if task.full_path.exists():
                return task.full_path
            return await self._download_stream(video_url, task, max_retries=max_retries)

    async def download_music(
        self,
        music_url: str,
        save_dir: str | Path,
        filename: str,
        task_id: str | None = None,
        max_retries: int | None = None,
    ) -> Path:
        """下载音乐文件（大小校验 + 进度回调 + 重试）"""
        max_retries = max_retries if max_retries is not None else self._max_retries
        async with self._semaphore:
            save_path = Path(save_dir)
            save_path.mkdir(parents=True, exist_ok=True)
            task = DownloadTask(url=music_url, save_path=save_path, filename=filename, suffix=".mp3")
            if task.full_path.exists():
                logger.info("[download] 音乐文件已存在，跳过: %s", task.full_path.name)
                task.status = "completed"
                return task.full_path
            return await self._download_stream(
                music_url, task, task_id=task_id,
                max_retries=max_retries, validate_size=True, use_jitter=True,
            )

    async def batch_download(
        self,
        tasks: list[dict],
    ) -> list[dict]:
        """
        批量下载（单个失败不影响其他任务）

        Args:
            tasks: [{"url": ..., "dir": ..., "filename": ..., "task_id": ...}, ...]

        Returns:
            [{"task_id": ..., "path": ...}, ...]
        """
        # Semaphore 已在 download_video 内层控制（由 self._max_concurrent 配置）
        results = []

        async def _download_one(t: dict):
            try:
                # url 字段可能已是 list（多 CDN），兼容 str
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
            except Exception as e:
                logger.error("批量下载中单个任务失败: %s - %s", t.get("task_id"), e)

        await asyncio.gather(*[_download_one(t) for t in tasks])
        return results

    async def download_cover(
        self,
        cover_url: str,
        save_dir: str | Path,
        filename: str,
        max_retries: int | None = None,
    ) -> Path:
        """下载封面图片（重试 + 断点续传，失败不影响主流程）"""
        max_retries = max_retries if max_retries is not None else self._max_retries
        if not cover_url:
            return None

        save_path = Path(save_dir)
        save_path.mkdir(parents=True, exist_ok=True)

        # 判断封面类型：动态封面 .webp，静态封面 .jpeg
        suffix = ".webp" if "animated_cover" in cover_url else ".jpeg"
        full_path = save_path / f"{filename}_cover{suffix}"

        if full_path.exists():
            return full_path

        task = DownloadTask(url=cover_url, save_path=save_path, filename=f"{filename}_cover", suffix=suffix)

        try:
            for attempt in range(max_retries):
                headers = self._get_headers(task)
                existing_size = 0
                if task.temp_path.exists():
                    existing_size = task.temp_path.stat().st_size
                    headers["Range"] = f"bytes={existing_size}-"
                try:
                    async with self._client.stream("GET", cover_url, headers=headers) as resp:
                        resp.raise_for_status()
                        content_length = int(resp.headers.get("content-length", 0))
                        if resp.status_code == 206:
                            task.total_size = existing_size + content_length
                        else:
                            task.total_size = content_length
                            existing_size = 0
                        chunk_size = get_chunk_size(task.total_size) if task.total_size > 0 else self.CHUNK_SIZE
                        mode = "ab" if existing_size > 0 and resp.status_code == 206 else "wb"
                        async with aiofiles.open(task.temp_path, mode) as f:
                            async for chunk in resp.aiter_bytes(chunk_size):
                                await f.write(chunk)
                    self._safe_rename(task.temp_path, full_path)
                    return full_path
                except (httpx.RemoteProtocolError, httpx.ReadTimeout, httpx.ConnectTimeout):
                    await asyncio.sleep((attempt + 1) * 2)
                except Exception:
                    await asyncio.sleep((attempt + 1) * 2)
            logger.warning("封面下载失败（已重试 %d 次）: %s", max_retries, cover_url)
            return None
        except Exception as e:
            logger.warning("封面下载失败: %s", e)
            return None

    async def download_desc(self, desc_text: str, save_dir: str | Path, filename: str) -> Path:
        """保存文案到文本文件"""
        save_path = Path(save_dir)
        save_path.mkdir(parents=True, exist_ok=True)
        full_path = save_path / f"{filename}_desc.txt"

        try:
            async with aiofiles.open(full_path, "w", encoding="utf-8") as f:
                await f.write(desc_text)
        except Exception as e:
            # 文案保存失败不影响主流程
            logger.warning("文案保存失败: %s", e)
            return None

        return full_path

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
        from collections import OrderedDict
        downloaded_segments: OrderedDict[str, None] = OrderedDict()

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
                        ts_content_length = await get_content_length(ts_url, headers, client=self._client)
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

                            downloaded_segments[segment.absolute_uri] = None

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
                        # 淘汰最早的一半记录，保留最近的分片避免重复下载
                        keep = MAX_SEGMENT_COUNT // 2
                        while len(downloaded_segments) > keep:
                            downloaded_segments.popitem(last=False)

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
