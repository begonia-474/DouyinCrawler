"""Token 管理 — msToken / ttwid / webid 生成"""

import random
import string
import httpx


class TokenManager:
    """Token 管理器"""

    MS_TOKEN_URL = "https://mssdk.bytedance.com/web/r/token"
    TTWID_URL = "https://ttwid.bytedance.com/ttwid/union/register/"
    WEBID_URL = "https://mcs.zijieapi.com/webid"

    @staticmethod
    def gen_false_ms_token() -> str:
        """生成随机假 msToken（184字符）"""
        chars = string.ascii_letters + string.digits + "-_"
        return "".join(random.choice(chars) for _ in range(184))

    @staticmethod
    async def gen_real_ms_token(timeout: int = 10) -> str:
        """
        从字节 SDK 获取真实 msToken

        失败时回退到随机生成
        """
        try:
            async with httpx.AsyncClient(timeout=timeout) as client:
                params = {"ms_appid": "6383", "msToken": ""}
                payload = {
                    "magic": 538969122,
                    "version": 1,
                    "dataType": 8,
                    "strData": "".join(random.choices(string.ascii_letters + string.digits, k=107)),
                }
                resp = await client.post(
                    TokenManager.MS_TOKEN_URL,
                    params=params,
                    json=payload,
                )
                # 从 cookie 中提取 msToken
                for cookie in resp.cookies.items():
                    if cookie[0] == "msToken":
                        token = cookie[1]
                        if len(token) in (164, 184):
                            return token
        except Exception:
            pass
        return TokenManager.gen_false_ms_token()

    @staticmethod
    async def gen_ttwid(timeout: int = 10) -> str:
        """
        获取 ttwid

        失败时返回空字符串
        """
        try:
            async with httpx.AsyncClient(timeout=timeout) as client:
                resp = await client.post(
                    TokenManager.TTWID_URL,
                    json={
                        "region": "cn",
                        "aid": 1768,
                        "needFid": False,
                        "service": "www.ixigua.com",
                        "migrate_info": {"ticket": "", "source": "node"},
                        "cbUrlProtocol": "https",
                        "union": True,
                    },
                )
                for cookie in resp.cookies.items():
                    if cookie[0] == "ttwid":
                        return cookie[1]
        except Exception:
            pass
        return ""

    @staticmethod
    async def gen_web_id(timeout: int = 10) -> str:
        """
        获取 webid

        失败时返回空字符串
        """
        try:
            async with httpx.AsyncClient(timeout=timeout) as client:
                resp = await client.post(
                    TokenManager.WEBID_URL,
                    params={"aid": "6383"},
                    json={
                        "app_id": 6383,
                        "url": "https://www.douyin.com/",
                        "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
                        "user_unique_id": "",
                    },
                )
                data = resp.json()
                return data.get("web_id", "")
        except Exception:
            pass
        return ""
