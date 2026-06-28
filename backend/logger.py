"""日志配置 — 基于标准库 logging"""

import sys
import logging
from pathlib import Path

LOG_DIR = Path(__file__).parent / "logs"
LOG_DIR.mkdir(exist_ok=True)

LOG_FORMAT = "%(asctime)s | %(levelname)-7s | %(name)s | %(message)s"
DATE_FORMAT = "%Y-%m-%d %H:%M:%S"


def setup_logging(level: int = logging.INFO):
    """配置全局日志"""
    root = logging.getLogger()
    root.handlers.clear()
    root.setLevel(level)

    # 控制台输出
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setLevel(level)
    console_handler.setFormatter(logging.Formatter(LOG_FORMAT, datefmt=DATE_FORMAT))
    root.addHandler(console_handler)

    # 文件输出 — app.log
    file_handler = logging.FileHandler(
        LOG_DIR / "app.log", encoding="utf-8"
    )
    file_handler.setLevel(level)
    file_handler.setFormatter(logging.Formatter(LOG_FORMAT, datefmt=DATE_FORMAT))
    root.addHandler(file_handler)

    # 文件输出 — error.log
    error_handler = logging.FileHandler(
        LOG_DIR / "error.log", encoding="utf-8"
    )
    error_handler.setLevel(logging.ERROR)
    error_handler.setFormatter(logging.Formatter(LOG_FORMAT, datefmt=DATE_FORMAT))
    root.addHandler(error_handler)


def get_logger(name: str):
    """获取 logger 实例"""
    return logging.getLogger(name)
