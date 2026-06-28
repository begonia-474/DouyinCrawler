"""服务配置数据类

所有业务服务的单一配置数据类，消除 16 个独立 __init__ 参数重复。
"""

from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional


@dataclass
class ServiceConfig:
    """所有业务服务的共享配置

    消除 16 个独立 __init__ 参数在 BaseService、DouyinHandler、task_manager 三处重复。
    """

    cookie: str = ""
    download_path: Path = field(default_factory=lambda: Path("Download"))
    naming: str = "{create}_{desc}"
    max_counts: float = float("inf")
    page_counts: int = 20
    timeout: int = 10
    encryption: str = "ab"
    proxies: Optional[dict] = None
    app_name: str = "douyin"
    folderize: bool = False
    music: bool = False
    cover: bool = False
    desc: bool = False
    interval: Optional[str] = None
    max_connections: int = 5
    max_retries: int = 5
    max_tasks: int = 10

    def to_dict(self) -> dict:
        """转换为 kwargs dict（兼容旧代码的 **self._config 模式）"""
        return {
            "cookie": self.cookie,
            "download_path": self.download_path,
            "naming": self.naming,
            "max_counts": self.max_counts,
            "page_counts": self.page_counts,
            "timeout": self.timeout,
            "encryption": self.encryption,
            "proxies": self.proxies,
            "app_name": self.app_name,
            "folderize": self.folderize,
            "music": self.music,
            "cover": self.cover,
            "desc": self.desc,
            "interval": self.interval,
            "max_connections": self.max_connections,
            "max_retries": self.max_retries,
            "max_tasks": self.max_tasks,
        }
