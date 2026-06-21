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

    # 控制台
    console = logging.StreamHandler(sys.stdout)
    console.setLevel(level)
    console.setFormatter(logging.Formatter(LOG_FORMAT, DATE_FORMAT))
    root.addHandler(console)

    # 文件 - 按级别分文件
    for lvl, filename in [(logging.INFO, "app.log"), (logging.ERROR, "error.log")]:
        fh = logging.FileHandler(LOG_DIR / filename, encoding="utf-8")
        fh.setLevel(lvl)
        fh.setFormatter(logging.Formatter(LOG_FORMAT, DATE_FORMAT))
        root.addHandler(fh)


def get_logger(name: str) -> logging.Logger:
    return logging.getLogger(name)
