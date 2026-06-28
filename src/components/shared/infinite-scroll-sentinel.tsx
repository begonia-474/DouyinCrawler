import { Loader2 } from "lucide-react";

interface InfiniteScrollSentinelProps {
  sentinelRef: React.RefObject<HTMLDivElement | null>;
  loadingMore: boolean;
  hasMore: boolean;
  total: number;
  label?: string;
  className?: string;
}

/** 统一无限滚动哨兵 — 供 likes/user/mix/favorites 列表页复用 */
export function InfiniteScrollSentinel({
  sentinelRef,
  loadingMore,
  hasMore,
  total,
  label = "条目",
  className = "",
}: InfiniteScrollSentinelProps) {
  return (
    <div className={className}>
      {/* 交叉观察器锚点 */}
      <div ref={sentinelRef} className="h-4" />

      {/* 加载更多指示器 */}
      {loadingMore && (
        <div className="flex justify-center py-4">
          <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
        </div>
      )}

      {/* 已全部加载 */}
      {!hasMore && total > 0 && (
        <p className="text-center text-xs text-muted-foreground py-4">
          已加载全部 {total} 个{label}
        </p>
      )}
    </div>
  );
}
