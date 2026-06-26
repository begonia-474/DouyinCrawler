"""工具类 — URL ID 提取、文件名格式化、M3U8 解析、文本清洗"""

import re
import time
import datetime
import httpx
import m3u8


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


# ============================================================
# 文本清洗与时间格式化（移植自 f2）
# ============================================================

_REPLACE_T_RE = re.compile(r"[^一-龥a-zA-Z0-9#]")


def replaceT(obj):
    """替换文案非法字符，保留中文、英文、数字、#。

    用于文件名安全和前端展示。支持 str 和 list 输入。
    """
    if isinstance(obj, list):
        return [re.sub(_REPLACE_T_RE, "_", i) if isinstance(i, str) else i or "" for i in obj]
    if isinstance(obj, str):
        return re.sub(_REPLACE_T_RE, "_", obj)
    return obj


def timestamp_2_str(timestamp, format: str = "%Y-%m-%d %H-%M-%S") -> str:
    """将 UNIX 时间戳转换为东八区北京时间格式化字符串。

    Args:
        timestamp: 秒或毫秒级 UNIX 时间戳（str/int/float）
        format: 日期格式

    Returns:
        格式化字符串，无效输入返回空字符串
    """
    if timestamp in (None, "None", "", 0, "0"):
        return ""
    try:
        ts = float(timestamp)
        if ts > 1e10:
            ts /= 1000
        tz = datetime.timezone(datetime.timedelta(hours=8))
        return datetime.datetime.fromtimestamp(ts, tz=tz).strftime(format)
    except (ValueError, TypeError, OSError):
        return ""


def sanitize_filename(name: str, max_len: int = 200) -> str:
    """清理文件名，移除非法字符和 emoji，按字节截断"""
    name = re.sub(r'[\\/:*?"<>|\n\r\t]', '_', name)
    # 过滤 emoji（SMP 平面 U+10000 以上）
    name = re.sub(r'[\U00010000-\U0010ffff]+', '', name)
    name = name.strip('. ')
    # 按字节截断，避免中文截断产生乱码
    encoded = name.encode('utf-8')
    if len(encoded) > max_len:
        name = encoded[:max_len].decode('utf-8', errors='ignore')
    return name


def format_filename(template: str, data: dict) -> str:
    """
    格式化文件名模板

    支持变量: {create}, {desc}, {caption}, {nickname}, {aweme_id}, {uid}
    """
    create_ts = data.get("create_time", 0)
    if isinstance(create_ts, str) and "-" in create_ts:
        # 已经是格式化字符串（如 "2024-06-25 16-00-00"），直接用
        create_str = create_ts.replace(" ", "_").replace(":", "").replace("-", "-", 2)
    else:
        create_str = time.strftime("%Y-%m-%d_%H%M%S", time.localtime(create_ts)) if create_ts else "unknown"

    # desc 和 caption 都指向 desc 字段（f2 兼容）
    # strip 尾部下划线，避免空 desc 时模板展开产生多余连接符
    desc = sanitize_filename(data.get("desc", "")).strip('_ ')

    result = template.format(
        create=create_str,
        desc=desc,
        caption=desc,  # caption 与 desc 相同，f2 兼容
        nickname=sanitize_filename(data.get("author", "")),
        aweme_id=data.get("aweme_id", ""),
        uid=data.get("author_uid", ""),
    )
    return sanitize_filename(result).rstrip('_')


# ============================================================
# 日期区间过滤
# ============================================================

def interval_2_timestamp(interval: str, date_type: str = "start") -> int:
    """
    将日期区间转换为时间戳（毫秒）

    Args:
        interval: 日期区间，格式 "YYYY-MM-DD|YYYY-MM-DD"
        date_type: "start" 或 "end"

    Returns:
        毫秒级时间戳
    """
    try:
        parts = interval.split("|")
        if len(parts) != 2:
            return 0

        date_str = parts[0] if date_type == "start" else parts[1]
        # 解析日期
        dt = time.strptime(date_str, "%Y-%m-%d")
        ts = int(time.mktime(dt))

        # end 日期需要加一天减一秒，包含全天
        if date_type == "end":
            ts += 86400 - 1

        return ts * 1000  # 转换为毫秒
    except Exception:
        return 0


def filter_by_date_interval(aweme_list: list, interval: str, field: str = "create_time") -> list:
    """
    按日期区间过滤作品列表

    Args:
        aweme_list: 作品列表
        interval: 日期区间，格式 "YYYY-MM-DD|YYYY-MM-DD" 或 "all"
        field: 日期字段名

    Returns:
        过滤后的作品列表
    """
    if not interval or interval == "all":
        return aweme_list

    start_ts = interval_2_timestamp(interval, "start")
    end_ts = interval_2_timestamp(interval, "end")

    if start_ts == 0 or end_ts == 0:
        return aweme_list

    filtered = []
    for item in aweme_list:
        # 获取创建时间（兼容 dict 和对象）
        if isinstance(item, dict):
            create_time = item.get(field, 0)
        else:
            create_time = getattr(item, field, 0)
        if isinstance(create_time, str):
            try:
                create_time = int(time.mktime(time.strptime(create_time, "%Y-%m-%d %H:%M:%S")))
            except Exception:
                continue

        # 转换为毫秒
        create_time_ms = create_time * 1000 if create_time < 1e12 else create_time

        if start_ts <= create_time_ms <= end_ts:
            filtered.append(item)

    return filtered


# ============================================================
# M3U8 直播流工具
# ============================================================

async def get_segments_from_m3u8(url: str):
    """从 m3u8 URL 获取 TS 分片列表，支持嵌套 m3u8"""
    try:
        m3u8_obj = m3u8.load(url)
    except Exception:
        return []
    segments = m3u8_obj.segments
    if not segments and m3u8_obj.playlists:
        nested_url = m3u8_obj.playlists[0].absolute_uri
        return await get_segments_from_m3u8(nested_url)
    return segments


async def get_content_length(
    url: str, headers: dict, proxies: dict = None, client: httpx.AsyncClient = None
) -> int:
    """获取远程文件大小（字节），HEAD 失败降级为 GET。可复用已有 client。"""

    async def _fetch(c: httpx.AsyncClient) -> int:
        try:
            resp = await c.head(url, headers=headers)
            resp.raise_for_status()
            return int(resp.headers.get("Content-Length", 0))
        except Exception:
            try:
                req = c.build_request("GET", url, headers=headers)
                resp = await c.send(req, stream=True)
                resp.raise_for_status()
                return int(resp.headers.get("Content-Length", 0))
            except Exception:
                return 0

    if client is not None:
        return await _fetch(client)

    proxy = proxies.get("https://") or proxies.get("http://") if proxies else None
    kwargs: dict = {"timeout": 10, "follow_redirects": True}
    if proxy:
        kwargs["proxy"] = proxy
    async with httpx.AsyncClient(**kwargs) as c:
        return await _fetch(c)


def get_chunk_size(file_size: int) -> int:
    """根据文件大小自适应下载块大小"""
    if file_size < 10 * 1024:
        return file_size
    elif file_size < 1 * 1024 * 1024:
        return file_size // 10
    elif file_size < 10 * 1024 * 1024:
        return file_size // 20
    elif file_size < 100 * 1024 * 1024:
        return file_size // 50
    else:
        return 1 * 1024 * 1024
