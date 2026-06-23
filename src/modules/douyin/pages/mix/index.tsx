import { useState, useCallback } from "react";
import { Header } from "@/components/layout/header";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { UrlInput } from "@/components/shared/url-input";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { Bezel } from "@/components/shared/bezel";
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
          <div className="flex items-center gap-2 p-4 rounded-2xl bg-destructive/[0.06] ring-1 ring-destructive/20 text-destructive text-sm">
            <AlertCircle className="h-4 w-4 shrink-0" />
            <span>{error}</span>
          </div>
        )}

        {videos.length > 0 && (
          <>
            <AnimateEntry>
              <Bezel radius="xl">
                <div className="p-6">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-4">
                      <div className="h-11 w-11 rounded-2xl bg-primary/10 ring-1 ring-primary/15 flex items-center justify-center shrink-0">
                        <Layers className="h-5 w-5 text-primary" />
                      </div>
                      <div>
                        <h3 className="font-heading font-semibold">{mixName}</h3>
                        <p className="text-sm text-muted-foreground tracking-wide">{videos.length} 个视频</p>
                      </div>
                    </div>
                    <Badge variant="secondary" className="rounded-full"><ListVideo className="h-3 w-3 mr-1" />合集</Badge>
                  </div>
                  {downloading && (
                    <div className="mt-5 space-y-1">
                      <Progress value={downloadProgress} />
                      <p className="text-xs text-muted-foreground tracking-wide text-right">{downloadedCount} / {videos.length}</p>
                    </div>
                  )}
                </div>
              </Bezel>
            </AnimateEntry>

            <div className="space-y-2">
              {videos.map((video, i) => (
                <AnimateEntry key={video.aweme_id} delay={i * 25}>
                  <Bezel radius="lg" padding="sm">
                    <div className={`flex items-center gap-4 p-4 bg-card transition-all duration-300 ${video.downloaded ? "bg-success/[0.02]" : ""}`}>
                      <div className="h-8 w-8 rounded-full bg-foreground/[0.04] ring-1 ring-foreground/[0.06] flex items-center justify-center text-sm font-medium shrink-0">
                        {video.downloaded ? <CheckCircle2 className="h-4 w-4 text-success" /> : i + 1}
                      </div>
                      <div className="flex-1 min-w-0">
                        <h4 className="text-sm font-medium truncate">{video.desc}</h4>
                        <p className="text-xs text-muted-foreground tracking-wide">
                          {formatDuration(video.duration)} · {formatCount(video.digg_count)} 赞 · {video.comment_count} 评论
                        </p>
                      </div>
                      <Button
                        variant={video.downloaded ? "capsule" : "default"}
                        size="sm"
                        disabled={video.downloaded}
                      >
                        {video.downloaded ? (
                          <><CheckCircle2 className="h-3.5 w-3.5 mr-1" />已下载</>
                        ) : (
                          <><Download className="h-3.5 w-3.5 mr-1" />下载</>
                        )}
                      </Button>
                    </div>
                  </Bezel>
                </AnimateEntry>
              ))}
            </div>
          </>
        )}
      </div>
    </>
  );
}
