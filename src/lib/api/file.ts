import { invoke } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";

// ============================================================
// 文件管理
// ============================================================

/** 在系统文件管理器中打开路径 */
export async function openFolder(path: string): Promise<void> {
  await openPath(path);
}

/** 导出数据为 JSON 文件 */
export async function exportData(dataType: string, savePath: string): Promise<string> {
  return invoke("export_data", { data_type: dataType, save_path: savePath });
}
