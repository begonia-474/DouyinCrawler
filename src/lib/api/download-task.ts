import { invoke } from "@tauri-apps/api/core";
import type { DownloadTask, TaskItem, TaskItemCounts, DownloadTaskDetail } from "../api-types";

// ============================================================
// 下载任务查询（DB 直接调用）
// ============================================================

export async function getDownloadTasks(limit = 50, offset = 0, status?: string, mode?: string): Promise<DownloadTask[]> {
  return invoke("get_download_tasks", { limit, offset, status: status || null, mode: mode || null });
}

export async function getDownloadTaskDetail(taskId: string): Promise<DownloadTaskDetail | null> {
  return invoke("get_download_task_detail", { task_id: taskId });
}

export async function getDownloadTaskItems(taskId: string, status?: string): Promise<TaskItem[]> {
  return invoke("get_download_task_items", { task_id: taskId, status: status || null });
}

export async function getDownloadTaskItemCounts(taskId: string): Promise<TaskItemCounts> {
  return invoke("get_download_task_item_counts", { task_id: taskId });
}

export async function deleteDownloadTask(taskId: string): Promise<void> {
  return invoke("delete_download_task", { task_id: taskId });
}
