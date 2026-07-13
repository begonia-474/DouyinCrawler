"""视频服务 — 单视频解析"""

import logging

from core.crawler_engine.filter import PostDetailFilter
from core.utils import AwemeIdFetcher

from .base import BaseService

logger = logging.getLogger(__name__)


class VideoService(BaseService):
    """单视频解析"""

    async def handle_parse_video(self, url: str) -> dict:
        """只解析视频信息，不下载"""
        logger.info("[handle_parse_video] 开始解析, url=%s", url[:80])
        aweme_id = await AwemeIdFetcher.get_aweme_id(url)
        if not aweme_id:
            logger.warning("[handle_parse_video] 无法提取 aweme_id")
            return {"success": False, "error": "无法从 URL 提取 aweme_id"}

        logger.info("[handle_parse_video] aweme_id=%s, 开始请求API", aweme_id)
        async with self._make_crawler() as crawler:
            data = await crawler.fetch_post_detail(aweme_id)

        logger.info("[handle_parse_video] API 返回 status_code=%s", data.get("status_code", "N/A"))
        if data.get("status_code", -1) != 0:
            logger.warning("[handle_parse_video] API 错误: %s", data.get("status_msg", "unknown"))
            return {"success": False, "error": f"API 错误: {data.get('status_msg', 'unknown')}"}

        detail = PostDetailFilter(data)
        if detail.is_prohibited:
            return {"success": False, "error": "视频侵权不可用"}

        logger.info("[handle_parse_video] 解析成功")
        return {"success": True, "detail": detail.to_db_dict()}
