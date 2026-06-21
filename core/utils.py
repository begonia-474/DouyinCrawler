"""工具类 — URL ID 提取、文件名格式化"""

import re
import time
import httpx


async def _follow_redirect(url: str, timeout: int = 10) -> str:
    """跟踪重定向，返回最终 URL"""
    async with httpx.AsyncClient(timeout=timeout, follow_redirects=True) as client:
        resp = await client.get(url, headers={
            "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        })
        return str(resp.url)


class AwemeIdFetcher:
    """从 URL 提取 aweme_id"""

    @staticmethod
    async def get_aweme_id(url: str) -> str:
        if "v.douyin.com" in url or "vm.tiktok.com" in url:
            url = await _follow_redirect(url)
        match = re.search(r'(?:video|note)/(\d+)', url)
        if match:
            return match.group(1)
        # 尝试从 URL 参数提取
        match = re.search(r'modal_id=(\d+)', url)
        if match:
            return match.group(1)
        return ""


class SecUserIdFetcher:
    """从 URL 提取 sec_user_id"""

    @staticmethod
    async def get_sec_user_id(url: str) -> str:
        if "v.douyin.com" in url:
            url = await _follow_redirect(url)
        match = re.search(r'user/([^/?]+)', url)
        if match:
            return match.group(1)
        # 从参数提取
        match = re.search(r'sec_user_id=([^&]+)', url)
        if match:
            return match.group(1)
        return ""


class MixIdFetcher:
    """从 URL 提取 mix_id"""

    @staticmethod
    async def get_mix_id(url: str) -> str:
        if "v.douyin.com" in url:
            url = await _follow_redirect(url)
        match = re.search(r'collection/(\d+)', url)
        if match:
            return match.group(1)
        match = re.search(r'mix_id=(\d+)', url)
        if match:
            return match.group(1)
        return ""


class WebCastIdFetcher:
    """从 URL 提取直播 ID"""

    @staticmethod
    async def get_webcast_id(url: str) -> str:
        if "v.douyin.com" in url:
            url = await _follow_redirect(url)
        match = re.search(r'live\.douyin\.com/(\d+)', url)
        if match:
            return match.group(1)
        return ""

    @staticmethod
    async def get_room_id(url: str) -> str:
        match = re.search(r'reflow/(\d+)', url)
        if match:
            return match.group(1)
        return ""


def detect_url_type(url: str) -> str:
    """
    自动检测 URL 类型

    Returns: one, post, like, collection, mix, live
    """
    if "live.douyin.com" in url or "webcast.amemv.com" in url:
        return "live"
    if "/video/" in url or "/note/" in url:
        return "one"
    if "/collection/" in url:
        return "mix"
    if "/user/" in url:
        return "post"  # 默认用户主页
    return "one"


def sanitize_filename(name: str, max_len: int = 80) -> str:
    """清理文件名"""
    name = re.sub(r'[\\/:*?"<>|\n\r\t]', '_', name)
    name = name.strip('. ')
    if len(name) > max_len:
        name = name[:max_len]
    return name or "untitled"


def format_filename(template: str, data: dict) -> str:
    """
    格式化文件名模板

    支持变量: {create}, {desc}, {nickname}, {aweme_id}, {uid}
    """
    create_ts = data.get("create_time", 0)
    create_str = time.strftime("%Y-%m-%d_%H%M%S", time.localtime(create_ts)) if create_ts else "unknown"

    result = template.format(
        create=create_str,
        desc=sanitize_filename(data.get("desc", "")),
        nickname=sanitize_filename(data.get("author", "")),
        aweme_id=data.get("aweme_id", ""),
        uid=data.get("author_uid", ""),
    )
    return sanitize_filename(result)
