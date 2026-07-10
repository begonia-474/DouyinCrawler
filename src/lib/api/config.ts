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

// ============================================================
// Firefox Cookie 自动获取
// ============================================================

export interface FirefoxProfile {
  name: string;
  path: string;
  is_default: boolean;
  has_cookies: boolean;
}

/**
 * 获取 Firefox profiles 列表
 */
export async function getFirefoxProfiles(): Promise<FirefoxProfile[]> {
  return invoke("get_firefox_profiles_command");
}

/**
 * 从默认 Firefox profile 获取抖音 cookie
 */
export async function getDouyinCookie(): Promise<string> {
  return invoke("get_douyin_cookie_command");
}

/**
 * 从指定 Firefox profile 获取抖音 cookie
 */
export async function getDouyinCookieFromProfile(profileName: string): Promise<string> {
  return invoke("get_douyin_cookie_from_profile_command", { profileName });
}

/**
 * 从 Firefox 获取指定域名的 cookie
 */
export async function getFirefoxCookie(domain: string): Promise<Record<string, string>> {
  return invoke("get_firefox_cookie_command", { domain });
}
