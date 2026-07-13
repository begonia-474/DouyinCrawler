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
  Check,
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
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
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

  const handleSelectChange = useCallback((awemeId: string, selected: boolean) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (selected) next.add(awemeId);
      else next.delete(awemeId);
      return next;
    });
  }, []);

  const handleSelectAll = useCallback(() => {
    if (selectedIds.size === videos.length) {
      setSelectedIds(new Set());
    } else {
      setSelectedIds(new Set(videos.map((v) => v.aweme_id)));
    }
  }, [selectedIds.size, videos]);

  const handleDownloadSelected = async () => {
    if (selectedIds.size === 0) return;
    setDownloading(true);
    setDownloadedCount(0);
    setActiveTaskId(null);

    const res = await downloadMix(currentUrl, Array.from(selectedIds));
    if (res.success && res.task_id) {
      setActiveTaskId(res.task_id);
      setSelectedIds(new Set());
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
            <div className="flex items-center gap-2">
              {selectedIds.size > 0 && (
                <Button variant="capsule" size="sm" onClick={handleDownloadSelected} disabled={downloading}>
                  下载选中 ({selectedIds.size})
                </Button>
              )}
              <DownloadAllButton
                downloading={downloading}
                downloadedCount={downloadedCount}
                total={videos.length}
                onClick={handleDownloadAll}
              />
            </div>
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

            <div className="flex items-center gap-2 mb-4">
              <button
                type="button"
                onClick={handleSelectAll}
                className="text-xs text-muted-foreground hover:text-foreground transition-colors"
              >
                {selectedIds.size === videos.length ? "取消全选" : "全选"}
              </button>
              {selectedIds.size > 0 && (
                <span className="text-xs text-muted-foreground">已选 {selectedIds.size} 个</span>
              )}
            </div>

            <div className="space-y-2">
              {videos.map((video, i) => (
                <AnimateEntry key={video.aweme_id} delay={Math.min(i * 25, 500)}>
                  <Bezel radius="lg" padding="sm">
                    <div
                      className={`flex items-center gap-4 p-4 bg-card transition-all duration-300 cursor-pointer hover:bg-foreground/[0.02] ${video.downloaded ? "bg-success/[0.02]" : ""}`}
                      onClick={() => navigate(`/douyin/video/${video.aweme_id}`, { state: { from: "合集", fromPath: "/douyin/mix" } })}
                    >
                      <button
                        type="button"
                        onClick={(e) => { e.stopPropagation(); handleSelectChange(video.aweme_id, !selectedIds.has(video.aweme_id)); }}
                        className={`h-5 w-5 rounded-full border-2 flex items-center justify-center shrink-0 transition-all ${
                          selectedIds.has(video.aweme_id)
                            ? "bg-primary border-primary text-primary-foreground"
                            : "border-foreground/20 hover:border-foreground/40"
                        }`}
                      >
                        {selectedIds.has(video.aweme_id) && <Check className="h-3 w-3" />}
                      </button>
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
                        onClick={(e) => {
                          e.stopPropagation();
                          downloadMix(currentUrl, [video.aweme_id]);
                        }}
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
