import { useState, useCallback, useRef, useEffect } from "react";
import { useLocation } from "react-router-dom";
import { Header } from "@/components/layout/header";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { UrlInput } from "@/components/shared/url-input";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Bezel } from "@/components/shared/bezel";
import { startLiveRecord, stopLiveRecord } from "@/lib/api";
import { useLiveInfoQuery } from "@/lib/queries";
import { useTaskStore } from "@/stores/task-store";
import type { LiveInfo as LiveInfoType } from "@/lib/api-types";
import {
  Radio,
  Users,
  Copy,
  CheckCircle2,
  Circle,
  Loader2,
  Disc,
  Square,
} from "lucide-react";
import { ErrorBanner } from "@/components/shared/error-banner";

export default function LivePage() {
  const location = useLocation();
  const initialUrl = (location.state as { url?: string })?.url || "";

  const [currentUrl, setCurrentUrl] = useState("");
  const [copied, setCopied] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [recordLoading, setRecordLoading] = useState(false);

  const lastParsedUrl = useRef("");

  const liveInfoQuery = useLiveInfoQuery(currentUrl || null);
  const liveInfo = liveInfoQuery.data?.data as LiveInfoType | undefined;

  // 使用全局 store
  const { tasks: allTasks, connect: connectLive, updateTask, removeTask } = useTaskStore();
  const liveTasks = Object.fromEntries(
    Object.entries(allTasks).filter(([, t]) => t.mode === "live")
  );

  // 查找当前正在进行的录制任务
  const activeTask = Object.values(liveTasks).find(
    (t) => t.status === "recording" || t.status === "starting" || t.status === "stopping"
  );
  const recording = !!activeTask;
  const recordTaskId = activeTask?.task_id || null;

  // 已通知过的完成/出错任务 ID（避免重复处理）
  const notifiedTaskIds = useRef<Set<string>>(new Set());

  // 连接 Tauri 事件
  useEffect(() => {
    connectLive();
  }, [connectLive]);

  // 监听任务完成/出错，显示错误并延迟清理
  useEffect(() => {
    const doneTasks = Object.values(liveTasks).filter((t) => t.status === "completed" || t.status === "error");
    for (const task of doneTasks) {
      if (notifiedTaskIds.current.has(task.task_id)) continue;
      notifiedTaskIds.current.add(task.task_id);

      if (task.status === "error") {
        setError(task.error || "录制出错");
      }
      setTimeout(() => removeTask(task.task_id), task.status === "completed" ? 3000 : 5000);
    }
  }, [liveTasks, removeTask]);

  const handleParse = useCallback((url: string) => {
    setError(null);
    lastParsedUrl.current = url;
    setCurrentUrl(url);
  }, []);

  const handleCopy = (text: string, type: string) => {
    navigator.clipboard.writeText(text);
    setCopied(type);
    window.setTimeout(() => setCopied(null), 2000);
  };

  const handleStartRecord = useCallback(async () => {
    if (!lastParsedUrl.current) return;
    setRecordLoading(true);
    const res = await startLiveRecord(lastParsedUrl.current);
    if (res.success && res.data) {
      useTaskStore.getState().addTask({
        task_id: res.data.task_id,
        task_type: "typed",
        mode: "live",
        url: lastParsedUrl.current,
        status: "starting",
      });
      setError(null);
    } else {
      setError(res.error || "启动录制失败");
    }
    setRecordLoading(false);
  }, []);

  const handleStopRecord = useCallback(async () => {
    if (!recordTaskId) return;
    setRecordLoading(true);
    const res = await stopLiveRecord(recordTaskId);
    if (res.success) {
      updateTask(recordTaskId, { status: "stopping" });
    } else {
      setError(res.error || "停止录制失败");
    }
    setRecordLoading(false);
  }, [recordTaskId, updateTask]);

  const queryError = liveInfoQuery.error?.message
    || (!liveInfoQuery.data?.success ? (liveInfoQuery.data?.error ?? null) : null)
    || error;

  return (
    <>
      <AnimateEntry>
        <Header title="直播" description="获取直播信息和流地址" parent={{ label: "首页", path: "/douyin" }} />
      </AnimateEntry>

      <div className="space-y-6">
        <UrlInput
          onSubmit={handleParse}
          loading={liveInfoQuery.isLoading}
          placeholder="粘贴直播间链接..."
          allowedTypes={["live"]}
          defaultValue={initialUrl}
          autoSubmit={!!initialUrl}
          autoDetect
        />

        <ErrorBanner message={queryError} />

        {liveInfoQuery.isLoading && (
          <div className="flex items-center justify-center py-16">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        )}

        {liveInfo && (
          <div className="space-y-5">
            <Bezel radius="xl">
              <div className="p-6">
                <div className="flex items-center justify-between mb-5">
                  <h3 className="font-heading text-lg font-semibold">{liveInfo.title}</h3>
                  <div className="flex items-center gap-2">
                    {recording && (
                      <Badge variant="destructive" className="animate-pulse rounded-full">
                        <Disc className="h-3 w-3 mr-1" />
                        {activeTask?.status === "stopping" ? "停止中" : "录制中"}
                      </Badge>
                    )}
                    <Badge variant={liveInfo.is_live ? "default" : "secondary"} className="rounded-full">
                      {liveInfo.is_live ? (
                        <><Circle className="h-2 w-2 fill-current mr-1" />直播中</>
                      ) : "未开播"}
                    </Badge>
                  </div>
                </div>
                <div className="grid grid-cols-3 gap-6 text-sm">
                  <div>
                    <p className="text-xs uppercase tracking-[0.15em] font-medium text-muted-foreground mb-1">主播</p>
                    <p className="font-medium">{liveInfo.nickname}</p>
                  </div>
                  <div>
                    <p className="text-xs uppercase tracking-[0.15em] font-medium text-muted-foreground mb-1">观看人数</p>
                    <p className="font-heading font-medium tabular-nums flex items-center gap-1">
                      <Users className="h-4 w-4" />
                      {liveInfo.user_count}
                    </p>
                  </div>
                  <div>
                    <p className="text-xs uppercase tracking-[0.15em] font-medium text-muted-foreground mb-1">房间号</p>
                    <p className="font-heading font-medium tabular-nums">{liveInfo.room_id}</p>
                  </div>
                </div>

                {liveInfo.is_live && (
                  <div className="mt-5 pt-5 border-t border-foreground/[0.06]">
                    {!recording ? (
                      <Button onClick={handleStartRecord} disabled={recordLoading}>
                        {recordLoading ? (
                          <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                        ) : (
                          <Disc className="h-4 w-4 mr-2" />
                        )}
                        开始录制
                      </Button>
                    ) : (
                      <Button variant="destructive" onClick={handleStopRecord} disabled={recordLoading || activeTask?.status === "stopping"}>
                        {recordLoading ? (
                          <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                        ) : (
                          <Square className="h-4 w-4 mr-2" />
                        )}
                        停止录制
                      </Button>
                    )}
                  </div>
                )}
              </div>
            </Bezel>

            {liveInfo.is_live && (
              <Bezel radius="xl">
                <div className="p-6 space-y-5">
                  <div className="flex items-center gap-2">
                    <Radio className="h-4 w-4 text-muted-foreground" />
                    <span className="text-sm font-medium">流地址</span>
                  </div>

                  {liveInfo.flv_urls?.length > 0 && (
                    <div className="space-y-2">
                      <Label className="text-xs uppercase tracking-[0.1em] text-muted-foreground">FLV</Label>
                      {liveInfo.flv_urls.map((url, i) => (
                        <div key={i} className="flex gap-2">
                          <Input value={url} readOnly className="flex-1 font-mono text-xs rounded-xl border-foreground/[0.08] bg-foreground/[0.03]" />
                          <Button variant="capsule" size="icon" onClick={() => handleCopy(url, `flv-${i}`)}>
                            {copied === `flv-${i}` ? <CheckCircle2 className="h-4 w-4 text-success" /> : <Copy className="h-4 w-4" />}
                          </Button>
                        </div>
                      ))}
                    </div>
                  )}

                  {liveInfo.m3u8_urls?.length > 0 && (
                    <div className="space-y-2">
                      <Label className="text-xs uppercase tracking-[0.1em] text-muted-foreground">M3U8</Label>
                      {liveInfo.m3u8_urls.map((url, i) => (
                        <div key={i} className="flex gap-2">
                          <Input value={url} readOnly className="flex-1 font-mono text-xs rounded-xl border-foreground/[0.08] bg-foreground/[0.03]" />
                          <Button variant="capsule" size="icon" onClick={() => handleCopy(url, `m3u8-${i}`)}>
                            {copied === `m3u8-${i}` ? <CheckCircle2 className="h-4 w-4 text-success" /> : <Copy className="h-4 w-4" />}
                          </Button>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </Bezel>
            )}
          </div>
        )}
      </div>
    </>
  );
}
