import type {
  ApiResponse, PostDetailResponse, DownloadResult,
  CommentItem, MusicItem, FollowItem, LiveInfo,
  LiveRecordTask, FollowingLiveItem,
} from "./api-types";

// 动态端口，由 Tauri sidecar 启动时设置
let BASE_URL = "http://127.0.0.1:8765";

export function setBackendPort(port: number) {
  BASE_URL = `http://127.0.0.1:${port}`;
}

export function getBackendUrl() {
  return BASE_URL;
}

async function request<T>(path: string, options?: RequestInit): Promise<ApiResponse<T>> {
  try {
    const res = await fetch(`${BASE_URL}${path}`, {
      headers: { "Content-Type": "application/json" },
      ...options,
    });
    return await res.json();
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "网络错误" };
  }
}

function post<T>(path: string, body: unknown): Promise<ApiResponse<T>> {
  return request(path, { method: "POST", body: JSON.stringify(body) });
}

// === 健康 & 配置 ===

export async function healthCheck(): Promise<ApiResponse<{ configured: boolean }>> {
  return request("/api/health");
}

export interface ConfigParams {
  cookie?: string;
  download_path?: string;
  naming?: string;
  encryption?: string;
  proxy?: string;
}

export async function updateConfig(params: ConfigParams): Promise<ApiResponse> {
  return post("/api/config", params);
}

// === 视频解析 & 下载 ===

export async function getPostDetail(url: string): Promise<ApiResponse<PostDetailResponse>> {
  return post("/api/post/detail", { url });
}

export async function getPostStats(url: string): Promise<ApiResponse> {
  return post("/api/post/stats", { url });
}

export async function downloadOne(url: string): Promise<ApiResponse<DownloadResult>> {
  return post("/api/download/one", { url });
}

// === 评论 ===

export async function getComments(url: string, cursor = 0, count = 20): Promise<ApiResponse<{ comments: CommentItem[]; has_more: boolean; cursor: number }>> {
  return post("/api/comments", { url, cursor, count });
}

export async function getCommentReplies(url: string, commentId: string, cursor = 0, count = 3): Promise<ApiResponse<{ comments: CommentItem[]; has_more: boolean; cursor: number }>> {
  return post("/api/comments/reply", { url, comment_id: commentId, cursor, count });
}

// === 搜索 ===

export async function search(keyword: string, offset = 0, count = 10): Promise<ApiResponse<PostDetailResponse>> {
  return post("/api/search", { keyword, offset, count });
}

// === Feed ===

export async function getTabFeed(count = 10): Promise<ApiResponse<PostDetailResponse>> {
  return post("/api/feed/tab", { count });
}

export async function getFollowFeed(cursor = 0, count = 10): Promise<ApiResponse<PostDetailResponse>> {
  return post("/api/feed/follow", { cursor, count });
}

export async function getFriendFeed(cursor = 0, count = 10): Promise<ApiResponse<PostDetailResponse>> {
  return post("/api/feed/friend", { cursor, count });
}

// === 音乐 ===

export async function getMusicCollection(cursor = 0, count = 18): Promise<ApiResponse<{ music_list: MusicItem[]; has_more: boolean }>> {
  return post("/api/music/collection", { cursor, count });
}

export async function downloadMusic(play_url: string, title: string, author = ""): Promise<ApiResponse<{ path: string }>> {
  return post("/api/music/download", { play_url, title, author });
}

// === 用户 ===

export async function getUserProfile(url: string): Promise<ApiResponse<PostDetailResponse>> {
  return post("/api/user/profile", { url });
}

export async function getUserPosts(url: string): Promise<ApiResponse<PostDetailResponse>> {
  return post("/api/user/posts", { url });
}

export async function getUserLikes(url: string): Promise<ApiResponse<PostDetailResponse>> {
  return post("/api/user/likes", { url });
}

export async function getUserCollects(): Promise<ApiResponse<PostDetailResponse>> {
  return post("/api/user/collects", {});
}

export async function getCollectsVideo(collectsId: string, cursor = 0): Promise<ApiResponse<PostDetailResponse>> {
  return post("/api/user/collects/video", { collects_id: collectsId, cursor });
}

export async function getCollectsVideoList(collectsId: string): Promise<ApiResponse<PostDetailResponse>> {
  return post("/api/user/collects/video/list", { collects_id: collectsId });
}

export async function getUserFollowing(url: string, offset = 0, count = 20): Promise<ApiResponse<{ followings: FollowItem[]; has_more: boolean }>> {
  return post("/api/user/following", { url, offset, count });
}

export async function getUserFollowers(url: string, offset = 0, count = 20): Promise<ApiResponse<{ followers: FollowItem[]; has_more: boolean }>> {
  return post("/api/user/followers", { url, offset, count });
}

// === 直播 ===

export async function getLiveInfo(url: string): Promise<ApiResponse<LiveInfo>> {
  return post("/api/live", { url });
}

// === 合集 ===

export async function getMixInfo(url: string): Promise<ApiResponse<PostDetailResponse>> {
  return post("/api/mix", { url });
}

// === 直播录制 ===

export async function startLiveRecord(url: string): Promise<ApiResponse<{ task_id: string }>> {
  return post("/api/live/record", { url });
}

export async function stopLiveRecord(taskId: string): Promise<ApiResponse<{ task_id: string }>> {
  return post("/api/live/stop", { url: taskId });
}

export async function getLiveStatus(): Promise<ApiResponse<Record<string, LiveRecordTask>>> {
  return request("/api/live/status");
}

export async function getFollowingLive(): Promise<ApiResponse<{ lives: FollowingLiveItem[] }>> {
  return request("/api/live/following");
}
