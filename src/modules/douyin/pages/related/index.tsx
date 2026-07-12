import { useState, useCallback, useEffect, useRef } from "react";
import { useSearchParams, useNavigate } from "react-router-dom";
import { Header } from "@/components/layout/header";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { VideoCard } from "@/components/shared/video-card";
import { InfiniteScrollSentinel } from "@/components/shared/infinite-scroll-sentinel";
import { LoadingSpinner } from "@/components/shared/loading-spinner";
import { ErrorBanner } from "@/components/shared/error-banner";
import { CommentDialog } from "@/components/shared/comment-dialog";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import { getRelated } from "@/lib/api/related";
import { formatDurationSec } from "@/lib/utils";
import type { VideoItem } from "@/lib/api-types";
import { Compass } from "lucide-react";

export default function RelatedPage() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const url = searchParams.get("url") ?? "";

  const [error, setError] = useState<string | null>(null);
  const seenIdsRef = useRef<Set<string>>(new Set());
  const [commentAwemeId, setCommentAwemeId] = useState<string | null>(null);

  const { items: videos, hasMore, loadingMore, initialLoading, sentinelRef, reset } = useInfiniteScroll<VideoItem>({
    fetchPage: useCallback(async () => {
      if (!url) return null;
      const filterGids = seenIdsRef.current.size > 0 ? [...seenIdsRef.current].join(",") + "," : "";
      const res = await getRelated(url, 20, filterGids);
      if (res.success && res.data) {
        const newIds = (res.data.videos as VideoItem[])
          .map((v) => v.aweme_id)
          .filter(Boolean);
        newIds.forEach((id) => seenIdsRef.current.add(id));
        return {
          items: res.data.videos as VideoItem[],
          nextCursor: 0,
          hasMore: res.data.has_more ?? false,
        };
      }
      setError(res.error || "获取相关推荐失败");
      return null;
    }, [url]),
    enabled: true,
  });

  // URL 变化时重置并加载第一页
  useEffect(() => {
    if (!url) return;
    setError(null);
    seenIdsRef.current = new Set();
    reset();
  }, [url, reset]);

  const handleVideoClick = useCallback(
    (video: VideoItem) => {
      const videoUrl = `https://www.douyin.com/video/${video.aweme_id}`;
      navigate(`/douyin/related?url=${encodeURIComponent(videoUrl)}&aweme_id=${video.aweme_id}`);
    },
    [navigate]
  );

  if (!url) {
    return (
      <>
        <AnimateEntry>
          <Header title="相关推荐" description="基于视频内容推荐相似作品" parent={{ label: "单视频下载", path: "/douyin/video" }} />
        </AnimateEntry>
        <div className="py-16 text-center text-muted-foreground text-sm">
          请先在「单视频下载」页面解析一个视频链接
        </div>
      </>
    );
  }

  return (
    <>
      <AnimateEntry>
        <Header title="相关推荐" description="基于视频内容推荐相似作品" parent={{ label: "单视频下载", path: "/douyin/video" }} />
      </AnimateEntry>

      <div className="space-y-6">
        {error && <ErrorBanner message={error} />}

        {initialLoading && <LoadingSpinner text="正在加载相关推荐…" />}

        {!initialLoading && videos.length === 0 && !error && (
          <div className="py-16 text-center">
            <Compass className="h-10 w-10 text-muted-foreground/30 mx-auto mb-3" />
            <p className="text-sm text-muted-foreground">暂无相关推荐</p>
          </div>
        )}

        {videos.length > 0 && (
          <AnimateEntry>
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
                  onClick={() => handleVideoClick(video)}
                />
              ))}
            </div>
            <InfiniteScrollSentinel
              sentinelRef={sentinelRef}
              loadingMore={loadingMore}
              hasMore={hasMore}
              total={videos.length}
              label="相关推荐"
            />
          </AnimateEntry>
        )}
      </div>

      <CommentDialog
        awemeId={commentAwemeId ?? ""}
        open={!!commentAwemeId}
        onOpenChange={(open) => !open && setCommentAwemeId(null)}
      />
    </>
  );
}
