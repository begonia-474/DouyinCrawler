import { useState, useCallback } from "react";

interface UsePaginationOptions {
  pageSize?: number;
}

interface UsePaginationReturn {
  page: number;
  pageSize: number;
  setPage: (page: number | ((prev: number) => number)) => void;
  goToFirst: () => void;
  offset: number;
  /** 搜索/筛选变更时重置到第一页 */
  resetPage: () => void;
}

/**
 * 通用分页状态 hook，封装 page/pageSize/offset。
 *
 * 用法：
 * ```tsx
 * const { page, pageSize, setPage, offset, resetPage } = usePagination();
 * const query = useSomeQuery({ limit: pageSize, offset });
 * ```
 */
export function usePagination({ pageSize = 20 }: UsePaginationOptions = {}): UsePaginationReturn {
  const [page, setPage] = useState(0);

  const goToFirst = useCallback(() => setPage(0), []);
  const resetPage = useCallback(() => setPage(0), []);

  return {
    page,
    pageSize,
    setPage,
    goToFirst,
    offset: page * pageSize,
    resetPage,
  };
}
