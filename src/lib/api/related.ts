import type { ApiResponse } from "../api-types";
import { pyCall } from "./core";

// ============================================================
// 相关推荐
// ============================================================

export interface RelatedResult {
  success: boolean;
  count: number;
  has_more: boolean;
  filter_gids: string;
  videos: Record<string, unknown>[];
}

/**
 * 获取相关推荐视频（单页，前端控制分页）
 *
 * @param url - 视频 URL
 * @param count - 每页数量（默认 20）
 * @param filterGids - 已看过的 aweme_id 逗号列表（首次为空，后续传上次返回的 filter_gids）
 */
export async function getRelated(url: string, count = 20, filterGids = ""): Promise<ApiResponse<RelatedResult>> {
  return pyCall("py_get_related", { url, count, filter_gids: filterGids });
}
