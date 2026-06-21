"""全模式端到端测试"""

import asyncio
from pathlib import Path

import yaml

CONFIG_PATH = Path(__file__).resolve().parent.parent / "docs" / "抖音相关" / "app.yaml"
with open(CONFIG_PATH, encoding="utf-8") as f:
    cfg = yaml.safe_load(f)
COOKIE = cfg["douyin"]["cookie"]

DOWNLOAD_DIR = str(Path(__file__).resolve().parent.parent / "Download")
USER_URL = "https://www.douyin.com/user/MS4wLjABAAAAICzXd4iEYZzFmurKgaK3xVYAviJyEU9KPVYZqL6mNYrpVikZvgVoZ3-K04VU2DFZ"
VIDEO_URL = "https://www.douyin.com/video/7650450403901017571"


async def test_mode(name, coro):
    print(f"\n{'='*50}")
    print(f"测试: {name}")
    print(f"{'='*50}")
    try:
        result = await coro
        if result.get("success"):
            print(f"  [OK]")
            for k, v in result.items():
                if k == "success":
                    continue
                if isinstance(v, list):
                    print(f"  {k}: {len(v)} items")
                elif isinstance(v, dict):
                    print(f"  {k}: {v}")
                else:
                    print(f"  {k}: {v}")
        else:
            print(f"  [FAIL] {result.get('error', 'unknown')}")
        return result
    except Exception as e:
        print(f"  [ERROR] {e}")
        return {"success": False, "error": str(e)}


async def main():
    from core.handler import DouyinHandler

    handler = DouyinHandler(
        cookie=COOKIE,
        download_path=DOWNLOAD_DIR,
        max_counts=3,  # 测试只下载3个
        page_counts=5,
    )

    # 1. 单视频下载
    await test_mode("单视频下载 (one)", handler.handle_one_video(VIDEO_URL))

    # 2. 用户资料
    await test_mode("用户资料 (profile)", handler.handle_user_profile(USER_URL))

    # 3. 用户主页视频 (只获取不下载)
    await test_mode("用户主页视频 (post)", handler.handle_user_post(USER_URL))

    # 4. 相关推荐
    await test_mode("相关推荐 (related)", handler.handle_related(VIDEO_URL))

    # 5. 评论
    await test_mode("评论 (comment)", handler.handle_post_comment(VIDEO_URL, count=5))

    # 6. 首页推荐
    await test_mode("首页推荐 (tab_feed)", handler.handle_tab_feed(count=3))

    # 7. 收藏夹列表
    await test_mode("收藏夹列表 (collects)", handler.handle_user_collects())

    # 8. 音乐收藏
    await test_mode("音乐收藏 (music)", handler.handle_user_music_collection(count=3))

    # 9. 关注列表
    await test_mode("关注列表 (following)", handler.handle_user_following(USER_URL, count=3))

    # 10. 粉丝列表
    await test_mode("粉丝列表 (follower)", handler.handle_user_follower(USER_URL, count=3))

    # 11. 直播信息 (用一个实际直播间测试)
    await test_mode("直播信息 (live)", handler.handle_user_live("https://live.douyin.com/870432280816"))

    # 12. 搜索
    await test_mode("搜索 (search)", handler.handle_search("美食", count=3))


if __name__ == "__main__":
    asyncio.run(main())
