# DouyinCrawler Desktop

抖音内容爬虫桌面应用。项目使用 Tauri 2 + React 19 构建桌面界面，Python 爬虫核心通过 PyO3 嵌入 Rust 进程，由 Tauri command 直接调用，不需要独立 HTTP 服务。

## 功能特性

- 单视频解析与下载
- 用户作品、点赞、收藏、收藏夹、合集批量下载
- 直播信息获取与直播录制
- 评论、评论回复、搜索、推荐流、关注流、朋友流获取
- 关注/粉丝列表与关注直播列表获取
- 视频、用户、音乐、下载记录本地 SQLite 管理
- 下载任务与直播录制任务实时进度事件
- 图集、音乐、视频、用户信息分页与排序浏览

## 技术栈

- 前端：React 19、TypeScript、Vite 7、Tailwind CSS 4、shadcn/ui、Zustand、React Query
- 桌面：Tauri 2、Rust、PyO3
- 爬虫核心：Python 3.11+、httpx、pydantic、ABogus/XBogus 签名算法
- 分发：Windows 安装包内嵌 Python 运行时，用户无需额外安装
- 数据库：SQLite、rusqlite、WAL 模式
- 测试：pytest、pytest-asyncio、前端 TypeScript 构建、Cargo check

## 环境准备

Python 使用项目目录下的虚拟环境：

```bash
source .venv/bin/activate
pip install -r requirements.txt -c requirements.lock
```

安装前端依赖：

```bash
pnpm install
```

## 开发

启动完整桌面应用：

```bash
pnpm tauri dev
```

仅启动前端开发服务器：

```bash
pnpm dev
```

## 生产构建

```bash
pnpm tauri build
```

产物在 `src-tauri/target/release/bundle/`。

### Windows 分发包

Windows 分发包内嵌完整的 Python 运行时，用户无需安装 Python。

```bash
# 首次：下载 Python 运行时并安装依赖（只需一次）
pnpm setup:python

# 构建
pnpm tauri build
```

生成 `.msi` 安装包，安装即用。

## 测试与检查

推荐提交前至少运行：

```bash
pnpm build
cd src-tauri && cargo check
cd ..
python -m pytest -m offline
```

完整集成验证需要有效 Cookie 和网络环境：

```bash
PYTHONIOENCODING=utf-8 python test/test_all_modes.py
PYTHONIOENCODING=utf-8 python test/e2e_verify.py
```

## 文档

- [架构文档](docs/reference/architecture.md) — 三层架构详解、核心模块职责、数据流
- [API 文档](docs/reference/api.md) — 所有 Tauri 命令接口说明
- [部署文档](docs/reference/deployment.md) — 环境要求、安装步骤、构建与测试

## 项目结构

```text
DouyinCrawler-desktop/
├── src/                     # React 前端
│   ├── modules/douyin/      # 抖音功能页面
│   ├── components/          # UI 组件
│   ├── hooks/               # 分页、无限滚动等 hooks
│   ├── lib/                 # API、React Query、类型和工具
│   └── stores/              # Zustand 实时任务状态
├── src-tauri/               # Rust/Tauri 后端
│   ├── src/lib.rs           # 应用初始化、共享命令、事件/DB/Python 注入
│   ├── src/commands/        # Tauri commands，按 Python/DB 边界拆分
│   ├── src/db.rs            # SQLite 数据库层与迁移
│   ├── src/config.rs        # 配置管理
│   └── src/python/          # PyO3 桥接封装
├── core/                    # Python 爬虫核心
│   ├── handler.py           # 业务门面，保持 py_bridge 调用接口
│   ├── services/            # 视频、用户、收藏、合集、直播、Feed、音乐等服务
│   ├── crawler.py           # HTTP 爬虫引擎
│   ├── downloader.py        # 下载器
│   ├── filter.py            # 响应数据过滤
│   ├── db.py                # Python 侧 DB 辅助
│   ├── db_bridge.py         # Rust 注入的 DB 写入桥
│   ├── py_bridge.py         # PyO3 调用入口
│   ├── tauri_bridge.py      # Tauri 事件桥
│   ├── task/                # 任务管理
│   │   ├── task_manager.py  # 配置、Handler 生命周期、事件广播门面
│   │   └── live_manager.py  # 直播录制任务
│   └── signature/           # ABogus/XBogus 签名算法
├── scripts/                 # 辅助脚本
├── test/                    # 离线单测和集成验证
├── config/                  # 运行时配置，已忽略
├── data/                    # SQLite 数据库，已忽略
└── Download/                # 下载目录，已忽略
```

## 架构概览

```text
React 前端
  ↓ Tauri invoke
Rust command
  ↓ PyO3
core.py_bridge
  ↓
DouyinHandler / core.services
  ↓
DouyinCrawler / Downloader / SQLite bridge

Python 任务进度
  ↓ core.tauri_bridge.emit()
Tauri event
  ↓
Zustand stores / React Query cache invalidation
```

前端通过 `src/lib/api.ts` 调用 Tauri command。Rust command 按边界拆在 `src-tauri/src/commands/` 中：Python 业务调用走 `commands/python.rs`，数据库 CRUD 走 `commands/db.rs`。Python 侧 `core/handler.py` 是稳定门面，具体业务逻辑位于 `core/services/`。

## 配置与数据

- 运行时配置文件：`config/app.json`
- 默认数据库：`data/douyin.db`
- 默认下载目录：`Download/`
- 前端开发端口：`5173`

`config/app.json` 可能包含有效抖音 Cookie，`config/` 已被 `.gitignore` 忽略，请不要提交或公开 Cookie。

## 类型生成

前端 Tauri 数据类型由 Rust 数据结构生成：

```bash
python scripts/gen_tauri_types.py
```

生成目标为 `src/lib/tauri-types.ts`。

## 致谢

本项目的签名算法和部分核心代码参考了 [f2](https://github.com/Johnserf-Seed/f2) 项目。感谢 f2 项目的作者 [Johnserf-Seed](https://github.com/Johnserf-Seed) 和所有贡献者。

## 许可证

MIT License
