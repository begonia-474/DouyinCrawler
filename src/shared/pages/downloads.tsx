import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";
import { useMergedTasks } from "@/hooks/use-merged-tasks";
import { Header } from "@/components/layout/header";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { Bezel } from "@/components/shared/bezel";
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
} from "@/components/ui/dropdown-menu";
import { TaskCard } from "@/components/shared/task-card";
import {
  AlertDialog, AlertDialogContent, AlertDialogHeader, AlertDialogFooter,
  AlertDialogTitle, AlertDialogDescription, AlertDialogAction, AlertDialogCancel,
} from "@/components/ui/alert-dialog";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Download,
  CheckCircle2,
  FolderOpen,
  Radio,
  Trash2,
  DownloadCloud,
} from "lucide-react";
import { openFolder, getConfig, exportData } from "@/lib/api";
import { useDeleteLiveRecord, useDeleteDownloadTask } from "@/lib/mutations";
import { useLiveRecordsQuery, useDownloadTasksQuery } from "@/lib/queries";
import { useTaskStore } from "@/stores/task-store";
import type { LiveRecord } from "@/lib/tauri-types";
import { formatFileSize } from "@/lib/utils";

export default function DownloadsPage() {
  const liveRecordsQuery = useLiveRecordsQuery({ limit: 50 });
  const dbTasksQuery = useDownloadTasksQuery({ limit: 100 });
  const liveRecords = liveRecordsQuery.data ?? [];
  // 直播任务的历史展示由 live_records 负责，避免在“下载任务”和“直播录制”重复出现。
  const dbTasks = useMemo(
    () => (dbTasksQuery.data ?? []).filter((task) => task.mode !== "live"),
    [dbTasksQuery.data]
  );
  const { tasks: liveTasks, connect, clearCompleted, removeTask } = useTaskStore();
  const deleteLive = useDeleteLiveRecord();
  const deleteTask = useDeleteDownloadTask();

  const [deleteLiveTarget, setDeleteLiveTarget] = useState<LiveRecord | null>(null);
  const [deleteFile, setDeleteFile] = useState(false);

  useEffect(() => {
    connect();
  }, [connect]);

  const handleOpenFolder = async () => {
    try {
      const config = await getConfig();
      await openFolder(config.download_path);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "打开文件夹失败");
    }
  };

  const EXPORT_TYPES: Record<string, string> = {
    downloads: "下载记录",
    videos: "视频列表",
    users: "用户列表",
    live_records: "直播录制",
    music: "音乐收藏",
  };

  const handleExport = async (dataType: string) => {
    try {
      const config = await getConfig();
      const ts = new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19);
      const savePath = `${config.download_path}/export_${dataType}_${ts}.json`;
      await exportData(dataType, savePath);
      toast.success(`导出成功：${savePath}`);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "导出失败");
    }
  };

  const handleConfirmDeleteLive = () => {
    if (!deleteLiveTarget) return;
    deleteLive.mutate({ id: deleteLiveTarget.id, deleteFile }, {
      onError: (err) => toast.error(err instanceof Error ? err.message : "删除失败"),
    });
    setDeleteLiveTarget(null);
    setDeleteFile(false);
  };

  const handleRemoveTask = (taskId: string) => {
    removeTask(taskId);
    deleteTask.mutate(taskId);
  };

  // 已完成的任务（用于"已完成" tab）
  const completedTasks = dbTasks.filter((t) => t.status === "completed" || t.status === "error");

  const { mergedTasks, runningCount, hasCompletedTasks } = useMergedTasks(dbTasks, liveTasks);

  return (
    <>
      <AnimateEntry>
        <Header title="下载管理" description="查看下载历史记录">
          <div className="flex items-center gap-2">
            {hasCompletedTasks && (
              <Button variant="capsule" size="sm" onClick={clearCompleted}>
                清除已完成任务
              </Button>
            )}
            <DropdownMenu>
              <DropdownMenuTrigger
                render={<Button variant="capsule" size="sm" />}
              >
                <DownloadCloud className="h-4 w-4 mr-1.5" />
                导出数据
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                {Object.entries(EXPORT_TYPES).map(([key, label]) => (
                  <DropdownMenuItem key={key} onClick={() => handleExport(key)}>
                    {label}
                  </DropdownMenuItem>
                ))}
              </DropdownMenuContent>
            </DropdownMenu>
            <Button variant="capsule" size="sm" onClick={handleOpenFolder}>
              <FolderOpen className="h-4 w-4 mr-1.5" />
              打开文件夹
            </Button>
          </div>
        </Header>
      </AnimateEntry>

      <Tabs defaultValue="batch">
        <TabsList>
          <TabsTrigger value="batch">
            下载任务
            {runningCount > 0 && (
              <Badge variant="default" className="ml-1.5 text-[10px]">
                {runningCount}
              </Badge>
            )}
          </TabsTrigger>
          <TabsTrigger value="completed">
            已完成
            {completedTasks.length > 0 && (
              <Badge variant="secondary" className="ml-1.5 text-[10px]">
                {completedTasks.length}
              </Badge>
            )}
          </TabsTrigger>
          <TabsTrigger value="live">
            直播录制
            {liveRecords.length > 0 && (
              <Badge variant="secondary" className="ml-1.5 text-[10px]">
                {liveRecords.length}
              </Badge>
            )}
          </TabsTrigger>
        </TabsList>

        <TabsContent value="batch" className="mt-8 space-y-2">
          {mergedTasks.length === 0 ? (
            <Bezel radius="xl">
              <div className="p-12 text-center text-muted-foreground">
                <Download className="h-10 w-10 mx-auto mb-4 opacity-30" />
                <p className="text-sm tracking-wide">没有下载任务</p>
              </div>
            </Bezel>
          ) : (
            mergedTasks.map((task, i) => (
              <AnimateEntry key={task.id} delay={i * 30}>
                <TaskCard
                  task={task}
                  liveState={liveTasks[task.id]}
                  onRemove={
                    task.status === "completed" || task.status === "error"
                      ? () => handleRemoveTask(task.id)
                      : undefined
                  }
                />
              </AnimateEntry>
            ))
          )}
        </TabsContent>

        <TabsContent value="completed" className="mt-8 space-y-2">
          {dbTasksQuery.isLoading ? (
            <div className="space-y-2">
              {Array.from({ length: 3 }).map((_, i) => (
                <div key={i} className="rounded-2xl bg-card p-4 space-y-3">
                  <div className="flex items-center gap-3">
                    <div className="h-9 w-9 rounded-xl bg-muted animate-pulse" />
                    <div className="flex-1 space-y-1.5">
                      <div className="h-4 w-48 bg-muted rounded animate-pulse" />
                      <div className="h-3 w-32 bg-muted rounded animate-pulse" />
                    </div>
                    <div className="h-5 w-16 bg-muted rounded-full animate-pulse" />
                  </div>
                  <div className="h-1.5 w-full bg-muted rounded-full animate-pulse" />
                </div>
              ))}
            </div>
          ) : completedTasks.length === 0 ? (
            <Bezel radius="xl">
              <div className="p-12 text-center text-muted-foreground">
                <CheckCircle2 className="h-10 w-10 mx-auto mb-4 opacity-30" />
                <p className="text-sm tracking-wide">没有已完成的下载</p>
              </div>
            </Bezel>
          ) : (
            completedTasks.map((task, i) => (
              <AnimateEntry key={task.id} delay={i * 30}>
                <TaskCard
                  task={task}
                  onRemove={() => handleRemoveTask(task.id)}
                />
              </AnimateEntry>
            ))
          )}
        </TabsContent>

        <TabsContent value="live" className="mt-8 space-y-2">
          {liveRecordsQuery.isLoading ? (
            <div className="space-y-2">
              {Array.from({ length: 3 }).map((_, i) => (
                <div key={i} className="rounded-2xl bg-card p-4">
                  <div className="flex items-center gap-4">
                    <div className="h-9 w-9 rounded-xl bg-muted animate-pulse" />
                    <div className="flex-1 space-y-1.5">
                      <div className="h-4 w-40 bg-muted rounded animate-pulse" />
                      <div className="h-3 w-24 bg-muted rounded animate-pulse" />
                    </div>
                    <div className="h-5 w-16 bg-muted rounded-full animate-pulse" />
                  </div>
                </div>
              ))}
            </div>
          ) : liveRecords.length === 0 ? (
            <Bezel radius="xl">
              <div className="p-12 text-center text-muted-foreground">
                <Radio className="h-10 w-10 mx-auto mb-4 opacity-30" />
                <p className="text-sm tracking-wide">没有直播录制记录</p>
              </div>
            </Bezel>
          ) : (
            liveRecords.map((item, i) => (
              <AnimateEntry key={item.id} delay={i * 30}>
                <Bezel radius="lg" padding="sm">
                  <div className="flex items-center gap-4 p-4 bg-card hover:bg-foreground/[0.02] transition-all duration-300">
                    <div className="h-9 w-9 rounded-xl bg-destructive/[0.06] ring-1 ring-destructive/10 flex items-center justify-center shrink-0">
                      <Radio className="h-4 w-4 text-destructive/70" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium truncate">
                        {item.file_path || item.title || "直播录制"}
                      </p>
                      <div className="flex items-center gap-3 mt-1">
                        {item.nickname && (
                          <span className="text-[11px] text-muted-foreground">{item.nickname}</span>
                        )}
                        {item.file_size > 0 && (
                          <span className="text-[11px] text-muted-foreground font-mono tabular-nums">
                            {formatFileSize(item.file_size)}
                          </span>
                        )}
                      </div>
                    </div>
                    <Badge variant={item.status === "completed" ? "default" : "destructive"} className="text-[11px] rounded-full">
                      {item.status === "completed" ? "已完成" : item.status}
                    </Badge>
                    <Button
                      variant="ghost"
                      size="icon-sm"
                      title="删除记录"
                      onClick={() => setDeleteLiveTarget(item)}
                    >
                      <Trash2 className="h-4 w-4 text-destructive" />
                    </Button>
                  </div>
                </Bezel>
              </AnimateEntry>
            ))
          )}
        </TabsContent>
      </Tabs>

      <AlertDialog open={!!deleteLiveTarget} onOpenChange={(open) => !open && setDeleteLiveTarget(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认删除</AlertDialogTitle>
            <AlertDialogDescription>
              确定删除这条直播录制记录？此操作不可撤销。
            </AlertDialogDescription>
          </AlertDialogHeader>
          {deleteLiveTarget?.file_path && (
            <label className="flex items-center gap-2 text-sm cursor-pointer">
              <Checkbox checked={deleteFile} onCheckedChange={(checked) => setDeleteFile(checked === true)} />
              同时删除本地文件
            </label>
          )}
          <AlertDialogFooter>
            <AlertDialogCancel onClick={() => setDeleteFile(false)}>取消</AlertDialogCancel>
            <AlertDialogAction variant="destructive" onClick={handleConfirmDeleteLive}>删除</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
