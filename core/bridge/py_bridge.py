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


def _get_context():
    """获取 ParsingContext 单例"""
    from core.bridge.parsing_context import context
    return context


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
    handler = _get_context().handler
    result = _run_async(handler.handle_parse_video(url))
    logger.info("[py_bridge] parse_video 返回: success=%s", result.get("success"))
    return result


@_safe_call
def get_live_info(url: str) -> dict:
    """获取直播信息"""
    logger.info("[py_bridge] get_live_info 调用, url=%s", url[:80])
    handler = _get_context().handler
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
    from core.models.live_record import LiveOutputV1, LivePlanV1
    from core.utils import sanitize_filename

    handler = _get_context().handler
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

    try:
        user_count = int(result.get("user_count") or 0)
    except (TypeError, ValueError):
        user_count = 0

    return LivePlanV1(
        web_rid=str(result.get("web_rid") or ""),
        room_id=str(result.get("room_id") or ""),
        title=str(result.get("title") or ""),
        nickname=str(result.get("nickname") or ""),
        sec_user_id=str(result.get("sec_user_id") or ""),
        user_id=str(result["user_id"]) if result.get("user_id") else None,
        cover_url=str(result.get("cover_url") or ""),
        user_count=user_count,
        m3u8_url=str(m3u8_url),
        output=LiveOutputV1(
            save_dir=str(save_dir), filename=filename, suffix=".flv"
        ),
        headers={
            "User-Agent": (
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
                "AppleWebKit/537.36 (KHTML, like Gecko) "
                "Chrome/130.0.0.0 Safari/537.36"
            ),
            "Referer": "https://www.douyin.com/",
            "Cookie": config.cookie,
        },
    ).model_dump(mode="json")


@_safe_call
def get_user_profile(url: str) -> dict:
    """获取用户信息"""
    logger.info("[py_bridge] get_user_profile 调用, url=%s", url[:80])
    handler = _get_context().handler
    result = _run_async(handler.handle_user_profile(url))
    logger.info("[py_bridge] get_user_profile 返回: success=%s", result.get("success"))
    return result


@_safe_call
def get_user_posts(url: str, cursor: int = 0, count: int = 20) -> dict:
    """获取用户作品列表（单页）"""
    handler = _get_context().handler
    return _run_async(handler.handle_user_post_list(url, cursor, count))


@_safe_call
def search_videos(keyword: str, offset: int = 0, count: int = 10) -> dict:
    """搜索视频"""
    handler = _get_context().handler
    return _run_async(handler.handle_search(keyword, offset, count))


@_safe_call
def get_mix_info(url: str, cursor: int = 0, count: int = 20) -> dict:
    """获取合集信息（单页）"""
    handler = _get_context().handler
    return _run_async(handler.handle_user_mix_list(url, cursor, count))


@_safe_call
def get_collects_list() -> dict:
    """获取收藏夹列表"""
    handler = _get_context().handler
    return _run_async(handler.handle_user_collects())


@_safe_call
def get_collects_video_list(collects_id: str, cursor: int = 0, count: int = 20) -> dict:
    """获取收藏夹视频列表（单页）"""
    handler = _get_context().handler
    return _run_async(handler.handle_collects_video_list(collects_id, cursor, count))


@_safe_call
def get_following_list(url: str, offset: int = 0, count: int = 20) -> dict:
    """获取关注列表"""
    handler = _get_context().handler
    return _run_async(handler.handle_user_following(url, offset, count))


@_safe_call
def get_follower_list(url: str, offset: int = 0, count: int = 20) -> dict:
    """获取粉丝列表"""
    handler = _get_context().handler
    return _run_async(handler.handle_user_follower(url, offset, count))


@_safe_call
def get_music_collection(cursor: int = 0, count: int = 18) -> dict:
    """获取音乐收藏"""
    handler = _get_context().handler
    return _run_async(handler.handle_user_music_collection(cursor, count))


@_safe_call
def download_music_batch(url: str) -> dict:
    """下载全部音乐（不写 task DB 表，返回结果供 Rust 处理）

    Returns:
        {"success": True, "music_list": [...], "results": [{music_id, title, author, path, file_size, success, error}, ...]}
    """
    import os
    handler = _get_context().handler

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
    handler = _get_context().handler
    result = _run_async(handler.handle_download_music(play_url, title, author))
    return result


@_safe_call
def get_following_live() -> dict:
    """获取关注直播列表"""
    handler = _get_context().handler
    return _run_async(handler.handle_following_live())


@_safe_call
def get_related(url: str, count: int = 20, filter_gids: str = "") -> dict:
    """获取相关推荐视频（单页，前端控制分页）"""
    handler = _get_context().handler
    return _run_async(handler.handle_related(url, count, filter_gids))


@_safe_call
def get_comments(url: str, cursor: int = 0, count: int = 20) -> dict:
    """获取评论"""
    handler = _get_context().handler
    return _run_async(handler.handle_post_comment(url, cursor, count))


@_safe_call
def get_comment_replies(url: str, comment_id: str, cursor: int = 0, count: int = 3) -> dict:
    """获取评论回复"""
    handler = _get_context().handler
    return _run_async(handler.handle_post_comment_reply(url, comment_id, cursor, count))


@_safe_call
def get_tab_feed(count: int = 10) -> dict:
    """获取推荐 Feed"""
    handler = _get_context().handler
    return _run_async(handler.handle_tab_feed(count))


@_safe_call
def get_follow_feed(cursor: int = 0, count: int = 10) -> dict:
    """获取关注 Feed"""
    handler = _get_context().handler
    return _run_async(handler.handle_follow_feed(cursor, count))


@_safe_call
def get_friend_feed(cursor: int = 0, count: int = 10) -> dict:
    """获取好友 Feed"""
    handler = _get_context().handler
    return _run_async(handler.handle_friend_feed(cursor, count))


@_safe_call
def get_user_likes(url: str, cursor: int = 0, count: int = 20) -> dict:
    """获取用户点赞列表（单页）"""
    handler = _get_context().handler
    return _run_async(handler.handle_user_like_list(url, cursor, count))


@_safe_call
def get_post_stats(url: str) -> dict:
    """获取作品统计"""
    handler = _get_context().handler
    return _run_async(handler.handle_post_stats(url))



@_safe_call
def resolve_single(url: str) -> dict:
    """解析单视频下载计划（typed, mode=one）

    返回 SingleDownloadPlanV1（含 contract_version=1, mode=one）。
    不执行下载；由 Rust DownloadEngine 消费。
    """
    from pathlib import Path
    from core.download.downloader import format_filename

    handler = _get_context().handler
    config = handler.config
    naming = config.naming
    music_enabled = config.music
    cover_enabled = config.cover
    desc_enabled = config.desc
    folderize = config.folderize
    download_path = config.download_path if isinstance(config.download_path, Path) else Path(config.download_path)
    app_name = config.app_name

    base_headers = {
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
        "Referer": "https://www.douyin.com/",
        "Cookie": config.cookie,
    }

    from core.utils import AwemeIdFetcher
    from core.crawler_engine.filter import PostDetailFilter

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

    user_dir = download_path / app_name / "one" / (detail.author_nickname or "unknown")
    save_dir = user_dir / format_filename(naming, detail.to_dict()) if folderize else user_dir

    from core.models.single_download import SingleDownloadPlanV1
    from core.services.media_plan import build_media_items_v1

    items = build_media_items_v1([detail], naming=naming, folderize=False, headers=base_headers)
    if not items:
        return {"success": False, "error": "无法获取视频下载链接"}
    for item in items:
        item.accessories = [
            accessory
            for accessory in item.accessories
            if (
                accessory.kind.value == "music" and music_enabled
                or accessory.kind.value == "cover" and cover_enabled
                or accessory.kind.value == "description" and desc_enabled
            )
        ]
    return SingleDownloadPlanV1(
        save_dir=str(save_dir), items=items, total=len(items)
    ).model_dump(mode="json")


@_safe_call
def resolve_music_urls(url: str) -> dict:
    """解析音乐下载 URL 列表（typed, mode=music-only）

    返回 ResolvedUrls 兼容格式 { items, save_dir, total }。
    仅用于音乐批量下载，不执行下载。
    """
    from pathlib import Path
    from core.crawler_engine.filter import MusicCollectionFilter

    handler = _get_context().handler
    config = handler.config
    download_path = config.download_path if isinstance(config.download_path, Path) else Path(config.download_path)
    app_name = config.app_name
    folderize = config.folderize

    base_headers = {
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
        "Referer": "https://www.douyin.com/",
        "Cookie": config.cookie,
    }

    async def fetch_music_list():
        async with handler._music._make_crawler() as crawler:
            data = await crawler.get_music_collection(0, 1000)
            music_filter = MusicCollectionFilter(data)
            return music_filter.get_music_list()

    music_list = _run_async(fetch_music_list())

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


@_safe_call
def resolve_page(mode: str, url: str, cursor: int = 0, count: int = 20, aweme_ids: list = None) -> dict:
    """解析单页下载 URL（分页模式）

    仅支持 post/like/mix/collects，返回一页 typed media plan；由 Rust
    驱动分页循环并根据配置决定是否下载附属文件。

    Args:
        mode: 分页下载模式 (post/like/mix/collects)
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
    from pathlib import Path

    logger.info("[py_bridge] resolve_page 调用, mode=%s, url=%s, cursor=%d, count=%d",
                mode, url[:80], cursor, count)

    if mode not in ("post", "like", "mix", "collects"):
        return {"success": False, "error": f"不支持的分页模式: {mode}"}

    handler = _get_context().handler
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

    from core.models.paged_download import PagedDownloadPlanV1, PagedUserProfileV1
    from core.services.media_plan import build_media_items_v1

    if mode in ("post", "like", "mix", "collects"):
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
            typed_items = build_media_items_v1(
                [detail], naming=naming, folderize=folderize, headers=base_headers
            )

            return PagedDownloadPlanV1(
                mode=mode,
                save_dir=str(save_dir),
                items=typed_items,
                next_cursor=None,
                has_more=False,
                page_aweme_ids=[target_id],
            ).model_dump(mode="json")

        from core.crawler_engine.filter import UserPostFilter, UserProfileFilter
        from core.utils import SecUserIdFetcher, MixIdFetcher, sanitize_filename

        user_profile = None
        directory_nickname = None
        all_details = []
        next_cursor = None
        has_more = False

        async def _fetch_single_page():
            nonlocal user_profile, directory_nickname, all_details, next_cursor, has_more
            async with handler._user._make_crawler() as crawler:
                if mode in ("post", "like"):
                    sec_user_id = await SecUserIdFetcher.get_sec_user_id(url)
                    if not sec_user_id:
                        return "无法从 URL 提取 sec_user_id"
                    if mode == "post":
                        # 仅首页获取用户资料（对齐 f2 get_or_add_user_data 行为）
                        if cursor == 0:
                            profile_data = await crawler.fetch_user_profile(sec_user_id)
                            profile = UserProfileFilter(profile_data)
                            user_profile = profile.to_dict()
                            directory_nickname = user_profile.get("nickname") or "unknown"
                        data = await crawler.fetch_user_post(sec_user_id, cursor, count)
                    else:
                        # f2 stores liked works under the target user's directory, not
                        # under whichever video author happens to appear first on a page.
                        # Resolve the profile on every stateless page request so save_dir
                        # remains stable; only page one exposes it for persistence.
                        profile_data = await crawler.fetch_user_profile(sec_user_id)
                        profile = UserProfileFilter(profile_data)
                        resolved_profile = profile.to_dict()
                        directory_nickname = resolved_profile.get("nickname") or "unknown"
                        if cursor == 0:
                            user_profile = resolved_profile
                        data = await crawler.fetch_user_favorite(sec_user_id, cursor, count)
                elif mode == "mix":
                    mix_id = await MixIdFetcher.get_mix_id(url)
                    if not mix_id:
                        return "无法从 URL 提取 mix_id"
                    data = await crawler.fetch_mix_aweme(mix_id, cursor, count)
                    # 对齐 f2：从合集第一个作品获取 sec_user_id，然后获取用户资料
                    # 仅首页获取，后续页使用第一页的 directory_nickname
                    if cursor == 0 and data:
                        try:
                            video_filter = UserPostFilter(data)
                            video_list = video_filter.get_video_list()
                            if video_list:
                                first = video_list[0]
                                sec_user_id = getattr(first, "author_sec_uid", None) or ""
                                if sec_user_id:
                                    profile_data = await crawler.fetch_user_profile(sec_user_id)
                                    profile = UserProfileFilter(profile_data)
                                    user_profile = profile.to_dict()
                                    directory_nickname = user_profile.get("nickname") or "unknown"
                        except Exception:
                            pass  # 测试环境可能没有完整的 filter 支持
                elif mode == "collects":
                    collects_id = url
                    # The collection contains works from unrelated authors. Use the
                    # collection identity as the stable directory component instead
                    # of the first author on each page.
                    directory_nickname = sanitize_filename(str(collects_id)) or "unknown"
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

        # 对齐 f2：download_path / app_name / mode / nickname
        nickname = directory_nickname or "unknown"
        if nickname == "unknown":
            if user_profile:
                nickname = user_profile.get("nickname") or "unknown"
            elif all_details:
                first = all_details[0]
                # 确保 fallback 也使用 sanitize_filename，与首页一致
                nickname = sanitize_filename(getattr(first, "author_nickname", None) or "unknown")
        save_dir = download_path / app_name / mode / nickname

        typed_items = build_media_items_v1(
            all_details, naming=naming, folderize=folderize, headers=base_headers
        )
        page_aweme_ids = []
        seen_ids = set()
        for d in all_details:
            aid = str(getattr(d, "aweme_id", "") or "").strip()
            if aid and aid not in seen_ids:
                seen_ids.add(aid)
                page_aweme_ids.append(aid)
        typed_profile = None
        if user_profile:
            typed_profile = PagedUserProfileV1.model_validate(
                {
                    field: user_profile.get(field)
                    for field in PagedUserProfileV1.model_fields
                    if field in user_profile
                }
            )
        return PagedDownloadPlanV1(
            mode=mode,
            save_dir=str(save_dir),
            items=typed_items,
            next_cursor=next_cursor,
            has_more=has_more,
            page_aweme_ids=page_aweme_ids,
            user_profile=typed_profile,
        ).model_dump(mode="json")


@_safe_call
def resolve_download_page(mode: str, url: str, save_dir: str, cursor: int = 0, count: int = 20) -> dict:
    """解析单页下载 URL（下载专用，save_dir 由 Rust 控制）

    与 resolve_page 的区别：
    - resolve_page: 前端展示用，Python 计算 save_dir
    - resolve_download_page: 下载用，Rust 传入 save_dir，保证一致性

    Args:
        mode: 分页下载模式 (post/like/mix/collects)
        url: 目标 URL
        save_dir: 保存目录（由 Rust 首页计算后传入）
        cursor: 分页游标
        count: 每页数量

    Returns:
        {
            "success": True,
            "items": [...],
            "save_dir": save_dir,  # 原样返回
            "next_cursor": ...,
            "has_more": ...,
            "page_aweme_ids": [...],
            "user_profile": {...},  # 仅首页返回
        }
    """
    from pathlib import Path

    logger.info("[py_bridge] resolve_download_page 调用, mode=%s, url=%s, save_dir=%s, cursor=%d",
                mode, url[:80], save_dir, cursor)

    if mode not in ("post", "like", "mix", "collects"):
        return {"success": False, "error": f"不支持的分页模式: {mode}"}

    handler = _get_context().handler
    config = handler.config
    cookie = config.cookie
    naming = config.naming

    base_headers = {
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
        "Referer": "https://www.douyin.com/",
        "Cookie": cookie,
    }

    from core.crawler_engine.filter import UserPostFilter, UserProfileFilter
    from core.utils import SecUserIdFetcher, MixIdFetcher
    from core.models.paged_download import PagedDownloadPlanV1, PagedUserProfileV1
    from core.services.media_plan import build_media_items_v1

    user_profile = None
    all_details = []
    next_cursor = None
    has_more = False

    async def _fetch_page():
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
                    # like 模式：仅首页获取用户资料
                    if cursor == 0:
                        profile_data = await crawler.fetch_user_profile(sec_user_id)
                        profile = UserProfileFilter(profile_data)
                        user_profile = profile.to_dict()
                    data = await crawler.fetch_user_favorite(sec_user_id, cursor, count)
            elif mode == "mix":
                mix_id = await MixIdFetcher.get_mix_id(url)
                if not mix_id:
                    return "无法从 URL 提取 mix_id"
                data = await crawler.fetch_mix_aweme(mix_id, cursor, count)
                # 仅首页获取用户资料
                if cursor == 0 and data:
                    from core.crawler_engine.filter import UserPostFilter as MixFilter
                    mix_filter = MixFilter(data)
                    if mix_filter.aweme_list:
                        first_aweme = mix_filter.aweme_list[0]
                        sec_user_id = first_aweme.get("author", {}).get("sec_uid", "")
                        if sec_user_id:
                            profile_data = await crawler.fetch_user_profile(sec_user_id)
                            profile = UserProfileFilter(profile_data)
                            user_profile = profile.to_dict()
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

    error = _run_async(_fetch_page())
    if error:
        return {"success": False, "error": error}

    # 使用 Rust 传入的 save_dir，不重新计算
    typed_items = build_media_items_v1(
        all_details, naming=naming, folderize=config.folderize, headers=base_headers
    )
    page_aweme_ids = []
    seen_ids = set()
    for d in all_details:
        aid = str(getattr(d, "aweme_id", "") or "").strip()
        if aid and aid not in seen_ids:
            seen_ids.add(aid)
            page_aweme_ids.append(aid)

    typed_profile = None
    if user_profile:
        typed_profile = PagedUserProfileV1.model_validate(
            {
                field: user_profile.get(field)
                for field in PagedUserProfileV1.model_fields
                if field in user_profile
            }
        )

    return PagedDownloadPlanV1(
        mode=mode,
        save_dir=save_dir,  # 原样返回 Rust 传入的 save_dir
        items=typed_items,
        next_cursor=next_cursor,
        has_more=has_more,
        page_aweme_ids=page_aweme_ids,
        user_profile=typed_profile,
    ).model_dump(mode="json")
