import { useState, useCallback } from "react";
import { Collapsible as CollapsiblePrimitive } from "@base-ui/react/collapsible";
import { cn } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { TaskItemRow } from "./task-item-row";
import { useDownloadTaskItemsQuery, useDownloadTaskItemCountsQuery } from "@/lib/queries";
import {
  Video,
  Image,
  Radio,
  Music,
  CheckCircle2,
  XCircle,
  Loader2,
  ChevronDown,
  Trash2,
} from "lucide-react";
import type { DownloadTask } from "@/lib/api-types";
import type { UnifiedTask } from "@/stores/task-store";

const modeLabels: Record<string, string> = {
  one: "单视频",
  post: "用户主页",
  like: "用户点赞",
  mix: "合集",
  collects: "收藏夹",
  live: "直播录制",
  music: "音乐",
};

const modeIcons: Record<string, typeof Video> = {
  one: Video,
  post: Video,
  like: Video,
  mix: Image,
  collects: Image,
  live: Radio,
  music: Music,
};

/** 状态 → 显示文本（未知状态不显示为"失败"） */
const statusLabels: Record<string, string> = {
  pending: "等待中",
  starting: "启动中",
  running: "下载中",
  recording: "录制中",
  stopping: "停止中",
  completed: "已完成",
  error: "失败",
  cancelled: "已取消",
};

/** 状态 → Badge variant */
const statusVariants: Record<string, "default" | "destructive" | "secondary"> = {
  pending: "secondary",
  starting: "secondary",
  running: "secondary",
  recording: "secondary",
  stopping: "secondary",
  completed: "default",
  error: "destructive",
  cancelled: "secondary",
};

interface TaskCardProps {
  task: DownloadTask;
  liveState?: UnifiedTask;   // 实时状态覆盖
  onRemove?: () => void;
}

export function TaskCard({ task, liveState, onRemove }: TaskCardProps) {
  const [open, setOpen] = useState(false);
  const [hasLoaded, setHasLoaded] = useState(false);

  // 实时状态优先
  const status = liveState?.status ?? task.status;
  const total = liveState?.total ?? task.total;
  const completed = liveState?.completed ?? task.completed;
  const failed = liveState?.failed ?? task.failed;
  const skipped = task.skipped;  // skipped 只有 DB 数据
  const errorMsg = liveState?.error ?? task.error_msg;
  const isRunning = status === "running" || status === "starting" || status === "recording" || status === "stopping";

  // 展开时加载子项
  const handleOpenChange = useCallback((newOpen: boolean) => {
    setOpen(newOpen);
    if (newOpen && !hasLoaded) {
      setHasLoaded(true);
    }
  }, [hasLoaded]);

  // 子项数据（仅在展开后请求）
  const itemsQuery = useDownloadTaskItemsQuery(
    hasLoaded ? task.id : "",
    undefined
  );
  const countsQuery = useDownloadTaskItemCountsQuery(
    hasLoaded ? task.id : ""
  );

  const items = itemsQuery.data ?? [];
  const counts = countsQuery.data;

  const Icon = modeIcons[task.mode] || Video;
  const label = modeLabels[task.mode] || task.mode;

  const progressPercent = total > 0 ? (completed / total) * 100 : 0;

  return (
    <CollapsiblePrimitive.Root open={open} onOpenChange={handleOpenChange}>
      <div className={cn(
        "rounded-2xl bg-card overflow-hidden transition-all duration-200 noise",
        isRunning && "ring-1 ring-primary/20"
      )}>
        {/* 卡片头部 —— 可点击展开 */}
        <CollapsiblePrimitive.Trigger className="w-full text-left cursor-pointer">
          <div className="p-4 hover:bg-foreground/[0.02] transition-colors">
            <div className="flex items-center gap-3">
              {/* 模式图标 */}
              <div className={cn(
                "h-9 w-9 rounded-xl flex items-center justify-center shrink-0 ring-1",
                isRunning
                  ? "bg-primary/10 ring-primary/20"
                  : status === "completed"
                  ? "bg-success/10 ring-success/20"
                  : status === "error"
                  ? "bg-destructive/10 ring-destructive/20"
                  : "bg-foreground/[0.04] ring-foreground/[0.06]"
              )}>
                {isRunning ? (
                  <Loader2 className="h-4 w-4 text-primary animate-spin" />
                ) : status === "completed" ? (
                  <CheckCircle2 className="h-4 w-4 text-success" />
                ) : status === "error" ? (
                  <XCircle className="h-4 w-4 text-destructive" />
                ) : (
                  <Icon className="h-4 w-4 text-muted-foreground" />
                )}
              </div>

              {/* 标题 + 作者 + URL */}
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium truncate">
                  {task.author_nickname
                    ? `${task.author_nickname} · ${label}`
                    : task.title || `${label}下载`}
                </p>
                <p className="text-xs text-muted-foreground truncate mt-0.5">
                  {task.url}
                </p>
              </div>

              {/* 状态 Badge */}
              <Badge
                variant={statusVariants[status] ?? "secondary"}
                className="text-[11px] rounded-full"
              >
                {statusLabels[status] ?? status}
              </Badge>

              {/* 删除按钮 */}
              {!isRunning && onRemove && (
                <Button
                  variant="ghost"
                  size="icon-sm"
                  title="移除任务"
                  onClick={(e) => {
                    e.stopPropagation();
                    onRemove();
                  }}
                >
                  <Trash2 className="h-4 w-4 text-muted-foreground" />
                </Button>
              )}

              {/* 展开箭头 */}
              <ChevronDown
                className={cn(
                  "h-4 w-4 text-muted-foreground transition-transform duration-200 shrink-0",
                  open && "rotate-180"
                )}
              />
            </div>

            {/* 进度条（运行中或有进度时显示） */}
            {(isRunning || total > 0) && (
              <div className="mt-3 space-y-1.5">
                <Progress
                  value={isRunning ? progressPercent : 100}
                  className="h-1.5"
                />
                <div className="flex items-center justify-between text-xs text-muted-foreground">
                  <span>
                    {isRunning
                      ? liveState?.current_item || "准备中..."
                      : `${completed} 完成${skipped > 0 ? ` · ${skipped} 跳过` : ""}${failed > 0 ? ` · ${failed} 失败` : ""}`}
                  </span>
                  <span className="font-mono tabular-nums">
                    {completed}/{total}
                  </span>
                </div>
              </div>
            )}

            {/* 错误信息 */}
            {status === "error" && errorMsg && (
              <p className="text-xs text-destructive mt-2">{errorMsg}</p>
            )}
          </div>
        </CollapsiblePrimitive.Trigger>

        {/* 展开面板 —— 子项列表 */}
        <CollapsiblePrimitive.Panel className="overflow-hidden">
          <div className="border-t border-foreground/[0.06] px-4 py-3">
            {/* 子项统计 */}
            {counts && (
              <div className="flex items-center gap-3 mb-3 text-xs text-muted-foreground">
                <span>共 {counts.total} 项</span>
                {counts.completed > 0 && (
                  <span className="text-success">{counts.completed} 完成</span>
                )}
                {counts.skipped > 0 && (
                  <span>{counts.skipped} 跳过</span>
                )}
                {counts.failed > 0 && (
                  <span className="text-destructive">{counts.failed} 失败</span>
                )}
                {counts.pending > 0 && (
                  <span>{counts.pending} 等待</span>
                )}
              </div>
            )}

            {/* 子项列表 */}
            {itemsQuery.isLoading ? (
              <div className="flex justify-center py-4">
                <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
              </div>
            ) : items.length === 0 ? (
              <p className="text-xs text-muted-foreground text-center py-4">
                暂无子项记录
              </p>
            ) : (
              <div className="max-h-[400px] overflow-y-auto -mx-1">
                {items.map((item) => (
                  <TaskItemRow key={item.id} item={item} />
                ))}
              </div>
            )}
          </div>
        </CollapsiblePrimitive.Panel>
      </div>
    </CollapsiblePrimitive.Root>
  );
}
