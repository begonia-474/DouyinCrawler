import type { ApiResponse, TabFeedResult, UserPostsResult } from "../api-types";
import { pyCall } from "./core";

// ============================================================
// Feed
// ============================================================

export async function getTabFeed(count = 10): Promise<ApiResponse<TabFeedResult>> {
  return pyCall("py_get_tab_feed", { count });
}

export async function getFollowFeed(cursor = 0, count = 10): Promise<ApiResponse<UserPostsResult>> {
  return pyCall("py_get_follow_feed", { cursor, count });
}

export async function getFriendFeed(cursor = 0, count = 10): Promise<ApiResponse<UserPostsResult>> {
  return pyCall("py_get_friend_feed", { cursor, count });
}
