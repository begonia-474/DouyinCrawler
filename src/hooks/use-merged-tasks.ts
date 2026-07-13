import { useMemo } from "react";
import type { DownloadTask } from "@/lib/api-types";
import type { UnifiedTask } from "@/stores/task-store";

interface UseMergedTasksResult {
  /** 合并后的任务列表（实时状态覆盖 DB，最新在前） */
  mergedTasks: DownloadTask[];
  /** 进行中的任务数 */
  runningCount: number;
  /** 是否有已完成或出错的任务 */
  hasCompletedTasks: boolean;
}

/**
 * 合并 DB 加载的任务与实时事件任务
 *
 * 实时任务（通过 Tauri task-update 事件接收）覆盖 DB 查询结果的对应字段，
 * 对于 DB 中不存在的实时任务（刚启动还没落库），创建临时条目。
 *
 * 供 downloads.tsx 及其他需要任务列表的页面复用。
 */
export function useMergedTasks(
  dbTasks: DownloadTask[],
  liveTasks: Record<string, UnifiedTask>,
): UseMergedTasksResult {
  const mergedTasks = useMemo(() => {
    const taskMap = new Map<string, DownloadTask>();

    // 先放入 DB 任务
    for (const task of dbTasks) {
      taskMap.set(task.id, task);
    }

    // 实时任务覆盖或补充
    for (const live of Object.values(liveTasks)) {
      const existing = taskMap.get(live.task_id);
      if (existing) {
        taskMap.set(live.task_id, {
          ...existing,
          status: (live.status as DownloadTask["status"]) ?? existing.status,
          total: live.total ?? existing.total,
          completed: live.completed ?? existing.completed,
          failed: live.failed ?? existing.failed,
          error_msg: live.error ?? existing.error_msg,
        });
      } else {
        // 实时任务不在 DB 中（刚启动还没写入），创建临时条目
        taskMap.set(live.task_id, {
          id: live.task_id,
          mode: (live.type as DownloadTask["mode"]) ?? "one",
          url: live.url ?? "",
          title: live.title ?? live.nickname ?? null,
          author_nickname: live.nickname ?? null,
          status: (live.status as DownloadTask["status"]) ?? "running",
          total: live.total ?? 0,
          completed: live.completed ?? 0,
          skipped: 0,
          failed: live.failed ?? 0,
          error_msg: live.error ?? null,
          created_at: 0,
          updated_at: 0,
        });
      }
    }

    // 按创建时间倒序（最新在前），实时任务（created_at=0）排最前面
    return Array.from(taskMap.values()).sort(
      (a, b) => b.created_at - a.created_at || b.id.localeCompare(a.id),
    );
  }, [dbTasks, liveTasks]);

  const runningCount = mergedTasks.filter(
    (t) =>
      t.status === "running" ||
      t.status === "starting" ||
      t.status === "recording" ||
      t.status === "stopping",
  ).length;

  const hasCompletedTasks = mergedTasks.some(
    (t) => t.status === "completed" || t.status === "error" || t.status === "interrupted",
  );

  return { mergedTasks, runningCount, hasCompletedTasks };
}
