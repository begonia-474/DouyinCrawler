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
    const { Effect, EffectState } = await import("@tauri-apps/api/window");
    const win = getCurrentWebviewWindow();
    await win.setEffects({
      effects: [Effect.Mica],
      state: EffectState.Active,
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
  downloadCount: number;
  theme: Theme;
  incrementDownloadCount: () => void;
  setDownloadCount: (count: number) => void;
  setTheme: (theme: Theme) => void;
}

export const useAppStore = create<AppState>((set) => ({
  downloadCount: 0,
  theme: initialTheme,
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
