# API 文档

本文档列出前端通过 Tauri IPC 可调用的所有命令。调用方式统一为：

```typescript
import { invoke } from "@tauri-apps/api/core";
const result = await invoke<ReturnType>("command_name", { param1, param2 });
```

项目中封装在 `src/lib/api/` 下，推荐使用封装函数而非直接 invoke。

---

## 配置 (`src/lib/api/config.ts`)

### `get_config`

获取应用配置。

- **参数**：无
- **返回**：`AppConfig`

### `set_config`

更新应用配置。

- **参数**：`key: string`, `value: unknown`
- **返回**：`void`

### `get_db_path`

获取数据库文件路径。

- **参数**：无
- **返回**：`string`

---

## Python 爬虫命令 (`src/lib/api/video.ts`, `user.ts`, `live.ts`, `feed.ts`, `comment.ts`, `search.ts`, `related.ts`, `music.ts`)

所有 `py_*` 命令通过 PyO3 调用 Python 爬虫函数，返回 JSON 数据。

### 视频

#### `py_parse_video`

解析单个视频。

- **参数**：`url: string`
- **返回**：`VideoParseResult`（视频详情、作者信息、统计数据）

#### `py_get_post_stats`

获取视频统计数据。

- **参数**：`aweme_id: string`
- **返回**：`PostStats`

### 用户

#### `py_get_user_profile`

获取用户资料。

- **参数**：`sec_uid: string`
- **返回**：`UserProfileResult`

#### `py_get_user_posts`

获取用户作品列表（分页）。

- **参数**：`sec_uid: string`, `cursor?: number`, `count?: number`
- **返回**：`UserPostsResult`（含 `has_more`、`cursor`）

#### `py_get_user_likes`

获取用户点赞列表（分页）。

- **参数**：`sec_uid: string`, `cursor?: number`, `count?: number`
- **返回**：`UserLikesResult`

### 直播

#### `py_get_live_info`

获取直播信息。

- **参数**：`url: string`
- **返回**：`LiveInfoResult`

#### `py_get_following_live`

获取关注直播列表。

- **参数**：`cursor?: number`, `count?: number`
- **返回**：`FollowingLiveResult`

### Feed

#### `py_get_tab_feed`

获取推荐 Feed。

- **参数**：`cursor?: number`, `count?: number`
- **返回**：`TabFeedResult`

#### `py_get_follow_feed`

获取关注 Feed。

- **参数**：`cursor?: number`, `count?: number`
- **返回**：`FollowFeedResult`

#### `py_get_friend_feed`

获取朋友 Feed。

- **参数**：`cursor?: number`, `count?: number`
- **返回**：`FriendFeedResult`

### 评论

#### `py_get_comments`

获取视频评论（分页）。

- **参数**：`aweme_id: string`, `cursor?: number`, `count?: number`
- **返回**：`CommentsResult`

#### `py_get_comment_replies`

获取评论回复（分页）。

- **参数**：`aweme_id: string`, `comment_id: string`, `cursor?: number`, `count?: number`
- **返回**：`CommentRepliesResult`

### 搜索

#### `py_search_videos`

搜索视频。

- **参数**：`keyword: string`, `cursor?: number`, `count?: number`
- **返回**：`SearchResult`

### 推荐

#### `py_get_related`

获取相关推荐。

- **参数**：`aweme_id: string`, `cursor?: number`, `count?: number`
- **返回**：`RelatedResult`

### 合集与收藏

#### `py_get_mix_info`

获取合集信息。

- **参数**：`mix_id: string`
- **返回**：`MixInfoResult`

#### `py_get_collects_list`

获取收藏夹列表。

- **参数**：`cursor?: number`, `count?: number`
- **返回**：`CollectsListResult`

#### `py_get_collects_video_list`

获取收藏夹视频列表。

- **参数**：`favorites_id: string`, `cursor?: number`, `count?: number`
- **返回**：`CollectsVideoListResult`

### 关注/粉丝

#### `py_get_following_list`

获取关注列表。

- **参数**：`sec_uid: string`, `cursor?: number`, `count?: number`
- **返回**：`FollowingListResult`

#### `py_get_follower_list`

获取粉丝列表。

- **参数**：`sec_uid: string`, `cursor?: number`, `count?: number`
- **返回**：`FollowerListResult`

### 音乐

#### `py_get_music_collection`

获取音乐合集。

- **参数**：`music_id: string`, `cursor?: number`, `count?: number`
- **返回**：`MusicCollectionResult`

#### `py_download_music`

下载单首音乐。

- **参数**：`music_id: string`, `save_path?: string`
- **返回**：`MusicDownloadResult`

---

## 下载任务 (`src/lib/api/download.ts`)

### `start_download`

创建并启动下载任务。

- **参数**：`mode: DownloadMode`, `url: string`, `aweme_ids?: string[]`
- **返回**：`DownloadTask`
- **说明**：`mode` 可选值：`"single"` | `"post"` | `"like"` | `"favorite"` | `"mix"` | `"collect"` | `"feed"` | `"music"` | `"related"`

### `cancel_task`

取消下载任务。

- **参数**：`task_id: string`
- **返回**：`void`

### `start_live_record`

启动直播录制。

- **参数**：`url: string`
- **返回**：`DownloadTask`

### `stop_live_record`

停止直播录制。

- **参数**：`task_id: string`
- **返回**：`void`

### `get_live_status`

获取直播录制状态。

- **参数**：`task_id: string`
- **返回**：`LiveStatus`

---

## 下载任务 CRUD (`src/lib/api/download-task.ts`)

### `create_download_task`

创建下载任务记录。

- **参数**：`task: CreateDownloadTaskParams`
- **返回**：`DownloadTask`

### `get_download_tasks`

获取下载任务列表。

- **参数**：`status?: TaskStatus`, `limit?: number`, `offset?: number`
- **返回**：`DownloadTask[]`

### `get_download_task_detail`

获取任务详情（含子任务项）。

- **参数**：`task_id: string`
- **返回**：`DownloadTaskDetail`

### `delete_download_task`

删除下载任务。

- **参数**：`task_id: string`
- **返回**：`void`

---

## 数据库查询 (`src/lib/api/db-query.ts`)

### 视频

#### `get_videos`

分页查询视频列表。

- **参数**：`limit?: number`, `offset?: number`, `sort_by?: string`, `order?: "asc" | "desc"`
- **返回**：`VideoInfo[]`

#### `get_video_count`

获取视频总数。

- **参数**：无
- **返回**：`number`

### 用户

#### `get_users`

分页查询用户列表。

- **参数**：`limit?: number`, `offset?: number`, `sort_by?: string`, `order?: "asc" | "desc"`
- **返回**：`UserInfo[]`

#### `get_user_count`

获取用户总数。

- **参数**：无
- **返回**：`number`

#### `get_user_by_sec_uid`

按 sec_uid 查询用户。

- **参数**：`sec_uid: string`
- **返回**：`UserInfo | null`

### 直播记录

#### `get_live_records`

分页查询直播记录。

- **参数**：`limit?: number`, `offset?: number`
- **返回**：`LiveRecord[]`

#### `get_live_record_count`

获取直播记录总数。

- **参数**：无
- **返回**：`number`

### 音乐合集

#### `get_music_collection`

分页查询音乐合集。

- **参数**：`limit?: number`, `offset?: number`
- **返回**：`MusicCollection[]`

#### `get_music_collection_count`

获取音乐合集总数。

- **参数**：无
- **返回**：`number`

---

## 统计 (`src/lib/api/db-query.ts`)

### `get_video_stats`

获取视频统计。

- **参数**：无
- **返回**：`VideoStats`

### `get_user_stats`

获取用户统计。

- **参数**：无
- **返回**：`UserStats`

### `get_download_trend`

获取下载趋势（按天）。

- **参数**：`days?: number`
- **返回**：`DownloadTrend[]`

### `get_top_authors`

获取热门作者排行。

- **参数**：`limit?: number`
- **返回**：`TopAuthor[]`

### `get_storage_analysis`

获取存储分析。

- **参数**：无
- **返回**：`StorageAnalysis`

### `db_health_check`

数据库健康检查。

- **参数**：无
- **返回**：`HealthCheckResult`

### `export_data`

导出数据。

- **参数**：`format: string`, `path?: string`
- **返回**：`string`（导出文件路径）

---

## 文件操作 (`src/lib/api/file.ts`)

### `openFolder`

打开文件夹（系统文件管理器）。

- **参数**：`path: string`
- **返回**：`void`

### `exportData`

导出数据到文件。

- **参数**：`data: unknown`, `filename: string`
- **返回**：`string`（文件路径）

---

## 删除操作 (`src/lib/api/delete.ts`)

### 单条删除

- `delete_video_info(aweme_id: string)` → `void`
- `delete_user_info(sec_uid: string)` → `void`
- `delete_live_record(id: string)` → `void`
- `delete_music_collection(id: string)` → `void`

### 批量删除

- `delete_video_info_batch(aweme_ids: string[])` → `void`
- `delete_user_info_batch(sec_uids: string[])` → `void`
- `delete_live_record_batch(ids: string[])` → `void`
- `delete_music_collection_batch(ids: string[])` → `void`

---

## Firefox Cookie (`src/lib/api/config.ts`)

### `get_firefox_profiles_command`

获取 Firefox 配置文件列表。

- **参数**：无
- **返回**：`FirefoxProfile[]`

### `get_douyin_cookie_command`

从 Firefox 获取抖音 Cookie。

- **参数**：无
- **返回**：`string`

### `get_douyin_cookie_from_profile_command`

从指定 Firefox 配置文件获取抖音 Cookie。

- **参数**：`profile_path: string`
- **返回**：`string`

### `get_firefox_cookie_command`

获取 Firefox Cookie。

- **参数**：`profile_path: string`, `domain?: string`
- **返回**：`string`

---

## 前端封装层 (`src/lib/`)

### React Query Hooks (`queries.ts`)

推荐使用封装好的 hooks 而非直接调用 invoke：

```typescript
// 查询
useVideoParse(url)
useUserProfile(secUid)
useUserPosts(secUid, cursor)
useLiveInfo(url)
useComments(awemeId, cursor)
useTabFeed(cursor)
useFollowingLive(cursor)
useMusicCollection(musicId, cursor)

// 变更
useDeleteVideo()
useDeleteUser()
useStartDownload()
useCancelTask()
useStartLiveRecord()
useStopLiveRecord()
```

### Query Keys (`query-keys.ts`)

统一的 query key 工厂，确保缓存一致性：

```typescript
queryKeys.video.parse(url)
queryKeys.user.profile(secUid)
queryKeys.user.posts(secUid, cursor)
queryKeys.live.info(url)
queryKeys.feed.tab(cursor)
queryKeys.task.list()
queryKeys.task.detail(taskId)
```
