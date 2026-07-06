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
    """清理文件名，移除非法字符和 emoji，按字节截断（对齐 f2 split_filename）"""
    name = re.sub(r'[\\/:*?"<>|\n\r\t]', '_', name)
    # 过滤 emoji（SMP 平面 U+10000 以上）
    name = re.sub(r'[\U00010000-\U0010ffff]+', '', name)
    # 合并连续空格（对齐 f2）
    name = re.sub(r'\s+', ' ', name).strip('. ')
    # 按字节截断，超长时保留首尾（对齐 f2 split_filename）
    encoded = name.encode('utf-8')
    if len(encoded) > max_len:
        # f2 策略：前 2/3 + "......" + 后 1/3
        split_first = (max_len - 6) * 2 // 3
        split_second = (max_len - 6) // 3
        first_part = encoded[:split_first].decode('utf-8', errors='ignore')
        second_part = encoded[-split_second:].decode('utf-8', errors='ignore')
        name = f"{first_part}......{second_part}"
    return name


def format_filename(template: str, data: dict) -> str:
    """
    格式化文件名模板

    支持变量: {create}, {desc}, {caption}, {nickname}, {aweme_id}, {uid}
    """
    create_ts = data.get("create_time", 0)
    if isinstance(create_ts, str) and "-" in create_ts:
        # 已经是格式化字符串，对齐 f2 格式：YYYY-MM-DD HH-MM-SS
        create_str = create_ts.replace(":", "-")
        # 确保日期和时间之间用空格分隔（f2 格式）
        if "_" in create_str:
            create_str = create_str.replace("_", " ")
    else:
        create_str = time.strftime("%Y-%m-%d %H-%M-%S", time.localtime(create_ts)) if create_ts else "unknown"

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
