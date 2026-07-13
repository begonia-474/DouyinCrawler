"""
Python API 模块
提供模块级别的函数供 PyO3 调用
"""

import asyncio
import logging
from typing import Optional

from core.models.responses import ErrorCode

logger = logging.getLogger(__name__)


def _cover_suffix(cover_url: str) -> str:
    """根据封面 URL 判断扩展名（对齐 f2：动态封面 .webp，静态封面 .jpeg）"""
    if cover_url and "animated_cover" in cover_url:
        return ".webp"
    return ".jpeg"


def _safe_call(func):
    """装饰器：统一捕获异常，根据异常类型设置 error_code，返回 {success: False, error_code: ..., error: ...}"""
    def wrapper(*args, **kwargs):
        try:
            return func(*args, **kwargs)
        except Exception as e:
            logger.error("[py_bridge] %s 异常: %s", func.__name__, e, exc_info=True)
            error_code = _classify_error(e)
            return {"success": False, "error_code": error_code.value, "error": str(e)}
    wrapper.__name__ = func.__name__
    wrapper.__doc__ = func.__doc__
    return wrapper


def _classify_error(exc: Exception) -> ErrorCode:
    """根据异常类型映射到 ErrorCode 枚举"""
    try:
        import httpx
        if isinstance(exc, httpx.TimeoutException):
            return ErrorCode.NETWORK_TIMEOUT
        if isinstance(exc, httpx.HTTPStatusError):
            status = exc.response.status_code
            if status == 429:
                return ErrorCode.RATE_LIMITED
            if status in (401, 403):
                return ErrorCode.COOKIE_EXPIRED
            if status == 404:
                return ErrorCode.VIDEO_NOT_FOUND
            return ErrorCode.NETWORK_ERROR
        if isinstance(exc, httpx.ConnectError):
            return ErrorCode.NETWORK_ERROR
    except ImportError:
        pass

    name = type(exc).__name__
    if "Timeout" in name:
        return ErrorCode.NETWORK_TIMEOUT
    if "Connection" in name or "Connect" in name:
        return ErrorCode.NETWORK_ERROR
    if "Cookie" in name or "Auth" in name:
        return ErrorCode.COOKIE_EXPIRED
    if "Signature" in name:
        return ErrorCode.SIGNATURE_ERROR
    if "Parse" in name:
        return ErrorCode.PARSE_ERROR
    return ErrorCode.UNKNOWN


# 延迟导入，避免循环导入
_task_manager = None

def _get_task_manager():
    """获取 task_manager 单例"""
    global _task_manager
    if _task_manager is None:
        from core.task.task_manager import task_manager
        _task_manager = task_manager

        logger.info("[py_bridge] task_manager 初始化完成")
    return _task_manager


def _run_async(coro):
    """运行异步函数。

    调用方来自 Tauri spawn_blocking 线程，无 running event loop，
    直接使用 asyncio.run 创建新事件循环执行协程。
    """
    return asyncio.run(coro)


@_safe_call
def parse_video(url: str) -> dict:
    """解析视频信息"""
    logger.info("[py_bridge] parse_video 调用, url=%s", url[:80])
    handler = _get_task_manager().handler
    result = _run_async(handler.handle_parse_video(url))
    logger.info("[py_bridge] parse_video 返回: success=%s", result.get("success"))
    return result


@_safe_call
def download_video(url: str) -> dict:
    """下载单个视频（仅下载，不写 DB，返回结果供 Rust 事务性完成）

    F2.1: 不再调用 download_single_sync，DB 写入由 Rust TaskApplicationService 负责。
    """
    import asyncio
    handler = _get_task_manager().handler
    result = asyncio.run(handler.handle_one_video(url))
    return result


@_safe_call
def get_live_info(url: str) -> dict:
    """获取直播信息"""
    logger.info("[py_bridge] get_live_info 调用, url=%s", url[:80])
    handler = _get_task_manager().handler
    result = _run_async(handler.handle_user_live(url))
    logger.info("[py_bridge] get_live_info 返回: success=%s", result.get("success"))
    if not result.get("success"):
        return result

    flv_urls = result.get("flv_urls") or list((result.get("flv_pull_url") or {}).values())
    m3u8_urls = result.get("m3u8_urls") or list((result.get("m3u8_pull_url") or {}).values())
    return {
        "success": True,
        "live_info": {
            "title": result.get("title", ""),
            "nickname": result.get("nickname", ""),
            "is_live": result.get("is_live", False),
            "user_count": result.get("user_count", 0),
            "room_id": result.get("room_id", ""),
            "cover": result.get("cover") or result.get("cover_url", ""),
            "flv_urls": flv_urls,
            "m3u8_urls": m3u8_urls,
        },
    }


@_safe_call
def resolve_live(url: str) -> dict:
    """解析直播录制参数，下载和任务生命周期由 Rust 负责。

    保持 f2 的直播录制约定：固定选择 FULL_HD1 HLS 流，文件名以
    ``_live.flv`` 结尾，保存到 ``download_path/app_name/live/nickname``。
    """
    import time
    from pathlib import Path

    from core.download.downloader import format_filename
    from core.utils import sanitize_filename

    handler = _get_task_manager().handler
    result = _run_async(handler.handle_user_live(url))
    if not result.get("success"):
        return result

    m3u8_urls = result.get("m3u8_pull_url") or {}
    m3u8_url = m3u8_urls.get("FULL_HD1")
    if not m3u8_url:
        return {"success": False, "error": "未获取到 FULL_HD1 直播流"}

    config = handler.config
    download_path = (
        config.download_path
        if isinstance(config.download_path, Path)
        else Path(config.download_path)
    )
    nickname = sanitize_filename(result.get("nickname") or "unknown")
    base_dir = download_path / config.app_name / "live" / nickname
    formatted_name = format_filename(
        config.naming,
        {
            "create_time": int(time.time()),
            "desc": result.get("title") or "live",
            "author": result.get("nickname") or "",
            "aweme_id": result.get("room_id") or "",
            "author_uid": result.get("user_id") or "",
        },
    )
    save_dir = base_dir / formatted_name if getattr(config, "folderize", False) else base_dir
    filename = formatted_name + "_live"

    return {
        "success": True,
        "web_rid": result.get("web_rid", ""),
        "room_id": result.get("room_id", ""),
        "title": result.get("title", ""),
        "nickname": result.get("nickname", ""),
        "sec_user_id": result.get("sec_user_id", ""),
        "user_id": result.get("user_id", ""),
        "cover_url": result.get("cover_url", ""),
        "user_count": result.get("user_count", 0),
        "m3u8_url": m3u8_url,
        "save_dir": str(save_dir),
        "filename": filename,
        "suffix": ".flv",
        "headers": {
            "User-Agent": (
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
                "AppleWebKit/537.36 (KHTML, like Gecko) "
                "Chrome/130.0.0.0 Safari/537.36"
            ),
            "Referer": "https://www.douyin.com/",
            "Cookie": config.cookie,
        },
    }


@_safe_call
def download_batch(mode: str, url: str) -> dict:
    """批量下载（不写 task DB 表，返回结果供 Rust 处理）

    Args:
        mode: 下载模式 (post/like/mix/collects)
        url: 目标 URL

    Returns:
        {"success": True, "count": N, "results": [{path, detail: {aweme_id, desc, author_nickname, ...}}, ...]}
    """
    import asyncio
    handler = _get_task_manager().handler

    logger.info("[py_bridge] download_batch 调用, mode=%s, url=%s", mode, url[:80])

    if mode == "post":
        result = asyncio.run(handler.handle_user_post(url))
    elif mode == "like":
        result = asyncio.run(handler.handle_user_like(url))
    elif mode == "mix":
        result = asyncio.run(handler.handle_user_mix(url))
    elif mode == "collects":
        result = asyncio.run(handler.handle_collects_video(url))
    else:
        return {"success": False, "error": f"未知的批量下载模式: {mode}"}

    logger.info("[py_bridge] download_batch 返回: success=%s, count=%s",
                result.get("success"), result.get("count"))
    return result


@_safe_call
def start_download(mode: str, url: str) -> dict:
    """统一下载入口（通过 mode 分发）"""
    logger.info("[py_bridge] start_download 调用, mode=%s, url=%s", mode, url[:80])
    task_id = _get_task_manager().start_download(mode, url)
    logger.info("[py_bridge] 下载任务已启动, task_id=%s", task_id)
    return {"success": True, "task_id": task_id}


@_safe_call
def get_user_profile(url: str) -> dict:
    """获取用户信息"""
    logger.info("[py_bridge] get_user_profile 调用, url=%s", url[:80])
    handler = _get_task_manager().handler
    result = _run_async(handler.handle_user_profile(url))
    logger.info("[py_bridge] get_user_profile 返回: success=%s", result.get("success"))
    return result


@_safe_call
def get_user_posts(url: str, cursor: int = 0, count: int = 20) -> dict:
    """获取用户作品列表（单页）"""
    handler = _get_task_manager().handler
    return _run_async(handler.handle_user_post_list(url, cursor, count))


@_safe_call
def search_videos(keyword: str, offset: int = 0, count: int = 10) -> dict:
    """搜索视频"""
    handler = _get_task_manager().handler
    return _run_async(handler.handle_search(keyword, offset, count))


@_safe_call
def get_mix_info(url: str, cursor: int = 0, count: int = 20) -> dict:
    """获取合集信息（单页）"""
    handler = _get_task_manager().handler
    return _run_async(handler.handle_user_mix_list(url, cursor, count))


@_safe_call
def get_collects_list() -> dict:
    """获取收藏夹列表"""
    handler = _get_task_manager().handler
    return _run_async(handler.handle_user_collects())


@_safe_call
def get_collects_video_list(collects_id: str, cursor: int = 0, count: int = 20) -> dict:
    """获取收藏夹视频列表（单页）"""
    handler = _get_task_manager().handler
    return _run_async(handler.handle_collects_video_list(collects_id, cursor, count))


@_safe_call
def get_following_list(url: str, offset: int = 0, count: int = 20) -> dict:
    """获取关注列表"""
    handler = _get_task_manager().handler
    return _run_async(handler.handle_user_following(url, offset, count))


@_safe_call
def get_follower_list(url: str, offset: int = 0, count: int = 20) -> dict:
    """获取粉丝列表"""
    handler = _get_task_manager().handler
    return _run_async(handler.handle_user_follower(url, offset, count))


@_safe_call
def get_music_collection(cursor: int = 0, count: int = 18) -> dict:
    """获取音乐收藏"""
    handler = _get_task_manager().handler
    return _run_async(handler.handle_user_music_collection(cursor, count))


@_safe_call
def download_music_batch(url: str) -> dict:
    """下载全部音乐（不写 task DB 表，返回结果供 Rust 处理）

    Returns:
        {"success": True, "music_list": [...], "results": [{music_id, title, author, path, file_size, success, error}, ...]}
    """
    import os
    handler = _get_task_manager().handler

    # 1. 获取音乐列表
    collection = _run_async(handler.handle_user_music_collection())
    if not collection.get("success"):
        return {"success": False, "error": collection.get("error", "获取音乐列表失败")}

    music_list = collection.get("music_list", [])
    results = []

    # 2. 逐首下载
    for music in music_list:
        music_id = music.get("music_id", "")
        title = music.get("title", "")
        author = music.get("author", "")
        play_url = music.get("play_url", "")

        try:
            dl_result = _run_async(handler.handle_download_music(play_url, title, author))
            file_path = dl_result.get("path", "") if dl_result.get("success") else ""
            file_size = 0
            if file_path:
                try:
                    file_size = os.path.getsize(file_path)
                except OSError:
                    pass
            results.append({
                "music_id": music_id,
                "title": title,
                "author": author,
                "play_url": play_url,
                "path": file_path,
                "file_size": file_size,
                "success": dl_result.get("success", False),
                "error": dl_result.get("error", ""),
            })
        except Exception as e:
            logger.error("[download_music_batch] 单曲下载失败: %s", e)
            results.append({
                "music_id": music_id,
                "title": title,
                "author": author,
                "play_url": play_url,
                "path": "",
                "file_size": 0,
                "success": False,
                "error": str(e),
            })

    return {"success": True, "music_list": music_list, "results": results}


@_safe_call
def download_music(play_url: str, title: str, author: str = "") -> dict:
    """下载音乐（不写 DB，返回结果供 Rust/前端处理持久化）"""
    handler = _get_task_manager().handler
    result = _run_async(handler.handle_download_music(play_url, title, author))
    return result


@_safe_call
def get_following_live() -> dict:
    """获取关注直播列表"""
    handler = _get_task_manager().handler
    return _run_async(handler.handle_following_live())


@_safe_call
def get_related(url: str, count: int = 20, filter_gids: str = "") -> dict:
    """获取相关推荐视频（单页，前端控制分页）"""
    handler = _get_task_manager().handler
    return _run_async(handler.handle_related(url, count, filter_gids))


@_safe_call
def get_comments(url: str, cursor: int = 0, count: int = 20) -> dict:
    """获取评论"""
    handler = _get_task_manager().handler
    return _run_async(handler.handle_post_comment(url, cursor, count))


@_safe_call
def get_comment_replies(url: str, comment_id: str, cursor: int = 0, count: int = 3) -> dict:
    """获取评论回复"""
    handler = _get_task_manager().handler
    return _run_async(handler.handle_post_comment_reply(url, comment_id, cursor, count))


@_safe_call
def get_tab_feed(count: int = 10) -> dict:
    """获取推荐 Feed"""
    handler = _get_task_manager().handler
    return _run_async(handler.handle_tab_feed(count))


@_safe_call
def get_follow_feed(cursor: int = 0, count: int = 10) -> dict:
    """获取关注 Feed"""
    handler = _get_task_manager().handler
    return _run_async(handler.handle_follow_feed(cursor, count))


@_safe_call
def get_friend_feed(cursor: int = 0, count: int = 10) -> dict:
    """获取好友 Feed"""
    handler = _get_task_manager().handler
    return _run_async(handler.handle_friend_feed(cursor, count))


@_safe_call
def get_user_likes(url: str, cursor: int = 0, count: int = 20) -> dict:
    """获取用户点赞列表（单页）"""
    handler = _get_task_manager().handler
    return _run_async(handler.handle_user_like_list(url, cursor, count))


@_safe_call
def get_post_stats(url: str) -> dict:
    """获取作品统计"""
    handler = _get_task_manager().handler
    return _run_async(handler.handle_post_stats(url))


@_safe_call
def start_live_record(url: str) -> dict:
    """开始直播录制"""
    task_id = _get_task_manager().start_live_record(url)
    return {"success": True, "task_id": task_id, "message": "直播录制已启动"}


@_safe_call
def stop_live_record(task_id: str) -> dict:
    """停止直播录制"""
    result = _get_task_manager().stop_live_record(task_id)
    if not result:
        return {"success": False, "error": "录制任务不存在"}
    return {"success": True, "task_id": task_id}


@_safe_call
def get_live_status() -> dict:
    """获取直播录制状态"""
    status = _get_task_manager().get_live_status()
    return {"success": True, "data": status}


@_safe_call
def get_batch_status() -> dict:
    """获取批量下载状态"""
    status = _get_task_manager().get_batch_status()
    return {"success": True, "data": status}




@_safe_call
def resolve_urls(mode: str, url: str) -> dict:
    """解析下载 URL 列表（不执行下载）
    
    将"知道下什么"和"执行下载"分离。
    返回下载所需的 URL 列表 + 元数据，供 Rust DownloadEngine 使用。
    
    Args:
        mode: 下载模式 (one/post/like/mix/collects/music)
        url: 目标 URL
        
    Returns:
        {
            "success": True,
            "items": [
                {
                    "aweme_id": "xxx",
                    "download_url": "https://...",   # 视频/文件下载 URL（可能是列表）
                    "filename": "描述_20240101",      # 文件名（不含扩展名）
                    "suffix": ".mp4",                 # 扩展名
                    "headers": {                      # 下载所需的 HTTP headers
                        "User-Agent": "...",
                        "Cookie": "...",
                        "Referer": "..."
                    },
                    "content_type": "video",          # video / image / music / cover / desc
                    "detail": { ... },                # 完整视频元数据（用于 DB 写入）
                    "accessories": [                  # 附属文件（音乐、封面、文案）
                        {"url": "...", "filename": "...", "suffix": ".mp3", "content_type": "music"},
                    ]
                },
                ...
            ],
            "save_dir": "/path/to/save",          # 建议的保存目录
            "total": 100                          # 总数
        }
    """
    import asyncio
    from pathlib import Path
    from core.download.downloader import format_filename
    
    logger.info("[py_bridge] resolve_urls 调用, mode=%s, url=%s", mode, url[:80])
    
    handler = _get_task_manager().handler

    # 从 handler.config 统一读取配置
    config = handler.config
    cookie = config.cookie
    naming = config.naming
    download_path = config.download_path if isinstance(config.download_path, Path) else Path(config.download_path)
    app_name = config.app_name
    folderize = config.folderize
    music_enabled = config.music
    cover_enabled = config.cover
    desc_enabled = config.desc
    
    # 通用 headers
    base_headers = {
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
        "Referer": "https://www.douyin.com/",
        "Cookie": cookie,
    }
    
    def build_download_url(detail, content_type="video"):
        """构建单个下载项"""
        aweme_id = detail.aweme_id
        filename = format_filename(naming, detail.to_dict())
        
        # 根据内容类型确定 URL 和后缀
        if content_type == "video":
            url = detail.video_urls if detail.video_urls else detail.video_url
            suffix = ".mp4"
        elif content_type == "image":
            url = detail.images[0] if detail.images else None
            suffix = ".webp"
        elif content_type == "music":
            url = detail.music_url
            suffix = ".mp3"
        elif content_type == "cover":
            url = detail.cover_url
            suffix = _cover_suffix(detail.cover_url)
        elif content_type == "desc":
            url = None
            suffix = ".txt"
        else:
            return None
        
        if not url:
            return None
        
        # 构建附属文件列表
        accessories = []
        if music_enabled and detail.music_url:
            accessories.append({
                "url": detail.music_url,
                "filename": f"{filename}_music",
                "suffix": ".mp3",
                "content_type": "music",
            })
        if cover_enabled and detail.cover_url:
            accessories.append({
                "url": detail.cover_url,
                "filename": f"{filename}_cover",
                "suffix": _cover_suffix(detail.cover_url),
                "content_type": "cover",
            })
        if desc_enabled and detail.desc:
            accessories.append({
                "url": None,
                "filename": f"{filename}_desc",
                "suffix": ".txt",
                "content_type": "desc",
                "content": detail.desc,  # 文案内容直接存储
            })
        
        return {
            "aweme_id": aweme_id,
            "download_url": url,
            "filename": f"{filename}_video",
            "suffix": suffix,
            "headers": base_headers.copy(),
            "content_type": content_type,
            "detail": detail.to_db_dict(),
            "accessories": accessories,
        }
    
    if mode == "one":
        # 单视频解析
        result = _run_async(handler._video.handle_parse_video(url))
        if not result.get("success"):
            return result
        
        # 重新获取完整 detail（handle_parse_video 返回的是 to_db_dict）
        from core.utils import AwemeIdFetcher
        from core.crawler_engine.filter import PostDetailFilter, UserProfileFilter
        
        aweme_id = _run_async(AwemeIdFetcher.get_aweme_id(url))
        if not aweme_id:
            return {"success": False, "error": "无法从 URL 提取 aweme_id"}
        
        async def fetch_detail():
            async with handler._video._make_crawler() as crawler:
                data = await crawler.fetch_post_detail(aweme_id)
                return PostDetailFilter(data)
        
        detail = _run_async(fetch_detail())
        
        if detail.is_prohibited:
            return {"success": False, "error": "视频侵权不可用"}
        
        # 确定保存目录
        user_dir = download_path / app_name / "one" / (detail.author_nickname or "unknown")
        save_dir = user_dir / format_filename(naming, detail.to_dict()) if folderize else user_dir

        from core.models.single_download import (
            SingleAccessory,
            SingleAccessoryKind,
            SingleDownloadItem,
            SingleDownloadPlanV1,
            SingleMediaKind,
            SingleOutputSpec,
            SingleVideoMetadata,
        )

        base_name = format_filename(naming, detail.to_dict())
        metadata = SingleVideoMetadata.model_validate(detail.to_db_dict())

        def single_accessories(*, include_description: bool) -> list[SingleAccessory]:
            accessories: list[SingleAccessory] = []
            if music_enabled and detail.music_url:
                accessories.append(SingleAccessory(
                    kind=SingleAccessoryKind.MUSIC,
                    url=detail.music_url,
                    output=SingleOutputSpec(
                        filename=f"{base_name}_music",
                        suffix=".mp3",
                        folder_name=None,
                    ),
                ))
            if cover_enabled and detail.cover_url:
                accessories.append(SingleAccessory(
                    kind=SingleAccessoryKind.COVER,
                    url=detail.cover_url,
                    output=SingleOutputSpec(
                        filename=f"{base_name}_cover",
                        suffix=_cover_suffix(detail.cover_url),
                        folder_name=None,
                    ),
                ))
            if include_description and desc_enabled and detail.desc:
                accessories.append(SingleAccessory(
                    kind=SingleAccessoryKind.DESCRIPTION,
                    content=detail.desc,
                    output=SingleOutputSpec(
                        filename=f"{base_name}_desc",
                        suffix=".txt",
                        folder_name=None,
                    ),
                ))
            return accessories

        def output(filename: str, suffix: str) -> SingleOutputSpec:
            # ``save_dir`` already includes the folderized post directory for mode=one.
            return SingleOutputSpec(
                filename=filename,
                suffix=suffix,
                folder_name=None,
            )
        
        # 构建下载项
        if detail.is_image_post and (detail.images or detail.images_video):
            # 图片帖子：先实况视频，再静态图（对齐 f2）
            items: list[SingleDownloadItem] = []

            # 实况视频（对齐 f2: _live_{i+1}.mp4）
            if detail.images_video:
                for i, live_url in enumerate(detail.images_video):
                    if live_url:
                        items.append(SingleDownloadItem(
                            aweme_id=detail.aweme_id,
                            urls=[live_url],
                            kind=SingleMediaKind.LIVE_PHOTO,
                            output=output(f"{base_name}_live_{i + 1}", ".mp4"),
                            headers=base_headers.copy(),
                            metadata=metadata,
                        ))

            # 静态图（对齐 f2: _image_{i+1}.webp）
            for i, img_url in enumerate(detail.images):
                if img_url:
                    items.append(SingleDownloadItem(
                        aweme_id=detail.aweme_id,
                        urls=[img_url],
                        kind=SingleMediaKind.IMAGE,
                        output=output(f"{base_name}_image_{i + 1}", ".webp"),
                        headers=base_headers.copy(),
                        metadata=metadata,
                    ))

            # 添加附属文件
            if items:
                items[0].accessories.extend(
                    single_accessories(include_description=False)
                )

            return SingleDownloadPlanV1(
                save_dir=str(save_dir),
                items=items,
                total=len(items),
            ).model_dump(mode="json")
        else:
            # 视频帖子
            urls = detail.video_urls or ([detail.video_url] if detail.video_url else [])
            urls = [download_url for download_url in urls if download_url]
            if not urls:
                return {"success": False, "error": "无法获取视频下载链接"}

            item = SingleDownloadItem(
                aweme_id=detail.aweme_id,
                urls=urls,
                kind=SingleMediaKind.VIDEO,
                output=output(f"{base_name}_video", ".mp4"),
                headers=base_headers.copy(),
                metadata=metadata,
                accessories=single_accessories(include_description=True),
            )
            return SingleDownloadPlanV1(
                save_dir=str(save_dir),
                items=[item],
                total=1,
            ).model_dump(mode="json")
    
    elif mode in ("post", "like", "mix", "collects"):
        # 批量解析 — 直接使用 crawler + PostDetailFilter 获取 to_db_dict() 格式
        from core.crawler_engine.filter import PostDetailFilter, UserProfileFilter
        from core.utils import SecUserIdFetcher, MixIdFetcher

        user_profile = None

        async def _fetch_batch_details():
            nonlocal user_profile
            async with handler._user._make_crawler() as crawler:
                # post/mix 模式获取用户资料
                if mode in ("post", "like"):
                    sec_user_id = await SecUserIdFetcher.get_sec_user_id(url)
                    if not sec_user_id:
                        return None, "无法从 URL 提取 sec_user_id"
                    if mode == "post":
                        profile_data = await crawler.fetch_user_profile(sec_user_id)
                        profile = UserProfileFilter(profile_data)
                        user_profile = profile.to_dict()
                        all_details = await handler._user._paginate_and_collect(
                            lambda c, n: crawler.fetch_user_post(sec_user_id, c, n),
                            skip_prohibited=True,
                        )
                    else:
                        all_details = await handler._user._paginate_and_collect(
                            lambda c, n: crawler.fetch_user_favorite(sec_user_id, c, n),
                            skip_prohibited=False,
                        )
                elif mode == "mix":
                    mix_id = await MixIdFetcher.get_mix_id(url)
                    if not mix_id:
                        return None, "无法从 URL 提取 mix_id"
                    all_details = await handler._mix._paginate_and_collect(
                        lambda c, n: crawler.fetch_mix_aweme(mix_id, c, n),
                        skip_prohibited=True,
                    )
                elif mode == "collects":
                    # collects 模式下 url 参数就是 collects_id
                    collects_id = url
                    all_details = await handler._collection._paginate_and_collect(
                        lambda c, n: crawler.fetch_user_collects_video(collects_id, c, n),
                        skip_prohibited=False,
                    )
                else:
                    all_details = []
                return all_details, None

        batch_result, batch_error = _run_async(_fetch_batch_details())
        if batch_error:
            return {"success": False, "error": batch_error}

        # 构建 items（使用 to_db_dict() 格式，对齐 Rust VideoInfo 结构体）
        items = []
        for detail in batch_result:
            if not isinstance(detail, PostDetailFilter):
                continue

            filename = format_filename(naming, detail.to_dict())
            headers = base_headers.copy()
            # folderize 子目录名（对齐 f2）
            folder = filename if folderize else None

            if detail.is_image_post and (detail.images or detail.images_video):
                # 图集帖子：先实况视频，再静态图（对齐 f2）

                # 实况视频（对齐 f2: _live_{i+1}.mp4）
                if detail.images_video:
                    for i, live_url in enumerate(detail.images_video):
                        if live_url:
                            items.append({
                                "aweme_id": detail.aweme_id,
                                "download_url": live_url,
                                "filename": f"{filename}_live_{i + 1}",
                                "suffix": ".mp4",
                                "headers": headers,
                                "content_type": "live_photo",
                                "detail": detail.to_db_dict(),
                                "accessories": [],
                                "folder_name": folder,
                            })

                # 静态图（对齐 f2: _image_{i+1}.webp）
                for i, img_url in enumerate(detail.images):
                    if img_url:
                        items.append({
                            "aweme_id": detail.aweme_id,
                            "download_url": img_url,
                            "filename": f"{filename}_image_{i + 1}",
                            "suffix": ".webp",
                            "headers": headers,
                            "content_type": "image",
                            "detail": detail.to_db_dict(),
                            "accessories": [],
                            "folder_name": folder,
                        })
            elif detail.video_urls or detail.video_url:
                # 视频帖子
                video_url = detail.video_urls if detail.video_urls else detail.video_url
                item = {
                    "aweme_id": detail.aweme_id,
                    "download_url": video_url,
                    "filename": f"{filename}_video",
                    "suffix": ".mp4",
                    "headers": headers,
                    "content_type": "video",
                    "detail": detail.to_db_dict(),
                    "accessories": [],
                    "folder_name": folder,
                }
                if music_enabled and detail.music_url:
                    item["accessories"].append({
                        "url": detail.music_url,
                        "filename": f"{filename}_music",
                        "suffix": ".mp3",
                        "content_type": "music",
                    })
                if cover_enabled and detail.cover_url:
                    item["accessories"].append({
                        "url": detail.cover_url,
                        "filename": f"{filename}_cover",
                        "suffix": _cover_suffix(detail.cover_url),
                        "content_type": "cover",
                    })
                items.append(item)

        # 确定保存目录（对齐 f2：download_path / app_name / mode / nickname）
        nickname = "unknown"
        if user_profile:
            nickname = user_profile.get("nickname") or "unknown"
        elif batch_result:
            first = next((d for d in batch_result if isinstance(d, PostDetailFilter)), None)
            if first:
                nickname = first.author_nickname or "unknown"
        save_dir = download_path / app_name / mode / nickname

        return {
            "success": True,
            "items": items,
            "save_dir": str(save_dir),
            "total": len(items),
            "user_profile": user_profile,
        }
    
    elif mode == "music":
        # 音乐批量解析
        from core.crawler_engine.filter import MusicCollectionFilter
        
        async def fetch_music_list():
            async with handler._music._make_crawler() as crawler:
                data = await crawler.get_music_collection(0, 1000)
                music_filter = MusicCollectionFilter(data)
                return music_filter.get_music_list()
        
        music_list = _run_async(fetch_music_list())

        # 对齐 f2：download_path / app_name / music / nickname
        nickname = music_list[0].get("author", "unknown") if music_list else "unknown"
        save_dir = download_path / app_name / "music" / nickname
        save_dir.mkdir(parents=True, exist_ok=True)
        
        items = []
        for music in music_list:
            if music.get("play_url"):
                music_title = music.get("title", "unknown")
                item = {
                    "aweme_id": music.get("music_id", ""),
                    "download_url": music["play_url"],
                    "filename": f"{music_title}_{music.get('author', 'unknown')}",
                    "suffix": ".mp3",
                    "headers": base_headers.copy(),
                    "content_type": "music",
                    "detail": music,
                    "accessories": [],
                    "folder_name": music_title if folderize else None,
                }
                items.append(item)
        
        return {
            "success": True,
            "items": items,
            "save_dir": str(save_dir),
            "total": len(items),
        }
    
    else:
        return {"success": False, "error": f"未知的下载模式: {mode}"}


@_safe_call
def resolve_page(mode: str, url: str, cursor: int = 0, count: int = 20, aweme_ids: list = None) -> dict:
    """解析单页下载 URL（分页模式）

    与 resolve_urls 的区别：只返回一页数据 + 分页元数据 (next_cursor, has_more)，
    由 Rust 驱动分页循环，实现"边解析边下载"。

    与 resolve_urls 的另一个区别：不根据 music/cover/desc 配置过滤附属文件，
    返回所有可用的附属文件，由 Rust 侧根据配置决定是否下载。

    Args:
        mode: 下载模式 (one/post/like/mix/collects/music)
        url: 目标 URL
        cursor: 分页游标（首页传 0）
        count: 每页数量

    Returns:
        {
            "success": True,
            "items": [...],
            "save_dir": "/path/to/save",
            "total": 10,
            "next_cursor": 12345,    # 下一页游标，无更多数据时为 None
            "has_more": True,         # 是否还有更多数据
            "user_profile": { ... },  # 仅 post 模式首次返回
        }
    """
    import asyncio
    from pathlib import Path
    from core.download.downloader import format_filename

    logger.info("[py_bridge] resolve_page 调用, mode=%s, url=%s, cursor=%d, count=%d",
                mode, url[:80], cursor, count)

    handler = _get_task_manager().handler
    config = handler.config
    cookie = config.cookie
    naming = config.naming
    download_path = config.download_path if isinstance(config.download_path, Path) else Path(config.download_path)
    app_name = config.app_name
    folderize = config.folderize

    # 通用 headers
    base_headers = {
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
        "Referer": "https://www.douyin.com/",
        "Cookie": cookie,
    }

    def _build_items_from_details(details, mode_for_dir):
        """从 PostDetailFilter 列表构建 items（不过滤附属文件）"""
        items = []
        for detail in details:
            if not hasattr(detail, 'aweme_id'):
                continue
            filename = format_filename(naming, detail.to_dict())
            folder = filename if folderize else None

            if detail.is_image_post and (detail.images or detail.images_video):
                # 实况视频（对齐 f2: _live_{i+1}.mp4）
                if detail.images_video:
                    for i, live_url in enumerate(detail.images_video):
                        if live_url:
                            items.append({
                                "aweme_id": detail.aweme_id,
                                "download_url": live_url,
                                "filename": f"{filename}_live_{i + 1}",
                                "suffix": ".mp4",
                                "headers": base_headers.copy(),
                                "content_type": "live_photo",
                                "detail": detail.to_db_dict(),
                                "accessories": _build_accessories(detail, filename),
                                "folder_name": folder,
                            })

                # 静态图（对齐 f2: _image_{i+1}.webp）
                for i, img_url in enumerate(detail.images):
                    if img_url:
                        items.append({
                            "aweme_id": detail.aweme_id,
                            "download_url": img_url,
                            "filename": f"{filename}_image_{i + 1}",
                            "suffix": ".webp",
                            "headers": base_headers.copy(),
                            "content_type": "image",
                            "detail": detail.to_db_dict(),
                            "accessories": _build_accessories(detail, filename),
                            "folder_name": folder,
                        })
            elif detail.video_urls or detail.video_url:
                video_url = detail.video_urls if detail.video_urls else detail.video_url
                items.append({
                    "aweme_id": detail.aweme_id,
                    "download_url": video_url,
                    "filename": f"{filename}_video",
                    "suffix": ".mp4",
                    "headers": base_headers.copy(),
                    "content_type": "video",
                    "detail": detail.to_db_dict(),
                    "accessories": _build_accessories(detail, filename),
                    "folder_name": folder,
                })
        return items

    def _build_accessories(detail, filename):
        """构建附属文件列表（不过滤，返回所有可用的）"""
        accessories = []
        if detail.music_url:
            accessories.append({
                "url": detail.music_url,
                "filename": f"{filename}_music",
                "suffix": ".mp3",
                "content_type": "music",
            })
        if detail.cover_url:
            accessories.append({
                "url": detail.cover_url,
                "filename": f"{filename}_cover",
                "suffix": _cover_suffix(detail.cover_url),
                "content_type": "cover",
            })
        if detail.desc:
            accessories.append({
                "url": None,
                "filename": f"{filename}_desc",
                "suffix": ".txt",
                "content_type": "desc",
                "content": detail.desc,
            })
        return accessories

    if mode == "one":
        # 单视频：无分页，委托给 resolve_urls
        result = resolve_urls(mode, url)
        if result.get("success"):
            result["next_cursor"] = None
            result["has_more"] = False
        return result

    elif mode in ("post", "like", "mix", "collects"):
        # 单个 aweme_id 优化：直接获取单个视频，跳过分页
        if aweme_ids and len(aweme_ids) == 1 and cursor == 0:
            from core.crawler_engine.filter import PostDetailFilter
            from core.utils import AwemeIdFetcher

            target_id = aweme_ids[0]
            logger.info("[py_bridge] 单视频优化: aweme_id=%s, 跳过分页", target_id)

            async def _fetch_single():
                async with handler._video._make_crawler() as crawler:
                    data = await crawler.fetch_post_detail(target_id)
                    return PostDetailFilter(data) if data else None

            detail = _run_async(_fetch_single())
            if not detail:
                return {"success": False, "error": f"无法获取视频 {target_id}"}

            nickname = getattr(detail, "author_nickname", None) or "unknown"
            save_dir = download_path / app_name / mode / nickname
            items = _build_items_from_details([detail], mode)

            return {
                "success": True,
                "items": items,
                "save_dir": str(save_dir),
                "total": len(items),
                "next_cursor": None,
                "has_more": False,
            }
        from core.crawler_engine.filter import UserPostFilter, UserProfileFilter
        from core.utils import SecUserIdFetcher, MixIdFetcher

        user_profile = None
        all_details = []
        next_cursor = None
        has_more = False

        async def _fetch_single_page():
            nonlocal user_profile, all_details, next_cursor, has_more
            async with handler._user._make_crawler() as crawler:
                if mode in ("post", "like"):
                    sec_user_id = await SecUserIdFetcher.get_sec_user_id(url)
                    if not sec_user_id:
                        return "无法从 URL 提取 sec_user_id"
                    if mode == "post":
                        # 仅首页获取用户资料
                        if cursor == 0:
                            profile_data = await crawler.fetch_user_profile(sec_user_id)
                            profile = UserProfileFilter(profile_data)
                            user_profile = profile.to_dict()
                        data = await crawler.fetch_user_post(sec_user_id, cursor, count)
                    else:
                        data = await crawler.fetch_user_favorite(sec_user_id, cursor, count)
                elif mode == "mix":
                    mix_id = await MixIdFetcher.get_mix_id(url)
                    if not mix_id:
                        return "无法从 URL 提取 mix_id"
                    data = await crawler.fetch_mix_aweme(mix_id, cursor, count)
                elif mode == "collects":
                    collects_id = url
                    data = await crawler.fetch_user_collects_video(collects_id, cursor, count)
                else:
                    data = None

                if not data:
                    has_more = False
                    return None

                video_filter = UserPostFilter(data)
                for detail in video_filter.get_video_list():
                    all_details.append(detail)

                has_more = video_filter.has_more
                next_cursor = video_filter.max_cursor if has_more else None
                return None

        error = _run_async(_fetch_single_page())
        if error:
            return {"success": False, "error": error}

        items = _build_items_from_details(all_details, mode)
        # 对齐 f2：download_path / app_name / mode / nickname
        nickname = "unknown"
        if user_profile:
            nickname = user_profile.get("nickname") or "unknown"
        elif all_details:
            first = all_details[0]
            nickname = getattr(first, "author_nickname", None) or "unknown"
        save_dir = download_path / app_name / mode / nickname

        result = {
            "success": True,
            "items": items,
            "save_dir": str(save_dir),
            "total": len(items),
            "next_cursor": next_cursor,
            "has_more": has_more,
        }
        if user_profile:
            result["user_profile"] = user_profile
        return result

    elif mode == "music":
        # 音乐：单次拉取，无分页
        result = resolve_urls(mode, url)
        if result.get("success"):
            result["next_cursor"] = None
            result["has_more"] = False
        return result

    else:
        return {"success": False, "error": f"未知的下载模式: {mode}"}
