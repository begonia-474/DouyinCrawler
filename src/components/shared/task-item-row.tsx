import { cn } from "@/lib/utils";
import { formatFileSize } from "@/lib/utils";
import {
  CheckCircle2,
  SkipForward,
  XCircle,
  Circle,
  Loader2,
} from "lucide-react";
import type { TaskItem } from "@/lib/api-types";

const statusConfig: Record<string, { icon: typeof CheckCircle2; color: string; label: string }> = {
  completed: { icon: CheckCircle2, color: "text-success", label: "完成" },
  skipped: { icon: SkipForward, color: "text-muted-foreground", label: "跳过" },
  failed: { icon: XCircle, color: "text-destructive", label: "失败" },
  downloading: { icon: Loader2, color: "text-primary", label: "下载中" },
  pending: { icon: Circle, color: "text-muted-foreground/50", label: "等待" },
};

export function TaskItemRow({ item }: { item: TaskItem }) {
  const config = statusConfig[item.status] || statusConfig.pending;
  const Icon = config.icon;

  return (
    <div className="flex items-center gap-3 py-2 px-3 rounded-lg hover:bg-foreground/[0.02] transition-colors">
      <Icon
        className={cn(
          "h-3.5 w-3.5 shrink-0",
          config.color,
          item.status === "downloading" && "animate-spin"
        )}
      />
      <div className="flex-1 min-w-0">
        <p className="text-xs truncate">
          {item.title || item.file_path || `#${item.id}`}
        </p>
        {item.error_msg && (
          <p className="text-[11px] text-destructive truncate mt-0.5">
            {item.error_msg}
          </p>
        )}
      </div>
      <div className="flex items-center gap-2 shrink-0">
        {item.file_size > 0 && (
          <span className="text-[11px] text-muted-foreground font-mono tabular-nums">
            {formatFileSize(item.file_size)}
          </span>
        )}
        <span className={cn("text-[11px]", config.color)}>
          {config.label}
        </span>
      </div>
    </div>
  );
}
