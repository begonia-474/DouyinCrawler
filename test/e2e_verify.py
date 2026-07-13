"""端到端验证：测试数据库写入完整性"""

import asyncio
import json
import sys
from pathlib import Path

# 确保能导入项目模块
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

# 加载 cookie
CONFIG_PATH = Path(__file__).resolve().parent.parent / "config" / "app.json"
with open(CONFIG_PATH, encoding="utf-8") as f:
    _cfg = json.load(f)
COOKIE = _cfg["douyin"]["cookie"]

TEST_VIDEO_URL = "https://www.douyin.com/video/7650450403901017571"

# 预期的 video_info 字段（61 个，含 aweme_id 主键）
EXPECTED_VIDEO_FIELDS = [
    "aweme_id", "desc", "aweme_type", "author_nickname", "author_sec_uid", "author_uid",
    "create_time", "duration", "video_url", "cover_url", "music_title",
    "digg_count", "comment_count", "share_count", "collect_count",
    "mix_id", "mix_name",
    # f2 对齐 - 作者
    "author_nickname_raw", "author_short_id", "author_unique_id",
    # f2 对齐 - 内容
    "desc_raw", "is_ads", "is_story", "is_top", "is_long_video",
    # f2 对齐 - 视频
    "video_bit_rate", "animated_cover", "private_status", "is_delete",
    # f2 对齐 - 音乐
    "music_author", "music_author_raw", "music_duration", "music_id", "music_mid",
    "pgc_author", "pgc_author_title", "pgc_music_type", "music_status",
    "music_owner_handle", "music_owner_id", "music_owner_nickname",
    "music_play_url", "is_commerce_music",
    # f2 对齐 - 合集
    "mix_desc", "mix_create_time", "mix_pic_type", "mix_type", "mix_share_url",
    # f2 对齐 - 权限
    "can_comment", "can_forward", "can_share", "download_setting",
    "allow_douplus", "allow_share",
    # f2 对齐 - 统计/标签/其他
    "admire_count", "hashtag_ids", "hashtag_names", "images", "region", "is_prohibited",
]

# 预期的 user_info 字段（30 个，含 sec_user_id 主键）
EXPECTED_USER_FIELDS = [
    "sec_user_id", "nickname", "uid", "avatar_url", "unique_id", "signature",
    "aweme_count", "follower_count", "following_count", "total_favorited",
    "ip_location", "live_status", "room_id",
    # f2 对齐
    "city", "country", "favoriting_count", "gender",
    "is_ban", "is_block", "is_blocked", "is_star",
    "mix_count", "mplatform_followers_count", "nickname_raw", "school_name",
    "short_id", "signature_raw", "user_age", "custom_verify",
]


async def test_to_db_dict():
    """测试 PostDetailFilter.to_db_dict() 返回完整字段"""
    from core.crawler import DouyinCrawler
    from core.filter import PostDetailFilter

    print("=" * 60)
    print("[测试1] PostDetailFilter.to_db_dict() 字段完整性")
    print("=" * 60)

    aweme_id = TEST_VIDEO_URL.split("/video/")[-1].split("?")[0]
    print(f"  aweme_id: {aweme_id}")

    async with DouyinCrawler(cookie=COOKIE, encryption="ab") as crawler:
        data = await crawler.fetch_post_detail(aweme_id)

    if data.get("status_code", -1) != 0:
        print(f"  [FAIL] API 返回错误: {data.get('status_msg')}")
        return None

    detail = PostDetailFilter(data)
    db_dict = detail.to_db_dict()

    print(f"  to_db_dict() 返回 {len(db_dict)} 个字段")
    print(f"  预期 {len(EXPECTED_VIDEO_FIELDS)} 个字段")

    # 检查每个字段是否存在
    missing = [f for f in EXPECTED_VIDEO_FIELDS if f not in db_dict]
    extra = [f for f in db_dict if f not in EXPECTED_VIDEO_FIELDS]

    if missing:
        print(f"  [FAIL] 缺少字段: {missing}")
    else:
        print(f"  [PASS] 所有预期字段都存在")

    if extra:
        print(f"  [INFO] 额外字段: {extra}")

    # 打印部分关键字段
    print(f"\n  关键字段值:")
    print(f"    aweme_id: {db_dict.get('aweme_id')}")
    print(f"    desc: {str(db_dict.get('desc', ''))[:50]}")
    print(f"    author_nickname: {db_dict.get('author_nickname')}")
    print(f"    digg_count: {db_dict.get('digg_count')}")
    print(f"    comment_count: {db_dict.get('comment_count')}")
    print(f"    video_url: {str(db_dict.get('video_url', ''))[:60]}...")

    return db_dict


async def test_resolve_single():
    """测试 resolve_single 返回 SingleDownloadPlanV1"""
    from core.py_bridge import resolve_single

    print("\n" + "=" * 60)
    print("[测试2] resolve_single() 返回 SingleDownloadPlanV1")
    print("=" * 60)

    result = resolve_single(TEST_VIDEO_URL)

    if not result.get("success"):
        print(f"  [FAIL] 解析失败: {result.get('error')}")
        return None

    print(f"  [PASS] 解析成功")
    print(f"  contract_version: {result.get('contract_version')}")
    print(f"  mode: {result.get('mode')}")
    print(f"  save_dir: {result.get('save_dir')}")
    print(f"  total: {result.get('total')}")

    return result


async def test_user_profile_to_dict():
    """测试 UserProfileFilter.to_dict() 返回完整字段"""
    from core.crawler import DouyinCrawler
    from core.filter import UserProfileFilter

    print("\n" + "=" * 60)
    print("[测试3] UserProfileFilter.to_dict() 字段完整性")
    print("=" * 60)

    # 用视频详情中的 author_sec_uid 来获取用户信息
    user_url = "https://www.douyin.com/user/MS4wLjABAAAAICzXd4iEYZzFmurKgaK3xVYAviJyEU9KPVYZqL6mNYrpVikZvgVoZ3-K04VU2DFZ"

    async with DouyinCrawler(cookie=COOKIE, encryption="ab") as crawler:
        sec_user_id = user_url.split("/user/")[-1].split("?")[0]
        data = await crawler.fetch_user_profile(sec_user_id)

    if not data:
        print(f"  [FAIL] 无法获取用户信息")
        return None

    user = UserProfileFilter(data)
    udict = user.to_dict()

    print(f"  to_dict() 返回 {len(udict)} 个字段")
    print(f"  预期 {len(EXPECTED_USER_FIELDS)} 个字段")

    missing = [f for f in EXPECTED_USER_FIELDS if f not in udict]
    if missing:
        print(f"  [FAIL] 缺少字段: {missing}")
    else:
        print(f"  [PASS] 所有预期字段都存在")

    print(f"\n  关键字段值:")
    print(f"    sec_user_id: {udict.get('sec_user_id')}")
    print(f"    nickname: {udict.get('nickname')}")
    print(f"    follower_count: {udict.get('follower_count')}")
    print(f"    aweme_count: {udict.get('aweme_count')}")

    return udict


async def main():
    print("DouyinCrawler 数据库对齐 f2 端到端验证\n")

    # 测试1: to_db_dict
    db_dict = await test_to_db_dict()

    # 测试2: resolve_single
    result = await test_resolve_single()

    # 测试3: user profile
    udict = await test_user_profile_to_dict()

    # 总结
    print("\n" + "=" * 60)
    print("验证总结")
    print("=" * 60)

    if db_dict:
        video_ok = len(db_dict) >= len(EXPECTED_VIDEO_FIELDS)
        print(f"  video_info 字段: {len(db_dict)}/{len(EXPECTED_VIDEO_FIELDS)} {'PASS' if video_ok else 'FAIL'}")
    else:
        print(f"  video_info 字段: SKIP (API 失败)")

    resolved_metadata = None
    if result and result.get("items"):
        resolved_metadata = result["items"][0].get("metadata")
    if resolved_metadata:
        detail_ok = len(resolved_metadata) >= len(EXPECTED_VIDEO_FIELDS)
        print(
            "  resolve_single metadata: "
            f"{len(resolved_metadata)}/{len(EXPECTED_VIDEO_FIELDS)} "
            f"{'PASS' if detail_ok else 'FAIL'}"
        )
    else:
        print("  resolve_single metadata: SKIP")

    if udict:
        user_ok = len(udict) >= len(EXPECTED_USER_FIELDS)
        print(f"  user_info 字段: {len(udict)}/{len(EXPECTED_USER_FIELDS)} {'PASS' if user_ok else 'FAIL'}")
    else:
        print(f"  user_info 字段: SKIP (API 失败)")


if __name__ == "__main__":
    asyncio.run(main())
