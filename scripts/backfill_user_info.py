"""回填 user_info 表中缺失字段的旧记录。

通过 fetch_user_profile 获取完整用户资料（28 字段），更新 user_info 表。
对比 fetch_post_detail 的 author 字段，能获取 aweme_count、following_count、ip_location 等。

用法：python scripts/backfill_user_info.py
"""

import asyncio
import json
import sqlite3
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from core.crawler import DouyinCrawler
from core.filter import UserProfileFilter

DB_PATH = Path(__file__).resolve().parent.parent / "data" / "douyin.db"


async def backfill():
    if not DB_PATH.exists():
        print(f"数据库不存在: {DB_PATH}")
        return

    db = sqlite3.connect(str(DB_PATH))
    db.row_factory = sqlite3.Row

    # 找出所有 user_info 记录
    users = db.execute(
        "SELECT sec_user_id, nickname, aweme_count, following_count, ip_location FROM user_info"
    ).fetchall()

    if not users:
        print("没有用户记录")
        db.close()
        return

    print(f"找到 {len(users)} 条 user_info 记录")

    cfg_path = Path(__file__).resolve().parent.parent / "config" / "app.json"
    cookie = json.load(open(cfg_path, encoding="utf-8"))["douyin"]["cookie"]

    for user in users:
        sec_uid = user["sec_user_id"]
        nickname = user["nickname"]
        needs_update = (
            user["aweme_count"] == 0
            or user["following_count"] == 0
            or not user["ip_location"]
        )

        if not needs_update:
            print(f"  跳过 {nickname}: 数据已完整")
            continue

        print(f"  更新 {nickname} ({sec_uid[:20]}...)...")

        try:
            async with DouyinCrawler(cookie=cookie, encryption="ab") as c:
                data = await c.fetch_user_profile(sec_uid)
            profile = UserProfileFilter(data)
            info = profile.to_dict()

            db.execute(
                """UPDATE user_info SET
                    nickname = COALESCE(NULLIF(?, ''), nickname),
                    uid = COALESCE(NULLIF(?, ''), uid),
                    avatar_url = COALESCE(NULLIF(?, ''), avatar_url),
                    unique_id = COALESCE(NULLIF(?, ''), unique_id),
                    signature = COALESCE(NULLIF(?, ''), signature),
                    aweme_count = CASE WHEN ? > 0 THEN ? ELSE aweme_count END,
                    follower_count = CASE WHEN ? > 0 THEN ? ELSE follower_count END,
                    following_count = CASE WHEN ? > 0 THEN ? ELSE following_count END,
                    total_favorited = CASE WHEN ? > 0 THEN ? ELSE total_favorited END,
                    ip_location = COALESCE(NULLIF(?, ''), ip_location),
                    city = COALESCE(NULLIF(?, ''), city),
                    country = COALESCE(NULLIF(?, ''), country),
                    favoriting_count = CASE WHEN ? > 0 THEN ? ELSE favoriting_count END,
                    gender = CASE WHEN ? > 0 THEN ? ELSE gender END,
                    is_ban = CASE WHEN ? > 0 THEN ? ELSE is_ban END,
                    is_block = CASE WHEN ? > 0 THEN ? ELSE is_block END,
                    is_blocked = CASE WHEN ? > 0 THEN ? ELSE is_blocked END,
                    is_star = CASE WHEN ? > 0 THEN ? ELSE is_star END,
                    mix_count = CASE WHEN ? > 0 THEN ? ELSE mix_count END,
                    mplatform_followers_count = CASE WHEN ? > 0 THEN ? ELSE mplatform_followers_count END,
                    nickname_raw = COALESCE(NULLIF(?, ''), nickname_raw),
                    school_name = COALESCE(NULLIF(?, ''), school_name),
                    short_id = COALESCE(NULLIF(?, ''), short_id),
                    signature_raw = COALESCE(NULLIF(?, ''), signature_raw),
                    user_age = CASE WHEN ? > 0 THEN ? ELSE user_age END,
                    custom_verify = COALESCE(NULLIF(?, ''), custom_verify)
                WHERE sec_user_id = ?""",
                (
                    info.get("nickname", ""),
                    info.get("uid", ""),
                    info.get("avatar_url", ""),
                    info.get("unique_id", ""),
                    info.get("signature", ""),
                    info.get("aweme_count", 0), info.get("aweme_count", 0),
                    info.get("follower_count", 0), info.get("follower_count", 0),
                    info.get("following_count", 0), info.get("following_count", 0),
                    info.get("total_favorited", 0), info.get("total_favorited", 0),
                    info.get("ip_location", ""),
                    info.get("city", ""),
                    info.get("country", ""),
                    info.get("favoriting_count", 0), info.get("favoriting_count", 0),
                    info.get("gender", 0), info.get("gender", 0),
                    info.get("is_ban", 0), info.get("is_ban", 0),
                    info.get("is_block", 0), info.get("is_block", 0),
                    info.get("is_blocked", 0), info.get("is_blocked", 0),
                    info.get("is_star", 0), info.get("is_star", 0),
                    info.get("mix_count", 0), info.get("mix_count", 0),
                    info.get("mplatform_followers_count", 0), info.get("mplatform_followers_count", 0),
                    info.get("nickname_raw", ""),
                    info.get("school_name", ""),
                    info.get("short_id", ""),
                    info.get("signature_raw", ""),
                    info.get("user_age", 0), info.get("user_age", 0),
                    info.get("custom_verify", ""),
                    sec_uid,
                ),
            )
            db.commit()
            print(
                f"    follower={info.get('follower_count')}, "
                f"aweme={info.get('aweme_count')}, "
                f"following={info.get('following_count')}, "
                f"ip={info.get('ip_location')}"
            )
        except Exception as e:
            print(f"    失败: {e}")

    db.close()
    print("回填完成")


if __name__ == "__main__":
    asyncio.run(backfill())
