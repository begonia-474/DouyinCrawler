import { useState, useEffect, useCallback } from "react";
import { Header } from "@/components/layout/header";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
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
} from "lucide-react";
import { getDownloads, getLiveRecords } from "@/lib/api";
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
  }, [loadData]);

  const completedDownloads = downloads.filter((d) => d.status === "completed");

  return (
    <>
      <AnimateEntry>
        <Header title="下载管理" description="查看下载历史记录">
          <Button variant="capsule" size="sm">
            <FolderOpen className="h-4 w-4 mr-1.5" />
            打开文件夹
          </Button>
        </Header>
      </AnimateEntry>

      <Tabs defaultValue="completed">
        <TabsList>
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
                          {item.title || item.file_path || "未知文件"}
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
                        {item.title || "直播录制"}
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
