import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import { queryClient } from "@/lib/query-client";

export interface UnifiedTask {
  task_id: string;
  task_type: "typed";
  // TaskEvent 字段
  event_type?: "started" | "progress" | "finished";
  mode?: string;       // download mode: one / post / like / mix / collects / live / music
  url?: string;
  status: string;      // starting | running | completed | error | recording | stopping
  total?: number;
  completed?: number;
  failed?: number;
  skipped?: number;
  current_item?: string;
  error_msg?: string;
  // batch/live 扩展字段（Python 侧可能发送）
  type?: string;
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

/** 记录待失效的 mode 类型，debounce 期间合并 */
let pendingModes = new Set<string>();

/** 300ms debounce 合并终态事件的 React Query 失效 */
let invalidateTimer: ReturnType<typeof setTimeout> | null = null;

function debouncedInvalidate(mode: string) {
  pendingModes.add(mode);
  if (invalidateTimer) return;
  invalidateTimer = setTimeout(() => {
    invalidateTimer = null;
    const modes = pendingModes;
    pendingModes = new Set();

    void queryClient.invalidateQueries({ queryKey: ["downloads"] });
    void queryClient.invalidateQueries({ queryKey: ["download-stats"] });
    void queryClient.invalidateQueries({ queryKey: ["download-tasks"] });
    void queryClient.invalidateQueries({ queryKey: ["download-task-detail"] });

    if (modes.has("live")) {
      void queryClient.invalidateQueries({ queryKey: ["live-records"] });
      void queryClient.invalidateQueries({ queryKey: ["live-record-count"] });
    }
    if (modes.size > 1 || !modes.has("live")) {
      void queryClient.invalidateQueries({ queryKey: ["video-count"] });
      void queryClient.invalidateQueries({ queryKey: ["video-stats"] });
      void queryClient.invalidateQueries({ queryKey: ["user-stats"] });
    }
  }, 300);
}

/**
 * 将 patch 中的非 undefined 字段合并到旧状态。
 * undefined 字段不覆盖现有值（patch 语义）。
 */
function mergePatch(
  old: UnifiedTask | undefined,
  patch: Record<string, unknown>,
): UnifiedTask {
  const base: UnifiedTask = old ?? {
    task_id: (patch.task_id as string) ?? "",
    task_type: "typed",
    status: "",
  };

  const merged: UnifiedTask = { ...base, task_type: "typed" };

  // 只覆盖 patch 中明确存在的字段（undefined 跳过）
  const fields: (keyof UnifiedTask)[] = [
    "event_type",
    "status", "url", "type", "mode",
    "total", "completed", "failed", "skipped", "current_item", "error_msg",
    "title", "nickname", "room_id", "web_rid",
    "file", "file_size", "duration_sec", "started_at", "ended_at",
    "cover_url", "error",
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

      const data = msg.data ?? {};

      // 统一处理所有事件（task_type 始终为 "typed"）
      // data 包含 TaskEvent 的所有字段（event_type, task_id, mode, url, patch.*）
      const patch: Record<string, unknown> = {
        task_id: taskId,
      };

      // 复制所有字段到 patch（前端 UnifiedTask 会处理）
      const fields = [
        "event_type", "status", "url", "type", "mode",
        "total", "completed", "failed", "skipped", "current_item", "error_msg",
        "title", "nickname", "room_id", "web_rid",
        "file", "file_size", "duration_sec", "started_at", "ended_at",
        "cover_url", "error",
      ];

      for (const key of fields) {
        const val = data[key];
        if (val !== undefined && val !== null) {
          patch[key] = val;
        }
      }

      set((state) => ({
        tasks: {
          ...state.tasks,
          [taskId]: mergePatch(state.tasks[taskId], patch),
        },
        connected: true,
      }));

      // 终态：debounced 刷新 DB 查询缓存（事件只是 hint，DB 是真相）
      const currentStatus = (data.status as string) ?? "";
      if (TERMINAL_STATUSES.has(currentStatus)) {
        debouncedInvalidate((data.mode as string) ?? "");
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
