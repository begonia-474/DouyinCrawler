"""FastAPI 服务入口"""

import time
import uuid
from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware

from backend.logger import setup_logging, get_logger
from backend.schemas import (
    ApiResponse, UrlRequest, KeywordRequest, CommentRequest,
    CommentReplyRequest, FeedRequest, MusicRequest, UserListRequest,
    CollectsVideoRequest, ConfigRequest,
)
from backend.task_manager import task_manager
from core.utils import detect_url_type

setup_logging()
logger = get_logger(__name__)

app = FastAPI(title="DouyinCrawler API")

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.middleware("http")
async def log_requests(request: Request, call_next):
    req_id = uuid.uuid4().hex[:8]
    path = request.url.path
    method = request.method
    logger.info("[%s] %s %s -> 开始", req_id, method, path)
    start = time.time()
    response = await call_next(request)
    elapsed = (time.time() - start) * 1000
    logger.info("[%s] %s %s -> %d (%.1fms)", req_id, method, path, response.status_code, elapsed)
    return response


async def safe_call(coro, label: str):
    try:
        result = await coro
        if result.get("success"):
            logger.info("%s 成功", label)
            return ApiResponse(success=True, data=result)
        logger.warning("%s 失败: %s", label, result.get("error"))
        return ApiResponse(success=False, error=result.get("error", "未知错误"))
    except Exception as e:
        logger.exception("%s 异常", label)
        return ApiResponse(success=False, error=str(e))


# === 健康 & 配置 ===

@app.get("/api/health")
async def health():
    return ApiResponse(success=True, data={"configured": task_manager.is_configured})


@app.post("/api/config")
async def update_config(req: ConfigRequest):
    logger.info("配置更新: cookie=%s...", (req.cookie or "")[:30])
    task_manager.update_config(
        cookie=req.cookie, download_path=req.download_path,
        naming=req.naming, encryption=req.encryption, proxy=req.proxy,
    )
    return ApiResponse(success=True, data=task_manager.config_summary)


# === 视频解析 & 下载 ===

@app.post("/api/post/detail")
async def post_detail(req: UrlRequest):
    url_type = detect_url_type(req.url)
    logger.info("解析: url=%s, 类型=%s", req.url, url_type)
    handler = task_manager.handler
    if url_type == "one":
        return await safe_call(handler.handle_one_video(req.url), "单视频解析")
    elif url_type == "post":
        return await safe_call(handler.handle_user_post(req.url), "用户主页解析")
    elif url_type == "mix":
        return await safe_call(handler.handle_user_mix(req.url), "合集解析")
    elif url_type == "live":
        return await safe_call(handler.handle_user_live(req.url), "直播解析")
    return await safe_call(handler.handle_one_video(req.url), "默认解析")


@app.post("/api/post/stats")
async def post_stats(req: UrlRequest):
    return await safe_call(task_manager.handler.handle_post_stats(req.url), "作品统计")


@app.post("/api/download/one")
async def download_one(req: UrlRequest):
    return await safe_call(task_manager.handler.handle_one_video(req.url), "单视频下载")


# === 评论 ===

@app.post("/api/comments")
async def get_comments(req: CommentRequest):
    return await safe_call(
        task_manager.handler.handle_post_comment(req.url, req.cursor, req.count),
        "评论获取",
    )


@app.post("/api/comments/reply")
async def get_comment_replies(req: CommentReplyRequest):
    return await safe_call(
        task_manager.handler.handle_post_comment_reply(req.url, req.comment_id, req.cursor, req.count),
        "评论回复",
    )


# === 搜索 ===

@app.post("/api/search")
async def search(req: KeywordRequest):
    return await safe_call(
        task_manager.handler.handle_search(req.keyword, req.offset, req.count),
        "搜索",
    )


# === Feed ===

@app.post("/api/feed/tab")
async def feed_tab(req: FeedRequest):
    return await safe_call(
        task_manager.handler.handle_tab_feed(req.count),
        "推荐Feed",
    )


@app.post("/api/feed/follow")
async def feed_follow(req: FeedRequest):
    return await safe_call(
        task_manager.handler.handle_follow_feed(req.cursor, req.count),
        "关注Feed",
    )


@app.post("/api/feed/friend")
async def feed_friend(req: FeedRequest):
    return await safe_call(
        task_manager.handler.handle_friend_feed(req.cursor, req.count),
        "朋友Feed",
    )


# === 音乐 ===

@app.post("/api/music/collection")
async def music_collection(req: MusicRequest):
    return await safe_call(
        task_manager.handler.handle_user_music_collection(req.cursor, req.count),
        "音乐收藏",
    )


# === 用户 ===

@app.post("/api/user/profile")
async def user_profile(req: UrlRequest):
    return await safe_call(
        task_manager.handler.handle_user_profile(req.url),
        "用户资料",
    )


@app.post("/api/user/posts")
async def user_posts(req: UrlRequest):
    return await safe_call(
        task_manager.handler.handle_user_post(req.url),
        "用户作品",
    )


@app.post("/api/user/likes")
async def user_likes(req: UrlRequest):
    return await safe_call(
        task_manager.handler.handle_user_like(req.url),
        "用户点赞",
    )


@app.post("/api/user/collection")
async def user_collection(req: UrlRequest):
    return await safe_call(
        task_manager.handler.handle_user_collection(),
        "用户收藏",
    )


@app.post("/api/user/collects")
async def user_collects(req: UrlRequest):
    return await safe_call(
        task_manager.handler.handle_user_collects(),
        "收藏夹列表",
    )


@app.post("/api/user/collects/video")
async def user_collects_video(req: CollectsVideoRequest):
    return await safe_call(
        task_manager.handler.handle_collects_video(req.collects_id),
        "收藏夹视频",
    )


@app.post("/api/user/following")
async def user_following(req: UserListRequest):
    return await safe_call(
        task_manager.handler.handle_user_following(req.url, req.offset, req.count),
        "关注列表",
    )


@app.post("/api/user/followers")
async def user_followers(req: UserListRequest):
    return await safe_call(
        task_manager.handler.handle_user_follower(req.url, req.offset, req.count),
        "粉丝列表",
    )


# === 直播 ===

@app.post("/api/live")
async def live_info(req: UrlRequest):
    return await safe_call(
        task_manager.handler.handle_user_live(req.url),
        "直播信息",
    )


# === 合集 ===

@app.post("/api/mix")
async def mix_info(req: UrlRequest):
    return await safe_call(
        task_manager.handler.handle_user_mix(req.url),
        "合集信息",
    )


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(
        "backend.server:app",
        host="127.0.0.1",
        port=8765,
        reload=True,
        reload_excludes=["backend/logs/*", "backend/config.json"],
    )
