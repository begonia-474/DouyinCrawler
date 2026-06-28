"""时间戳格式化与日期区间过滤"""

import time
import datetime


def timestamp_2_str(timestamp, format: str = "%Y-%m-%d %H-%M-%S") -> str:
    """将 UNIX 时间戳转换为东八区北京时间格式化字符串。

    Args:
        timestamp: 秒或毫秒级 UNIX 时间戳（str/int/float）
        format: 日期格式

    Returns:
        格式化字符串，无效输入返回空字符串
    """
    if timestamp in (None, "None", "", 0, "0"):
        return ""
    try:
        ts = float(timestamp)
        if ts > 1e10:
            ts /= 1000
        tz = datetime.timezone(datetime.timedelta(hours=8))
        return datetime.datetime.fromtimestamp(ts, tz=tz).strftime(format)
    except (ValueError, TypeError, OSError):
        return ""


def interval_2_timestamp(interval: str, date_type: str = "start") -> int:
    """
    将日期区间转换为时间戳（毫秒）

    Args:
        interval: 日期区间，格式 "YYYY-MM-DD|YYYY-MM-DD"
        date_type: "start" 或 "end"

    Returns:
        毫秒级时间戳
    """
    try:
        parts = interval.split("|")
        if len(parts) != 2:
            return 0

        date_str = parts[0] if date_type == "start" else parts[1]
        # 解析日期
        dt = time.strptime(date_str, "%Y-%m-%d")
        ts = int(time.mktime(dt))

        # end 日期需要加一天减一秒，包含全天
        if date_type == "end":
            ts += 86400 - 1

        return ts * 1000  # 转换为毫秒
    except Exception:
        return 0


def filter_by_date_interval(aweme_list: list, interval: str, field: str = "create_time") -> list:
    """
    按日期区间过滤作品列表

    Args:
        aweme_list: 作品列表
        interval: 日期区间，格式 "YYYY-MM-DD|YYYY-MM-DD" 或 "all"
        field: 日期字段名

    Returns:
        过滤后的作品列表
    """
    if not interval or interval == "all":
        return aweme_list

    start_ts = interval_2_timestamp(interval, "start")
    end_ts = interval_2_timestamp(interval, "end")

    if start_ts == 0 or end_ts == 0:
        return aweme_list

    filtered = []
    for item in aweme_list:
        # 获取创建时间（兼容 dict 和对象）
        if isinstance(item, dict):
            create_time = item.get(field, 0)
        else:
            create_time = getattr(item, field, 0)
        if isinstance(create_time, str):
            try:
                create_time = int(time.mktime(time.strptime(create_time, "%Y-%m-%d %H:%M:%S")))
            except Exception:
                continue

        # 转换为毫秒
        create_time_ms = create_time * 1000 if create_time < 1e12 else create_time

        if start_ts <= create_time_ms <= end_ts:
            filtered.append(item)

    return filtered
