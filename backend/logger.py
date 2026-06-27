"""日志配置 — 基于 loguru，自动捕获 stdlib logging 输出"""

import sys
from pathlib import Path
from loguru import logger

LOG_DIR = Path(__file__).parent / "logs"
LOG_DIR.mkdir(exist_ok=True)

# 移除 loguru 默认 handler
logger.remove()

LOG_FORMAT = (
    "<green>{time:YYYY-MM-DD HH:mm:ss}</green> | "
    "<level>{level: <7}</level> | "
    "<cyan>{name}</cyan> | "
    "<level>{message}</level>"
)

# 控制台输出
logger.add(sys.stdout, format=LOG_FORMAT, level="INFO", colorize=True)

# 文件输出 — 按级别分文件，自动轮转
logger.add(
    LOG_DIR / "app.log",
    format=LOG_FORMAT,
    level="INFO",
    rotation="10 MB",
    retention="7 days",
    encoding="utf-8",
)
logger.add(
    LOG_DIR / "error.log",
    format=LOG_FORMAT,
    level="ERROR",
    rotation="10 MB",
    retention="30 days",
    encoding="utf-8",
)

# 拦截 stdlib logging — core/ 模块的 logging.getLogger(__name__) 输出也会被 loguru 接管
import logging


class InterceptHandler(logging.Handler):
    """将 stdlib logging 转发到 loguru"""

    def emit(self, record: logging.LogRecord) -> None:
        try:
            level = logger.level(record.levelname).name
        except ValueError:
            level = record.levelno
        logger.opt(depth=6, exception=record.exc_info).log(level, record.getMessage())


def setup_logging(level: int = logging.INFO):
    """配置全局日志，拦截 stdlib logging"""
    root = logging.getLogger()
    root.handlers.clear()
    root.setLevel(level)
    root.addHandler(InterceptHandler())


def get_logger(name: str):
    """兼容接口 — 直接返回 loguru logger"""
    return logger
