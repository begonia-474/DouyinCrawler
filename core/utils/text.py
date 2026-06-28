"""文本清洗与文件名格式化（移植自 f2）"""

import re
import time


_REPLACE_T_RE = re.compile(r"[^一-龥a-zA-Z0-9#]")


def replaceT(obj):
    """替换文案非法字符，保留中文、英文、数字、#。

    用于文件名安全和前端展示。支持 str 和 list 输入。
    """
    if isinstance(obj, list):
        return [re.sub(_REPLACE_T_RE, "_", i) if isinstance(i, str) else i or "" for i in obj]
    if isinstance(obj, str):
        return re.sub(_REPLACE_T_RE, "_", obj)
    return obj


def sanitize_filename(name: str, max_len: int = 200) -> str:
    """清理文件名，移除非法字符和 emoji，按字节截断"""
    name = re.sub(r'[\\/:*?"<>|\n\r\t]', '_', name)
    # 过滤 emoji（SMP 平面 U+10000 以上）
    name = re.sub(r'[\U00010000-\U0010ffff]+', '', name)
    name = name.strip('. ')
    # 按字节截断，避免中文截断产生乱码
    encoded = name.encode('utf-8')
    if len(encoded) > max_len:
        name = encoded[:max_len].decode('utf-8', errors='ignore')
    return name


def format_filename(template: str, data: dict) -> str:
    """
    格式化文件名模板

    支持变量: {create}, {desc}, {caption}, {nickname}, {aweme_id}, {uid}
    """
    create_ts = data.get("create_time", 0)
    if isinstance(create_ts, str) and "-" in create_ts:
        # 已经是格式化字符串（如 "2024-06-25 16-00-00"），直接用
        create_str = create_ts.replace(" ", "_").replace(":", "").replace("-", "-", 2)
    else:
        create_str = time.strftime("%Y-%m-%d_%H%M%S", time.localtime(create_ts)) if create_ts else "unknown"

    # desc 和 caption 都指向 desc 字段（f2 兼容）
    # strip 尾部下划线，避免空 desc 时模板展开产生多余连接符
    desc = sanitize_filename(data.get("desc", "")).strip('_ ')

    result = template.format(
        create=create_str,
        desc=desc,
        caption=desc,  # caption 与 desc 相同，f2 兼容
        nickname=sanitize_filename(data.get("author", "")),
        aweme_id=data.get("aweme_id", ""),
        uid=data.get("author_uid", ""),
    )
    return sanitize_filename(result).rstrip('_')
