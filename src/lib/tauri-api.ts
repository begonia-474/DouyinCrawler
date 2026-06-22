import { invoke } from "@tauri-apps/api/core";
import type { DownloadRecord, DownloadStats, LiveRecord } from "./tauri-types";

/** 获取下载记录（分页 + 筛选） */
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

/** 获取下载统计 */
export async function getDownloadStats(): Promise<DownloadStats> {
  return invoke("get_download_stats");
}

/** 获取直播录制记录（分页） */
export async function getLiveRecords(params: {
  limit?: number;
  offset?: number;
}): Promise<LiveRecord[]> {
  return invoke("get_live_records", {
    limit: params.limit ?? 20,
    offset: params.offset ?? 0,
  });
}
