import { invoke } from "@tauri-apps/api/core";
import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";
import type {
  ApiResponse, PostDetailResponse, DownloadResult,
  CommentItem, MusicItem, FollowItem, LiveInfo,
  LiveRecordTask, FollowingLiveItem,
  DownloadMode, DownloadTask, TaskItem, TaskItemCounts, DownloadTaskDetail,
} from "./api-types";
import type { DownloadRecord, DownloadStats, LiveRecord, VideoInfo, UserInfo, VideoStats, UserStats, TrendPoint, AuthorStat, StorageStat, DbHealth } from "./tauri-types";

// ============================================================
// 辅助：包装后端返回值到 ApiResponse.data
// ============================================================

/** 后端（Python/Rust）返回的原始响应结构 */
interface BackendResponse {
  success: boolean;
  data?: unknown;
  error?: string;
  [key: string]: unknown;
}

function wrap<T = unknown>(raw: BackendResponse): ApiResponse<T> {
  if (raw && typeof raw === "object" && "success" in raw) {
    const hasDataKey = "data" in raw;
    return {
      success: raw.success,
      data: (hasDataKey ? raw.data : raw) as T,
      error: raw.error,
    };
  }
  return { success: false, error: "无效的响应格式" };
}

// ============================================================
// 配置
// ============================================================

export interface AppConfig {
  cookie: string;
  download_path: string;
  naming: string;
  encryption: string;
  proxy: string;
  app_name: string;
  folderize: boolean;
  music: boolean;
  cover: boolean;
  desc: boolean;
  interval: string | null;
  page_counts: number;
  max_counts: number;
  timeout: number;
  max_connections: number;
  max_retries: number;
  max_tasks: number;
}

export async function getConfig(): Promise<AppConfig> {
  return invoke("get_config");
}

export async function setConfig(updates: Record<string, string>): Promise<void> {
  await invoke("set_config", { updates });
}

// ============================================================
// 文件管理（tauri-plugin-opener）
// ============================================================

/** 在系统文件管理器中打开路径 */
export async function openFolder(path: string): Promise<void> {
  await openPath(path);
}

/** 在系统文件管理器中定位到文件 */
export async function revealInFolder(path: string): Promise<void> {
  await revealItemInDir(path);
}

/** 导出数据为 JSON 文件 */
export async function exportData(dataType: string, savePath: string): Promise<string> {
  return invoke("export_data", { data_type: dataType, save_path: savePath });
}

// ============================================================
// 视频解析 & 下载
// ============================================================

export async function getPostDetail(url: string): Promise<ApiResponse<PostDetailResponse>> {
  try {
    return wrap(await invoke("py_parse_video", { url }));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

export async function getPostStats(url: string): Promise<ApiResponse> {
  try {
    return wrap(await invoke("py_get_post_stats", { url }));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

export async function downloadOne(url: string): Promise<ApiResponse<DownloadResult>> {
  try {
    return wrap(await invoke("py_download_video", { url }));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

/** 通用批量下载入口，返回 task_id 供页面订阅进度 */
export async function startBatchDownload(download_type: string, url: string): Promise<ApiResponse & { task_id?: string }> {
  try {
    const raw = await invoke<BackendResponse>("py_start_batch_download", { url, download_type });
    let taskId: string | undefined;
    if (raw?.success && raw.task_id != null) {
      taskId = String(raw.task_id);
      const { useTaskStore } = await import("@/stores/task-store");
      useTaskStore.getState().addTask({
        task_id: taskId,
        task_type: "batch",
        type: download_type,
        url,
        status: "starting",
        total: 0,
        completed: 0,
        failed: 0,
        skipped: 0,
        current_item: "",
        error: "",
      });
    }
    return { ...wrap(raw), task_id: taskId };
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

export const downloadUserPosts = (url: string) => startBatchDownload("user_post", url);
export const downloadUserLikes = (url: string) => startBatchDownload("user_like", url);
export const downloadMix = (url: string) => startBatchDownload("mix", url);
export const downloadCollectsVideo = (collectsId: string) => startBatchDownload("collects", collectsId);

/** 统一下载入口（通过 mode 分发） */
export async function startDownload(mode: DownloadMode, url: string): Promise<ApiResponse & { task_id?: string }> {
  try {
    const raw = await invoke<BackendResponse>("py_start_download", { mode, url });
    let taskId: string | undefined;
    if (raw?.success && raw.task_id != null) {
      taskId = String(raw.task_id);
      const { useTaskStore } = await import("@/stores/task-store");
      useTaskStore.getState().addTask({
        task_id: taskId,
        task_type: "batch",
        type: mode,
        url,
        status: "starting",
        total: 0,
        completed: 0,
        failed: 0,
        skipped: 0,
        current_item: "",
        error: "",
      });
    }
    return { ...wrap(raw), task_id: taskId };
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

// ============================================================
// 下载任务查询（DB 直接调用）
// ============================================================

export async function getDownloadTasks(limit = 50, offset = 0, status?: string, mode?: string): Promise<DownloadTask[]> {
  return invoke("get_download_tasks", { limit, offset, status: status || null, mode: mode || null });
}

export async function getDownloadTaskDetail(taskId: string): Promise<DownloadTaskDetail | null> {
  return invoke("get_download_task_detail", { task_id: taskId });
}

export async function getDownloadTaskItems(taskId: string, status?: string): Promise<TaskItem[]> {
  return invoke("get_download_task_items", { task_id: taskId, status: status || null });
}

export async function getDownloadTaskItemCounts(taskId: string): Promise<TaskItemCounts> {
  return invoke("get_download_task_item_counts", { task_id: taskId });
}

export async function deleteDownloadTask(taskId: string): Promise<void> {
  return invoke("delete_download_task", { task_id: taskId });
}

// ============================================================
// 评论
// ============================================================

export async function getComments(url: string, cursor = 0, count = 20): Promise<ApiResponse<{ comments: CommentItem[]; has_more: boolean; cursor: number }>> {
  try {
    return wrap(await invoke("py_get_comments", { url, cursor, count }));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

export async function getCommentReplies(url: string, commentId: string, cursor = 0, count = 3): Promise<ApiResponse<{ comments: CommentItem[]; has_more: boolean; cursor: number }>> {
  try {
    return wrap(await invoke("py_get_comment_replies", { url, comment_id: commentId, cursor, count }));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

// ============================================================
// 搜索
// ============================================================

export async function search(keyword: string, offset = 0, count = 10): Promise<ApiResponse<PostDetailResponse>> {
  try {
    return wrap(await invoke("py_search_videos", { keyword, offset, count }));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

// ============================================================
// Feed
// ============================================================

export async function getTabFeed(count = 10): Promise<ApiResponse<PostDetailResponse>> {
  try {
    return wrap(await invoke("py_get_tab_feed", { count }));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

export async function getFollowFeed(cursor = 0, count = 10): Promise<ApiResponse<PostDetailResponse>> {
  try {
    return wrap(await invoke("py_get_follow_feed", { cursor, count }));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

export async function getFriendFeed(cursor = 0, count = 10): Promise<ApiResponse<PostDetailResponse>> {
  try {
    return wrap(await invoke("py_get_friend_feed", { cursor, count }));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

// ============================================================
// 音乐
// ============================================================

export async function getMusicCollection(cursor = 0, count = 18): Promise<ApiResponse<{ music_list: MusicItem[]; has_more: boolean }>> {
  try {
    return wrap(await invoke("py_get_music_collection", { cursor, count }));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

export async function downloadMusic(play_url: string, title: string, author = ""): Promise<ApiResponse<{ path: string }>> {
  try {
    return wrap(await invoke("py_download_music", { play_url, title, author }));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

// ============================================================
// 用户
// ============================================================

export async function getUserProfile(url: string): Promise<ApiResponse<PostDetailResponse>> {
  try {
    return wrap(await invoke("py_get_user_profile", { url }));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

export async function getUserPosts(url: string, cursor: number = 0, count: number = 20): Promise<ApiResponse<PostDetailResponse>> {
  try {
    return wrap(await invoke("py_get_user_posts", { url, cursor, count }));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

export async function getUserLikes(url: string, cursor: number = 0, count: number = 20): Promise<ApiResponse<PostDetailResponse>> {
  try {
    return wrap(await invoke("py_get_user_likes", { url, cursor, count }));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

export async function getUserCollects(): Promise<ApiResponse<PostDetailResponse>> {
  try {
    return wrap(await invoke("py_get_collects_list"));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

export async function getCollectsVideoList(collectsId: string, cursor: number = 0, count: number = 20): Promise<ApiResponse<PostDetailResponse>> {
  try {
    return wrap(await invoke("py_get_collects_video_list", { collects_id: collectsId, cursor, count }));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

export async function getUserFollowing(url: string, offset = 0, count = 20): Promise<ApiResponse<{ followings: FollowItem[]; has_more: boolean }>> {
  try {
    return wrap(await invoke("py_get_following_list", { url, offset, count }));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

export async function getUserFollowers(url: string, offset = 0, count = 20): Promise<ApiResponse<{ followers: FollowItem[]; has_more: boolean }>> {
  try {
    return wrap(await invoke("py_get_follower_list", { url, offset, count }));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

// ============================================================
// 直播
// ============================================================

export async function getLiveInfo(url: string): Promise<ApiResponse<LiveInfo>> {
  try {
    return wrap(await invoke("py_get_live_info", { url }));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

// ============================================================
// 合集
// ============================================================

export async function getMixInfo(url: string, cursor: number = 0, count: number = 20): Promise<ApiResponse<PostDetailResponse>> {
  try {
    return wrap(await invoke("py_get_mix_info", { url, cursor, count }));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

// ============================================================
// 直播录制
// ============================================================

export async function startLiveRecord(url: string): Promise<ApiResponse<{ task_id: string }>> {
  try {
    return wrap(await invoke("py_start_live_record", { url }));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

export async function stopLiveRecord(taskId: string): Promise<ApiResponse<{ task_id: string }>> {
  try {
    return wrap(await invoke("py_stop_live_record", { task_id: taskId }));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

export async function getLiveStatus(): Promise<ApiResponse<Record<string, LiveRecordTask>>> {
  try {
    return wrap(await invoke("py_get_live_status"));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}

/** @deprecated 直播记录已由后端 live_manager 自动保存，此函数不再使用 */
export async function saveLiveRecordAfterStop(task: LiveRecordTask): Promise<void> {
  if (!task.file) return;
  await invoke("save_live_record_record", {
    record: {
      room_id: task.room_id || "",
      web_rid: task.web_rid || null,
      title: task.title || "",
      nickname: task.nickname || "",
      sec_user_id: null,
      file_path: task.file,
      file_size: task.file_size || 0,
      duration_sec: task.duration_sec || 0,
      status: "completed",
      started_at: task.started_at || 0,
      ended_at: task.ended_at || 0,
      cover_url: task.cover_url || null,
    },
  });
}

export async function getFollowingLive(): Promise<ApiResponse<{ lives: FollowingLiveItem[] }>> {
  try {
    return wrap(await invoke("py_get_following_live"));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}


// ============================================================
// 数据库查询 (Tauri 直接调用)
// ============================================================

export async function getDownloads(params: {
  limit?: number;
  offset?: number;
  status?: string;
  download_type?: string;
}): Promise<DownloadRecord[]> {
  return invoke("get_downloads", {
    limit: params.limit ?? 20,
    offset: params.offset ?? 0,
    status: params.status ?? null,
    download_type: params.download_type ?? null,
  });
}

export async function getDownloadStats(): Promise<DownloadStats> {
  return invoke("get_download_stats");
}

export async function getLiveRecords(params: {
  limit?: number;
  offset?: number;
}): Promise<LiveRecord[]> {
  return invoke("get_live_records", {
    limit: params.limit ?? 20,
    offset: params.offset ?? 0,
  });
}

export async function getLiveRecordCount(): Promise<number> {
  return invoke("get_live_record_count");
}

// === video_info / user_info 查询 ===

export async function getVideos(params: {
  limit?: number;
  offset?: number;
  keyword?: string;
  author_sec_uid?: string;
  sort_by?: string;
  sort_order?: string;
  post_type?: string;
}): Promise<VideoInfo[]> {
  return invoke("get_videos", {
    limit: params.limit ?? 20,
    offset: params.offset ?? 0,
    keyword: params.keyword ?? null,
    author_sec_uid: params.author_sec_uid ?? null,
    sort_by: params.sort_by ?? null,
    sort_order: params.sort_order ?? null,
    post_type: params.post_type ?? null,
  });
}

export async function getVideoCount(params?: {
  keyword?: string;
  author_sec_uid?: string;
  post_type?: string;
}): Promise<number> {
  return invoke("get_video_count", {
    keyword: params?.keyword ?? null,
    author_sec_uid: params?.author_sec_uid ?? null,
    post_type: params?.post_type ?? null,
  });
}

export async function getUsers(params: {
  limit?: number;
  offset?: number;
  keyword?: string;
  sort_by?: string;
  sort_order?: string;
}): Promise<UserInfo[]> {
  return invoke("get_users", {
    limit: params.limit ?? 20,
    offset: params.offset ?? 0,
    keyword: params.keyword ?? null,
    sort_by: params.sort_by ?? null,
    sort_order: params.sort_order ?? null,
  });
}

export async function getUserCount(params?: {
  keyword?: string;
}): Promise<number> {
  return invoke("get_user_count", {
    keyword: params?.keyword ?? null,
  });
}

export async function getUserBySecUid(secUserId: string): Promise<UserInfo | null> {
  return invoke("get_user_by_sec_uid", { sec_user_id: secUserId });
}

export async function getVideoStats(): Promise<VideoStats> {
  return invoke("get_video_stats");
}

export async function getUserStats(): Promise<UserStats> {
  return invoke("get_user_stats");
}

export async function getDownloadTrend(range: string): Promise<TrendPoint[]> {
  return invoke("get_download_trend", { range });
}

export async function getTopAuthors(limit = 10): Promise<AuthorStat[]> {
  return invoke("get_top_authors", { limit });
}

export async function getStorageAnalysis(): Promise<StorageStat[]> {
  return invoke("get_storage_analysis");
}

export async function dbHealthCheck(dbPath: string): Promise<DbHealth> {
  return invoke("db_health_check", { db_path: dbPath });
}

export async function getDbPath(): Promise<string> {
  return invoke("get_db_path");
}

export async function isVideoDownloaded(awemeId: string): Promise<boolean> {
  return invoke("is_video_downloaded", { aweme_id: awemeId });
}

// === 音乐收藏 ===

export interface MusicCollectionItem {
  music_id: string;
  mid: string | null;
  title: string | null;
  author: string | null;
  owner_nickname: string | null;
  duration: number;
  cover: string | null;
  play_url: string | null;
  file_path: string | null;
  status: string;
  created_at: number;
}

export interface NewMusicCollectionItem {
  music_id: string;
  mid?: string;
  title?: string;
  author?: string;
  owner_nickname?: string;
  duration: number;
  cover?: string;
  play_url?: string;
}

export async function getMusicCollectionFromDB(params: {
  limit?: number;
  offset?: number;
  keyword?: string;
  status?: string;
}): Promise<MusicCollectionItem[]> {
  return invoke("get_music_collection", {
    limit: params.limit ?? 20,
    offset: params.offset ?? 0,
    keyword: params.keyword ?? null,
    status: params.status ?? null,
  });
}

export async function getMusicCollectionCountFromDB(keyword?: string, status?: string): Promise<number> {
  return invoke("get_music_collection_count", { keyword: keyword ?? null, status: status ?? null });
}

export async function saveMusicCollection(music: NewMusicCollectionItem): Promise<void> {
  return invoke("save_music_collection", { music });
}

export async function saveMusicCollectionBatch(musics: NewMusicCollectionItem[]): Promise<void> {
  return invoke("save_music_collection_batch", { musics });
}

export async function updateMusicFilePath(musicId: string, filePath: string): Promise<void> {
  return invoke("update_music_file_path", { music_id: musicId, file_path: filePath });
}

export async function deleteDownloadRecord(id: number, deleteFile = false): Promise<void> {
  return invoke("delete_download_record", { id, delete_file: deleteFile });
}

export async function deleteLiveRecord(id: number, deleteFile = false): Promise<void> {
  return invoke("delete_live_record", { id, delete_file: deleteFile });
}

export async function deleteVideoInfo(awemeId: string): Promise<void> {
  return invoke("delete_video_info", { aweme_id: awemeId });
}

export async function deleteUserInfo(secUserId: string): Promise<void> {
  return invoke("delete_user_info", { sec_user_id: secUserId });
}

export async function deleteMusicCollection(musicId: string, deleteFile = false): Promise<void> {
  return invoke("delete_music_collection", { music_id: musicId, delete_file: deleteFile });
}

// ============================================================
// 测试
// ============================================================

export async function testEmit(): Promise<ApiResponse> {
  try {
    return wrap(await invoke("py_test_emit"));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}
