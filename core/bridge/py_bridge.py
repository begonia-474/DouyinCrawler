"""
Python API 模块
提供模块级别的函数供 PyO3 调用
"""

import asyncio
import json
import uuid
import logging
from typing import Optional

logger = logging.getLogger(__name__)


def _safe_call(func):
    """装饰器：统一捕获异常，返回 {success: False, error: ...}"""
    def wrapper(*args, **kwargs):
        try:
            return func(*args, **kwargs)
        except Exception as e:
            logger.error("[py_bridge] %s 异常: %s", func.__name__, e, exc_info=True)
            return {"success": False, "error": str(e)}
    wrapper.__name__ = func.__name__
    wrapper.__doc__ = func.__doc__
    return wrapper


# 延迟导入，避免循环导入
_task_manager = None

def _get_task_manager():
    """获取 task_manager 单例"""
    global _task_manager
    if _task_manager is None:
        from core.task.task_manager import task_manager
        _task_manager = task_manager

        # 检查 Cookie 是否为空，如果为空则从 config.json 加载
        if not _task_manager._cookie:
            logger.info("[py_bridge] cookie 为空，尝试从 config 加载")
            _task_manager._load_config()
        logger.info("[py_bridge] task_manager 初始化完成, cookie 长度=%d", len(_task_manager._cookie))
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
    return result


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
    """下载音乐"""
    handler = _get_task_manager().handler
    result = _run_async(handler.handle_download_music(play_url, title, author))
    # 下载成功后写入数据库
    if result.get("success"):
        from core.db import save_download_record
        save_download_record(
            download_type="music",
            title=title,
            author_nickname=author,
            file_path=result.get("path"),
            status="completed",
        )
    return result


@_safe_call
def get_following_live() -> dict:
    """获取关注直播列表"""
    handler = _get_task_manager().handler
    return _run_async(handler.handle_following_live())


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


