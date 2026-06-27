import { invoke } from "@tauri-apps/api/core";

// ============================================================
// 配置
// ============================================================

export interface AppConfig {
  cookie: string;
  download_path: string;
  naming: string;
  encryption: string;
  proxy: string;
  app_name: string;
  folderize: boolean;
  music: boolean;
  cover: boolean;
  desc: boolean;
  interval: string | null;
  page_counts: number;
  max_counts: number;
  timeout: number;
  max_connections: number;
  max_retries: number;
  max_tasks: number;
}

export async function getConfig(): Promise<AppConfig> {
  return invoke("get_config");
}

export async function setConfig(updates: Record<string, string>): Promise<void> {
  await invoke("set_config", { updates });
}
