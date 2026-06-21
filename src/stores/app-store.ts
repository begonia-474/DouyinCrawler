import { create } from "zustand";

interface AppState {
  connected: boolean;
  downloadCount: number;
  setConnected: (connected: boolean) => void;
  incrementDownloadCount: () => void;
  setDownloadCount: (count: number) => void;
}

export const useAppStore = create<AppState>((set) => ({
  connected: false,
  downloadCount: 0,
  setConnected: (connected) => set({ connected }),
  incrementDownloadCount: () =>
    set((state) => ({ downloadCount: state.downloadCount + 1 })),
  setDownloadCount: (count) => set({ downloadCount: count }),
}));
