import { invoke } from "@tauri-apps/api/core";

// ============================================================
// 删除操作（DB 直接调用）
// ============================================================

export async function deleteDownloadRecord(id: number, deleteFile = false): Promise<void> {
  return invoke("delete_download_record", { id, delete_file: deleteFile });
}

export async function deleteLiveRecord(id: number, deleteFile = false): Promise<void> {
  return invoke("delete_live_record", { id, delete_file: deleteFile });
}

export async function deleteVideoInfo(awemeId: string): Promise<void> {
  return invoke("delete_video_info", { aweme_id: awemeId });
}

export async function deleteUserInfo(secUserId: string): Promise<void> {
  return invoke("delete_user_info", { sec_user_id: secUserId });
}
