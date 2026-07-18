# 架构文档

## 总体架构

DouyinCrawler Desktop 是一个三层桌面应用：

```
┌─────────────────────────────────────────────┐
│              React 19 前端 (TypeScript)       │
│  Pages / Stores / React Query / shadcn/ui    │
└──────────────────────┬──────────────────────┘
                       │ Tauri invoke (IPC)
                       ▼
┌─────────────────────────────────────────────┐
│            Rust / Tauri 2 后端               │
│  Commands / Services / Database / PyO3 Bridge│
└──────────────────────┬──────────────────────┘
                       │ PyO3 (嵌入式 Python)
                       ▼
┌─────────────────────────────────────────────┐
│            Python 爬虫核心 (core/)            │
│  Handler / Services / Crawler / Signature    │
└─────────────────────────────────────────────┘
```

前端通过 Tauri IPC 调用 Rust command；Rust 通过 PyO3 嵌入 Python 解释器直接调用爬虫函数，无需独立 HTTP 服务。

---

## 前端层 (`src/`)

### 目录结构

```
src/
├── main.tsx                 # 入口：QueryClientProvider 包裹 App
├── App.tsx                  # BrowserRouter + useRoutes + Toaster
├── modules/douyin/          # 抖音功能模块（按页面拆分）
│   ├── route.tsx            # 路由定义（lazy 加载）
│   └── pages/               # 各功能页面
├── shared/                  # 共享路由与页面（downloads、settings）
├── components/
│   ├── ui/                  # shadcn/ui 基础组件（24 个）
│   ├── shared/              # 业务共享组件（video-card、task-card 等）
│   └── layout/              # 布局组件（sidebar、header、status-bar）
├── hooks/                   # 自定义 hooks（分页、无限滚动、任务合并等）
├── stores/                  # Zustand 全局状态
│   ├── app-store.ts         # 主题、下载计数
│   └── task-store.ts        # 实时任务状态（监听 Tauri 事件）
└── lib/
    ├── api/                 # Tauri invoke 封装（按功能拆文件）
    ├── queries.ts           # React Query hooks（15+ 查询）
    ├── mutations.ts         # useMutation hooks（10+ 变更）
    ├── query-keys.ts        # Query key 工厂（20+ key）
    ├── tauri-types.ts       # 自动生成的 Rust→TS 类型
    └── bindings.ts          # tauri-specta 生成的类型
```

### 路由

使用 React Router v7，所有页面 lazy 加载：

| 路径 | 页面 |
|------|------|
| `/douyin` | 首页 / URL 解析 |
| `/douyin/video/:awemeId` | 视频详情 |
| `/douyin/user` | 用户主页 |
| `/douyin/live` | 直播信息 |
| `/douyin/following-live` | 关注直播列表 |
| `/douyin/likes` | 用户点赞 |
| `/douyin/mix` | 合集 |
| `/douyin/feed` | 推荐/关注/朋友流 |
| `/douyin/favorites` | 收藏夹列表 |
| `/douyin/favorites/:id` | 收藏夹详情 |
| `/douyin/music` | 音乐合集 |
| `/douyin/related` | 相关推荐 |
| `/douyin/library` | 本地数据库浏览 |
| `/downloads` | 下载任务历史 |
| `/settings` | 应用设置 |

### 状态管理

- **Zustand**：全局 UI 状态（主题、任务进度）。`task-store.ts` 监听 Tauri `task-update` 事件，增量合并任务状态变更。
- **React Query**：服务端状态缓存（视频、用户、直播、音乐等），30s stale time，5min GC time，任务完成时自动失效。

---

## Rust 后端层 (`src-tauri/src/`)

### 目录结构

```
src-tauri/src/
├── lib.rs                   # 应用初始化：注册所有 command、注入 DB/Python/Config
├── main.rs                  # 入口，调用 lib::run()
├── config.rs                # ConfigManager：读写 config/app.json
├── state.rs                 # AppState：Database + ConfigManager + PythonBridge + cancel_signals
├── error.rs                 # AppError + ErrorCode 枚举
├── commands/
│   ├── python.rs            # 20+ py_* 命令（通过 run_python_blocking 调用 Python）
│   ├── db.rs                # 40+ DB CRUD 命令
│   └── tasks.rs             # 任务系统：start_download、cancel_task、直播录制
├── python/
│   ├── bridge.rs            # PythonBridge：PyO3 初始化、py_to_json_value()
│   ├── handler.rs           # Rust→Python 调用封装（25+ 函数）
│   ├── runtime.rs           # run_python_blocking：spawn_blocking 避免阻塞 tokio
│   ├── config.rs            # 同步 Rust config 到 Python ParsingContext
│   ├── db_bridge.rs         # 注入 save_video_info、save_user_info、has_user 到 Python
│   └── responses.rs         # 类型化响应结构体
├── database/
│   ├── connection.rs        # Database：WAL 模式、事务管理
│   ├── migration.rs         # Schema 迁移（v1-v10）
│   ├── models.rs            # Serde DTO
│   ├── video_repo.rs        # 视频信息 CRUD
│   ├── user_repo.rs         # 用户信息 CRUD
│   ├── task_repo.rs         # 下载任务 CRUD
│   ├── music_repo.rs        # 音乐合集 CRUD
│   ├── download_repo.rs     # 直播记录 CRUD
│   └── stats.rs             # 统计查询（趋势、热门作者、存储分析）
└── services/
    └── download/
        ├── task_service.rs  # TaskApplicationService：任务生命周期（核心）
        ├── engine.rs        # DownloadEngine：reqwest 流式下载、重试、并发
        ├── live.rs          # LiveRecorder：M3U8 HLS 录制
        └── contract.rs      # 下载计划契约类型
```

### 核心职责

| 模块 | 职责 |
|------|------|
| `commands/python.rs` | 前端→Python 的桥梁，将 Tauri command 转发到 Python |
| `commands/db.rs` | 数据库 CRUD，所有 DB 写入的主路径 |
| `commands/tasks.rs` | 下载/录制任务的创建、取消、状态查询 |
| `python/handler.rs` | 封装 PyO3 调用，处理 GIL、类型转换、错误映射 |
| `python/runtime.rs` | `spawn_blocking` 隔离 Python GIL，不阻塞 tokio |
| `python/db_bridge.rs` | Python→Rust DB 写入的唯一通道（仅 3 个函数） |
| `services/download/task_service.rs` | 下载任务全生命周期：创建→解析→下载→完成 |
| `services/download/engine.rs` | HTTP 流式下载：并发控制、指数退避重试、CDN 回退 |
| `services/download/live.rs` | HLS 直播录制 |
| `database/` | SQLite WAL 模式，10 次 schema 迁移，按实体拆分 repo |

---

## Python 爬虫层 (`core/`)

### 目录结构

```
core/
├── bridge/                  # PyO3 桥接层（Python↔Rust 接口）
│   ├── handler.py           # DouyinHandler 门面类
│   ├── py_bridge.py         # 模块级函数供 PyO3 调用
│   ├── parsing_context.py   # ParsingContext：懒加载单例 + 配置
│   └── db_bridge.py         # DB 写入桩（由 Rust 注入实现）
├── crawler_engine/          # HTTP 爬虫核心
│   ├── crawler.py           # DouyinCrawler：httpx 客户端
│   ├── api.py               # DouyinAPIEndpoints：URL 构造
│   ├── filter.py            # 响应过滤器（PostDetailFilter 等）
│   ├── signature/           # ABogus / XBogus 签名算法
│   ├── tokens/              # Token 管理
│   └── services/            # 8 个业务服务
├── models/                  # Pydantic v2 数据模型
│   ├── requests.py          # 请求模型
│   ├── responses.py         # 响应类型 + ErrorCode
│   ├── single_download.py   # SingleDownloadPlanV1 契约
│   ├── paged_download.py    # PagedDownloadPlanV1 契约
│   └── live_record.py       # LivePlanV1 契约
├── download/                # 下载辅助（文件名格式化）
├── utils/                   # 工具函数（sanitize、M3U8 解析、URL 处理）
└── config.py                # Config 类（读取 config/app.json）
```

### 核心职责

| 模块 | 职责 |
|------|------|
| `bridge/handler.py` | 稳定门面，所有 Python 功能的统一入口 |
| `bridge/py_bridge.py` | PyO3 直接调用的模块级函数 |
| `bridge/parsing_context.py` | 懒加载 DouyinHandler 单例，管理配置 |
| `crawler_engine/crawler.py` | httpx 异步 HTTP 客户端 |
| `crawler_engine/signature/` | ABogus/XBogus 请求签名算法 |
| `crawler_engine/services/` | 8 个服务：Video、User、Collection、Mix、Live、Feed、Content、Music |
| `models/` | 版本化下载计划契约（Rust 侧消费） |

### 服务列表

| 服务 | 功能 |
|------|------|
| `VideoService` | 视频解析、作品列表 |
| `UserService` | 用户资料、作品、点赞 |
| `CollectionService` | 收藏夹列表与详情 |
| `MixService` | 合集信息 |
| `LiveService` | 直播信息、M3U8 地址解析 |
| `FeedService` | 推荐流、关注流、朋友流 |
| `ContentService` | 评论、评论回复、相关推荐、统计数据 |
| `MusicService` | 音乐合集、音乐下载 |

---

## 数据流

### 查询流（以视频解析为例）

```
用户输入 URL
  → useVideoParseQuery(url)
  → pyCall("py_parse_video", {url})
  → Tauri IPC → commands/python.rs
  → run_python_blocking() [spawn_blocking 线程]
  → python/handler.rs → PyO3 → core/bridge/py_bridge.py
  → DouyinHandler → VideoService → DouyinCrawler → httpx → 抖音 API
  → PostDetailFilter 过滤响应
  → 返回 dict → py_to_json_value() → serde_json::Value
  → Tauri 序列化 → 前端接收 → React Query 缓存
```

### 下载流

```
前端: startDownload("post", url)
  → invoke("start_download", {mode, url, aweme_ids})
  → TaskApplicationService::start_batch_download()
  → 创建 DB 任务记录
  → tokio::spawn 后台任务：
      1. Python 解析下载计划（URL、文件名、元数据）
      2. DownloadEngine 流式下载（并发 + 重试 + CDN 回退）
      3. 进度事件 → Tauri event → 前端 task-store
      4. DB 写入任务项、视频信息、用户信息
      5. 分页循环直到 has_more=false
  → 最终状态 → React Query 失效
```

### DB 写入路径

有两条 DB 写入路径：

1. **Rust 直写**（主路径）：`commands/db.rs` → `state.db.*()` → rusqlite。所有前端 CRUD、任务生命周期、统计、导出均走此路径。
2. **Python 桥接**（受限）：Python → `db_bridge.py` 桩 → Rust PyO3 闭包 → rusqlite。仅 `save_video_info`、`save_user_info`、`has_user` 三个函数。Python 不直接执行 SQL。

---

## 关键设计模式

### Shim 层

旧导入路径（`core/handler.py`、`core/py_bridge.py` 等）保留为 shim，通过 `sys.modules[__name__] = _real` 重定向到 `core/bridge/` 和 `core/crawler_engine/`。保持向后兼容。

### Rust 接管下载

所有下载和直播录制执行已从 Python 迁移到 Rust（`TaskApplicationService` + `DownloadEngine`）。Python 仅负责解析下载计划（URL、文件名、元数据）。

### 版本化契约

下载计划使用版本化 Pydantic 模型（`SingleDownloadPlanV1`、`PagedDownloadPlanV1`、`LivePlanV1`），序列化为 JSON 由 Rust 侧消费。

### 事件驱动 UI

任务进度通过 `Tauri event → Zustand task-store → React Query invalidation` 流动。前端使用 patch-merge 语义做增量更新。

### 路径安全

所有文件操作经过 `validate_path_in_project()` 校验，防止 `../` 目录遍历攻击。

### GIL 管理

`run_python_blocking()` 使用 `spawn_blocking` 将 Python GIL 持有代码移出 tokio 运行时。`db_bridge.rs` 使用 `py.allow_threads()` 在获取 DB 锁前释放 GIL。
