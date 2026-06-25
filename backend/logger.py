"""日志配置"""

import logging
import sys
from pathlib import Path

LOG_DIR = Path(__file__).parent / "logs"
LOG_DIR.mkdir(exist_ok=True)

LOG_FORMAT = "%(asctime)s | %(levelname)-7s | %(name)s | %(message)s"
DATE_FORMAT = "%Y-%m-%d %H:%M:%S"


def setup_logging(level: int = logging.INFO):
    """配置全局日志"""
    root = logging.getLogger()
    root.setLevel(level)

    # 清除已有 handler（避免重复）
    root.handlers.clear()

    # 控制台 — 实时刷新
    console = logging.StreamHandler(sys.stdout)
    console.setLevel(level)
    console.setFormatter(logging.Formatter(LOG_FORMAT, DATE_FORMAT))
    console.stream.reconfigure(line_buffering=True) if hasattr(console.stream, 'reconfigure') else None
    root.addHandler(console)

    # 文件 - 按级别分文件，实时刷新
    for lvl, filename in [(logging.INFO, "app.log"), (logging.ERROR, "error.log")]:
        fh = logging.FileHandler(LOG_DIR / filename, encoding="utf-8", mode="a")
        fh.setLevel(lvl)
        fh.setFormatter(logging.Formatter(LOG_FORMAT, DATE_FORMAT))
        # 每次写入后立即刷新
        fh.stream.reconfigure(line_buffering=True) if hasattr(fh.stream, 'reconfigure') else None
        root.addHandler(fh)

    # 确保所有日志立即刷新
    import atexit
    atexit.register(logging.shutdown)


def get_logger(name: str) -> logging.Logger:
    return logging.getLogger(name)
