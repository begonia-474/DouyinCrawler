export interface ApiResponse<T = unknown> {
  success: boolean;
  data?: T;
  error?: string;
}

export interface PostDetail {
  type?: string;
  title?: string;
  desc?: string;
  author?: string;
  duration?: number;
  aweme_id?: string;
  awemeId?: string;
  video_url?: string;
  images?: string[];
  path?: string;
  paths?: string[];
  digg_count?: number;
  comment_count?: number;
  share_count?: number;
  collect_count?: number;
  is_image_post?: boolean;
}

export interface VideoItem {
  aweme_id: string;
  desc: string;
  author: string;
  duration: number;
  digg_count: number;
  comment_count: number;
  share_count: number;
  collect_count?: number;
  cover_url: string;
  create_time?: number;
}

// ============================================================
// 领域特定响应类型（拆分自 PostDetailResponse）
// ============================================================

export interface VideoParseResult {
  detail?: PostDetail;
  path?: string;
  paths?: string[];
}

export interface UserProfileResult {
  profile?: UserProfile;
}

export interface UserPostsResult {
  videos?: VideoItem[];
  has_more?: boolean;
  cursor?: number;
  next_cursor?: number;
}

export interface MixInfoResult {
  videos?: VideoItem[];
  has_more?: boolean;
  cursor?: number;
  next_cursor?: number;
  detail?: { desc?: string };
}

export interface CommentsResult {
  comments?: CommentItem[];
  has_more?: boolean;
  cursor?: number;
}

export interface FollowingListResult {
  followings?: FollowItem[];
  has_more?: boolean;
  cursor?: number;
}

export interface FollowerListResult {
  followers?: FollowItem[];
  has_more?: boolean;
  cursor?: number;
}

export interface CollectsListResult {
  collects?: CollectsFolder[];
}

export interface MusicCollectionResult {
  music_list?: MusicItem[];
  has_more?: boolean;
  cursor?: number;
}

export interface TabFeedResult {
  videos?: VideoItem[];
  has_more?: boolean;
  cursor?: number;
  next_cursor?: number;
}

export interface SearchResult {
  videos?: VideoItem[];
  has_more?: boolean;
  cursor?: number;
  count?: number;
}

/** @deprecated 使用领域特定类型替代，保留用于向后兼容 */
export interface PostDetailResponse {
  success?: boolean;
  type?: string;
  detail?: PostDetail;
  path?: string;
  paths?: string[];
  videos?: VideoItem[];
  profile?: UserProfile;
  comments?: CommentItem[];
  has_more?: boolean;
  cursor?: number;
  next_cursor?: number;
  music_list?: MusicItem[];
  followings?: FollowItem[];
  followers?: FollowItem[];
  collects?: CollectsFolder[];
}

export interface UserProfile {
  nickname: string;
  avatar: string;
  follower_count: number;
  following_count: number;
  aweme_count: number;
  total_favorited: number;
  signature: string;
  sec_user_id: string;
}

export interface CommentItem {
  cid: string;
  text: string;
  user: {
    nickname: string;
    avatar: string;
  };
  digg_count: number;
  create_time: number;
  reply_comment_total: number;
  replies?: CommentItem[];
}

export interface MusicItem {
  music_id: string;
  mid: string;
  title: string;
  author: string;
  owner_nickname: string;
  duration: number;
  cover: string;
  play_url: string;
}

export interface FollowItem {
  uid: string;
  nickname: string;
  avatar: string;
  signature: string;
  follower_count: number;
}

export interface CollectsFolder {
  id: string;
  name: string;
  count: number;
}

export interface LiveInfo {
  title: string;
  nickname: string;
  is_live: boolean;
  user_count: number;
  room_id: string;
  cover: string;
  flv_urls: string[];
  m3u8_urls: string[];
}

export interface LiveRecordTask {
  task_id: string;
  status: string;
  url?: string;
  file?: string;
  room_id?: string;
  web_rid?: string;
  title?: string;
  nickname?: string;
  file_size?: number;
  duration_sec?: number;
  started_at?: number;
  ended_at?: number;
  cover_url?: string;
  error?: string;
}

export interface FollowingLiveItem {
  web_rid: string;
  room_id: string;
  title: string;
  nickname: string;
  avatar: string;
  cover: string;
  user_count: number;
  tag_name: string;
}

// ============================================================
// 统一任务系统
// ============================================================

export type DownloadMode = "one" | "post" | "like" | "mix" | "collects" | "live" | "music";

export interface DownloadTask {
  id: string;
  mode: DownloadMode;
  url: string;
  title: string | null;
  author_nickname: string | null;
  status: "running" | "starting" | "completed" | "error" | "cancelled" | "recording" | "stopping";
  total: number;
  completed: number;
  skipped: number;
  failed: number;
  error_msg: string | null;
  created_at: number;
  updated_at: number;
}

export interface TaskItem {
  id: number;
  task_id: string;
  aweme_id: string | null;
  title: string | null;
  author_nickname: string | null;
  cover_url: string | null;
  file_path: string | null;
  file_size: number;
  status: "pending" | "downloading" | "completed" | "skipped" | "failed";
  error_msg: string | null;
  created_at: number;
}

export interface TaskItemCounts {
  total: number;
  completed: number;
  skipped: number;
  failed: number;
  pending: number;
}

export interface DownloadTaskDetail {
  task: DownloadTask;
  items: TaskItem[];
}

export interface DownloadResult {
  type: "video" | "images";
  path?: string;
  paths?: string[];
  detail?: PostDetail;
}

// ============================================================
// 类型化任务事件系统（对齐 Rust src/tasks/mod.rs）
// ============================================================

/** 任务状态（对齐 Rust TaskStatus 枚举） */
export type TaskStatus = "pending" | "starting" | "running" | "recording" | "stopping" | "completed" | "error" | "cancelled";

/** 任务子项状态（对齐 Rust TaskItemStatus 枚举） */
export type TaskItemStatus = "pending" | "downloading" | "completed" | "skipped" | "failed";

/** 任务事件类型（对齐 Rust TaskEventType 枚举） */
export type TaskEventType = "started" | "progress" | "finished";

/** 任务补丁（对齐 Rust TaskPatch）None 字段不覆盖现有值 */
export interface TaskPatch {
  task_id: string;
  status?: TaskStatus;
  total?: number;
  completed?: number;
  skipped?: number;
  failed?: number;
  error_msg?: string;
  current_item?: string;
}

/** 类型化任务事件（对齐 Rust TaskEvent，通过 Tauri event 发射） */
export interface TaskEvent {
  event_type: TaskEventType;
  task_id: string;
  mode?: DownloadMode;
  url?: string;
  /** 补丁字段（通过 serde(flatten) 展开） */
  patch: TaskPatch;
}

// ============================================================
// 错误码（对齐 Rust ErrorCode 枚举）
// ============================================================

/** 错误码枚举（对齐 Rust ErrorCode） */
export type ErrorCode =
  // 网络层
  | "network_timeout"
  | "network_error"
  | "rate_limited"
  | "proxy_error"
  // 认证层
  | "cookie_expired"
  | "cookie_invalid"
  | "login_required"
  // 内容层
  | "video_not_found"
  | "user_not_found"
  | "content_deleted"
  // 处理层
  | "signature_error"
  | "parse_error"
  // 系统层
  | "database_error"
  | "file_system_error"
  | "config_error"
  // 未知
  | "unknown";

/** 错误码分类 */
export type ErrorCategory = "network" | "auth" | "content" | "processing" | "system" | "unknown";

/** 根据错误码判断是否可重试 */
export function isRetryable(code: ErrorCode): boolean {
  const retryable: ErrorCode[] = [
    "network_timeout", "network_error", "rate_limited", "proxy_error",
    "signature_error", "database_error", "unknown",
  ];
  return retryable.includes(code);
}

/** 根据错误码判断是否需要跳转设置（Cookie 配置） */
export function needsSettingsRedirect(code: ErrorCode): boolean {
  return code === "cookie_expired" || code === "cookie_invalid" || code === "login_required";
}
