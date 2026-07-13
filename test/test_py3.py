"""
PyO3 集成测试脚本
测试所有 PyO3 调用是否正常工作
"""

import json
import sys
import pytest
from pathlib import Path

pytestmark = [pytest.mark.integration]

# 添加项目根目录到 Python 路径
project_root = Path(__file__).parent.parent
sys.path.insert(0, str(project_root))


def test_config():
    """测试配置加载"""
    print("=" * 50)
    print("测试配置加载")
    print("=" * 50)

    from core.py_bridge import _get_context
    ctx = _get_context()

    print(f"Cookie 长度: {len(ctx._cookie)}")
    print(f"下载路径: {ctx._download_path}")
    print(f"命名模板: {ctx._naming}")
    print(f"应用名称: {ctx._app_name}")
    print()


def test_parse_video():
    """测试视频解析"""
    print("=" * 50)
    print("测试视频解析")
    print("=" * 50)

    from core.py_bridge import parse_video

    url = "https://www.douyin.com/video/7650450403901017571"
    print(f"URL: {url}")

    result = parse_video(url)
    print(f"成功: {result.get('success')}")

    if result.get('success'):
        detail = result.get('detail', {})
        print(f"作者昵称: {detail.get('author_nickname', 'N/A')}")
        print(f"作者 sec_uid: {detail.get('author_sec_uid', 'N/A')}")
        print(f"视频描述: {detail.get('desc', 'N/A')[:50]}")
    else:
        print(f"错误: {result.get('error')}")
    print()


def test_get_user_profile():
    """测试用户主页解析"""
    print("=" * 50)
    print("测试用户主页解析")
    print("=" * 50)

    from core.py_bridge import get_user_profile

    url = "https://www.douyin.com/user/MS4wLjABAAAAICzXd4iEYZzFmurKgaK3xVYAviJyEU9KPVYZqL6mNYrpVikZvgVoZ3-K04VU2DFZ"
    print(f"URL: {url}")

    result = get_user_profile(url)
    print(f"成功: {result.get('success')}")

    if result.get('success'):
        profile = result.get('profile', {})
        print(f"用户昵称: {profile.get('nickname', 'N/A')}")
        print(f"粉丝数: {profile.get('follower_count', 0)}")
    else:
        print(f"错误: {result.get('error')}")
    print()


def test_get_following_live():
    """测试关注直播列表"""
    print("=" * 50)
    print("测试关注直播列表")
    print("=" * 50)

    from core.py_bridge import get_following_live

    result = get_following_live()
    print(f"成功: {result.get('success')}")

    if result.get('success'):
        lives = result.get('lives', [])
        print(f"直播数量: {len(lives)}")
        for i, live in enumerate(lives[:5]):
            print(f"  {i + 1}. {live.get('nickname', 'N/A')}: {live.get('title', 'N/A')}")
    else:
        print(f"错误: {result.get('error')}")
    print()


def main():
    """主测试函数"""
    print("PyO3 集成测试")
    print()

    try:
        test_config()
        test_parse_video()
        test_get_user_profile()
        test_get_following_live()

        print("=" * 50)
        print("测试完成")
        print("=" * 50)
    except Exception as e:
        print(f"测试失败: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main()
