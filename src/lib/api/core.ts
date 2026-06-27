import { invoke } from "@tauri-apps/api/core";
import type { ApiResponse } from "../api-types";

// ============================================================
// 辅助：包装后端返回值到 ApiResponse.data
// ============================================================

/** 后端（Python/Rust）返回的原始响应结构 */
export interface BackendResponse {
  success: boolean;
  data?: unknown;
  error?: string;
  [key: string]: unknown;
}

export function wrap<T = unknown>(raw: BackendResponse): ApiResponse<T> {
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

/** 通用 Python 桥接调用封装 */
export async function pyCall<T = unknown>(command: string, args?: Record<string, unknown>): Promise<ApiResponse<T>> {
  try {
    return wrap(await invoke(command, args));
  } catch (e) {
    return { success: false, error: e instanceof Error ? e.message : "调用失败" };
  }
}
