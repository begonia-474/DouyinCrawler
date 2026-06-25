import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";

export interface LiveTask {
  task_id: string;
  url: string;
  status: "starting" | "recording" | "completed" | "error" | "stopping";
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

interface LiveState {
  tasks: Record<string, LiveTask>;
  connected: boolean;

  addTask: (task: LiveTask) => void;
  updateTask: (taskId: string, updates: Partial<LiveTask>) => void;
  removeTask: (taskId: string) => void;

  connect: () => void;
  disconnect: () => void;
}

let unlisten: (() => void) | null = null;

export const useLiveStore = create<LiveState>((set) => ({
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

  connect: () => {
    if (unlisten) return;

    listen("task-update", (event) => {
      const msg = event.payload as {
        task_id: string;
        task_type: string;
        data: LiveTask;
      };
      if (msg.task_type !== "live") return;

      set((state) => ({
        tasks: { ...state.tasks, [msg.data.task_id]: msg.data },
        connected: true,
      }));
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
