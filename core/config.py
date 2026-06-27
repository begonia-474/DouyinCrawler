"""应用配置管理

与 Rust ConfigManager 共用 config/app.json，
结构: {"douyin": {...}, "tiktok": {...}}
"""

import json
import logging
from pathlib import Path

logger = logging.getLogger(__name__)

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

# 与 Rust AppConfig::default() 对齐的默认值
DEFAULTS = {
    "cookie": "",
    "download_path": "Download",
    "naming": "{create}_{desc}",
    "encryption": "ab",
    "proxy": "",
    "app_name": "douyin",
    "folderize": False,
    "music": False,
    "cover": False,
    "desc": False,
    "interval": None,
    "page_counts": 20,
    "max_counts": 0,
    "timeout": 10,
    "max_connections": 5,
    "max_retries": 5,
    "max_tasks": 10,
}


def _find_config_path() -> Path:
    """查找 config/app.json，优先项目根目录"""
    # 从 core/config.py 向上找项目根目录
    project_root = Path(__file__).resolve().parent.parent
    config_path = project_root / "config" / "app.json"
    if config_path.exists():
        return config_path

    # 回退：当前工作目录
    cwd_config = Path.cwd() / "config" / "app.json"
    if cwd_config.exists():
        return cwd_config

    # 都不存在，返回默认路径（项目根目录）
    return config_path


def _load_app_json(path: Path) -> dict:
    """加载 config/app.json，返回 douyin 配置段"""
    if not path.exists():
        logger.warning("[config] 配置文件不存在: %s，使用默认值", path)
        return {}

    try:
        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)
        # 提取 douyin 配置段
        douyin = data.get("douyin", {})
        logger.info("[config] 已加载配置: %s", path)
        return douyin
    except Exception as e:
        logger.error("[config] 加载配置失败: %s", e)
        return {}


class Config:
    """应用配置

    优先从 config/app.json 读取（与 Rust 共用），
    支持运行时通过 update() 接收 Rust 推送的配置。
    """

    def __init__(self):
        self._data: dict = {}
        self._path = _find_config_path()
        self._load()

    def _load(self):
        self._data = _load_app_json(self._path)

    def update(self, **kwargs):
        """接收 Rust 推送的配置更新（由 task_manager.update_config 调用）"""
        for k, v in kwargs.items():
            if k == "headers":
                continue
            self._data[k] = v

    def get(self, key: str, default=None):
        if key in self._data:
            return self._data[key]
        if default is not None:
            return default
        return DEFAULTS.get(key)

    def set(self, key: str, value):
        self._data[key] = value

    def to_kwargs(self) -> dict:
        """导出为 kwargs 字典，供爬虫使用"""
        kwargs = dict(DEFAULTS)
        kwargs.update(self._data)
        kwargs["headers"] = DEFAULT_HEADERS
        # 代理配置：proxy 字符串转 requests 格式
        proxy = kwargs.get("proxy", "")
        if proxy:
            kwargs["proxies"] = {"http://": proxy, "https://": proxy}
        else:
            kwargs["proxies"] = {"http://": None, "https://": None}
        return kwargs


# 全局配置实例
config = Config()
