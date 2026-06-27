import { invoke } from "@tauri-apps/api/core";
import type { ApiResponse, MusicItem } from "../api-types";
import { pyCall } from "./core";

// ============================================================
// 音乐（Python API 调用）
// ============================================================

export async function getMusicCollection(cursor = 0, count = 18): Promise<ApiResponse<{ music_list: MusicItem[]; has_more: boolean }>> {
  return pyCall("py_get_music_collection", { cursor, count });
}

export async function downloadMusic(play_url: string, title: string, author = ""): Promise<ApiResponse<{ path: string }>> {
  return pyCall("py_download_music", { play_url, title, author });
}

// ============================================================
// 音乐收藏（DB 直接调用）
// ============================================================

export interface MusicCollectionItem {
  music_id: string;
  mid: string | null;
  title: string | null;
  author: string | null;
  owner_nickname: string | null;
  duration: number;
  cover: string | null;
  play_url: string | null;
  file_path: string | null;
  status: string;
  created_at: number;
}

export interface NewMusicCollectionItem {
  music_id: string;
  mid?: string;
  title?: string;
  author?: string;
  owner_nickname?: string;
  duration: number;
  cover?: string;
  play_url?: string;
}

export async function getMusicCollectionFromDB(params: {
  limit?: number;
  offset?: number;
  keyword?: string;
  status?: string;
}): Promise<MusicCollectionItem[]> {
  return invoke("get_music_collection", {
    limit: params.limit ?? 20,
    offset: params.offset ?? 0,
    keyword: params.keyword ?? null,
    status: params.status ?? null,
  });
}

export async function getMusicCollectionCountFromDB(keyword?: string, status?: string): Promise<number> {
  return invoke("get_music_collection_count", { keyword: keyword ?? null, status: status ?? null });
}

export async function saveMusicCollection(music: NewMusicCollectionItem): Promise<void> {
  return invoke("save_music_collection", { music });
}

export async function saveMusicCollectionBatch(musics: NewMusicCollectionItem[]): Promise<void> {
  return invoke("save_music_collection_batch", { musics });
}

export async function updateMusicFilePath(musicId: string, filePath: string): Promise<void> {
  return invoke("update_music_file_path", { music_id: musicId, file_path: filePath });
}

export async function deleteMusicCollection(musicId: string, deleteFile = false): Promise<void> {
  return invoke("delete_music_collection", { music_id: musicId, delete_file: deleteFile });
}
