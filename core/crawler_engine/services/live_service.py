"""直播服务 — 直播信息、录制、关注直播列表"""

import time
import logging

from core.crawler_engine.filter import UserLive2Filter, UserLiveFilter, FollowingUserLiveFilter
from core.utils import WebCastIdFetcher, sanitize_filename

from .base import BaseService

logger = logging.getLogger(__name__)


class LiveService(BaseService):
    """直播相关业务"""

    async def handle_user_live(self, url: str, progress_callback=None) -> dict:
        """获取直播信息"""
        webcast_id = await WebCastIdFetcher.get_webcast_id(url)
        if not webcast_id:
            return {"success": False, "error": "无法从 URL 提取直播 ID"}

        async with self._make_crawler() as crawler:
            if len(webcast_id) == 19:
                data = await crawler.fetch_live_info_by_room_id(webcast_id)
                live_filter = UserLive2Filter(data)
            else:
                data = await crawler.fetch_live_info(web_rid=webcast_id)
                live_filter = UserLiveFilter(data)
        if not live_filter.is_live:
            return {"success": False, "error": f"未在直播中 (status={live_filter.live_status})"}

        flv_dict = live_filter.flv_pull_url or {}
        m3u8_dict = live_filter.m3u8_pull_url or {}

        return {
            "success": True,
            "title": live_filter.live_title,
            "nickname": live_filter.nickname,
            "is_live": live_filter.is_live,
            "user_count": live_filter.user_count,
            "room_id": live_filter.room_id,
            "cover": live_filter.cover_url,
            "flv_urls": list(flv_dict.values()),
            "m3u8_urls": list(m3u8_dict.values()),
            "web_rid": getattr(live_filter, "web_rid", "") or webcast_id,
            "cover_url": live_filter.cover_url,
            "user_id": live_filter.user_id,
            "sec_user_id": live_filter.sec_user_id,
            "flv_pull_url": flv_dict,
            "m3u8_pull_url": m3u8_dict,
        }

    async def handle_live_record(self, url: str, task_id: str, progress_callback=None, stop_event=None) -> dict:
        """录制直播流"""
        webcast_id = await WebCastIdFetcher.get_webcast_id(url)
        if not webcast_id:
            return {"success": False, "error": "无法从 URL 提取直播 ID"}

        async with self._make_crawler() as crawler:
            data = await crawler.fetch_live_info(web_rid=webcast_id)

        live_filter = UserLiveFilter(data)
        if not live_filter.is_live:
            return {"success": False, "error": f"未在直播中 (status={live_filter.live_status})"}

        m3u8_urls = live_filter.m3u8_pull_url
        if not m3u8_urls:
            return {"success": False, "error": "未获取到 m3u8 拉流地址"}

        m3u8_url = m3u8_urls.get("FULL_HD1") or m3u8_urls.get("HD1") or next(iter(m3u8_urls.values()))
        if not m3u8_url:
            return {"success": False, "error": "拉流地址为空"}

        nickname = sanitize_filename(live_filter.nickname or "unknown")
        save_dir = self.download_path / self.app_name / "live" / nickname
        create_str = time.strftime("%Y-%m-%d_%H-%M-%S")
        title = sanitize_filename(live_filter.live_title or "live")
        filename = f"{create_str}_{title}_live.flv"
        full_path = save_dir / filename

        started_at = int(time.time())
        async with self._make_downloader(progress_callback) as dl:
            if stop_event:
                dl._stop_event = stop_event
            await dl.download_m3u8_stream(task_id, m3u8_url, full_path)
        ended_at = int(time.time())

        file_size = full_path.stat().st_size if full_path.exists() else 0

        return {
            "success": True,
            "file": str(full_path),
            "room_id": live_filter.room_id,
            "web_rid": webcast_id,
            "title": live_filter.live_title,
            "nickname": live_filter.nickname,
            "file_size": file_size,
            "duration_sec": ended_at - started_at,
            "started_at": started_at,
            "ended_at": ended_at,
            "cover_url": live_filter.cover_url,
        }

    async def handle_following_live(self) -> dict:
        """获取关注用户中正在直播的列表"""
        try:
            async with self._make_crawler() as crawler:
                data = await crawler.fetch_following_user_live()
        except Exception as e:
            return {"success": False, "error": f"网络请求失败: {e}"}

        if data.get("status_code", -1) != 0:
            return {"success": False, "error": f"API 错误: {data.get('data', {}).get('message', 'unknown')}"}

        live_filter = FollowingUserLiveFilter(data)
        rooms = live_filter.live_rooms

        live_list = []
        for item in rooms:
            room = item.get("room", {})
            owner = room.get("owner", {})
            cover = room.get("cover", {})

            live_list.append({
                "web_rid": item.get("web_rid", ""),
                "room_id": room.get("id_str", ""),
                "title": room.get("title", ""),
                "nickname": owner.get("nickname", ""),
                "avatar": (owner.get("avatar_thumb", {}).get("url_list", [""]))[0],
                "cover": (cover.get("url_list", [""]))[0],
                "user_count": room.get("user_count", 0),
                "tag_name": item.get("tag_name", ""),
            })

        return {"success": True, "count": len(live_list), "lives": live_list}
