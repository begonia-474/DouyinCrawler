import type { ApiResponse, VideoParseResult } from "../api-types";
import { pyCall } from "./core";

// ============================================================
// 视频解析
// ============================================================

export async function getPostDetail(url: string): Promise<ApiResponse<VideoParseResult>> {
  return pyCall("py_parse_video", { url });
}

export async function getPostStats(url: string): Promise<ApiResponse> {
  return pyCall("py_get_post_stats", { url });
}
