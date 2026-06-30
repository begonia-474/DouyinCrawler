import { useQuery, useInfiniteQuery } from "@tanstack/react-query";
import { getDownloadStats, getDownloads, getLiveRecordCount, getLiveRecords, getMusicCollectionCountFromDB, getMusicCollectionFromDB, getUserCount, getUserStats, getUsers, getVideoCount, getVideos, getVideoStats, getDownloadTasks, getDownloadTaskDetail, getDownloadTaskItems, getDownloadTaskItemCounts, getDownloadTrend, getTopAuthors, getStorageAnalysis, dbHealthCheck, getPostDetail, getTabFeed, getFollowFeed, getFriendFeed, getUserCollects, getFollowingLive, getUserProfile, getUserPosts, getUserFollowing, getUserFollowers, getUserLikes, getCollectsVideoList, getMixInfo, getLiveInfo, getLiveStatus } from "@/lib/api";
import { queryKeys } from "@/lib/query-keys";

export function useDownloadsQuery(params: {
  limit?: number;
  offset?: number;
  status?: string;
  download_type?: string;
}) {
  return useQuery({
    queryKey: queryKeys.downloads(params),
    queryFn: () => getDownloads(params),
  });
}

export function useDownloadStatsQuery() {
  return useQuery({
    queryKey: queryKeys.downloadStats(),
    queryFn: () => getDownloadStats(),
  });
}

export function useLiveRecordsQuery(params: { limit?: number; offset?: number }) {
  return useQuery({
    queryKey: queryKeys.liveRecords(params),
    queryFn: () => getLiveRecords(params),
  });
}

export function useLiveRecordCountQuery() {
  return useQuery({
    queryKey: queryKeys.liveRecordCount(),
    queryFn: () => getLiveRecordCount(),
  });
}

export function useVideoStatsQuery() {
  return useQuery({
    queryKey: queryKeys.videoStats(),
    queryFn: () => getVideoStats(),
  });
}

export function useVideosQuery(params: {
  limit?: number;
  offset?: number;
  keyword?: string;
  author_sec_uid?: string;
  sort_by?: string;
  sort_order?: string;
  post_type?: string;
}) {
  return useQuery({
    queryKey: queryKeys.videos(params),
    queryFn: () => getVideos(params),
  });
}

export function useUserStatsQuery() {
  return useQuery({
    queryKey: queryKeys.userStats(),
    queryFn: () => getUserStats(),
  });
}

export function useVideoCountQuery(params: {
  keyword?: string;
  post_type?: string;
}) {
  return useQuery({
    queryKey: queryKeys.videoCount(params),
    queryFn: () => getVideoCount(params),
  });
}

export function useMusicCollectionQuery(params: {
  limit?: number;
  offset?: number;
  keyword?: string;
  status?: string;
}) {
  return useQuery({
    queryKey: queryKeys.musicCollection(params),
    queryFn: () => getMusicCollectionFromDB(params),
  });
}

export function useMusicCountQuery(params: { keyword?: string; status?: string }) {
  return useQuery({
    queryKey: queryKeys.musicCount(params),
    queryFn: () => getMusicCollectionCountFromDB(params.keyword, params.status),
  });
}

export function useUsersQuery(params: {
  limit?: number;
  offset?: number;
  keyword?: string;
  sort_by?: string;
  sort_order?: string;
}) {
  return useQuery({
    queryKey: queryKeys.users(params),
    queryFn: () => getUsers(params),
  });
}

export function useUserCountQuery(params?: { keyword?: string }) {
  return useQuery({
    queryKey: queryKeys.userCount(params ?? {}),
    queryFn: () => getUserCount(params),
  });
}

export function useDownloadTasksQuery(params?: { limit?: number; status?: string; mode?: string }) {
  return useQuery({
    queryKey: queryKeys.downloadTasks(params ?? {}),
    queryFn: () => getDownloadTasks(params?.limit, 0, params?.status, params?.mode),
  });
}

export function useDownloadTaskDetailQuery(taskId: string) {
  return useQuery({
    queryKey: queryKeys.downloadTaskDetail(taskId),
    queryFn: () => getDownloadTaskDetail(taskId),
    enabled: !!taskId,
  });
}

export function useDownloadTaskItemsQuery(taskId: string, status?: string) {
  return useQuery({
    queryKey: queryKeys.downloadTaskItems(taskId, status),
    queryFn: () => getDownloadTaskItems(taskId, status),
    enabled: !!taskId,
  });
}

export function useDownloadTaskItemCountsQuery(taskId: string) {
  return useQuery({
    queryKey: queryKeys.downloadTaskItemCounts(taskId),
    queryFn: () => getDownloadTaskItemCounts(taskId),
    enabled: !!taskId,
  });
}

export function useDownloadTrendQuery(range: string) {
  return useQuery({
    queryKey: queryKeys.downloadTrend(range),
    queryFn: () => getDownloadTrend(range),
  });
}

export function useTopAuthorsQuery(limit = 10) {
  return useQuery({
    queryKey: queryKeys.topAuthors(limit),
    queryFn: () => getTopAuthors(limit),
  });
}

export function useStorageAnalysisQuery() {
  return useQuery({
    queryKey: queryKeys.storageAnalysis(),
    queryFn: () => getStorageAnalysis(),
  });
}

export function useDbHealthQuery() {
  return useQuery({
    queryKey: queryKeys.dbHealth(),
    queryFn: () => dbHealthCheck(),
  });
}

/** 视频解析（React Query 缓存，按 URL 去重） */
export function useVideoParseQuery(url: string | null) {
  return useQuery({
    queryKey: queryKeys.videoParse(url ?? ""),
    queryFn: () => getPostDetail(url!),
    enabled: !!url,
    staleTime: 5 * 60 * 1000, // 5 分钟内不重新请求
  });
}

// ============================================================
// Feed 查询
// ============================================================

/** 推荐 Feed */
export function useTabFeedQuery(enabled = true) {
  return useQuery({
    queryKey: queryKeys.tabFeed(),
    queryFn: () => getTabFeed(),
    enabled,
    staleTime: 2 * 60 * 1000,
  });
}

/** 关注 Feed */
export function useFollowFeedQuery(enabled = true) {
  return useQuery({
    queryKey: queryKeys.followFeed(),
    queryFn: () => getFollowFeed(),
    enabled,
    staleTime: 2 * 60 * 1000,
  });
}

/** 朋友 Feed */
export function useFriendFeedQuery(enabled = true) {
  return useQuery({
    queryKey: queryKeys.friendFeed(),
    queryFn: () => getFriendFeed(),
    enabled,
    staleTime: 2 * 60 * 1000,
  });
}

// ============================================================
// 收藏夹 / 直播
// ============================================================

/** 我的收藏夹列表 */
export function useCollectsListQuery() {
  return useQuery({
    queryKey: queryKeys.collectsList(),
    queryFn: () => getUserCollects(),
    staleTime: 5 * 60 * 1000,
  });
}

/** 关注直播列表 */
export function useFollowingLiveQuery(enabled = true) {
  return useQuery({
    queryKey: queryKeys.followingLive(),
    queryFn: () => getFollowingLive(),
    enabled,
    staleTime: 30 * 1000, // 直播数据 30s 缓存
  });
}

// ============================================================
// 用户页面
// ============================================================

/** 用户资料 */
export function useUserProfileQuery(url: string | null) {
  return useQuery({
    queryKey: queryKeys.userProfile(url ?? ""),
    queryFn: () => getUserProfile(url!),
    enabled: !!url,
    staleTime: 5 * 60 * 1000,
  });
}

/** 用户作品（无限滚动） */
export function useUserPostsInfiniteQuery(url: string | null) {
  return useInfiniteQuery({
    queryKey: queryKeys.userPosts(url ?? ""),
    queryFn: ({ pageParam }) => getUserPosts(url!, pageParam, 20),
    initialPageParam: 0,
    getNextPageParam: (lastPage) => {
      if (!lastPage?.data) return undefined;
      return lastPage.data.has_more ? (lastPage.data.next_cursor ?? 0) : undefined;
    },
    enabled: !!url,
    staleTime: 2 * 60 * 1000,
  });
}

/** 用户关注列表 */
export function useUserFollowingQuery(url: string | null) {
  return useQuery({
    queryKey: queryKeys.userFollowing(url ?? ""),
    queryFn: () => getUserFollowing(url!),
    enabled: !!url,
    staleTime: 5 * 60 * 1000,
  });
}

/** 用户粉丝列表 */
export function useUserFollowersQuery(url: string | null) {
  return useQuery({
    queryKey: queryKeys.userFollowers(url ?? ""),
    queryFn: () => getUserFollowers(url!),
    enabled: !!url,
    staleTime: 5 * 60 * 1000,
  });
}

/** 用户点赞（无限滚动） */
export function useUserLikesInfiniteQuery(url: string | null) {
  return useInfiniteQuery({
    queryKey: queryKeys.userLikes(url ?? ""),
    queryFn: ({ pageParam }) => getUserLikes(url!, pageParam, 20),
    initialPageParam: 0,
    getNextPageParam: (lastPage) => {
      if (!lastPage?.data) return undefined;
      return lastPage.data.has_more ? (lastPage.data.next_cursor ?? 0) : undefined;
    },
    enabled: !!url,
    staleTime: 2 * 60 * 1000,
  });
}

// ============================================================
// 收藏夹详情
// ============================================================

/** 收藏夹视频（无限滚动） */
export function useCollectsVideosInfiniteQuery(id: string | null) {
  return useInfiniteQuery({
    queryKey: queryKeys.collectsVideos(id ?? ""),
    queryFn: ({ pageParam }) => getCollectsVideoList(id!, pageParam, 20),
    initialPageParam: 0,
    getNextPageParam: (lastPage) => {
      if (!lastPage?.data) return undefined;
      return lastPage.data.has_more ? (lastPage.data.next_cursor ?? 0) : undefined;
    },
    enabled: !!id,
    staleTime: 2 * 60 * 1000,
  });
}

// ============================================================
// 合集
// ============================================================

/** 合集信息 */
export function useMixInfoQuery(url: string | null) {
  return useQuery({
    queryKey: queryKeys.mixInfo(url ?? ""),
    queryFn: () => getMixInfo(url!, 0, 1),
    enabled: !!url,
    staleTime: 5 * 60 * 1000,
  });
}

/** 合集视频（无限滚动） */
export function useMixVideosInfiniteQuery(url: string | null) {
  return useInfiniteQuery({
    queryKey: queryKeys.mixInfo(url ?? "", { scope: "videos" }),
    queryFn: ({ pageParam }) => getMixInfo(url!, pageParam, 20),
    initialPageParam: 0,
    getNextPageParam: (lastPage) => {
      if (!lastPage?.data) return undefined;
      return lastPage.data.has_more ? (lastPage.data.next_cursor ?? 0) : undefined;
    },
    enabled: !!url,
    staleTime: 2 * 60 * 1000,
  });
}

// ============================================================
// 直播
// ============================================================

/** 直播信息 */
export function useLiveInfoQuery(url: string | null) {
  return useQuery({
    queryKey: queryKeys.liveInfo(url ?? ""),
    queryFn: () => getLiveInfo(url!),
    enabled: !!url,
    staleTime: 30 * 1000,
  });
}

/** 直播录制状态 */
export function useLiveStatusQuery() {
  return useQuery({
    queryKey: queryKeys.liveStatus(),
    queryFn: () => getLiveStatus(),
    refetchInterval: 5000,
  });
}
