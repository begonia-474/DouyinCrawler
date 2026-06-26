"""集成测试：获取视频详情 + 下载

需要网络和真实 cookie。
"""

import pytest
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from core.crawler import DouyinCrawler
from core.filter import PostDetailFilter
from core.downloader import Downloader, format_filename

pytestmark = [pytest.mark.integration]

TEST_VIDEO_URL = "https://www.douyin.com/video/7650450403901017571"


@pytest.mark.asyncio
async def test_fetch_and_download(cookie, download_dir):
    """获取视频详情并下载"""
    aweme_id = TEST_VIDEO_URL.split("/video/")[-1].split("?")[0]

    # 获取视频详情
    async with DouyinCrawler(cookie=cookie, encryption="ab") as crawler:
        data = await crawler.fetch_post_detail(aweme_id)

    assert data.get("status_code") == 0, f"API 返回错误: {data.get('status_msg')}"

    detail = PostDetailFilter(data)
    assert detail.desc, "视频描述不应为空"
    assert detail.author_nickname, "作者昵称不应为空"

    if not detail.video_url:
        pytest.skip("无法获取视频下载链接（可能是图文帖）")

    # 下载视频
    filename = format_filename("{create}_{desc}", detail.to_dict())
    async with Downloader(cookie=cookie) as dl:
        path = await dl.download_video(
            video_url=detail.video_url,
            save_dir=download_dir,
            filename=filename,
        )
        assert path, "下载路径不应为空"
        assert Path(path).exists(), f"下载文件不存在: {path}"
