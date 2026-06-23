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
import { getLiveInfo, startLiveRecord, stopLiveRecord, getLiveStatus, saveLiveRecordAfterStop } from "@/lib/api";
import type { LiveInfo as LiveInfoType } from "@/lib/api-types";
import {
  Radio,
  Users,
  Copy,
  CheckCircle2,
  Circle,
  AlertCircle,
  Loader2,
  Disc,
  Square,
} from "lucide-react";

export default function LivePage() {
  const location = useLocation();
  const initialUrl = (location.state as { url?: string })?.url || "";

  const [loading, setLoading] = useState(false);
  const [liveInfo, setLiveInfo] = useState<LiveInfoType | null>(null);
  const [copied, setCopied] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [recording, setRecording] = useState(false);
  const [recordTaskId, setRecordTaskId] = useState<string | null>(null);
  const [recordLoading, setRecordLoading] = useState(false);

  const lastParsedUrl = useRef("");
  const pollTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    return () => {
      if (pollTimerRef.current) clearInterval(pollTimerRef.current);
    };
  }, []);

  const handleParse = useCallback(async (url: string) => {
    setLoading(true);
    setLiveInfo(null);
    setError(null);
    lastParsedUrl.current = url;

    const res = await getLiveInfo(url);
    if (res.success && res.data) {
      setLiveInfo(res.data);
    } else {
      setError(res.error || "获取直播信息失败");
    }
    setLoading(false);
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
      setRecording(true);
      setRecordTaskId(res.data.task_id);
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
      const taskId = recordTaskId;
      let attempts = 0;
      pollTimerRef.current = setInterval(async () => {
        attempts++;
        try {
          const statusRes = await getLiveStatus();
          if (statusRes.success && statusRes.data) {
            const task = statusRes.data[taskId];
            if (task?.status === "completed") {
              if (pollTimerRef.current) clearInterval(pollTimerRef.current);
              try {
                await saveLiveRecordAfterStop(task);
              } catch (saveErr) {
                console.error("保存录制记录失败:", saveErr);
                setError("录制已完成，但保存记录失败");
              }
              setRecording(false);
              setRecordTaskId(null);
              setRecordLoading(false);
              return;
            } else if (task?.status === "error") {
              if (pollTimerRef.current) clearInterval(pollTimerRef.current);
              setError(task.error || "录制出错");
              setRecording(false);
              setRecordTaskId(null);
              setRecordLoading(false);
              return;
            }
          }
        } catch {
          // ignore
        }
        if (attempts >= 30) {
          if (pollTimerRef.current) clearInterval(pollTimerRef.current);
          setRecording(false);
          setRecordTaskId(null);
          setRecordLoading(false);
        }
      }, 1000);
    } else {
      setError(res.error || "停止录制失败");
      setRecordLoading(false);
    }
  }, [recordTaskId]);

  return (
    <>
      <AnimateEntry>
        <Header title="直播" description="获取直播信息和流地址" parent={{ label: "首页", path: "/douyin" }} />
      </AnimateEntry>

      <div className="space-y-6">
        <UrlInput
          onSubmit={handleParse}
          loading={loading}
          placeholder="粘贴直播间链接..."
          allowedTypes={["live"]}
          defaultValue={initialUrl}
          autoSubmit={!!initialUrl}
        />

        {error && (
          <div className="flex items-center gap-2 p-4 rounded-2xl bg-destructive/[0.06] ring-1 ring-destructive/20 text-destructive text-sm">
            <AlertCircle className="h-4 w-4 shrink-0" />
            <span>{error}</span>
          </div>
        )}

        {loading && (
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
                        录制中
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
                      <Button variant="destructive" onClick={handleStopRecord} disabled={recordLoading}>
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
