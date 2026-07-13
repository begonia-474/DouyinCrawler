import { useState, useCallback, useEffect, useRef } from "react";
import { useNavigate } from "react-router-dom";
import { Header } from "@/components/layout/header";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { UrlInput } from "@/components/shared/url-input";
import { VideoCard } from "@/components/shared/video-card";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Bezel } from "@/components/shared/bezel";
import { DownloadAllButton } from "@/components/shared/download-all-button";
import { Button } from "@/components/ui/button";
import { DownloadProgressOverlay } from "@/components/shared/download-progress-overlay";
import { InfiniteScrollSentinel } from "@/components/shared/infinite-scroll-sentinel";
import { LoadingSpinner } from "@/components/shared/loading-spinner";
import { ErrorBanner } from "@/components/shared/error-banner";
import { getUserProfile, getUserLikes, downloadUserLikes } from "@/lib/api";
import { useActiveTask } from "@/hooks/use-active-task";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import { usePersistedUrl } from "@/hooks/use-persisted-url";
import type { UserProfile as UserProfileType, VideoItem } from "@/lib/api-types";
import { formatDurationSec } from "@/lib/utils";

export default function LikesPage() {
  const navigate = useNavigate();
  const [loading, setLoading] = useState(false);
  const [profile, setProfile] = useState<UserProfileType | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [downloadedCount, setDownloadedCount] = useState(0);
  const [activeTaskId, setActiveTaskId] = useState<string | null>(null);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [currentUrl, setCurrentUrl] = usePersistedUrl("likes");
  const batchTask = useActiveTask(activeTaskId);
  const downloadProgress = batchTask ? ((batchTask.total ?? 0) > 0 ? Math.round(((batchTask.completed ?? 0) / (batchTask.total ?? 1)) * 100) : 0) : 0;

  const { items: likes, setItems: setLikes, hasMore, loadingMore, sentinelRef, reset } = useInfiniteScroll<VideoItem>({
    fetchPage: useCallback(async (cursor: number) => {
      if (!currentUrl) return null;
      const res = await getUserLikes(currentUrl, cursor, 20);
      if (res.success && res.data?.videos) {
        return {
          items: res.data.videos,
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
    setProfile(null);
    setLikes([]);
    setError(null);
    setCurrentUrl(url);

    const profileRes = await getUserProfile(url);
    if (profileRes.success && profileRes.data?.profile) {
      setProfile(profileRes.data.profile as unknown as UserProfileType);
    } else {
      setError(profileRes.error || "获取用户信息失败");
      setLoading(false);
      return;
    }

    // 首次加载点赞列表
    const likesRes = await getUserLikes(url, 0, 20);
    if (likesRes.success && likesRes.data?.videos) {
      reset(async () => ({
        items: likesRes.data!.videos!,
        nextCursor: likesRes.data!.next_cursor ?? 0,
        hasMore: likesRes.data!.has_more ?? false,
      }));
    }

    setLoading(false);
  }, [reset, setLikes, setCurrentUrl]);

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

  const handleSelectChange = useCallback((awemeId: string, selected: boolean) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (selected) next.add(awemeId);
      else next.delete(awemeId);
      return next;
    });
  }, []);

  const handleSelectAll = useCallback(() => {
    if (selectedIds.size === likes.length) {
      setSelectedIds(new Set());
    } else {
      setSelectedIds(new Set(likes.map((v) => v.aweme_id)));
    }
  }, [selectedIds.size, likes]);

  const handleDownloadSelected = async () => {
    if (selectedIds.size === 0) return;
    setDownloading(true);
    setDownloadedCount(0);
    setActiveTaskId(null);

    const url = profile?.sec_user_id ? `https://www.douyin.com/user/${profile.sec_user_id}` : "";
    const res = await downloadUserLikes(url, Array.from(selectedIds));
    if (res.success && res.task_id) {
      setActiveTaskId(res.task_id);
      setSelectedIds(new Set());
    } else {
      setError(res.error || "下载失败");
    }
    setDownloading(false);
  };

  const handleCardDownload = useCallback((video: VideoItem) => {
    const url = profile?.sec_user_id ? `https://www.douyin.com/user/${profile.sec_user_id}` : "";
    downloadUserLikes(url, [video.aweme_id]);
  }, [profile]);

  return (
    <>
      <AnimateEntry>
        <Header title="用户点赞" description="查看用户的点赞列表" parent={{ label: "首页", path: "/douyin" }}>
          {likes.length > 0 && (
            <div className="flex items-center gap-2">
              {selectedIds.size > 0 && (
                <Button variant="capsule" size="sm" onClick={handleDownloadSelected} disabled={downloading}>
                  下载选中 ({selectedIds.size})
                </Button>
              )}
              <DownloadAllButton
                downloading={downloading}
                downloadedCount={downloadedCount}
                total={likes.length}
                onClick={handleDownloadAll}
                variant="capsule"
                size="sm"
              />
            </div>
          )}
        </Header>
      </AnimateEntry>

      <div className="space-y-6">
        <UrlInput onSubmit={handleParse} loading={loading} placeholder="粘贴用户主页链接..." allowedTypes={["user"]} autoDetect />

        <ErrorBanner message={error} />

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

            {likes.length > 0 && (
              <div className="flex items-center gap-2 mb-4">
                <button
                  type="button"
                  onClick={handleSelectAll}
                  className="text-xs text-muted-foreground hover:text-foreground transition-colors"
                >
                  {selectedIds.size === likes.length ? "取消全选" : "全选"}
                </button>
                {selectedIds.size > 0 && (
                  <span className="text-xs text-muted-foreground">已选 {selectedIds.size} 个</span>
                )}
              </div>
            )}
            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-5">
              {likes.map((video) => (
                <VideoCard
                  key={video.aweme_id}
                  title={video.desc}
                  author={profile.nickname}
                  cover={video.cover_url}
                  duration={formatDurationSec(video.duration)}
                  diggCount={video.digg_count}
                  commentCount={video.comment_count}
                  shareCount={video.share_count}
                  onClick={() => navigate(`/douyin/video/${video.aweme_id}`, { state: { from: "点赞", fromPath: "/douyin/likes" } })}
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
              total={likes.length}
              label="点赞"
            />
          </>
        )}
      </div>
    </>
  );
}
