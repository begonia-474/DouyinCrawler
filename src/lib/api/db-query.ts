import { invoke } from "@tauri-apps/api/core";
import type {
  LiveRecord, VideoInfo, UserInfo,
  VideoStats, UserStats, TrendPoint, AuthorStat, StorageStat, DbHealth,
} from "../tauri-types";

// ============================================================
// 数据库查询 (Tauri 直接调用)
// ============================================================

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

export async function dbHealthCheck(): Promise<DbHealth> {
  return invoke("db_health_check");
}

export async function getDbPath(): Promise<string> {
  return invoke("get_db_path");
}

