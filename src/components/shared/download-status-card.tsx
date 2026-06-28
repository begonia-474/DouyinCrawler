import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Bezel } from "@/components/shared/bezel";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { useTaskStore } from "@/stores/task-store";
import { Loader2, CheckCircle2, XCircle, AlertTriangle } from "lucide-react";

export function DownloadStatusCard() {
  const navigate = useNavigate();
  const { tasks, connect } = useTaskStore();
  const [dismissed, setDismissed] = useState(false);

  // 全局注册事件监听（确保任何页面都能收到事件）
  useEffect(() => {
    connect();
  }, [connect]);

  // 找到当前正在进行的任务（batch 或 live）
  const activeTask = Object.values(tasks).find(
    (t) => t.status === "running" || t.status === "starting" || t.status === "recording" || t.status === "stopping"
  );
  const recentCompletedTask = Object.values(tasks)
    .filter((t) => t.status === "completed" || t.status === "error")
    .sort((a, b) => {
      return b.task_id.localeCompare(a.task_id);
    })[0];

  // 新任务开始时重置 dismissed 状态
  useEffect(() => {
    if (activeTask) setDismissed(false);
  }, [activeTask?.task_id]); // eslint-disable-line react-hooks/exhaustive-deps -- activeTask identity changes every render; task_id is the stable trigger

  // 已完成/失败的任务 5 秒后自动隐藏
  useEffect(() => {
    if (!activeTask && recentCompletedTask) {
      const timer = setTimeout(() => setDismissed(true), 5000);
      return () => clearTimeout(timer);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- derived objects change every render; task_id is the stable trigger
  }, [activeTask, recentCompletedTask?.task_id]);

  // 没有任务或已隐藏时不显示
  if ((!activeTask && !recentCompletedTask) || dismissed) return null;

  const task = activeTask || recentCompletedTask;
  if (!task) return null;

  const isRunning = task.status === "running" || task.status === "starting" || task.status === "recording" || task.status === "stopping";
  const total = task.total ?? 0;
  const completed = task.completed ?? 0;
  const failed = task.failed ?? 0;
  const skipped = task.skipped ?? 0;
  const progressPercent = total > 0 ? Math.round(((completed + skipped) / total) * 100) : 0;

  return (
    <div className="fixed bottom-4 right-4 z-50 animate-in slide-in-from-bottom-4">
      <Bezel radius="xl">
        <div className="p-4 min-w-[280px] space-y-2.5">
          {/* 头部：图标 + 状态文字 + 操作按钮 */}
          <div className="flex items-center gap-3">
            {isRunning ? (
              <Loader2 className="h-4 w-4 animate-spin text-primary shrink-0" />
            ) : task.status === "completed" ? (
              <CheckCircle2 className="h-4 w-4 text-success shrink-0" />
            ) : (
              <XCircle className="h-4 w-4 text-destructive shrink-0" />
            )}

            <span className="text-sm font-medium flex-1">
              {isRunning
                ? task.status === "recording" ? "录制中" : task.status === "stopping" ? "停止中" : "下载中"
                : task.status === "completed"
                ? "任务完成"
                : "任务失败"}
            </span>

            {(task.status === "completed" || task.status === "error") && (
              <Button
                size="sm"
                variant="capsule"
                onClick={() => navigate("/downloads")}
              >
                查看记录
              </Button>
            )}
          </div>

          {/* 进度条 + 计数（batch 任务运行中） */}
          {task.task_type === "batch" && (isRunning || total > 0) && (
            <div className="space-y-1.5">
              <Progress value={isRunning ? progressPercent : 100} className="h-1.5" />
              <div className="flex items-center justify-between text-xs text-muted-foreground">
                <span className="flex items-center gap-2">
                  <span className="font-mono tabular-nums">{completed + skipped}/{total}</span>
                  {skipped > 0 && (
                    <span className="text-muted-foreground/70">跳{skipped}</span>
                  )}
                  {failed > 0 && (
                    <span className="flex items-center gap-0.5 text-destructive">
                      <AlertTriangle className="h-3 w-3" />
                      {failed}
                    </span>
                  )}
                </span>
                {isRunning && total > 0 && (
                  <span className="font-mono tabular-nums">{progressPercent}%</span>
                )}
              </div>
            </div>
          )}

          {/* 当前下载项（运行中时显示） */}
          {isRunning && task.current_item && (
            <p className="text-xs text-muted-foreground truncate" title={task.current_item}>
              {task.current_item}
            </p>
          )}

          {/* 完成/失败的摘要 */}
          {!isRunning && total > 0 && (
            <p className="text-xs text-muted-foreground">
              {completed} 完成
              {skipped > 0 && ` · ${skipped} 跳过`}
              {failed > 0 && ` · ${failed} 失败`}
            </p>
          )}

          {/* 错误信息 */}
          {task.status === "error" && task.error && (
            <p className="text-xs text-destructive truncate">{task.error}</p>
          )}
        </div>
      </Bezel>
    </div>
  );
}
