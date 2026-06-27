import type { ApiResponse, CommentItem } from "../api-types";
import { pyCall } from "./core";

// ============================================================
// 评论（待实现 UI，后端已就绪）
// ============================================================

export async function getComments(url: string, cursor = 0, count = 20): Promise<ApiResponse<{ comments: CommentItem[]; has_more: boolean; cursor: number }>> {
  return pyCall("py_get_comments", { url, cursor, count });
}

export async function getCommentReplies(url: string, commentId: string, cursor = 0, count = 3): Promise<ApiResponse<{ comments: CommentItem[]; has_more: boolean; cursor: number }>> {
  return pyCall("py_get_comment_replies", { url, comment_id: commentId, cursor, count });
}
