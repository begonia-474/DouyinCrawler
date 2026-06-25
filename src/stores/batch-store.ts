import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";

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
    if (unlisten) {
      console.log("[BatchStore] 已经连接，跳过重复注册");
      return;
    }

    console.log("[BatchStore] 开始监听 task-update 事件");
    listen("task-update", async (event) => {
      console.log("[BatchStore] 收到原始事件:", event.payload);
      const msg = event.payload as {
        task_id: string;
        task_type: string;
        data: BatchTask;
      };
      if (msg.task_type !== "batch") {
        console.log("[BatchStore] 忽略非 batch 类型事件:", msg.task_type);
        return;
      }

      const task = msg.data;
      console.log("[BatchStore] 收到任务更新:", task.task_id, "status:", task.status, "full data:", task);
      set((state) => ({
        tasks: { ...state.tasks, [task.task_id]: task },
        connected: true,
      }));

      // 任务完成时通知页面刷新数据（数据库已由 Python 直接写入）
      if (task.status === "completed" || task.status === "error") {
        console.log("[BatchStore] 任务完成，通知页面刷新");
        window.dispatchEvent(new CustomEvent("download-records-updated"));
      }
    }).then((fn) => {
      unlisten = fn;
      console.log("[BatchStore] 事件监听已注册");
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
