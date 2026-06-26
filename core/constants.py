"""统一常量定义"""


class DownloadMode:
    """下载模式枚举，贯穿前后端"""
    ONE = "one"
    POST = "post"
    LIKE = "like"
    MIX = "mix"
    COLLECTS = "collects"
    LIVE = "live"
    MUSIC = "music"

    ALL = {ONE, POST, LIKE, MIX, COLLECTS, LIVE, MUSIC}
