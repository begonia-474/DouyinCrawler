import { invoke } from "@tauri-apps/api/core";
import type { ApiResponse, PostDetailResponse, DownloadResult, DownloadMode } from "../api-types";
import { wrap, pyCall, type BackendResponse } from "./core";

// ============================================================
// 视频下载 & 批量下载
// ============================================================

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

// ============================================================
// 合集信息
// ============================================================

export async function getMixInfo(url: string, cursor: number = 0, count: number = 20): Promise<ApiResponse<PostDetailResponse>> {
  return pyCall("py_get_mix_info", { url, cursor, count });
}

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
