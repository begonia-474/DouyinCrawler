"""签名管理器 — 封装 ABogus/XBogus 调用

参考: https://github.com/Johnserf-Seed/f2
"""

from urllib.parse import urlencode

from core.signature.abogus import ABogus
from core.signature.xbogus import XBogus
from core.signature.fingerprint import BrowserFingerprintGenerator


class XBogusManager:
    """X-Bogus 签名管理器"""

    @classmethod
    def str_2_endpoint(cls, user_agent: str, endpoint: str) -> str:
        """从字符串参数生成带签名的完整 URL"""
        xb = XBogus(user_agent)
        result = xb.getXBogus(endpoint)
        return result[0]

    @classmethod
    def model_2_endpoint(
        cls,
        user_agent: str,
        base_endpoint: str,
        params: dict,
    ) -> str:
        """从参数字典生成带签名的完整 URL

        Args:
            user_agent: 浏览器 User-Agent
            base_endpoint: API 基础端点
            params: 请求参数字典

        Returns:
            带签名的完整 URL
        """
        if not isinstance(params, dict):
            raise TypeError("参数必须是字典类型")

        param_str = urlencode(params)

        try:
            xb = XBogus(user_agent)
            xb_value = xb.getXBogus(param_str)
        except Exception as e:
            raise RuntimeError(f"生成 X-Bogus 失败: {e}")

        # 检查 base_endpoint 是否已有查询参数
        separator = "&" if "?" in base_endpoint else "?"
        final_endpoint = f"{base_endpoint}{separator}{param_str}&X-Bogus={xb_value[1]}"

        return final_endpoint


class ABogusManager:
    """A-Bogus 签名管理器"""

    @classmethod
    def str_2_endpoint(
        cls,
        user_agent: str,
        params: str,
        body: str = "",
    ) -> str:
        """从字符串参数生成带签名的完整 URL"""
        browser_fp = BrowserFingerprintGenerator.generate_fingerprint("Edge")
        ab = ABogus(user_agent, browser_fp)
        result = ab.generate(params, body)
        return result[0]

    @classmethod
    def model_2_endpoint(
        cls,
        user_agent: str,
        base_endpoint: str,
        params: dict,
        body: str = "",
    ) -> str:
        """从参数字典生成带签名的完整 URL

        Args:
            user_agent: 浏览器 User-Agent
            base_endpoint: API 基础端点
            params: 请求参数字典
            body: POST 请求体（可选）

        Returns:
            带签名的完整 URL
        """
        if not isinstance(params, dict):
            raise TypeError("参数必须是字典类型")

        param_str = urlencode(params)

        try:
            browser_fp = BrowserFingerprintGenerator.generate_fingerprint("Edge")
            ab = ABogus(user_agent, browser_fp)
            ab_value = ab.generate(param_str, body)
        except Exception as e:
            raise RuntimeError(f"生成 A-Bogus 失败: {e}")

        # 检查 base_endpoint 是否已有查询参数
        separator = "&" if "?" in base_endpoint else "?"
        final_endpoint = f"{base_endpoint}{separator}{param_str}&a_bogus={ab_value[1]}"

        return final_endpoint
