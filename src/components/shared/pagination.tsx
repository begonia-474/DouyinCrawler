import { Button } from "@/components/ui/button";

interface PaginationProps {
  page: number;
  totalPages: number;
  onPageChange: (page: number | ((prev: number) => number)) => void;
}

/**
 * 通用分页控件，显示上一页/下一页按钮和页码。
 *
 * 用法：
 * ```tsx
 * <Pagination page={page} totalPages={Math.ceil(total / pageSize)} onPageChange={setPage} />
 * ```
 */
export function Pagination({ page, totalPages, onPageChange }: PaginationProps) {
  return (
    <div className="flex justify-between items-center pt-4">
      <Button
        variant="capsule"
        size="sm"
        disabled={page === 0}
        onClick={() => onPageChange((p) => p - 1)}
      >
        上一页
      </Button>
      <span className="text-sm text-muted-foreground">
        第 {page + 1} / {totalPages || 1} 页
      </span>
      <Button
        variant="capsule"
        size="sm"
        disabled={page + 1 >= totalPages}
        onClick={() => onPageChange((p) => p + 1)}
      >
        下一页
      </Button>
    </div>
  );
}
