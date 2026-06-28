"""URL 提取、ID 解析、类型检测"""

import re
import httpx


# ============================================================
# URL 提取与清理
# ============================================================

_URL_PATTERN = re.compile(r"https?://\S+")
_TRAILING_PUNCT = re.compile(r"[,，。.!！?？)）\]】}>」』\s]+$")


def extract_valid_urls(text: str) -> str:
    """从任意文本中提取第一个有效 URL（支持分享口令）。

    用户从抖音 App 复制的分享口令通常格式为：
        "0.53 复制打开抖音 https://v.douyin.com/xxx/ 复制此链接"
    此函数提取其中的 URL 部分，去掉尾部粘连的标点。
    如果输入本身已是纯 URL，原样返回（去掉尾部标点后）。
    """
    match = _URL_PATTERN.search(text)
    if match:
        url = match.group(0)
        url = _TRAILING_PUNCT.sub("", url)
        return url
    return text.strip()


def _is_short_link(url: str) -> bool:
    """判断是否为抖音短链接"""
    return "v.douyin.com" in url or "iesdouyin.com" in url


async def _follow_redirect(url: str, timeout: int = 10) -> str:
    """跟踪重定向，返回最终 URL。网络失败时返回原始 URL。"""
    try:
        async with httpx.AsyncClient(timeout=timeout, follow_redirects=True) as client:
            resp = await client.get(url, headers={
                "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 "
                              "(KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0",
            })
            return str(resp.url)
    except Exception:
        return url


class AwemeIdFetcher:
    """从 URL 提取 aweme_id"""

    @staticmethod
    async def get_aweme_id(url: str) -> str:
        url = extract_valid_urls(url)
        if _is_short_link(url) or "vm.tiktok.com" in url:
            url = await _follow_redirect(url)
        # 路径匹配: video/{id} 或 note/{id}
        match = re.search(r'(?:video|note)/([^/?]+)', url)
        if match:
            return match.group(1)
        # iesdouyin 分享链接: /share/video/{id}/
        match = re.search(r'share/video/([^/?]+)', url)
        if match:
            return match.group(1)
        # URL 参数匹配: modal_id={id}
        match = re.search(r'modal_id=(\d+)', url)
        if match:
            return match.group(1)
        return ""


class SecUserIdFetcher:
    """从 URL 提取 sec_user_id"""

    @staticmethod
    async def get_sec_user_id(url: str) -> str:
        url = extract_valid_urls(url)
        if _is_short_link(url):
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
        url = extract_valid_urls(url)
        if _is_short_link(url):
            url = await _follow_redirect(url)
        match = re.search(r'collection/([^/?]+)', url)
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
        url = extract_valid_urls(url)
        if _is_short_link(url):
            url = await _follow_redirect(url)
        # Web 端直播链接
        match = re.search(r'live\.douyin\.com/(\d+)', url)
        if match:
            return match.group(1)
        # APP 端分享链接 (reflow)
        match = re.search(r'reflow/(\d+)', url)
        if match:
            return match.group(1)
        return ""


def detect_url_type(url: str) -> str:
    """
    自动检测 URL 类型（支持分享口令自动提取 URL）

    Returns: one, post, like, collection, mix, live
    """
    url = extract_valid_urls(url)
    if "live.douyin.com" in url or "webcast.amemv.com" in url:
        return "live"
    if "/video/" in url or "/note/" in url or "iesdouyin.com" in url:
        return "one"
    if "/collection/" in url:
        return "mix"
    if "/user/" in url:
        return "post"  # 默认用户主页
    return "one"
