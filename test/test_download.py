"""端到端测试：获取视频详情 + 下载"""

import asyncio
from pathlib import Path

import yaml

from core.crawler import DouyinCrawler
from core.filter import PostDetailFilter
from core.downloader import Downloader, format_filename

# 从 yaml 配置加载 cookie
CONFIG_PATH = Path(__file__).resolve().parent.parent / "docs" / "抖音相关" / "app.yaml"
with open(CONFIG_PATH, encoding="utf-8") as f:
    _cfg = yaml.safe_load(f)
COOKIE = _cfg["douyin"]["cookie"]

TEST_VIDEO_URL = "https://www.douyin.com/video/7650450403901017571"
DOWNLOAD_DIR = str(Path(__file__).resolve().parent.parent / "Download")


async def main():
    # 从 URL 提取 aweme_id
    aweme_id = TEST_VIDEO_URL.split("/video/")[-1].split("?")[0]
    print(f"[1] aweme_id: {aweme_id}")

    # 获取视频详情
    print("[2] 获取视频详情...")
    async with DouyinCrawler(cookie=COOKIE, encryption="ab") as crawler:
        try:
            data = await crawler.fetch_post_detail(aweme_id)
        except Exception as e:
            print(f"[ERROR] 请求失败: {e}")
            return

    # 解析响应
    if data.get("status_code", -1) != 0:
        print(f"[ERROR] API 返回错误: status_code={data.get('status_code')}")
        print(f"  status_msg: {data.get('status_msg', 'unknown')}")
        return

    detail = PostDetailFilter(data)
    print(f"[3] 视频信息:")
    print(f"  标题: {detail.desc}")
    print(f"  作者: {detail.author_nickname}")
    print(f"  点赞: {detail.digg_count}")
    print(f"  评论: {detail.comment_count}")
    print(f"  视频URL: {detail.video_url[:80]}..." if detail.video_url else "  视频URL: 空")

    if not detail.video_url:
        print("[ERROR] 无法获取视频下载链接")
        return

    # 下载视频
    print("[4] 开始下载...")
    filename = format_filename("{create}_{desc}", detail.to_dict())
    print(f"  文件名: {filename}.mp4")

    async with Downloader(cookie=COOKIE) as dl:
        progress = {"last": 0}

        def on_progress(task_id, downloaded, total):
            if total > 0:
                pct = downloaded * 100 // total
                if pct >= progress["last"] + 10:
                    progress["last"] = pct
                    print(f"  进度: {pct}% ({downloaded // 1024}KB / {total // 1024}KB)")

        dl.progress_callback = on_progress
        try:
            path = await dl.download_video(
                video_url=detail.video_url,
                save_dir=DOWNLOAD_DIR,
                filename=filename,
            )
            print(f"[5] 下载完成: {path}")
        except Exception as e:
            print(f"[ERROR] 下载失败: {e}")


if __name__ == "__main__":
    asyncio.run(main())
