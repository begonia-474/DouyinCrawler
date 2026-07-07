import { invoke } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";
import { resolve } from "@tauri-apps/api/path";

// ============================================================
// 文件管理
// ============================================================

/** 在系统文件管理器中打开路径 */
export async function openFolder(path: string): Promise<void> {
  const absolutePath = await resolve(path);
  await openPath(absolutePath);
}

/** 根据 music_id 获取音乐文件所在目录 */
export async function getDownloadDirByMusicId(musicId: string): Promise<string | null> {
  return invoke("get_download_dir_by_music_id", { music_id: musicId });
}

/** 根据 aweme_id 获取下载文件所在目录 */
export async function getDownloadDirByAwemeId(awemeId: string): Promise<string | null> {
  return invoke("get_download_dir_by_aweme_id", { aweme_id: awemeId });
}

/** 根据 sec_user_id 获取用户下载目录 */
export async function getUserDownloadDir(secUserId: string): Promise<string | null> {
  return invoke("get_user_download_dir", { sec_user_id: secUserId });
}

/** 导出数据为 JSON 文件 */
export async function exportData(dataType: string, savePath: string): Promise<string> {
  return invoke("export_data", { data_type: dataType, save_path: savePath });
}
