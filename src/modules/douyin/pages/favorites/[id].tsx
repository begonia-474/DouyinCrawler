import { useState, useCallback, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { Header } from "@/components/layout/header";
import { VideoCard } from "@/components/shared/video-card";
import { Button } from "@/components/ui/button";
import { Bezel } from "@/components/shared/bezel";
import { DownloadStatusCard } from "@/components/shared/download-status-card";
import { DownloadAllButton } from "@/components/shared/download-all-button";
import { DownloadProgressOverlay } from "@/components/shared/download-progress-overlay";
import { InfiniteScrollSentinel } from "@/components/shared/infinite-scroll-sentinel";
import { LoadingSpinner } from "@/components/shared/loading-spinner";
import { getCollectsVideoList, downloadCollectsVideo } from "@/lib/api";
import { useActiveTask } from "@/hooks/use-active-task";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import type { VideoItem } from "@/lib/api-types";
import { ArrowLeft } from "lucide-react";
import { ErrorBanner } from "@/components/shared/error-banner";
import { formatDurationSec } from "@/lib/utils";

export default function CollectsDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [error, setError] = useState<string | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [downloadedCount, setDownloadedCount] = useState(0);
  const [activeTaskId, setActiveTaskId] = useState<string | null>(null);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const batchTask = useActiveTask(activeTaskId);
  const downloadProgress = batchTask ? ((batchTask.total ?? 0) > 0 ? Math.round(((batchTask.completed ?? 0) / (batchTask.total ?? 1)) * 100) : 0) : 0;

  const { items: videos, hasMore, loadingMore, initialLoading, sentinelRef, reset } = useInfiniteScroll<VideoItem>({
    fetchPage: useCallback(async (cursor: number) => {
      if (!id) return null;
      const res = await getCollectsVideoList(id, cursor, 20);
      if (res.success && res.data?.videos) {
        return {
          items: res.data.videos,
          nextCursor: res.data.next_cursor ?? 0,
          hasMore: res.data.has_more ?? false,
        };
      }
      return null;
    }, [id]),
    enabled: !!id,
  });

  useEffect(() => {
    if (!id) return;
    setError(null);
    // reset() 触发 useInfiniteScroll 内部重新加载，loading 由其内部状态管理
    reset();
  }, [id, reset]);

  const handleDownloadAll = async () => {
    if (!id) return;
    setDownloading(true);
    setDownloadedCount(0);
    setActiveTaskId(null);

    const res = await downloadCollectsVideo(id);
    if (res.success && res.task_id) {
      setActiveTaskId(res.task_id);
    } else if (res.success) {
      setDownloadedCount(videos.length);
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
    if (selectedIds.size === 0 || !id) return;
    setDownloading(true);
    setDownloadedCount(0);
    setActiveTaskId(null);

    const res = await downloadCollectsVideo(id, Array.from(selectedIds));
    if (res.success && res.task_id) {
      setActiveTaskId(res.task_id);
      setSelectedIds(new Set());
    } else {
      setError(res.error || "下载失败");
    }
    setDownloading(false);
  };

  const handleCardDownload = useCallback((video: VideoItem) => {
    downloadCollectsVideo(id!, [video.aweme_id]);
  }, [id]);

  return (
    <>
      <Header title="收藏夹详情" description={`共 ${videos.length}${hasMore ? "+" : ""} 个视频`} parent={{ label: "我的收藏", path: "/douyin/favorites" }}>
        <div className="flex gap-2">
          <Button variant="capsule" size="sm" onClick={() => navigate("/douyin/favorites")}>
            <ArrowLeft className="h-4 w-4 mr-1" />
            返回
          </Button>
          {videos.length > 0 && (
            <>
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
                size="sm"
              />
            </>
          )}
        </div>
      </Header>

      <div className="space-y-6">
        <ErrorBanner message={error} />

        {downloading && (
          <DownloadProgressOverlay
            progress={downloadProgress}
            current={downloadedCount}
            total={videos.length}
          />
        )}

        {initialLoading && <LoadingSpinner />}

        {!initialLoading && videos.length === 0 && !error && (
          <Bezel radius="xl">
            <div className="p-12 text-center">
              <p className="text-muted-foreground">暂无视频</p>
            </div>
          </Bezel>
        )}

        {!initialLoading && videos.length > 0 && (
          <>
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
            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-5">
              {videos.map((video) => (
                <VideoCard
                  key={video.aweme_id}
                  title={video.desc}
                  author={video.author}
                  cover={video.cover_url}
                  duration={formatDurationSec(video.duration)}
                  diggCount={video.digg_count}
                  commentCount={video.comment_count}
                  shareCount={video.share_count}
                  onClick={() => navigate(`/douyin/video/${video.aweme_id}`, { state: { from: "收藏夹", fromPath: "/douyin/favorites" } })}
                  selectable
                  selected={selectedIds.has(video.aweme_id)}
                  onSelectChange={(sel) => handleSelectChange(video.aweme_id, sel)}
                  onDownload={() => handleCardDownload(video)}
                />
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
