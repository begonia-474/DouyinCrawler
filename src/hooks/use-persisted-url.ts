import { useState, useCallback } from "react";

/**
 * 持久化 URL 到 sessionStorage。
 *
 * 组件挂载时自动恢复上次保存的 URL，
 * 导航离开再返回时无需重新输入。
 *
 * @param key sessionStorage key（建议按页面区分，如 "douyin:user"）
 */
export function usePersistedUrl(key: string): [string, (url: string) => void] {
  const storageKey = `douyin:${key}:url`;

  const [url, setUrlState] = useState<string>(() => {
    try {
      return sessionStorage.getItem(storageKey) ?? "";
    } catch {
      return "";
    }
  });

  const setUrl = useCallback(
    (newUrl: string) => {
      setUrlState(newUrl);
      try {
        if (newUrl) {
          sessionStorage.setItem(storageKey, newUrl);
        } else {
          sessionStorage.removeItem(storageKey);
        }
      } catch {
        // sessionStorage 不可用时静默失败
      }
    },
    [storageKey]
  );

  return [url, setUrl];
}
