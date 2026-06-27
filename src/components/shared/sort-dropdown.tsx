import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { ChevronDown, ArrowDownWideNarrow, ArrowUpWideNarrow } from "lucide-react";

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
 * 基于 shadcn DropdownMenu 实现。
 */
export function SortDropdown({
  options,
  sortBy,
  sortOrder,
  onSortByChange,
  onSortOrderChange,
}: SortDropdownProps) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        render={<Button variant="capsule" size="sm" className="gap-1 h-11" />}
      >
        {options.find((o) => o.value === sortBy)?.label}
        <ChevronDown className="h-3 w-3" />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" sideOffset={4}>
        {options.map((opt) => (
          <DropdownMenuItem
            key={opt.value}
            data-active={sortBy === opt.value ? "" : undefined}
            onClick={() => onSortByChange(opt.value)}
          >
            {opt.label}
          </DropdownMenuItem>
        ))}
        <DropdownMenuSeparator />
        <DropdownMenuItem onClick={() => onSortOrderChange(sortOrder === "desc" ? "asc" : "desc")}>
          {sortOrder === "desc" ? (
            <span className="flex items-center gap-2">
              <ArrowDownWideNarrow className="h-3.5 w-3.5" /> 降序
            </span>
          ) : (
            <span className="flex items-center gap-2">
              <ArrowUpWideNarrow className="h-3.5 w-3.5" /> 升序
            </span>
          )}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
