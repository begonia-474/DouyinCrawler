import { useEffect, useRef } from "react";
import { Loader2 } from "lucide-react";

interface InfiniteScrollSentinelProps {
  sentinelRef?: React.RefObject<HTMLDivElement | null>;
  loadingMore: boolean;
  hasMore: boolean;
  total: number;
  label?: string;
  className?: string;
  /** 当哨兵进入视口时调用（用于 React Query fetchNextPage 等场景） */
  onVisible?: () => void;
}

/** 统一无限滚动哨兵 — 供 likes/user/mix/favorites 列表页复用 */
export function InfiniteScrollSentinel({
  sentinelRef,
  loadingMore,
  hasMore,
  total,
  label = "条目",
  className = "",
  onVisible,
}: InfiniteScrollSentinelProps) {
  const internalRef = useRef<HTMLDivElement>(null);
  const ref = sentinelRef ?? internalRef;

  useEffect(() => {
    if (!onVisible || !ref.current) return;
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && hasMore && !loadingMore) {
          onVisible();
        }
      },
      { threshold: 0.1 },
    );
    observer.observe(ref.current);
    return () => observer.disconnect();
  }, [onVisible, hasMore, loadingMore, ref]);

  return (
    <div className={className}>
      <div ref={ref} className="h-4" />

      {loadingMore && (
        <div className="flex justify-center py-4">
          <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
        </div>
      )}

      {!hasMore && total > 0 && (
        <p className="text-center text-xs text-muted-foreground py-4">
          已加载全部 {total} 个{label}
        </p>
      )}
    </div>
  );
}
