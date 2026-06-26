"""视频服务 — 单视频解析和下载"""

import logging

from core.downloader import format_filename
from core.filter import PostDetailFilter
from core.utils import AwemeIdFetcher

from core.services.base import BaseService, run_concurrent

logger = logging.getLogger(__name__)


class VideoService(BaseService):
    """单视频解析和下载"""

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

    async def handle_one_video(self, url: str, progress_callback=None) -> dict:
        """下载单个视频"""
        aweme_id = await AwemeIdFetcher.get_aweme_id(url)
        if not aweme_id:
            return {"success": False, "error": "无法从 URL 提取 aweme_id"}

        async with self._make_crawler() as crawler:
            data = await crawler.fetch_post_detail(aweme_id)

        if data.get("status_code", -1) != 0:
            return {"success": False, "error": f"API 错误: {data.get('status_msg', 'unknown')}"}

        detail = PostDetailFilter(data)
        if detail.is_prohibited:
            return {"success": False, "error": "视频侵权不可用"}

        filename = format_filename(self.naming, detail.to_dict())
        user_dir = self.download_path / self.app_name / "one" / detail.author_nickname
        save_dir = user_dir / filename if self.folderize else user_dir
        save_dir.mkdir(parents=True, exist_ok=True)

        async with self._make_downloader(progress_callback) as dl:
            # 附属文件并发下载
            accessory = []
            if self.music and detail.music_url:
                accessory.append(dl.download_music(detail.music_url, save_dir, filename))
            if self.cover and detail.cover_url:
                accessory.append(dl.download_cover(detail.cover_url, save_dir, filename))
            if self.desc and detail.desc:
                accessory.append(dl.download_desc(detail.desc, save_dir, filename))
            if accessory:
                await run_concurrent(accessory)

            if detail.is_image_post and (detail.images or detail.images_video):
                paths = []
                for i, live_url in enumerate(detail.images_video):
                    if live_url:
                        path = await dl.download_live_image(live_url, save_dir, f"{filename}_live_{i + 1}")
                        paths.append(str(path))
                for i, img_url in enumerate(detail.images):
                    if img_url:
                        path = await dl.download_image(img_url, save_dir, f"{filename}_image_{i + 1}")
                        paths.append(str(path))
                return {"success": True, "type": "images", "paths": paths, "detail": detail.to_db_dict()}
            else:
                if not detail.video_url:
                    return {"success": False, "error": "无法获取视频下载链接"}
                path = await dl.download_video(detail.video_url, save_dir, f"{filename}_video")
                return {"success": True, "type": "video", "path": str(path), "detail": detail.to_db_dict()}
