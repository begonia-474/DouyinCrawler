import { useState, useCallback } from "react";
import { Header } from "@/components/layout/header";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { UrlInput } from "@/components/shared/url-input";
import { VideoCard } from "@/components/shared/video-card";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Bezel } from "@/components/shared/bezel";
import { DownloadAllButton } from "@/components/shared/download-all-button";
import { DownloadProgressOverlay } from "@/components/shared/download-progress-overlay";
import { InfiniteScrollSentinel } from "@/components/shared/infinite-scroll-sentinel";
import { LoadingSpinner } from "@/components/shared/loading-spinner";
import { ErrorBanner } from "@/components/shared/error-banner";
import { useUserProfileQuery, useUserLikesInfiniteQuery } from "@/lib/queries";
import { downloadUserLikes } from "@/lib/api";
import { useTaskStore } from "@/stores/task-store";
import type { UserProfile as UserProfileType, VideoItem } from "@/lib/api-types";
import { formatDurationSec } from "@/lib/utils";

export default function LikesPage() {
  const [currentUrl, setCurrentUrl] = useState("");
  const [downloading, setDownloading] = useState(false);
  const [downloadedCount, setDownloadedCount] = useState(0);
  const [activeTaskId, setActiveTaskId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const batchTask = useTaskStore((s) => activeTaskId ? s.tasks[activeTaskId] : null);
  const downloadProgress = batchTask ? ((batchTask.total ?? 0) > 0 ? Math.round(((batchTask.completed ?? 0) / (batchTask.total ?? 1)) * 100) : 0) : 0;

  const profileQuery = useUserProfileQuery(currentUrl || null);
  const likesQuery = useUserLikesInfiniteQuery(currentUrl || null);

  const profile = (profileQuery.data?.data?.profile as unknown as UserProfileType) || null;
  const likes: VideoItem[] = likesQuery.data?.pages.flatMap((p) => p.data?.videos ?? []) ?? [];
  const hasMore = likesQuery.hasNextPage;
  const loadingMore = likesQuery.isFetchingNextPage;
  const loading = profileQuery.isLoading;

  const fetchNextPage = useCallback(() => {
    if (likesQuery.hasNextPage && !likesQuery.isFetchingNextPage) {
      likesQuery.fetchNextPage();
    }
  }, [likesQuery]);

  const handleParse = useCallback((url: string) => {
    setError(null);
    setCurrentUrl(url);
  }, []);

  const handleDownloadAll = async () => {
    setDownloading(true);
    setDownloadedCount(0);
    setActiveTaskId(null);
    setError(null);

    const res = await downloadUserLikes(profile?.sec_user_id ? `https://www.douyin.com/user/${profile.sec_user_id}` : "");
    if (res.success && res.task_id) {
      setActiveTaskId(res.task_id);
    } else if (res.success) {
      setDownloadedCount(likes.length);
    } else {
      setError(res.error || "下载失败");
    }
    setDownloading(false);
  };

  const queryError = profileQuery.error?.message || likesQuery.error?.message
    || (!profileQuery.data?.success ? (profileQuery.data?.error ?? null) : null)
    || error;

  return (
    <>
      <AnimateEntry>
        <Header title="用户点赞" description="查看用户的点赞列表" parent={{ label: "首页", path: "/douyin" }}>
          {likes.length > 0 && (
            <DownloadAllButton
              downloading={downloading}
              downloadedCount={downloadedCount}
              total={likes.length}
              onClick={handleDownloadAll}
              variant="capsule"
              size="sm"
            />
          )}
        </Header>
      </AnimateEntry>

      <div className="space-y-6">
        <UrlInput onSubmit={handleParse} loading={loading} placeholder="粘贴用户主页链接..." allowedTypes={["user"]} autoDetect />

        <ErrorBanner message={queryError} />

        {loading && <LoadingSpinner size={24} />}

        {downloading && (
          <DownloadProgressOverlay progress={downloadProgress} current={downloadedCount} total={likes.length} />
        )}

        {profile && !loading && (
          <>
            <AnimateEntry>
              <Bezel radius="xl" padding="sm">
                <div className="flex items-center gap-4 p-5 bg-card">
                  <Avatar className="h-12 w-12">
                    <AvatarImage src={profile.avatar} />
                    <AvatarFallback>{profile.nickname?.[0] || "?"}</AvatarFallback>
                  </Avatar>
                  <div>
                    <h3 className="font-semibold">{profile.nickname}</h3>
                    <p className="text-sm text-muted-foreground tracking-wide">{likes.length}{hasMore ? "+" : ""} 个点赞</p>
                  </div>
                </div>
              </Bezel>
            </AnimateEntry>

            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-5">
              {likes.map((video) => (
                <VideoCard
                  key={video.aweme_id}
                  title={video.desc}
                  author={profile.nickname}
                  duration={formatDurationSec(video.duration)}
                  diggCount={video.digg_count}
                  commentCount={video.comment_count}
                  shareCount={video.share_count}
                />
              ))}
            </div>
            <InfiniteScrollSentinel
              loadingMore={loadingMore}
              hasMore={!!hasMore}
              total={likes.length}
              label="点赞"
              onVisible={fetchNextPage}
            />
          </>
        )}
      </div>
    </>
  );
}
