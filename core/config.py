"""应用配置管理"""

import json
from pathlib import Path


# 默认浏览器指纹
DEFAULT_BROWSER = {
    "name": "Edge",
    "version": "130.0.0.0",
    "platform": "Win32",
    "language": "zh-CN",
    "engine_name": "Blink",
    "engine_version": "130.0.0.0",
    "os_name": "Windows",
    "os_version": "10",
}

DEFAULT_HEADERS = {
    "User-Agent": (
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
        "AppleWebKit/537.36 (KHTML, like Gecko) "
        "Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0"
    ),
    "Referer": "https://www.douyin.com/",
}


class Config:
    """应用配置"""

    DEFAULTS = {
        "cookie": "",
        "download_path": "Download",
        "naming": "{create}_{desc}",
        "timeout": 10,
        "max_retries": 5,
        "max_connections": 5,
        "max_tasks": 5,
        "encryption": "ab",
        "proxies": {"http://": None, "https://": None},
    }

    def __init__(self, config_path: str | Path = "config.json"):
        self._path = Path(config_path)
        self._data: dict = {}
        self._load()

    def _load(self):
        if self._path.exists():
            with open(self._path, "r", encoding="utf-8") as f:
                self._data = json.load(f)

    def save(self):
        with open(self._path, "w", encoding="utf-8") as f:
            json.dump(self._data, f, ensure_ascii=False, indent=2)

    def get(self, key: str, default=None):
        if key in self._data:
            return self._data[key]
        if default is not None:
            return default
        return self.DEFAULTS.get(key)

    def set(self, key: str, value):
        self._data[key] = value

    def to_kwargs(self) -> dict:
        """导出为 kwargs 字典，供爬虫使用"""
        kwargs = dict(self.DEFAULTS)
        kwargs.update(self._data)
        kwargs["headers"] = DEFAULT_HEADERS
        return kwargs


# 全局配置实例
config = Config()
