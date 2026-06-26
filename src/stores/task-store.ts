import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import { queryClient } from "@/lib/query-client";

export interface UnifiedTask {
  task_id: string;
  task_type: "batch" | "live";
  // batch 字段
  type?: string;       // download_type: user_post / user_like / mix / collects
  url: string;
  status: string;      // starting | running | completed | error | recording | stopping
  total?: number;
  completed?: number;
  failed?: number;
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
        if (task.status === "running" || task.status === "starting" || task.status === "recording" || task.status === "stopping") {
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
        data: Record<string, unknown>;
      };

      // 统一处理 batch 和 live 事件
      if (msg.task_type !== "batch" && msg.task_type !== "live") return;

      const data = msg.data;
      const task: UnifiedTask = {
        task_id: data.task_id as string ?? msg.task_id,
        task_type: msg.task_type as "batch" | "live",
        url: data.url as string ?? "",
        status: data.status as string ?? "",
        type: data.type as string | undefined,
        total: data.total as number | undefined,
        completed: data.completed as number | undefined,
        failed: data.failed as number | undefined,
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
        tasks: { ...state.tasks, [task.task_id]: task },
        connected: true,
      }));

      // 任务完成时刷新数据库查询缓存
      if (task.status === "completed" || task.status === "error") {
        void queryClient.invalidateQueries({ queryKey: ["downloads"] });
        void queryClient.invalidateQueries({ queryKey: ["download-stats"] });
        void queryClient.invalidateQueries({ queryKey: ["download-tasks"] });
        void queryClient.invalidateQueries({ queryKey: ["download-task-detail"] });
        if (task.task_type === "live") {
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
