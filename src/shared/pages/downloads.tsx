import { useState, useEffect, useCallback } from "react";
import { Header } from "@/components/layout/header";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
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
import { getDownloads, getLiveRecords } from "@/lib/tauri-api";
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
      console.log("[Downloads] 开始加载数据...");
      const [dl, lr] = await Promise.all([
        getDownloads({ limit: 50, status: "completed" }),
        getLiveRecords({ limit: 50 }),
      ]);
      console.log("[Downloads] 下载记录:", dl);
      console.log("[Downloads] 直播记录:", lr);
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
      <Header title="下载管理" description="查看下载历史记录">
        <Button variant="outline" size="sm">
          <FolderOpen className="h-4 w-4 mr-1" />
          打开文件夹
        </Button>
      </Header>

      <Tabs defaultValue="completed">
        <TabsList>
          <TabsTrigger value="completed">
            已完成
            {completedDownloads.length > 0 && (
              <Badge variant="secondary" className="ml-1.5">
                {completedDownloads.length}
              </Badge>
            )}
          </TabsTrigger>
          <TabsTrigger value="live">
            直播录制
            {liveRecords.length > 0 && (
              <Badge variant="secondary" className="ml-1.5">
                {liveRecords.length}
              </Badge>
            )}
          </TabsTrigger>
        </TabsList>

        <TabsContent value="completed" className="mt-4 space-y-3">
          {loading ? (
            <div className="flex justify-center py-12">
              <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
            </div>
          ) : completedDownloads.length === 0 ? (
            <Card>
              <CardContent className="p-8 text-center text-muted-foreground">
                <CheckCircle2 className="h-10 w-10 mx-auto mb-3" />
                <p>没有已完成的下载</p>
              </CardContent>
            </Card>
          ) : (
            completedDownloads.map((item) => {
              const Icon = typeIcons[item.download_type] || Download;
              return (
                <div
                  key={item.id}
                  className="flex items-center gap-4 p-4 border rounded-lg"
                >
                  <Icon className="h-4 w-4 text-muted-foreground shrink-0" />
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium truncate">
                      {item.title || item.file_path || "未知文件"}
                    </p>
                    <div className="flex items-center gap-2 mt-1">
                      {item.file_size > 0 && (
                        <span className="text-xs text-muted-foreground">
                          {formatFileSize(item.file_size)}
                        </span>
                      )}
                      <span className="text-xs text-muted-foreground">
                        {formatTimestamp(item.created_at)}
                      </span>
                      {item.author_nickname && (
                        <span className="text-xs text-muted-foreground">
                          {item.author_nickname}
                        </span>
                      )}
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    {item.file_path && (
                      <Button variant="ghost" size="icon" title="打开文件所在文件夹">
                        <FolderOpen className="h-4 w-4" />
                      </Button>
                    )}
                  </div>
                </div>
              );
            })
          )}
        </TabsContent>

        <TabsContent value="live" className="mt-4 space-y-3">
          {loading ? (
            <div className="flex justify-center py-12">
              <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
            </div>
          ) : liveRecords.length === 0 ? (
            <Card>
              <CardContent className="p-8 text-center text-muted-foreground">
                <Radio className="h-10 w-10 mx-auto mb-3" />
                <p>没有直播录制记录</p>
              </CardContent>
            </Card>
          ) : (
            liveRecords.map((item) => (
              <div
                key={item.id}
                className="flex items-center gap-4 p-4 border rounded-lg"
              >
                <Radio className="h-4 w-4 text-red-500 shrink-0" />
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium truncate">
                    {item.title || "直播录制"}
                  </p>
                  <div className="flex items-center gap-2 mt-1">
                    {item.nickname && (
                      <span className="text-xs text-muted-foreground">
                        {item.nickname}
                      </span>
                    )}
                    {item.file_size > 0 && (
                      <span className="text-xs text-muted-foreground">
                        {formatFileSize(item.file_size)}
                      </span>
                    )}
                  </div>
                </div>
                <Badge variant={item.status === "completed" ? "default" : "destructive"}>
                  {item.status === "completed" ? "已完成" : item.status}
                </Badge>
              </div>
            ))
          )}
        </TabsContent>
      </Tabs>
    </>
  );
}
