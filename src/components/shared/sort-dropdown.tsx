import { useState, useRef, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { ChevronDown } from "lucide-react";

export interface SortOption {
  value: string;
  label: string;
}

interface SortDropdownProps {
  options: SortOption[];
  sortBy: string;
  sortOrder: "asc" | "desc";
  onSortByChange: (value: string) => void;
  onSortOrderChange: (order: "asc" | "desc") => void;
}

/**
 * 通用排序下拉组件，支持字段选择 + 升降序切换。
 *
 * 用法：
 * ```tsx
 * <SortDropdown
 *   options={SORT_OPTIONS}
 *   sortBy={sortBy}
 *   sortOrder={sortOrder}
 *   onSortByChange={setSortBy}
 *   onSortOrderChange={setSortOrder}
 * />
 * ```
 */
export function SortDropdown({
  options,
  sortBy,
  sortOrder,
  onSortByChange,
  onSortOrderChange,
}: SortDropdownProps) {
  const [showSort, setShowSort] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // 点击外部关闭
  useEffect(() => {
    if (!showSort) return;
    const handleClickOutside = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setShowSort(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [showSort]);

  return (
    <div className="relative" ref={dropdownRef}>
      <Button
        variant="capsule"
        size="sm"
        onClick={() => setShowSort(!showSort)}
        className="gap-1 h-11"
      >
        {options.find((o) => o.value === sortBy)?.label}
        <ChevronDown className="h-3 w-3" />
      </Button>
      {showSort && (
        <div className="absolute right-0 top-full mt-1 z-10 bg-popover border rounded-md shadow-md py-1 min-w-[140px]">
          {options.map((opt) => (
            <button
              key={opt.value}
              className={`w-full text-left px-3 py-1.5 text-sm hover:bg-accent transition-colors ${
                sortBy === opt.value ? "font-medium" : ""
              }`}
              onClick={() => {
                onSortByChange(opt.value);
                setShowSort(false);
              }}
            >
              {opt.label}
            </button>
          ))}
          <div className="border-t mt-1 pt-1">
            <button
              className="w-full text-left px-3 py-1.5 text-sm hover:bg-accent"
              onClick={() => {
                onSortOrderChange(sortOrder === "desc" ? "asc" : "desc");
                setShowSort(false);
              }}
            >
              {sortOrder === "desc" ? "降序 ↓" : "升序 ↑"}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
