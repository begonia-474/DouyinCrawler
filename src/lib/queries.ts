import { useQuery } from "@tanstack/react-query";
import { getDownloadStats, getDownloads, getLiveRecordCount, getLiveRecords, getMusicCollectionCountFromDB, getMusicCollectionFromDB, getUserCount, getUserStats, getUsers, getVideoCount, getVideos, getVideoStats, getDownloadTasks, getDownloadTaskDetail, getDownloadTaskItems, getDownloadTaskItemCounts, getDownloadTrend, getTopAuthors, getStorageAnalysis, dbHealthCheck } from "@/lib/api";
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
