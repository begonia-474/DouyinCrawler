"""集成测试：PostDetailFilter.to_db_dict() 的 author_* 字段验证

需要网络和真实 cookie。
"""

import pytest
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from core.crawler import DouyinCrawler
from core.filter import PostDetailFilter

pytestmark = [pytest.mark.integration]

TEST_VIDEO_ID = "7650450403901017571"


@pytest.mark.asyncio
async def test_author_fields_in_db_dict(cookie):
    """to_db_dict() 应包含 author_* 前缀字段"""
    async with DouyinCrawler(cookie=cookie, encryption="ab") as c:
        data = await c.fetch_post_detail(TEST_VIDEO_ID)

    assert data.get("status_code") == 0, f"API 返回错误: {data.get('status_msg')}"

    detail = PostDetailFilter(data)
    db = detail.to_db_dict()

    # author_* 字段应存在
    assert "author_avatar_url" in db, "缺少 author_avatar_url"
    assert "author_signature" in db, "缺少 author_signature"
    assert "author_follower_count" in db, "缺少 author_follower_count"
    assert "author_aweme_count" in db, "缺少 author_aweme_count"
    assert "author_following_count" in db, "缺少 author_following_count"
    assert "author_total_favorited" in db, "缺少 author_total_favorited"
    assert "author_ip_location" in db, "缺少 author_ip_location"

    # 字段总数应 >= 60
    assert len(db) >= 60, f"字段数 {len(db)}，预期 >= 60"
