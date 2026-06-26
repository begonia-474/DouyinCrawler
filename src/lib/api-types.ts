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
