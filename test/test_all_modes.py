"""集成测试：全模式端到端测试

需要网络和真实 cookie。
"""

import pytest
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

pytestmark = [pytest.mark.integration]

USER_URL = "https://www.douyin.com/user/MS4wLjABAAAAICzXd4iEYZzFmurKgaK3xVYAviJyEU9KPVYZqL6mNYrpVikZvgVoZ3-K04VU2DFZ"
VIDEO_URL = "https://www.douyin.com/video/7650450403901017571"


@pytest.fixture(scope="module")
def handler(cookie, download_dir):
    """创建 DouyinHandler 实例"""
    from core.handler import DouyinHandler
    return DouyinHandler(
        cookie=cookie,
        download_path=download_dir,
        max_counts=3,
        page_counts=5,
    )


@pytest.mark.asyncio
async def test_user_profile(handler):
    """用户资料"""
    result = await handler.handle_user_profile(USER_URL)
    assert result.get("success"), f"失败: {result.get('error')}"


@pytest.mark.asyncio
async def test_related(handler):
    """相关推荐"""
    result = await handler.handle_related(VIDEO_URL)
    assert result.get("success"), f"失败: {result.get('error')}"


@pytest.mark.asyncio
async def test_comment(handler):
    """评论"""
    result = await handler.handle_post_comment(VIDEO_URL, count=5)
    assert result.get("success"), f"失败: {result.get('error')}"


@pytest.mark.asyncio
async def test_tab_feed(handler):
    """首页推荐"""
    result = await handler.handle_tab_feed(count=3)
    assert result.get("success"), f"失败: {result.get('error')}"


@pytest.mark.asyncio
async def test_collects(handler):
    """收藏夹列表"""
    result = await handler.handle_user_collects()
    assert result.get("success"), f"失败: {result.get('error')}"


@pytest.mark.asyncio
async def test_music_collection(handler):
    """音乐收藏"""
    result = await handler.handle_user_music_collection(count=3)
    assert result.get("success"), f"失败: {result.get('error')}"


@pytest.mark.asyncio
async def test_following(handler):
    """关注列表"""
    result = await handler.handle_user_following(USER_URL, count=3)
    assert result.get("success"), f"失败: {result.get('error')}"


@pytest.mark.asyncio
async def test_follower(handler):
    """粉丝列表"""
    result = await handler.handle_user_follower(USER_URL, count=3)
    assert result.get("success"), f"失败: {result.get('error')}"


@pytest.mark.asyncio
async def test_live_info(handler):
    """直播信息"""
    result = await handler.handle_user_live("https://live.douyin.com/870432280816")
    assert result.get("success"), f"失败: {result.get('error')}"


@pytest.mark.asyncio
async def test_search(handler):
    """搜索"""
    result = await handler.handle_search("美食", count=3)
    assert result.get("success"), f"失败: {result.get('error')}"
