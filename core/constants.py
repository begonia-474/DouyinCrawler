"""向后兼容 shim — DownloadMode 已迁移到 core.models.download

原 core/constants.py 中的 DownloadMode 枚举现在定义在 core/models/download.py。
此文件保留为向后兼容重导出，所有 `from core.constants import DownloadMode` 调用继续工作。
"""
from core.models.download import DownloadMode  # noqa: F401
