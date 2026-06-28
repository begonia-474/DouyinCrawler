import { Inbox } from "lucide-react";

interface EmptyStateProps {
  icon?: React.ReactNode;
  title?: string;
  description?: string;
  className?: string;
}

/** 统一空状态占位 — 供列表页复用 */
export function EmptyState({
  icon,
  title = "暂无数据",
  description,
  className = "",
}: EmptyStateProps) {
  return (
    <div className={`flex flex-col items-center justify-center gap-2 py-16 text-center ${className}`}>
      {icon || <Inbox className="h-12 w-12 text-muted-foreground/40" />}
      <p className="text-sm font-medium text-muted-foreground">{title}</p>
      {description && <p className="text-xs text-muted-foreground/60">{description}</p>}
    </div>
  );
}
