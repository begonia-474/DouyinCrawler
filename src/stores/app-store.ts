import { create } from "zustand";

type Theme = "light" | "dark" | "system";

async function applyTheme(theme: Theme) {
  const root = document.documentElement;
  root.classList.remove("light", "dark");
  const resolved: "light" | "dark" =
    theme === "system"
      ? window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light"
      : theme;
  root.classList.add(resolved);

  try {
    const { getCurrentWebviewWindow } = await import("@tauri-apps/api/webviewWindow");
    const win = getCurrentWebviewWindow();
    await win.setEffects({
      effects: [resolved === "dark" ? "micaDark" : "micaLight"],
      state: "active",
    });
  } catch {
    // Win10 or non-Tauri environment
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
