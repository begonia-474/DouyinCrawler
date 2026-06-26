import { useState, useCallback, useEffect, useRef } from "react";

interface UseInfiniteScrollOptions<T> {
  /** 获取下一页数据的函数，返回 items 和分页信息 */
  fetchPage: (cursor: number) => Promise<{
    items: T[];
    nextCursor: number;
    hasMore: boolean;
  } | null>;
  /** 是否有前置条件（如 URL 已解析）才启用加载 */
  enabled?: boolean;
}

interface UseInfiniteScrollReturn<T> {
  items: T[];
  setItems: React.Dispatch<React.SetStateAction<T[]>>;
  hasMore: boolean;
  loadingMore: boolean;
  sentinelRef: React.RefObject<HTMLDivElement | null>;
  /** 重置所有状态并触发首次加载 */
  reset: (fetchFn?: (cursor: number) => Promise<{ items: T[]; nextCursor: number; hasMore: boolean } | null>) => void;
}

/**
 * 通用无限滚动 hook，封装 IntersectionObserver + cursor 分页。
 *
 * 用法：
 * ```tsx
 * const { items, setItems, hasMore, loadingMore, sentinelRef, reset } = useInfiniteScroll({
 *   fetchPage: async (cursor) => {
 *     const res = await getSomeData(url, cursor, 20);
 *     if (res.success && res.data) {
 *       return { items: res.data.items, nextCursor: res.data.next_cursor ?? 0, hasMore: res.data.has_more ?? false };
 *     }
 *     return null;
 *   },
 *   enabled: !!currentUrl,
 * });
 * ```
 */
export function useInfiniteScroll<T>({
  fetchPage,
  enabled = true,
}: UseInfiniteScrollOptions<T>): UseInfiniteScrollReturn<T> {
  const [items, setItems] = useState<T[]>([]);
  const [cursor, setCursor] = useState(0);
  const [hasMore, setHasMore] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);
  const sentinelRef = useRef<HTMLDivElement>(null);

  // 用 ref 存储最新的 fetchPage 避免 useEffect 依赖问题
  const fetchPageRef = useRef(fetchPage);
  fetchPageRef.current = fetchPage;

  const loadMore = useCallback(async () => {
    if (!hasMore || loadingMore || !enabled) return;
    setLoadingMore(true);
    const result = await fetchPageRef.current(cursor);
    if (result) {
      setItems((prev) => [...prev, ...result.items]);
      setCursor(result.nextCursor);
      setHasMore(result.hasMore);
    }
    setLoadingMore(false);
  }, [cursor, hasMore, loadingMore, enabled]);

  // IntersectionObserver 自动触发
  useEffect(() => {
    const el = sentinelRef.current;
    if (!el) return;

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && hasMore && !loadingMore && enabled) {
          loadMore();
        }
      },
      { threshold: 0.1 }
    );

    observer.observe(el);
    return () => observer.disconnect();
  }, [hasMore, loadingMore, loadMore, enabled]);

  const reset = useCallback(
    (fetchFn?: (cursor: number) => Promise<{ items: T[]; nextCursor: number; hasMore: boolean } | null>) => {
      const fn = fetchFn ?? fetchPageRef.current;
      setItems([]);
      setCursor(0);
      setHasMore(false);
      setLoadingMore(false);
      // 首次加载
      (async () => {
        const result = await fn(0);
        if (result) {
          setItems(result.items);
          setCursor(result.nextCursor);
          setHasMore(result.hasMore);
        }
      })();
    },
    []
  );

  return { items, setItems, hasMore, loadingMore, sentinelRef, reset };
}
