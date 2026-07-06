import { invoke } from "@tauri-apps/api/core";

// ============================================================
// 删除操作（DB 直接调用）
// ============================================================

export async function deleteLiveRecord(id: number, deleteFile = false): Promise<void> {
  return invoke("delete_live_record", { id, delete_file: deleteFile });
}

export async function deleteVideoInfo(awemeId: string): Promise<void> {
  return invoke("delete_video_info", { aweme_id: awemeId });
}

export async function deleteUserInfo(secUserId: string, deleteFile = false): Promise<void> {
  return invoke("delete_user_info", { sec_user_id: secUserId, delete_file: deleteFile });
}

// ============================================================
// 批量删除（事务保证原子性）
// ============================================================

export async function deleteVideoInfoBatch(awemeIds: string[]): Promise<void> {
  return invoke("delete_video_info_batch", { aweme_ids: awemeIds });
}

export async function deleteUserInfoBatch(secUserIds: string[], deleteFile = false): Promise<void> {
  return invoke("delete_user_info_batch", { sec_user_ids: secUserIds, delete_file: deleteFile });
}

export async function deleteLiveRecordBatch(ids: number[], deleteFile = false): Promise<void> {
  return invoke("delete_live_record_batch", { ids, delete_file: deleteFile });
}

export async function deleteMusicCollectionBatch(musicIds: string[], deleteFile = false): Promise<void> {
  return invoke("delete_music_collection_batch", { music_ids: musicIds, delete_file: deleteFile });
}
