import { invoke } from "@tauri-apps/api/core";
import type { ApiResponse, LiveInfo, LiveRecordTask, FollowingLiveItem } from "../api-types";
import { pyCall, wrap, type BackendResponse } from "./core";

// ============================================================
// 直播 & 直播录制
// ============================================================

function invokeError(error: unknown, fallback: string): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string" && error) return error;
  return fallback;
}

export async function getLiveInfo(url: string): Promise<ApiResponse<LiveInfo>> {
  const response = await pyCall<{ live_info?: LiveInfo }>("py_get_live_info", { url });
  if (!response.success) return { success: false, error: response.error };
  const liveInfo = response.data?.live_info;
  if (!liveInfo) return { success: false, error: "直播信息响应缺少 live_info" };
  return { success: true, data: liveInfo };
}

export async function startLiveRecord(url: string): Promise<ApiResponse<{ task_id: string }>> {
  try {
    return wrap(await invoke<BackendResponse>("start_live_record", { url }));
  } catch (e) {
    return { success: false, error: invokeError(e, "启动录制失败") };
  }
}

export async function stopLiveRecord(taskId: string): Promise<ApiResponse<{ task_id: string }>> {
  try {
    return wrap(await invoke<BackendResponse>("stop_live_record", { task_id: taskId }));
  } catch (e) {
    return { success: false, error: invokeError(e, "停止录制失败") };
  }
}

export async function getLiveStatus(): Promise<ApiResponse<Record<string, LiveRecordTask>>> {
  try {
    return wrap(await invoke<BackendResponse>("get_live_status"));
  } catch (e) {
    return { success: false, error: invokeError(e, "获取录制状态失败") };
  }
}

export async function getFollowingLive(): Promise<ApiResponse<{ lives: FollowingLiveItem[] }>> {
  return pyCall("py_get_following_live");
}
