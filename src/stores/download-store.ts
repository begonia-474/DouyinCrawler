import { create } from "zustand";

export interface DownloadTask {
  id: string;
  filename: string;
  total: number;
  downloaded: number;
  status: "pending" | "downloading" | "completed" | "error";
  error?: string;
}

interface DownloadState {
  tasks: DownloadTask[];
  addTask: (task: DownloadTask) => void;
  updateTask: (id: string, updates: Partial<DownloadTask>) => void;
  removeTask: (id: string) => void;
  clearCompleted: () => void;
}

export const useDownloadStore = create<DownloadState>((set) => ({
  tasks: [],
  addTask: (task) =>
    set((state) => ({ tasks: [...state.tasks, task] })),
  updateTask: (id, updates) =>
    set((state) => ({
      tasks: state.tasks.map((t) => (t.id === id ? { ...t, ...updates } : t)),
    })),
  removeTask: (id) =>
    set((state) => ({ tasks: state.tasks.filter((t) => t.id !== id) })),
  clearCompleted: () =>
    set((state) => ({
      tasks: state.tasks.filter((t) => t.status !== "completed"),
    })),
}));
