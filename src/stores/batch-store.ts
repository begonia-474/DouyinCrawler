import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import { queryClient } from "@/lib/query-client";

export interface BatchTask {
  task_id: string;
  type: string;
  url: string;
  status: "starting" | "running" | "completed" | "error";
  total: number;
  completed: number;
  failed: number;
  current_item: string;
  error: string;
}

interface BatchState {
  tasks: Record<string, BatchTask>;
  connected: boolean;

  addTask: (task: BatchTask) => void;
  updateTask: (taskId: string, updates: Partial<BatchTask>) => void;
  removeTask: (taskId: string) => void;
  clearCompleted: () => void;

  connect: () => void;
  disconnect: () => void;
}

let unlisten: (() => void) | null = null;

export const useBatchStore = create<BatchState>((set) => ({
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
      const tasks: Record<string, BatchTask> = {};
      for (const [id, task] of Object.entries(state.tasks)) {
        if (task.status === "running" || task.status === "starting") {
          tasks[id] = task;
        }
      }
      return { tasks };
    }),

  connect: () => {
    if (unlisten) return;

    listen("task-update", async (event) => {
      const msg = event.payload as {
        task_id: string;
        task_type: string;
        data: BatchTask;
      };
      if (msg.task_type !== "batch") return;

      const task = msg.data;
      set((state) => ({
        tasks: { ...state.tasks, [task.task_id]: task },
        connected: true,
      }));

      // 任务完成时刷新数据库查询缓存（数据库已由 Python 直接写入）
      if (task.status === "completed" || task.status === "error") {
        void queryClient.invalidateQueries({ queryKey: ["downloads"] });
        void queryClient.invalidateQueries({ queryKey: ["download-stats"] });
        void queryClient.invalidateQueries({ queryKey: ["video-count"] });
        void queryClient.invalidateQueries({ queryKey: ["video-stats"] });
        void queryClient.invalidateQueries({ queryKey: ["user-stats"] });
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
