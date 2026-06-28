import { invoke } from "@tauri-apps/api/core";
import type { ApiResponse, PostDetailResponse, DownloadResult, DownloadMode } from "../api-types";
import { wrap, pyCall, type BackendResponse } from "./core";

// ============================================================
// 视频下载 & 批量下载
// ============================================================

/**
 * 单视频下载 — 委托给 startDownload("one", url)。
 * 所有下载路径统一走 Rust TaskApplicationService。
 */
export async function downloadOne(url: string): Promise<ApiResponse<DownloadResult>> {
  return startDownload("one", url) as Promise<ApiResponse<DownloadResult>>;
}

/** 通用批量下载入口，返回 task_id 供页面订阅进度。所有模式走 Rust-owned start_download。 */
export async function startBatchDownload(download_type: string, url: string): Promise<ApiResponse & { task_id?: string }> {
  try {
    const typeToMode: Record<string, DownloadMode> = {
      user_post: "post",
      user_like: "like",
      mix: "mix",
      collects: "collects",
    };
    const mode = typeToMode[download_type] ?? download_type;
    const raw = await invoke<BackendResponse>("start_download", { mode, url });
    let taskId: string | undefined;
    if (raw?.success && raw.task_id != null) {
      taskId = String(raw.task_id);
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
    // one/music/post/like/mix/collects 走 Rust-owned 路径，live 暂走 Python
    const rustModes = new Set(["one", "music", "post", "like", "mix", "collects"]);
    const command = rustModes.has(mode) ? "start_download" : "py_start_download";
    const raw = await invoke<BackendResponse>(command, { mode, url });
    let taskId: string | undefined;
    if (raw?.success && raw.task_id != null) {
      taskId = String(raw.task_id);
    }
    return { ...wrap(raw), task_id: taskId };
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}
