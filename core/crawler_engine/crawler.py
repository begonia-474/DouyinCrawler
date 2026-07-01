"""HTTP 爬虫引擎

能力分级：
- 未标注的方法均为 [active]（被 service 层调用）
- [reserved] 标注的方法已实现但暂无 service 调用方，属于预留核心能力
"""

import httpx
import asyncio
import random
import logging
from urllib.parse import urlencode

from core.crawler_engine.api import DouyinAPIEndpoints as ep
from core.crawler_engine.signature.abogus import ABogus
from core.crawler_engine.signature.xbogus import XBogus
from core.crawler_engine.signature.fingerprint import BrowserFingerprintGenerator

logger = logging.getLogger(__name__)


class DouyinCrawler:
    """抖音异步 HTTP 爬虫，自动注入签名、重试、限流检测"""

    UA = (
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
        "AppleWebKit/537.36 (KHTML, like Gecko) "
        "Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0"
    )

    def __init__(self, cookie: str, proxies: dict | None = None,
                 encryption: str = "ab", max_retries: int = 5,
                 max_connections: int = 5, timeout: int = 10):
        self.cookie = cookie
        self.encryption = encryption
        self.max_retries = max_retries
        client_kwargs = {
            "timeout": timeout,
            "limits": httpx.Limits(max_connections=max_connections, max_keepalive_connections=max_connections),
            "headers": {"User-Agent": self.UA, "Referer": "https://www.douyin.com/"},
            "follow_redirects": True,
            "transport": httpx.AsyncHTTPTransport(retries=max_retries),
        }
        if proxies:
            client_kwargs["proxy"] = proxies.get("https://") or proxies.get("http://")
        self._client = httpx.AsyncClient(**client_kwargs)

    async def close(self):
        await self._client.aclose()

    async def __aenter__(self):
        return self

    async def __aexit__(self, *args):
        await self.close()

    def _sign_url(self, base_url: str, params: dict, body: str = "") -> str:
        param_str = urlencode(params)
        if self.encryption == "ab":
            fp = BrowserFingerprintGenerator.generate_fingerprint("Edge")
            ab = ABogus(fp=fp, user_agent=self.UA)
            signed, _, _, _ = ab.generate_abogus(param_str, body)
        else:
            xb = XBogus(self.UA)
            signed, _, _ = xb.generate(param_str)
        sep = "&" if "?" in base_url else "?"
        return f"{base_url}{sep}{signed}"

    async def _get_json(self, url: str) -> dict:
        """GET 请求，带重试和 429 限流检测"""
        last_error = None
        for attempt in range(self.max_retries):
            try:
                if attempt == 0:
                    logger.info("[_get_json] 请求URL: %s", url[:120])
                resp = await self._client.get(url, headers={"Cookie": self.cookie})

                # 429 限流：等待后重试
                if resp.status_code == 429:
                    wait = (attempt + 1) * 5 + random.uniform(1, 3)
                    logger.warning("触发限流 (429)，等待 %.1f 秒后重试 (%d/%d)", wait, attempt + 1, self.max_retries)
                    await asyncio.sleep(wait)
                    continue

                resp.raise_for_status()

                if not resp.content:
                    logger.warning("[_get_json] 空响应, status=%d", resp.status_code)
                    return {"status_code": -1, "status_msg": "empty response"}

                try:
                    data = resp.json()
                    api_status = data.get("status_code", "N/A")
                    api_msg = data.get("status_msg", "N/A")
                    logger.info("[_get_json] API 响应: status_code=%s, status_msg=%s", api_status, api_msg)
                    return data
                except Exception:
                    logger.warning("[_get_json] JSON 解析失败, 响应前200字符: %s", resp.text[:200])
                    return {"status_code": -1, "status_msg": "invalid json response"}

            except (httpx.RemoteProtocolError, httpx.ReadTimeout, httpx.ConnectTimeout) as e:
                last_error = e
                wait = (attempt + 1) * 2 + random.uniform(0, 1)
                logger.warning("请求异常 (%s)，等待 %.1f 秒后重试 (%d/%d)", type(e).__name__, wait, attempt + 1, self.max_retries)
                await asyncio.sleep(wait)
            except httpx.HTTPStatusError as e:
                if e.response.status_code in (500, 502, 503):
                    last_error = e
                    wait = (attempt + 1) * 3 + random.uniform(0, 2)
                    logger.warning("服务端错误 (%d)，等待 %.1f 秒后重试 (%d/%d)", e.response.status_code, wait, attempt + 1, self.max_retries)
                    await asyncio.sleep(wait)
                    continue
                raise

        logger.error("请求失败，已重试 %d 次: %s", self.max_retries, last_error)
        return {"status_code": -1, "status_msg": f"request failed after {self.max_retries} retries: {last_error}"}

    async def _post_json(self, url: str, json_data: dict = None, form_data: dict = None) -> dict:
        """POST 请求，带重试和 429 限流检测"""
        last_error = None
        for attempt in range(self.max_retries):
            try:
                resp = await self._client.post(
                    url, headers={"Cookie": self.cookie},
                    json=json_data, data=form_data,
                )

                # 429 限流：等待后重试
                if resp.status_code == 429:
                    wait = (attempt + 1) * 5 + random.uniform(1, 3)
                    logger.warning("触发限流 (429)，等待 %.1f 秒后重试 (%d/%d)", wait, attempt + 1, self.max_retries)
                    await asyncio.sleep(wait)
                    continue

                resp.raise_for_status()

                if not resp.content:
                    return {"status_code": -1, "status_msg": "empty response"}

                try:
                    return resp.json()
                except Exception:
                    return {"status_code": -1, "status_msg": "invalid json"}

            except (httpx.RemoteProtocolError, httpx.ReadTimeout, httpx.ConnectTimeout) as e:
                last_error = e
                wait = (attempt + 1) * 2 + random.uniform(0, 1)
                logger.warning("请求异常 (%s)，等待 %.1f 秒后重试 (%d/%d)", type(e).__name__, wait, attempt + 1, self.max_retries)
                await asyncio.sleep(wait)
            except httpx.HTTPStatusError as e:
                if e.response.status_code in (500, 502, 503):
                    last_error = e
                    wait = (attempt + 1) * 3 + random.uniform(0, 2)
                    logger.warning("服务端错误 (%d)，等待 %.1f 秒后重试 (%d/%d)", e.response.status_code, wait, attempt + 1, self.max_retries)
                    await asyncio.sleep(wait)
                    continue
                raise

        logger.error("请求失败，已重试 %d 次: %s", self.max_retries, last_error)
        return {"status_code": -1, "status_msg": f"request failed after {self.max_retries} retries: {last_error}"}

    def _get_token(self) -> str:
        from core.crawler_engine.tokens.token_manager import TokenManager
        return TokenManager.gen_false_ms_token()

    # ============================================================
    # 用户相关
    # ============================================================

    async def fetch_user_profile(self, sec_user_id: str) -> dict:
        from core.models import UserProfile
        params = UserProfile(sec_user_id=sec_user_id, msToken=self._get_token()).model_dump()
        return await self._get_json(self._sign_url(ep.USER_DETAIL, params))

    async def fetch_user_post(self, sec_user_id: str, max_cursor: int = 0, count: int = 18) -> dict:
        from core.models import UserPost
        params = UserPost(sec_user_id=sec_user_id, max_cursor=max_cursor, count=count, msToken=self._get_token()).model_dump()
        return await self._get_json(self._sign_url(ep.USER_POST, params))

    async def fetch_user_favorite(self, sec_user_id: str, max_cursor: int = 0, count: int = 18) -> dict:
        from core.models import UserFavorite
        params = UserFavorite(sec_user_id=sec_user_id, max_cursor=max_cursor, count=count, msToken=self._get_token()).model_dump()
        return await self._get_json(self._sign_url(ep.USER_FAVORITE, params))

    async def fetch_user_collection(self, cursor: int = 0, count: int = 18) -> dict:
        from core.models import UserCollection
        params = UserCollection(cursor=cursor, count=count, msToken=self._get_token()).model_dump()
        return await self._post_json(self._sign_url(ep.USER_COLLECTION, params), json_data=params)

    async def fetch_user_collects(self, cursor: int = 0, count: int = 18) -> dict:
        from core.models import UserCollects
        params = UserCollects(cursor=cursor, count=count, msToken=self._get_token()).model_dump()
        return await self._get_json(self._sign_url(ep.USER_COLLECTS, params))

    async def fetch_user_collects_video(self, collects_id: str, cursor: int = 0, count: int = 18) -> dict:
        from core.models import UserCollectsVideo
        params = UserCollectsVideo(collects_id=collects_id, cursor=cursor, count=count, msToken=self._get_token()).model_dump()
        return await self._get_json(self._sign_url(ep.USER_COLLECTS_VIDEO, params))

    async def fetch_user_music_collection(self, cursor: int = 0, count: int = 18) -> dict:
        from core.models import UserMusicCollection
        params = UserMusicCollection(cursor=cursor, count=count, msToken=self._get_token()).model_dump()
        return await self._get_json(self._sign_url(ep.USER_MUSIC_COLLECTION, params))

    async def fetch_user_following(self, sec_user_id: str, offset: int = 0, count: int = 20) -> dict:
        from core.models import UserFollowing
        params = UserFollowing(sec_user_id=sec_user_id, offset=offset, count=count, msToken=self._get_token()).model_dump()
        return await self._post_json(self._sign_url(ep.USER_FOLLOWING, params), json_data=params)

    async def fetch_user_follower(self, sec_user_id: str, offset: int = 0, count: int = 20) -> dict:
        from core.models import UserFollower
        params = UserFollower(sec_user_id=sec_user_id, offset=offset, count=count, msToken=self._get_token()).model_dump()
        return await self._post_json(self._sign_url(ep.USER_FOLLOWER, params), json_data=params)

    async def fetch_query_user(self) -> dict:  # [reserved] 无 service/py_bridge 调用
        from core.models import QueryUser
        params = QueryUser(msToken=self._get_token()).model_dump()
        return await self._get_json(self._sign_url(ep.QUERY_USER, params))

    # ============================================================
    # 视频相关
    # ============================================================

    async def fetch_post_detail(self, aweme_id: str) -> dict:
        from core.models import PostDetail
        params = PostDetail(aweme_id=aweme_id, msToken=self._get_token()).model_dump()
        return await self._get_json(self._sign_url(ep.POST_DETAIL, params))

    async def fetch_post_related(self, aweme_id: str, count: int = 20) -> dict:
        from core.models import PostRelated
        params = PostRelated(aweme_id=aweme_id, count=count, msToken=self._get_token()).model_dump()
        return await self._get_json(self._sign_url(ep.POST_RELATED, params))

    async def fetch_post_comment(self, aweme_id: str, cursor: int = 0, count: int = 20) -> dict:
        from core.models import PostComment
        params = PostComment(aweme_id=aweme_id, cursor=cursor, count=count, msToken=self._get_token()).model_dump()
        return await self._get_json(self._sign_url(ep.POST_COMMENT, params))

    async def fetch_post_comment_reply(self, item_id: str, comment_id: str, cursor: int = 0, count: int = 20) -> dict:
        from core.models import PostCommentReply
        params = PostCommentReply(item_id=item_id, comment_id=comment_id, cursor=cursor, count=count, msToken=self._get_token()).model_dump()
        return await self._get_json(self._sign_url(ep.POST_COMMENT_REPLY, params))

    async def fetch_post_stats(self, aweme_id: str, aweme_type: int = 0) -> dict:
        from core.models import PostStats
        params = PostStats(item_id=aweme_id, aweme_type=aweme_type, msToken=self._get_token()).model_dump()
        body = urlencode(params)
        return await self._post_json(self._sign_url(ep.POST_STATS, params, body), form_data=params)

    async def fetch_locate_post(self, params: dict) -> dict:
        """定位作品 — 用于跳页定位"""
        return await self._get_json(self._sign_url(ep.LOCATE_POST, params))

    async def fetch_mix_aweme(self, mix_id: str, cursor: int = 0, count: int = 20) -> dict:
        from core.models import UserMix
        params = UserMix(mix_id=mix_id, cursor=cursor, count=count, msToken=self._get_token()).model_dump()
        return await self._get_json(self._sign_url(ep.MIX_AWEME, params))

    # ============================================================
    # 搜索
    # ============================================================

    async def fetch_post_search(self, keyword: str, offset: int = 0, count: int = 10) -> dict:
        from core.models import PostSearch
        params = PostSearch(keyword=keyword, offset=offset, count=count, msToken=self._get_token()).model_dump()
        return await self._get_json(self._sign_url(ep.POST_SEARCH, params))

    async def fetch_home_post_search(self, keyword: str, offset: int = 0, count: int = 10) -> dict:  # [reserved] 无 service 调用
        from core.models import HomePostSearch
        params = HomePostSearch(keyword=keyword, offset=offset, count=count, msToken=self._get_token()).model_dump()
        return await self._get_json(self._sign_url(ep.HOME_POST_SEARCH, params))

    async def fetch_suggest_word(self, query: str, count: int = 10) -> dict:  # [reserved] 无 service 调用
        from core.models import SuggestWord
        params = SuggestWord(query=query, count=count, msToken=self._get_token()).model_dump()
        return await self._get_json(self._sign_url(ep.SUGGEST_WORDS, params))

    # ============================================================
    # 信息流
    # ============================================================

    async def fetch_tab_feed(self, count: int = 10, tag_id: str = "", refresh_index: int = 1) -> dict:
        from core.models import TabFeed
        params = TabFeed(count=count, tag_id=tag_id, refresh_index=refresh_index, msToken=self._get_token()).model_dump()
        return await self._get_json(self._sign_url(ep.TAB_FEED, params))

    async def fetch_follow_feed(self, cursor: int = 0, count: int = 10) -> dict:
        from core.models import FollowFeed
        params = FollowFeed(cursor=cursor, count=count, msToken=self._get_token()).model_dump()
        return await self._get_json(self._sign_url(ep.FOLLOW_FEED, params))

    async def fetch_friend_feed(self, cursor: int = 0, count: int = 10) -> dict:
        from core.models import FriendFeed
        params = FriendFeed(cursor=cursor, count=count, msToken=self._get_token()).model_dump()
        return await self._get_json(self._sign_url(ep.FRIEND_FEED, params))

    # ============================================================
    # 直播
    # ============================================================

    async def fetch_live_info(self, web_rid: str = "", room_id_str: str = "") -> dict:
        from core.models import UserLive
        params = UserLive(web_rid=web_rid, room_id_str=room_id_str).model_dump()
        return await self._get_json(self._sign_url(ep.LIVE_INFO, params))

    async def fetch_live_info_by_room_id(self, room_id: str) -> dict:  # [reserved] 无 service 调用
        from core.models import UserLive2
        params = UserLive2(room_id=room_id).model_dump()
        return await self._get_json(self._sign_url(ep.LIVE_INFO_ROOM_ID, params))

    async def fetch_user_live_status(self, user_ids: str) -> dict:  # [reserved] 无 service 调用
        from core.models import UserLiveStatus
        params = UserLiveStatus(user_ids=user_ids, msToken=self._get_token()).model_dump()
        return await self._get_json(self._sign_url(ep.USER_LIVE_STATUS, params))

    async def fetch_following_user_live(self) -> dict:
        from core.models import FollowingUserLive
        params = FollowingUserLive(msToken=self._get_token()).model_dump()
        return await self._get_json(self._sign_url(ep.FOLLOW_USER_LIVE, params))

    async def fetch_live_im(self, room_id: str, user_unique_id: str) -> dict:  # [reserved] 无 service 调用
        from core.models import LiveImFetch
        params = LiveImFetch(room_id=room_id, user_unique_id=user_unique_id).model_dump()
        return await self._get_json(self._sign_url(ep.LIVE_IM_FETCH, params))
