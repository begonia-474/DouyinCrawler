# DouyinCrawler

抖音内容爬虫 - 轻量级、高效的抖音数据采集工具

## 功能特性

- 单视频下载
- 用户主页视频批量下载
- 用户点赞视频下载
- 用户收藏视频下载
- 收藏夹视频下载
- 合集视频下载
- 直播信息获取
- 评论获取
- 搜索功能
- 关注/粉丝列表获取

## 安装

```bash
pip install -r requirements.txt
```

## 使用方法

```python
import asyncio
from core.handler import DouyinHandler

async def main():
    handler = DouyinHandler(
        cookie="your_cookie_here",
        download_path="Download",
    )

    # 下载单个视频
    result = await handler.handle_one_video("https://www.douyin.com/video/xxx")
    print(result)

asyncio.run(main())
```

## 项目结构

```
DouyinCrawler/
├── core/
│   ├── api.py              # API 端点定义
│   ├── config.py           # 配置管理
│   ├── config_manager.py   # 客户端配置管理器
│   ├── crawler.py          # HTTP 爬虫引擎
│   ├── db.py               # 数据库层
│   ├── downloader.py       # 下载器
│   ├── filter.py           # 响应数据过滤
│   ├── handler.py          # 业务处理器
│   ├── models.py           # 请求参数模型
│   ├── utils.py            # 工具函数
│   ├── signature/          # 签名算法
│   │   ├── abogus.py       # ABogus 签名
│   │   ├── xbogus.py       # XBogus 签名
│   │   ├── fingerprint.py  # 浏览器指纹生成
│   │   └── manager.py      # 签名管理器
│   └── tokens/             # Token 管理
│       └── token_manager.py
├── test/                   # 测试文件
├── requirements.txt        # 依赖列表
└── README.md              # 项目说明
```

## 致谢

本项目的签名算法和部分核心代码参考了 [f2](https://github.com/Johnserf-Seed/f2) 项目。

f2 是一个功能强大的社交媒体数据采集工具，支持多个平台。本项目专注于抖音平台，采用了 f2 的核心签名算法和部分实现，以保持与抖音反爬机制的同步更新。

感谢 f2 项目的作者 [Johnserf-Seed](https://github.com/Johnserf-Seed) 和所有贡献者的辛勤工作。

## 许可证

MIT License
