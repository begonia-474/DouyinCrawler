import { useState, useCallback } from "react";
import { Header } from "@/components/layout/header";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { UrlInput } from "@/components/shared/url-input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { getMixInfo, downloadMix } from "@/lib/api";
import type { VideoItem } from "@/lib/api-types";
import {
  Download,
  Layers,
  Loader2,
  CheckCircle2,
  ListVideo,
  AlertCircle,
} from "lucide-react";

function formatCount(n: number): string {
  if (n >= 10000) return `${(n / 10000).toFixed(1)}w`;
  return n.toLocaleString();
}

function formatDuration(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
}

export default function MixPage() {
  const [loading, setLoading] = useState(false);
  const [downloading, setDownloading] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [downloadedCount, setDownloadedCount] = useState(0);
  const [mixName, setMixName] = useState("");
  const [videos, setVideos] = useState<(VideoItem & { downloaded?: boolean })[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [currentUrl, setCurrentUrl] = useState("");

  const handleParse = useCallback(async (url: string) => {
    setLoading(true);
    setVideos([]);
    setMixName("");
    setError(null);
    setCurrentUrl(url);

    const res = await getMixInfo(url);
    if (res.success && res.data?.videos) {
      setMixName(res.data.detail?.desc || "合集");
      setVideos(res.data.videos.map((v) => ({ ...v, downloaded: false })));
    } else {
      setError(res.error || "获取合集失败");
    }
    setLoading(false);
  }, []);

  const handleDownloadAll = async () => {
    setDownloading(true);
    setDownloadProgress(0);
    setDownloadedCount(0);

    const res = await downloadMix(currentUrl);
    if (res.success) {
      setDownloadedCount(videos.length);
      setDownloadProgress(100);
      setVideos((prev) => prev.map((v) => ({ ...v, downloaded: true })));
    } else {
      setError(res.error || "下载失败");
    }
    setDownloading(false);
  };

  return (
    <>
      <AnimateEntry>
        <Header title="合集" description="下载整个合集/播放列表" parent={{ label: "首页", path: "/douyin" }}>
          {videos.length > 0 && (
            <Button onClick={handleDownloadAll} disabled={downloading}>
              {downloading ? <Loader2 className="h-4 w-4 mr-2 animate-spin" /> : <Download className="h-4 w-4 mr-2" />}
              {downloading ? `下载中 ${downloadedCount}/${videos.length}` : "全部下载"}
            </Button>
          )}
        </Header>
      </AnimateEntry>

      <div className="space-y-6">
        <UrlInput onSubmit={handleParse} loading={loading} placeholder="粘贴合集链接..." allowedTypes={["mix"]} />

        {error && (
          <div className="flex items-center gap-2 p-3 rounded-lg bg-destructive/10 text-destructive text-sm">
            <AlertCircle className="h-4 w-4 shrink-0" />
            <span>{error}</span>
          </div>
        )}

        {videos.length > 0 && (
          <>
            <Card className="border-border/40 bg-card/60">
              <CardHeader>
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <div className="h-10 w-10 rounded-lg bg-primary/10 flex items-center justify-center">
                      <Layers className="h-5 w-5 text-primary" />
                    </div>
                    <div>
                      <CardTitle>{mixName}</CardTitle>
                      <p className="text-sm text-muted-foreground tracking-wide">{videos.length} 个视频</p>
                    </div>
                  </div>
                  <Badge variant="secondary"><ListVideo className="h-3 w-3 mr-1" />合集</Badge>
                </div>
              </CardHeader>
              {downloading && (
                <CardContent className="pt-0">
                  <div className="space-y-1">
                    <Progress value={downloadProgress} />
                    <p className="text-xs text-muted-foreground tracking-wide text-right">{downloadedCount} / {videos.length}</p>
                  </div>
                </CardContent>
              )}
            </Card>

            <div className="space-y-3">
              {videos.map((video, i) => (
                <Card key={video.aweme_id} className={`border-border/40 bg-card/60 hover:-translate-y-1 transition-all duration-500 ${video.downloaded ? "border-success/30" : ""}`}>
                  <CardContent className="p-4 flex items-center gap-4">
                    <div className="h-8 w-8 rounded-full bg-muted flex items-center justify-center text-sm font-medium shrink-0">
                      {video.downloaded ? <CheckCircle2 className="h-4 w-4 text-success" /> : i + 1}
                    </div>
                    <div className="flex-1 min-w-0">
                      <h4 className="text-sm font-medium truncate">{video.desc}</h4>
                      <p className="text-xs text-muted-foreground tracking-wide">
                        {formatDuration(video.duration)} · {formatCount(video.digg_count)} 赞 · {video.comment_count} 评论
                      </p>
                    </div>
                    <Button
                      variant={video.downloaded ? "outline" : "default"}
                      size="sm"
                      disabled={video.downloaded}
                    >
                      {video.downloaded ? (
                        <><CheckCircle2 className="h-3.5 w-3.5 mr-1" />已下载</>
                      ) : (
                        <><Download className="h-3.5 w-3.5 mr-1" />下载</>
                      )}
                    </Button>
                  </CardContent>
                </Card>
              ))}
            </div>
          </>
        )}
      </div>
    </>
  );
}
