import { create } from "zustand";

type Theme = "light" | "dark" | "system";

function applyTheme(theme: Theme) {
  const root = document.documentElement;
  root.classList.remove("light", "dark");
  if (theme === "system") {
    const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    root.classList.add(prefersDark ? "dark" : "light");
  } else {
    root.classList.add(theme);
  }
}

function getInitialTheme(): Theme {
  const stored = localStorage.getItem("theme") as Theme | null;
  if (stored && ["light", "dark", "system"].includes(stored)) return stored;
  return "system";
}

const initialTheme = getInitialTheme();

interface AppState {
  connected: boolean;
  downloadCount: number;
  theme: Theme;
  setConnected: (connected: boolean) => void;
  incrementDownloadCount: () => void;
  setDownloadCount: (count: number) => void;
  setTheme: (theme: Theme) => void;
}

export const useAppStore = create<AppState>((set) => ({
  connected: false,
  downloadCount: 0,
  theme: initialTheme,
  setConnected: (connected) => set({ connected }),
  incrementDownloadCount: () =>
    set((state) => ({ downloadCount: state.downloadCount + 1 })),
  setDownloadCount: (count) => set({ downloadCount: count }),
  setTheme: (theme) => {
    localStorage.setItem("theme", theme);
    applyTheme(theme);
    set({ theme });
  },
}));

applyTheme(initialTheme);
