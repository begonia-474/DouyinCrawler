import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Bezel } from "@/components/shared/bezel";
import { Button } from "@/components/ui/button";
import { useBatchStore } from "@/stores/batch-store";
import { useLiveStore } from "@/stores/live-store";
import { Loader2, CheckCircle2, XCircle } from "lucide-react";

export function DownloadStatusCard() {
  const navigate = useNavigate();
  const { tasks, connect } = useBatchStore();
  const { connect: connectLive } = useLiveStore();
  const [dismissed, setDismissed] = useState(false);

  // 全局注册事件监听（确保任何页面都能收到事件）
  useEffect(() => {
    connect();
    connectLive();
  }, [connect, connectLive]);

  // 找到当前正在进行的任务
  const activeTask = Object.values(tasks).find(
    (t) => t.status === "running" || t.status === "starting"
  );
  const recentCompletedTask = Object.values(tasks)
    .filter((t) => t.status === "completed" || t.status === "error")
    .sort((a, b) => {
      // 按 task_id 排序（简单的时间排序）
      return b.task_id.localeCompare(a.task_id);
    })[0];

  // 新任务开始时重置 dismissed 状态
  useEffect(() => {
    if (activeTask) setDismissed(false);
  }, [activeTask?.task_id]);

  // 已完成/失败的任务 5 秒后自动隐藏
  useEffect(() => {
    if (!activeTask && recentCompletedTask) {
      const timer = setTimeout(() => setDismissed(true), 5000);
      return () => clearTimeout(timer);
    }
  }, [activeTask, recentCompletedTask?.task_id]);

  // 没有任务或已隐藏时不显示
  if ((!activeTask && !recentCompletedTask) || dismissed) return null;

  const task = activeTask || recentCompletedTask;
  if (!task) return null;

  return (
    <div className="fixed bottom-4 right-4 z-50 animate-in slide-in-from-bottom-4">
      <Bezel radius="xl">
        <div className="p-4 flex items-center gap-3 min-w-[200px]">
          {task.status === "running" || task.status === "starting" ? (
            <Loader2 className="h-4 w-4 animate-spin text-primary" />
          ) : task.status === "completed" ? (
            <CheckCircle2 className="h-4 w-4 text-success" />
          ) : (
            <XCircle className="h-4 w-4 text-destructive" />
          )}

          <span className="text-sm">
            {task.status === "running" || task.status === "starting"
              ? "正在下载..."
              : task.status === "completed"
              ? "下载完成"
              : "下载失败"}
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
      </Bezel>
    </div>
  );
}
