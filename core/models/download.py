"""下载模式枚举 — 贯穿 Python / Rust / Frontend 三端

Rust 侧对应定义在 src-tauri/src/services/download/mod.rs。
前端侧通过 tauri-specta 自动生成（或手动定义在 api-types.ts）。
"""


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
