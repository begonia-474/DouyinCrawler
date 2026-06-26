import { useEffect, useMemo } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { Header } from "@/components/layout/header";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { Bezel } from "@/components/shared/bezel";
import { TaskCard } from "@/components/shared/task-card";
import {
  Download,
  CheckCircle2,
  Loader2,
  FolderOpen,
  Video,
  Image,
  Radio,
  Music,
  Trash2,
} from "lucide-react";
import { deleteDownloadRecord, deleteLiveRecord, deleteDownloadTask } from "@/lib/api";
import { queryKeys } from "@/lib/query-keys";
import { useDownloadsQuery, useLiveRecordsQuery, useDownloadTasksQuery } from "@/lib/queries";
import { useTaskStore } from "@/stores/task-store";
import type { DownloadRecord, LiveRecord } from "@/lib/tauri-types";
import type { DownloadTask } from "@/lib/api-types";
import { formatFileSize, formatTimestamp } from "@/lib/utils";

const typeIcons: Record<string, typeof Video> = {
  video: Video,
  images: Image,
  live: Radio,
  music: Music,
};

export default function DownloadsPage() {
  const queryClient = useQueryClient();
  const downloadsQuery = useDownloadsQuery({ limit: 50, status: "completed" });
  const liveRecordsQuery = useLiveRecordsQuery({ limit: 50 });
  const dbTasksQuery = useDownloadTasksQuery({ limit: 100 });
  const downloads = downloadsQuery.data ?? [];
  const liveRecords = liveRecordsQuery.data ?? [];
  const dbTasks = dbTasksQuery.data ?? [];
  const loading = downloadsQuery.isLoading || liveRecordsQuery.isLoading;
  const { tasks: liveTasks, connect, clearCompleted, removeTask } = useTaskStore();

  useEffect(() => {
    connect();
  }, [connect]);

  const askDeleteFile = (filePath: string | null) => {
    if (!filePath) return false;
    return window.confirm("是否同时删除这条记录对应的本地文件？\n\n取消则只删除记录。");
  };

  const handleDeleteDownload = async (item: DownloadRecord) => {
    if (!window.confirm("确定删除这条下载记录？")) return;
    try {
      await deleteDownloadRecord(item.id, askDeleteFile(item.file_path));
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: queryKeys.downloads() }),
        queryClient.invalidateQueries({ queryKey: queryKeys.downloadStats() }),
      ]);
    } catch (err) {
      window.alert(err instanceof Error ? err.message : "删除失败");
    }
  };

  const handleDeleteLiveRecord = async (item: LiveRecord) => {
    if (!window.confirm("确定删除这条直播录制记录？")) return;
    try {
      await deleteLiveRecord(item.id, askDeleteFile(item.file_path));
      await queryClient.invalidateQueries({ queryKey: queryKeys.liveRecords() });
    } catch (err) {
      window.alert(err instanceof Error ? err.message : "删除失败");
    }
  };

  const handleRemoveTask = async (taskId: string) => {
    removeTask(taskId);
    try {
      await deleteDownloadTask(taskId);
      await queryClient.invalidateQueries({ queryKey: queryKeys.downloadTasks() });
    } catch {
      // DB 删除失败不影响 UI 移除
    }
  };

  const completedDownloads = downloads.filter((d) => d.status === "completed");

  // 合并 DB 任务和实时任务：实时状态覆盖 DB 数据
  const mergedTasks = useMemo(() => {
    const taskMap = new Map<string, DownloadTask>();

    // 先放入 DB 任务
    for (const task of dbTasks) {
      taskMap.set(task.id, task);
    }

    // 实时任务覆盖或补充
    for (const live of Object.values(liveTasks)) {
      const existing = taskMap.get(live.task_id);
      if (existing) {
        // 覆盖实时字段
        taskMap.set(live.task_id, {
          ...existing,
          status: (live.status as DownloadTask["status"]) ?? existing.status,
          total: live.total ?? existing.total,
          completed: live.completed ?? existing.completed,
          failed: live.failed ?? existing.failed,
          error_msg: live.error ?? existing.error_msg,
        });
      } else {
        // 实时任务不在 DB 中（刚启动还没写入），创建临时条目
        taskMap.set(live.task_id, {
          id: live.task_id,
          mode: (live.type as DownloadTask["mode"]) ?? "one",
          url: live.url,
          title: live.title ?? live.nickname ?? null,
          status: (live.status as DownloadTask["status"]) ?? "running",
          total: live.total ?? 0,
          completed: live.completed ?? 0,
          skipped: 0,
          failed: live.failed ?? 0,
          error_msg: live.error ?? null,
          created_at: 0,
          updated_at: 0,
        });
      }
    }

    // 按创建时间倒序（最新在前），实时任务（created_at=0）排最前面
    return Array.from(taskMap.values()).sort((a, b) => b.created_at - a.created_at || b.id.localeCompare(a.id));
  }, [dbTasks, liveTasks]);

  const runningCount = mergedTasks.filter(
    (t) => t.status === "running" || t.status === "starting" || t.status === "recording" || t.status === "stopping"
  ).length;

  const hasCompletedTasks = mergedTasks.some(
    (t) => t.status === "completed" || t.status === "error"
  );

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
            <Button variant="capsule" size="sm">
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
            {completedDownloads.length > 0 && (
              <Badge variant="secondary" className="ml-1.5 text-[10px]">
                {completedDownloads.length}
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
          {loading ? (
            <div className="flex justify-center py-16">
              <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
            </div>
          ) : completedDownloads.length === 0 ? (
            <Bezel radius="xl">
              <div className="p-12 text-center text-muted-foreground">
                <CheckCircle2 className="h-10 w-10 mx-auto mb-4 opacity-30" />
                <p className="text-sm tracking-wide">没有已完成的下载</p>
              </div>
            </Bezel>
          ) : (
            completedDownloads.map((item, i) => {
              const Icon = typeIcons[item.download_type] || Download;
              return (
                <AnimateEntry key={item.id} delay={i * 30}>
                  <Bezel radius="lg" padding="sm">
                    <div className="flex items-center gap-4 p-4 bg-card hover:bg-foreground/[0.02] transition-all duration-300">
                      <div className="h-9 w-9 rounded-xl bg-foreground/[0.04] ring-1 ring-foreground/[0.06] flex items-center justify-center shrink-0">
                        <Icon className="h-4 w-4 text-muted-foreground" />
                      </div>
                      <div className="flex-1 min-w-0">
                        <p className="text-sm font-medium truncate">
                          {item.file_path || item.title || "未知文件"}
                        </p>
                        <div className="flex items-center gap-3 mt-1">
                          {item.file_size > 0 && (
                            <span className="text-[11px] text-muted-foreground font-mono tabular-nums">
                              {formatFileSize(item.file_size)}
                            </span>
                          )}
                          <span className="text-[11px] text-muted-foreground">
                            {formatTimestamp(item.created_at)}
                          </span>
                          {item.author_nickname && (
                            <span className="text-[11px] text-muted-foreground">
                              {item.author_nickname}
                            </span>
                          )}
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        {item.file_path && (
                          <Button variant="ghost" size="icon-sm" title="打开文件所在文件夹">
                            <FolderOpen className="h-4 w-4" />
                          </Button>
                        )}
                        <Button
                          variant="ghost"
                          size="icon-sm"
                          title="删除记录"
                          onClick={() => handleDeleteDownload(item)}
                        >
                          <Trash2 className="h-4 w-4 text-destructive" />
                        </Button>
                      </div>
                    </div>
                  </Bezel>
                </AnimateEntry>
              );
            })
          )}
        </TabsContent>

        <TabsContent value="live" className="mt-8 space-y-2">
          {loading ? (
            <div className="flex justify-center py-16">
              <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
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
                      onClick={() => handleDeleteLiveRecord(item)}
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
    </>
  );
}
