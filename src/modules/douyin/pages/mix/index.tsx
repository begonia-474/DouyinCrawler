import { useState, useCallback, useEffect, useRef } from "react";
import { useNavigate } from "react-router-dom";
import { Header } from "@/components/layout/header";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { UrlInput } from "@/components/shared/url-input";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Bezel } from "@/components/shared/bezel";
import { DownloadStatusCard } from "@/components/shared/download-status-card";
import { DownloadAllButton } from "@/components/shared/download-all-button";
import { DownloadProgressOverlay } from "@/components/shared/download-progress-overlay";
import { InfiniteScrollSentinel } from "@/components/shared/infinite-scroll-sentinel";
import { getMixInfo, downloadMix } from "@/lib/api";
import { useActiveTask } from "@/hooks/use-active-task";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import { usePersistedUrl } from "@/hooks/use-persisted-url";
import type { VideoItem } from "@/lib/api-types";
import {
  Download,
  Layers,
  CheckCircle2,
  ListVideo,
} from "lucide-react";
import { ErrorBanner } from "@/components/shared/error-banner";
import { formatCount, formatDurationSec } from "@/lib/utils";

type MixVideo = VideoItem & { downloaded?: boolean };

export default function MixPage() {
  const navigate = useNavigate();
  const [loading, setLoading] = useState(false);
  const [downloading, setDownloading] = useState(false);
  const [downloadedCount, setDownloadedCount] = useState(0);
  const [activeTaskId, setActiveTaskId] = useState<string | null>(null);
  const batchTask = useActiveTask(activeTaskId);
  const downloadProgress = batchTask ? ((batchTask.total ?? 0) > 0 ? Math.round(((batchTask.completed ?? 0) / (batchTask.total ?? 1)) * 100) : 0) : 0;
  const [mixName, setMixName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [currentUrl, setCurrentUrl] = usePersistedUrl("mix");

  const { items: videos, setItems: setVideos, hasMore, loadingMore, sentinelRef, reset } = useInfiniteScroll<MixVideo>({
    fetchPage: useCallback(async (cursor: number) => {
      if (!currentUrl) return null;
      const res = await getMixInfo(currentUrl, cursor, 20);
      if (res.success && res.data?.videos) {
        return {
          items: res.data.videos.map((v) => ({ ...v, downloaded: false })),
          nextCursor: res.data.next_cursor ?? 0,
          hasMore: res.data.has_more ?? false,
        };
      }
      return null;
    }, [currentUrl]),
    enabled: !!currentUrl,
  });

  const handleParse = useCallback(async (url: string) => {
    setLoading(true);
    setMixName("");
    setError(null);
    setCurrentUrl(url);

    const res = await getMixInfo(url, 0, 20);
    if (res.success && res.data?.videos) {
      setMixName(res.data.detail?.desc || "合集");
      reset(async () => ({
        items: res.data!.videos!.map((v) => ({ ...v, downloaded: false })),
        nextCursor: res.data!.next_cursor ?? 0,
        hasMore: res.data!.has_more ?? false,
      }));
    } else {
      setError(res.error || "获取合集失败");
    }
    setLoading(false);
  }, [reset, setCurrentUrl]);

  // 挂载时自动恢复上次解析
  const initRef = useRef(true);
  useEffect(() => {
    if (initRef.current && currentUrl) {
      initRef.current = false;
      handleParse(currentUrl);
    }
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  const handleDownloadAll = async () => {
    setDownloading(true);
    setDownloadedCount(0);
    setActiveTaskId(null);

    const res = await downloadMix(currentUrl);
    if (res.success && res.task_id) {
      setActiveTaskId(res.task_id);
    } else if (res.success) {
      setDownloadedCount(videos.length);
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
            <DownloadAllButton
              downloading={downloading}
              downloadedCount={downloadedCount}
              total={videos.length}
              onClick={handleDownloadAll}
            />
          )}
        </Header>
      </AnimateEntry>

      <div className="space-y-6">
        <UrlInput onSubmit={handleParse} loading={loading} placeholder="粘贴合集链接..." allowedTypes={["mix"]} autoDetect />

        <ErrorBanner message={error} />

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
                        <p className="text-sm text-muted-foreground tracking-wide">{videos.length}{hasMore ? "+" : ""} 个视频</p>
                      </div>
                    </div>
                    <Badge variant="secondary" className="rounded-full"><ListVideo className="h-3 w-3 mr-1" />合集</Badge>
                  </div>
                  {downloading && (
                    <DownloadProgressOverlay
                      progress={downloadProgress}
                      current={downloadedCount}
                      total={videos.length}
                      className="mt-5"
                    />
                  )}
                </div>
              </Bezel>
            </AnimateEntry>

            <div className="space-y-2">
              {videos.map((video, i) => (
                <AnimateEntry key={video.aweme_id} delay={Math.min(i * 25, 500)}>
                  <Bezel radius="lg" padding="sm">
                    <div
                      className={`flex items-center gap-4 p-4 bg-card transition-all duration-300 cursor-pointer hover:bg-foreground/[0.02] ${video.downloaded ? "bg-success/[0.02]" : ""}`}
                      onClick={() => navigate(`/douyin/video/${video.aweme_id}`, { state: { from: "合集", fromPath: "/douyin/mix" } })}
                    >
                      <div className="h-8 w-8 rounded-full bg-foreground/[0.04] ring-1 ring-foreground/[0.06] flex items-center justify-center text-sm font-medium shrink-0">
                        {video.downloaded ? <CheckCircle2 className="h-4 w-4 text-success" /> : i + 1}
                      </div>
                      <div className="flex-1 min-w-0">
                        <h4 className="text-sm font-medium truncate">{video.desc}</h4>
                        <p className="text-xs text-muted-foreground tracking-wide">
                          {formatDurationSec(video.duration)} · {formatCount(video.digg_count)} 赞 · {video.comment_count} 评论
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
            <InfiniteScrollSentinel
              sentinelRef={sentinelRef}
              loadingMore={loadingMore}
              hasMore={hasMore}
              total={videos.length}
              label="视频"
            />
          </>
        )}
      </div>

      <DownloadStatusCard />
    </>
  );
}
