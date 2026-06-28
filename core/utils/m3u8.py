"""M3U8 直播流工具 — 分段解析、内容长度、块大小"""

import httpx
import m3u8


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
