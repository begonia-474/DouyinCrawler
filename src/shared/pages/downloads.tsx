import { useState, useEffect, useCallback } from "react";
import { Header } from "@/components/layout/header";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { Bezel } from "@/components/shared/bezel";
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
  XCircle,
  Disc,
} from "lucide-react";
import { deleteDownloadRecord, deleteLiveRecord, getDownloads, getLiveRecords } from "@/lib/api";
import { useBatchStore } from "@/stores/batch-store";
import { useLiveStore } from "@/stores/live-store";
import type { DownloadRecord, LiveRecord } from "@/lib/tauri-types";
import { formatFileSize, formatTimestamp } from "@/lib/utils";

const typeIcons: Record<string, typeof Video> = {
  video: Video,
  images: Image,
  live: Radio,
  music: Music,
};

export default function DownloadsPage() {
  const [downloads, setDownloads] = useState<DownloadRecord[]>([]);
  const [liveRecords, setLiveRecords] = useState<LiveRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const { tasks: batchTasks, connect: connectBatch, clearCompleted, removeTask: removeBatchTask } = useBatchStore();
  const { tasks: liveTasks, removeTask: removeLiveTask } = useLiveStore();

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [dl, lr] = await Promise.all([
        getDownloads({ limit: 50, status: "completed" }),
        getLiveRecords({ limit: 50 }),
      ]);
      setDownloads(dl);
      setLiveRecords(lr);
    } catch (err) {
      console.error("[Downloads] 加载下载记录失败:", err);
    }
    setLoading(false);
  }, []);

  useEffect(() => {
    loadData();
    connectBatch();

    // 监听下载记录更新事件
    const handleUpdate = () => loadData();
    window.addEventListener("download-records-updated", handleUpdate);
    return () => window.removeEventListener("download-records-updated", handleUpdate);
  }, [loadData, connectBatch]);

  const askDeleteFile = (filePath: string | null) => {
    if (!filePath) return false;
    return window.confirm("是否同时删除这条记录对应的本地文件？\n\n取消则只删除记录。");
  };

  const handleDeleteDownload = async (item: DownloadRecord) => {
    if (!window.confirm("确定删除这条下载记录？")) return;
    try {
      await deleteDownloadRecord(item.id, askDeleteFile(item.file_path));
      await loadData();
    } catch (err) {
      window.alert(err instanceof Error ? err.message : "删除失败");
    }
  };

  const handleDeleteLiveRecord = async (item: LiveRecord) => {
    if (!window.confirm("确定删除这条直播录制记录？")) return;
    try {
      await deleteLiveRecord(item.id, askDeleteFile(item.file_path));
      await loadData();
    } catch (err) {
      window.alert(err instanceof Error ? err.message : "删除失败");
    }
  };

  const completedDownloads = downloads.filter((d) => d.status === "completed");
  const batchTaskList = Object.values(batchTasks);
  const runningBatchTasks = batchTaskList.filter((t) => t.status === "running" || t.status === "starting");
  const completedBatchTasks = batchTaskList.filter((t) => t.status === "completed" || t.status === "error");

  const liveTaskList = Object.values(liveTasks);
  const runningLiveTasks = liveTaskList.filter((t) => t.status === "recording" || t.status === "starting" || t.status === "stopping");
  const completedLiveTasks = liveTaskList.filter((t) => t.status === "completed" || t.status === "error");

  const typeLabels: Record<string, string> = {
    user_post: "用户主页",
    user_like: "用户点赞",
    mix: "合集",
    collects: "收藏夹",
  };

  return (
    <>
      <AnimateEntry>
        <Header title="下载管理" description="查看下载历史记录">
          <div className="flex items-center gap-2">
            {(completedBatchTasks.length > 0 || completedLiveTasks.length > 0) && (
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
            {(runningBatchTasks.length + runningLiveTasks.length) > 0 && (
              <Badge variant="default" className="ml-1.5 text-[10px]">
                {runningBatchTasks.length + runningLiveTasks.length}
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
          {batchTaskList.length === 0 && liveTaskList.length === 0 ? (
            <Bezel radius="xl">
              <div className="p-12 text-center text-muted-foreground">
                <Download className="h-10 w-10 mx-auto mb-4 opacity-30" />
                <p className="text-sm tracking-wide">没有下载任务</p>
              </div>
            </Bezel>
          ) : (
            <>
              {/* 批量下载任务 */}
              {batchTaskList.map((task, i) => (
              <AnimateEntry key={task.task_id} delay={i * 30}>
                <Bezel radius="lg" padding="sm">
                  <div className="p-4">
                    <div className="flex items-center gap-4 mb-3">
                      <div className="h-9 w-9 rounded-xl bg-primary/10 ring-1 ring-primary/20 flex items-center justify-center shrink-0">
                        {task.status === "running" || task.status === "starting" ? (
                          <Loader2 className="h-4 w-4 text-primary animate-spin" />
                        ) : task.status === "completed" ? (
                          <CheckCircle2 className="h-4 w-4 text-success" />
                        ) : (
                          <XCircle className="h-4 w-4 text-destructive" />
                        )}
                      </div>
                      <div className="flex-1 min-w-0">
                        <p className="text-sm font-medium truncate">
                          {typeLabels[task.type] || task.type}下载
                        </p>
                        <p className="text-xs text-muted-foreground truncate mt-0.5">
                          {task.url}
                        </p>
                      </div>
                      <Badge
                        variant={task.status === "completed" ? "default" : task.status === "error" ? "destructive" : "secondary"}
                        className="text-[11px] rounded-full"
                      >
                        {task.status === "running" ? "下载中" : task.status === "starting" ? "启动中" : task.status === "completed" ? "已完成" : "失败"}
                      </Badge>
                      {(task.status === "completed" || task.status === "error") && (
                        <Button
                          variant="ghost"
                          size="icon-sm"
                          title="移除任务"
                          onClick={() => removeBatchTask(task.task_id)}
                        >
                          <Trash2 className="h-4 w-4 text-muted-foreground" />
                        </Button>
                      )}
                    </div>

                    {(task.status === "running" || task.status === "starting") && (
                      <div className="space-y-2">
                        <Progress value={task.total > 0 ? (task.completed / task.total) * 100 : 0} className="h-1.5" />
                        <div className="flex items-center justify-between text-xs text-muted-foreground">
                          <span className="truncate">{task.current_item || "准备中..."}</span>
                          <span className="font-mono tabular-nums">{task.completed}/{task.total}</span>
                        </div>
                      </div>
                    )}

                    {task.status === "error" && task.error && (
                      <p className="text-xs text-destructive mt-2">{task.error}</p>
                    )}
                  </div>
                </Bezel>
              </AnimateEntry>
            ))}

              {/* 直播录制任务 */}
              {liveTaskList.map((task, i) => (
                <AnimateEntry key={task.task_id} delay={(batchTaskList.length + i) * 30}>
                  <Bezel radius="lg" padding="sm">
                    <div className="p-4">
                      <div className="flex items-center gap-4 mb-3">
                        <div className="h-9 w-9 rounded-xl bg-destructive/10 ring-1 ring-destructive/20 flex items-center justify-center shrink-0">
                          {task.status === "recording" || task.status === "starting" || task.status === "stopping" ? (
                            <Loader2 className="h-4 w-4 text-destructive animate-spin" />
                          ) : task.status === "completed" ? (
                            <CheckCircle2 className="h-4 w-4 text-success" />
                          ) : (
                            <XCircle className="h-4 w-4 text-destructive" />
                          )}
                        </div>
                        <div className="flex-1 min-w-0">
                          <p className="text-sm font-medium truncate">
                            直播录制 - {task.nickname || "未知主播"}
                          </p>
                          <p className="text-xs text-muted-foreground truncate mt-0.5">
                            {task.title || task.url}
                          </p>
                        </div>
                        <Badge
                          variant={task.status === "completed" ? "default" : task.status === "error" ? "destructive" : "secondary"}
                          className="text-[11px] rounded-full"
                        >
                          {task.status === "recording" ? "录制中" : task.status === "starting" ? "启动中" : task.status === "stopping" ? "停止中" : task.status === "completed" ? "已完成" : "失败"}
                        </Badge>
                        {(task.status === "completed" || task.status === "error") && (
                          <Button
                            variant="ghost"
                            size="icon-sm"
                            title="移除任务"
                            onClick={() => removeLiveTask(task.task_id)}
                          >
                            <Trash2 className="h-4 w-4 text-muted-foreground" />
                          </Button>
                        )}
                      </div>

                      {(task.status === "recording" || task.status === "starting") && (
                        <div className="flex items-center gap-2 text-xs text-muted-foreground">
                          <Disc className="h-3 w-3 animate-pulse text-destructive" />
                          <span>正在录制中...</span>
                        </div>
                      )}

                      {task.status === "error" && task.error && (
                        <p className="text-xs text-destructive mt-2">{task.error}</p>
                      )}
                    </div>
                  </Bezel>
                </AnimateEntry>
              ))}
            </>
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
