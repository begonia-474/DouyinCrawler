import type { ApiResponse, SearchResult } from "../api-types";
import { pyCall } from "./core";

// ============================================================
// 搜索（待实现 UI，后端已就绪）
// ============================================================

export async function search(keyword: string, offset = 0, count = 10): Promise<ApiResponse<SearchResult>> {
  return pyCall("py_search_videos", { keyword, offset, count });
}
