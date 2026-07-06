import { useState, useCallback } from "react";

/**
 * 列表项选择状态管理 Hook
 *
 * 用于批量操作场景，管理 Set<string> 类型的选中状态。
 * 仅当前页有效（翻页后选中状态重置）。
 */
export function useSelection() {
  const [selected, setSelected] = useState<Set<string>>(new Set());

  const isSelected = useCallback(
    (id: string) => selected.has(id),
    [selected]
  );

  const toggle = useCallback((id: string) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }, []);

  const selectAll = useCallback((ids: string[]) => {
    setSelected(new Set(ids));
  }, []);

  const clearSelection = useCallback(() => {
    setSelected(new Set());
  }, []);

  const selectedCount = selected.size;

  const isAllSelected = useCallback(
    (ids: string[]) => ids.length > 0 && ids.every((id) => selected.has(id)),
    [selected]
  );

  const isIndeterminate = useCallback(
    (ids: string[]) => {
      const selectedInPage = ids.filter((id) => selected.has(id)).length;
      return selectedInPage > 0 && selectedInPage < ids.length;
    },
    [selected]
  );

  return {
    selected,
    isSelected,
    toggle,
    selectAll,
    clearSelection,
    selectedCount,
    isAllSelected,
    isIndeterminate,
  };
}
