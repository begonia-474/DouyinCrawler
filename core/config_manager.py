"""客户端配置管理器

参考: https://github.com/Johnserf-Seed/f2
"""

from core.config import DEFAULT_BROWSER, DEFAULT_HEADERS


class ClientConfManager:
    """客户端配置管理器"""

    @classmethod
    def encryption(cls) -> str:
        """获取加密方式"""
        return "ab"  # 默认使用 ABogus

    @classmethod
    def base_request_model(cls) -> dict:
        """获取基础请求模型配置"""
        return {
            "version": {
                "version_code": "290100",
                "version_name": "29.1.0",
            },
            "browser": {
                "name": DEFAULT_BROWSER["name"],
                "version": DEFAULT_BROWSER["version"],
                "platform": DEFAULT_BROWSER["platform"],
                "language": DEFAULT_BROWSER["language"],
            },
            "engine": {
                "name": DEFAULT_BROWSER["engine_name"],
                "version": DEFAULT_BROWSER["engine_version"],
            },
            "os": {
                "name": DEFAULT_BROWSER["os_name"],
                "version": DEFAULT_BROWSER["os_version"],
            },
        }

    @classmethod
    def brm_version(cls) -> dict:
        """获取版本配置"""
        return cls.base_request_model().get("version", {})

    @classmethod
    def brm_browser(cls) -> dict:
        """获取浏览器配置"""
        return cls.base_request_model().get("browser", {})

    @classmethod
    def brm_engine(cls) -> dict:
        """获取引擎配置"""
        return cls.base_request_model().get("engine", {})

    @classmethod
    def brm_os(cls) -> dict:
        """获取操作系统配置"""
        return cls.base_request_model().get("os", {})

    @classmethod
    def headers(cls) -> dict:
        """获取请求头配置"""
        return DEFAULT_HEADERS

    @classmethod
    def user_agent(cls) -> str:
        """获取 User-Agent"""
        return DEFAULT_HEADERS.get("User-Agent", "")

    @classmethod
    def referer(cls) -> str:
        """获取 Referer"""
        return DEFAULT_HEADERS.get("Referer", "")

    @classmethod
    def proxies(cls) -> dict:
        """获取代理配置"""
        return {"http://": None, "https://": None}
