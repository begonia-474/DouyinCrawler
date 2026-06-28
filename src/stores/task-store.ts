import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import { queryClient } from "@/lib/query-client";

export interface UnifiedTask {
  task_id: string;
  task_type: "batch" | "live" | "typed";
  // TaskEvent 对齐字段（Python / Rust 统一发射）
  event_type?: "started" | "progress" | "finished";
  // batch/typed 字段
  type?: string;       // download_type: user_post / user_like / mix / collects
  url?: string;
  status: string;      // starting | running | completed | error | recording | stopping
  mode?: string;       // download mode: one / post / like / mix / collects / live / music
  total?: number;
  completed?: number;
  failed?: number;
  skipped?: number;
  current_item?: string;
  // live 字段
  title?: string;
  nickname?: string;
  room_id?: string;
  web_rid?: string;
  file?: string;
  file_size?: number;
  duration_sec?: number;
  started_at?: number;
  ended_at?: number;
  cover_url?: string;
  error?: string;
  error_msg?: string;
}

interface TaskState {
  tasks: Record<string, UnifiedTask>;
  connected: boolean;

  addTask: (task: UnifiedTask) => void;
  updateTask: (taskId: string, updates: Partial<UnifiedTask>) => void;
  removeTask: (taskId: string) => void;
  clearCompleted: () => void;

  connect: () => void;
  disconnect: () => void;
}

let unlisten: (() => void) | null = null;

/** 终态状态集合（DB 不会再变化，需要收敛到 DB 真相） */
const TERMINAL_STATUSES = new Set<string>(["completed", "error", "cancelled"]);

/**
 * 将 patch 中的非 undefined 字段合并到旧状态。
 * undefined 字段不覆盖现有值（patch 语义）。
 */
function mergePatch(
  old: UnifiedTask | undefined,
  patch: Record<string, unknown>,
  taskType: "batch" | "live" | "typed",
): UnifiedTask {
  const base: UnifiedTask = old ?? {
    task_id: (patch.task_id as string) ?? "",
    task_type: taskType,
    status: "",
  };

  const merged: UnifiedTask = { ...base, task_type: taskType };

  // 只覆盖 patch 中明确存在的字段（undefined 跳过）
  const fields: (keyof UnifiedTask)[] = [
    "event_type",
    "status", "url", "type", "mode",
    "total", "completed", "failed", "skipped", "current_item",
    "title", "nickname", "room_id", "web_rid",
    "file", "file_size", "duration_sec", "started_at", "ended_at",
    "cover_url", "error", "error_msg",
  ];

  for (const key of fields) {
    const val = patch[key];
    if (val !== undefined && val !== null) {
      // @ts-expect-error 动态赋值
      merged[key] = val;
    }
  }

  return merged;
}

export const useTaskStore = create<TaskState>((set) => ({
  tasks: {},
  connected: false,

  addTask: (task) =>
    set((state) => ({
      tasks: { ...state.tasks, [task.task_id]: task },
    })),

  updateTask: (taskId, updates) =>
    set((state) => ({
      tasks: {
        ...state.tasks,
        [taskId]: { ...state.tasks[taskId], ...updates },
      },
    })),

  removeTask: (taskId) =>
    set((state) => {
      const { [taskId]: _, ...rest } = state.tasks;
      return { tasks: rest };
    }),

  clearCompleted: () =>
    set((state) => {
      const tasks: Record<string, UnifiedTask> = {};
      for (const [id, task] of Object.entries(state.tasks)) {
        if (!TERMINAL_STATUSES.has(task.status)) {
          tasks[id] = task;
        }
      }
      return { tasks };
    }),

  connect: () => {
    if (unlisten) return;

    listen("task-update", async (event) => {
      const msg = event.payload as {
        task_id?: string;
        task_type?: string;
        data?: Record<string, unknown>;
      };

      // 拒绝没有 task_id 的事件
      const taskId = msg.task_id ?? msg.data?.task_id as string | undefined;
      if (!taskId) {
        console.warn("[task-store] 收到无 task_id 的事件，忽略", msg);
        return;
      }

      const taskType = msg.task_type;
      const data = msg.data ?? {};

      // 处理 batch/live 事件（Python 路径，已对齐 event_type 字段）
      if (taskType === "batch" || taskType === "live") {
        const patch = {
          task_id: taskId,
          event_type: data.event_type as string | undefined,
          status: data.status as string | undefined,
          url: data.url as string | undefined,
          type: data.type as string | undefined,
          mode: data.mode as string | undefined,
          total: data.total as number | undefined,
          completed: data.completed as number | undefined,
          failed: data.failed as number | undefined,
          skipped: data.skipped as number | undefined,
          current_item: data.current_item as string | undefined,
          title: data.title as string | undefined,
          nickname: data.nickname as string | undefined,
          room_id: data.room_id as string | undefined,
          web_rid: data.web_rid as string | undefined,
          file: data.file as string | undefined,
          file_size: data.file_size as number | undefined,
          duration_sec: data.duration_sec as number | undefined,
          started_at: data.started_at as number | undefined,
          ended_at: data.ended_at as number | undefined,
          cover_url: data.cover_url as string | undefined,
          error: data.error as string | undefined,
        };

        set((state) => ({
          tasks: {
            ...state.tasks,
            [taskId]: mergePatch(state.tasks[taskId], patch, taskType),
          },
          connected: true,
        }));
      }

      // 处理 typed 事件（Rust 新路径，TaskEvent 通过 serde(flatten) 展开在 data 中）
      if (taskType === "typed") {
        // TaskEvent 字段直接在 data 顶层（serde(flatten) + event_type）
        const patch = {
          task_id: taskId,
          event_type: data.event_type as string | undefined,
          status: data.status as string | undefined,
          url: data.url as string | undefined,
          mode: data.mode as string | undefined,
          total: data.total as number | undefined,
          completed: data.completed as number | undefined,
          failed: data.failed as number | undefined,
          skipped: data.skipped as number | undefined,
          current_item: data.current_item as string | undefined,
          error_msg: data.error_msg as string | undefined,
        };

        set((state) => ({
          tasks: {
            ...state.tasks,
            [taskId]: mergePatch(state.tasks[taskId], patch, taskType),
          },
          connected: true,
        }));
      }

      // 终态：刷新 DB 查询缓存（事件只是 hint，DB 是真相）
      const currentStatus = (data.status as string) ?? "";
      if (TERMINAL_STATUSES.has(currentStatus)) {
        void queryClient.invalidateQueries({ queryKey: ["downloads"] });
        void queryClient.invalidateQueries({ queryKey: ["download-stats"] });
        void queryClient.invalidateQueries({ queryKey: ["download-tasks"] });
        void queryClient.invalidateQueries({ queryKey: ["download-task-detail"] });
        if (taskType === "live") {
          void queryClient.invalidateQueries({ queryKey: ["live-records"] });
          void queryClient.invalidateQueries({ queryKey: ["live-record-count"] });
        } else {
          void queryClient.invalidateQueries({ queryKey: ["video-count"] });
          void queryClient.invalidateQueries({ queryKey: ["video-stats"] });
          void queryClient.invalidateQueries({ queryKey: ["user-stats"] });
        }
      }
    }).then((fn) => {
      unlisten = fn;
    });
  },

  disconnect: () => {
    if (unlisten) {
      unlisten();
      unlisten = null;
    }
    set({ connected: false });
  },
}));
