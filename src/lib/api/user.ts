import type { ApiResponse, PostDetailResponse, FollowItem } from "../api-types";
import { pyCall } from "./core";

// ============================================================
// 用户
// ============================================================

export async function getUserProfile(url: string): Promise<ApiResponse<PostDetailResponse>> {
  return pyCall("py_get_user_profile", { url });
}

export async function getUserPosts(url: string, cursor: number = 0, count: number = 20): Promise<ApiResponse<PostDetailResponse>> {
  return pyCall("py_get_user_posts", { url, cursor, count });
}

export async function getUserLikes(url: string, cursor: number = 0, count: number = 20): Promise<ApiResponse<PostDetailResponse>> {
  return pyCall("py_get_user_likes", { url, cursor, count });
}

export async function getUserCollects(): Promise<ApiResponse<PostDetailResponse>> {
  return pyCall("py_get_collects_list");
}

export async function getCollectsVideoList(collectsId: string, cursor: number = 0, count: number = 20): Promise<ApiResponse<PostDetailResponse>> {
  return pyCall("py_get_collects_video_list", { collects_id: collectsId, cursor, count });
}

export async function getUserFollowing(url: string, offset = 0, count = 20): Promise<ApiResponse<{ followings: FollowItem[]; has_more: boolean }>> {
  return pyCall("py_get_following_list", { url, offset, count });
}

export async function getUserFollowers(url: string, offset = 0, count = 20): Promise<ApiResponse<{ followers: FollowItem[]; has_more: boolean }>> {
  return pyCall("py_get_follower_list", { url, offset, count });
}
