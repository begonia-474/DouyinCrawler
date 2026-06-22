import { useState, useCallback } from "react";
import { useLocation } from "react-router-dom";
import { Header } from "@/components/layout/header";
import { UrlInput } from "@/components/shared/url-input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { getLiveInfo, startLiveRecord, stopLiveRecord } from "@/lib/api";
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

  const lastParsedUrl = { current: "" };

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
      setRecording(false);
      setRecordTaskId(null);
    } else {
      setError(res.error || "停止录制失败");
    }
    setRecordLoading(false);
  }, [recordTaskId]);

  return (
    <>
      <Header title="直播" description="获取直播信息和流地址" parent={{ label: "首页", path: "/douyin" }} />

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
          <div className="flex items-center gap-2 p-3 rounded-lg bg-destructive/10 text-destructive text-sm">
            <AlertCircle className="h-4 w-4 shrink-0" />
            <span>{error}</span>
          </div>
        )}

        {loading && (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        )}

        {liveInfo && (
          <div className="space-y-4">
            <Card>
              <CardHeader>
                <div className="flex items-center justify-between">
                  <CardTitle className="text-lg">{liveInfo.title}</CardTitle>
                  <div className="flex items-center gap-2">
                    {recording && (
                      <Badge variant="destructive" className="animate-pulse">
                        <Disc className="h-3 w-3 mr-1" />
                        录制中
                      </Badge>
                    )}
                    <Badge variant={liveInfo.is_live ? "default" : "secondary"}>
                      {liveInfo.is_live ? (
                        <><Circle className="h-2 w-2 fill-current mr-1" />直播中</>
                      ) : "未开播"}
                    </Badge>
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <div className="grid grid-cols-3 gap-4 text-sm">
                  <div>
                    <p className="text-muted-foreground">主播</p>
                    <p className="font-medium">{liveInfo.nickname}</p>
                  </div>
                  <div>
                    <p className="text-muted-foreground">观看人数</p>
                    <p className="font-medium flex items-center gap-1">
                      <Users className="h-4 w-4" />
                      {liveInfo.user_count}
                    </p>
                  </div>
                  <div>
                    <p className="text-muted-foreground">房间号</p>
                    <p className="font-medium">{liveInfo.room_id}</p>
                  </div>
                </div>

                {liveInfo.is_live && (
                  <div className="mt-4 pt-4 border-t">
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
              </CardContent>
            </Card>

            {liveInfo.is_live && (
              <Card>
                <CardHeader>
                  <CardTitle className="text-base flex items-center gap-2">
                    <Radio className="h-4 w-4" />
                    流地址
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  {liveInfo.flv_urls?.length > 0 && (
                    <div className="space-y-2">
                      <Label>FLV</Label>
                      {liveInfo.flv_urls.map((url, i) => (
                        <div key={i} className="flex gap-2">
                          <Input value={url} readOnly className="flex-1 font-mono text-xs" />
                          <Button variant="outline" size="icon" onClick={() => handleCopy(url, `flv-${i}`)}>
                            {copied === `flv-${i}` ? <CheckCircle2 className="h-4 w-4 text-green-600" /> : <Copy className="h-4 w-4" />}
                          </Button>
                        </div>
                      ))}
                    </div>
                  )}

                  {liveInfo.m3u8_urls?.length > 0 && (
                    <div className="space-y-2">
                      <Label>M3U8</Label>
                      {liveInfo.m3u8_urls.map((url, i) => (
                        <div key={i} className="flex gap-2">
                          <Input value={url} readOnly className="flex-1 font-mono text-xs" />
                          <Button variant="outline" size="icon" onClick={() => handleCopy(url, `m3u8-${i}`)}>
                            {copied === `m3u8-${i}` ? <CheckCircle2 className="h-4 w-4 text-green-600" /> : <Copy className="h-4 w-4" />}
                          </Button>
                        </div>
                      ))}
                    </div>
                  )}
                </CardContent>
              </Card>
            )}
          </div>
        )}
      </div>
    </>
  );
}
