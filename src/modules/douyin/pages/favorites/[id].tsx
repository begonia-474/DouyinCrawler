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
import { useTaskStore } from "@/stores/task-store";
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
  const batchTask = useTaskStore((s) => activeTaskId ? s.tasks[activeTaskId] : null);
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

  return (
    <>
      <Header title="收藏夹详情" description={`共 ${videos.length}${hasMore ? "+" : ""} 个视频`} parent={{ label: "我的收藏", path: "/douyin/favorites" }}>
        <div className="flex gap-2">
          <Button variant="capsule" size="sm" onClick={() => navigate("/douyin/favorites")}>
            <ArrowLeft className="h-4 w-4 mr-1" />
            返回
          </Button>
          {videos.length > 0 && (
            <DownloadAllButton
              downloading={downloading}
              downloadedCount={downloadedCount}
              total={videos.length}
              onClick={handleDownloadAll}
              size="sm"
            />
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
            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-5">
              {videos.map((video) => (
                <VideoCard
                  key={video.aweme_id}
                  title={video.desc}
                  author={video.author}
                  duration={formatDurationSec(video.duration)}
                  diggCount={video.digg_count}
                  commentCount={video.comment_count}
                  shareCount={video.share_count}
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
