import { useState, useCallback } from "react";

interface UseAsyncActionOptions<T> {
  action: () => Promise<T>;
  onSuccess?: (result: T) => void;
  onError?: (error: string) => void;
}

interface UseAsyncActionReturn<T> {
  run: () => Promise<T | undefined>;
  loading: boolean;
  error: string | null;
  clearError: () => void;
}

/**
 * 统一异步操作 hook
 *
 * 封装 loading + error 状态，替代各页面重复的
 * `setLoading(true); try { ... } catch { ... } finally { setLoading(false) }` 模式。
 */
export function useAsyncAction<T = void>({
  action,
  onSuccess,
  onError,
}: UseAsyncActionOptions<T>): UseAsyncActionReturn<T> {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const clearError = useCallback(() => setError(null), []);

  const run = useCallback(async (): Promise<T | undefined> => {
    setLoading(true);
    setError(null);
    try {
      const result = await action();
      onSuccess?.(result);
      return result;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
      onError?.(msg);
      return undefined;
    } finally {
      setLoading(false);
    }
  }, [action, onSuccess, onError]);

  return { run, loading, error, clearError };
}
