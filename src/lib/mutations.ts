import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  deleteVideoInfo, deleteUserInfo, deleteLiveRecord,
  deleteDownloadTask, deleteMusicCollection,
  deleteVideoInfoBatch, deleteUserInfoBatch, deleteLiveRecordBatch, deleteMusicCollectionBatch,
  downloadMusic, saveMusicCollectionBatch, updateMusicFilePath,
} from "@/lib/api";
import { queryKeys } from "@/lib/query-keys";
import type { NewMusicCollectionItem } from "@/lib/api";

// ============================================================
// 删除操作 mutations
// ============================================================

export function useDeleteVideoInfo() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (awemeId: string) => deleteVideoInfo(awemeId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.videos() });
      qc.invalidateQueries({ queryKey: queryKeys.videoCount() });
      qc.invalidateQueries({ queryKey: queryKeys.videoStats() });
      qc.invalidateQueries({ queryKey: queryKeys.userStats() });
    },
  });
}

export function useDeleteUserInfo() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ secUserId, deleteFile }: { secUserId: string; deleteFile?: boolean }) =>
      deleteUserInfo(secUserId, deleteFile),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.users() });
      qc.invalidateQueries({ queryKey: queryKeys.userCount() });
      qc.invalidateQueries({ queryKey: queryKeys.userStats() });
      // 级联删除会同时清理 video_info 和 live_records
      qc.invalidateQueries({ queryKey: queryKeys.videos() });
      qc.invalidateQueries({ queryKey: queryKeys.videoCount() });
      qc.invalidateQueries({ queryKey: queryKeys.videoStats() });
      qc.invalidateQueries({ queryKey: queryKeys.liveRecords() });
      qc.invalidateQueries({ queryKey: queryKeys.liveRecordCount() });
    },
  });
}

export function useDeleteLiveRecord() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, deleteFile }: { id: number; deleteFile?: boolean }) =>
      deleteLiveRecord(id, deleteFile),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.liveRecords() });
      qc.invalidateQueries({ queryKey: queryKeys.liveRecordCount() });
    },
  });
}

export function useDeleteDownloadTask() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (taskId: string) => deleteDownloadTask(taskId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.downloadTasks() });
      // 前缀匹配：使所有 download-task-detail 缓存失效
      qc.invalidateQueries({ queryKey: ["download-task-detail"] });
    },
  });
}

export function useDeleteMusicCollection() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ musicId, deleteFile }: { musicId: string; deleteFile?: boolean }) =>
      deleteMusicCollection(musicId, deleteFile),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.musicCollection() });
      qc.invalidateQueries({ queryKey: queryKeys.musicCount() });
    },
  });
}

// ============================================================
// 音乐操作 mutations
// ============================================================

export function useDownloadMusic() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ play_url, title, author }: { play_url: string; title: string; author?: string }) =>
      downloadMusic(play_url, title, author),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.musicCollection() });
      qc.invalidateQueries({ queryKey: queryKeys.musicCount() });
    },
  });
}

export function useSaveMusicCollectionBatch() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (musics: NewMusicCollectionItem[]) => saveMusicCollectionBatch(musics),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.musicCollection() });
      qc.invalidateQueries({ queryKey: queryKeys.musicCount() });
    },
  });
}

export function useUpdateMusicFilePath() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ musicId, filePath }: { musicId: string; filePath: string }) =>
      updateMusicFilePath(musicId, filePath),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.musicCollection() });
      qc.invalidateQueries({ queryKey: queryKeys.musicCount() });
    },
  });
}

// ============================================================
// 批量删除 mutations
// ============================================================

export function useDeleteVideoInfoBatch() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (awemeIds: string[]) => deleteVideoInfoBatch(awemeIds),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.videos() });
      qc.invalidateQueries({ queryKey: queryKeys.videoCount() });
      qc.invalidateQueries({ queryKey: queryKeys.videoStats() });
      qc.invalidateQueries({ queryKey: queryKeys.userStats() });
    },
  });
}

export function useDeleteUserInfoBatch() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ secUserIds, deleteFile }: { secUserIds: string[]; deleteFile?: boolean }) =>
      deleteUserInfoBatch(secUserIds, deleteFile),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.users() });
      qc.invalidateQueries({ queryKey: queryKeys.userCount() });
      qc.invalidateQueries({ queryKey: queryKeys.userStats() });
      qc.invalidateQueries({ queryKey: queryKeys.videos() });
      qc.invalidateQueries({ queryKey: queryKeys.videoCount() });
      qc.invalidateQueries({ queryKey: queryKeys.videoStats() });
      qc.invalidateQueries({ queryKey: queryKeys.liveRecords() });
      qc.invalidateQueries({ queryKey: queryKeys.liveRecordCount() });
    },
  });
}

export function useDeleteLiveRecordBatch() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ ids, deleteFile }: { ids: number[]; deleteFile?: boolean }) =>
      deleteLiveRecordBatch(ids, deleteFile),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.liveRecords() });
      qc.invalidateQueries({ queryKey: queryKeys.liveRecordCount() });
    },
  });
}

export function useDeleteMusicCollectionBatch() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ musicIds, deleteFile }: { musicIds: string[]; deleteFile?: boolean }) =>
      deleteMusicCollectionBatch(musicIds, deleteFile),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.musicCollection() });
      qc.invalidateQueries({ queryKey: queryKeys.musicCount() });
    },
  });
}
