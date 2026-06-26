import { useQuery } from "@tanstack/react-query";
import { getDownloadStats, getDownloads, getLiveRecordCount, getLiveRecords, getMusicCollectionCountFromDB, getMusicCollectionFromDB, getUserCount, getUserStats, getUsers, getVideoCount, getVideos, getVideoStats } from "@/lib/api";
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
