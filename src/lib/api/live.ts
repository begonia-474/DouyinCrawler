import type { ApiResponse, LiveInfo, LiveRecordTask, FollowingLiveItem } from "../api-types";
import { pyCall } from "./core";

// ============================================================
// 直播 & 直播录制
// ============================================================

export async function getLiveInfo(url: string): Promise<ApiResponse<LiveInfo>> {
  return pyCall("py_get_live_info", { url });
}

export async function startLiveRecord(url: string): Promise<ApiResponse<{ task_id: string }>> {
  return pyCall("py_start_live_record", { url });
}

export async function stopLiveRecord(taskId: string): Promise<ApiResponse<{ task_id: string }>> {
  return pyCall("py_stop_live_record", { task_id: taskId });
}

export async function getLiveStatus(): Promise<ApiResponse<Record<string, LiveRecordTask>>> {
  return pyCall("py_get_live_status");
}

export async function getFollowingLive(): Promise<ApiResponse<{ lives: FollowingLiveItem[] }>> {
  return pyCall("py_get_following_live");
}
