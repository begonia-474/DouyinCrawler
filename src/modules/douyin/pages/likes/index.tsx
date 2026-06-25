import { useState, useCallback, useEffect, useRef } from "react";
import { Header } from "@/components/layout/header";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { UrlInput } from "@/components/shared/url-input";
import { VideoCard } from "@/components/shared/video-card";
import { Button } from "@/components/ui/button";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Bezel } from "@/components/shared/bezel";
import { getUserProfile, getUserLikes, downloadUserLikes } from "@/lib/api";
import type { UserProfile as UserProfileType, VideoItem } from "@/lib/api-types";
import { Loader2, AlertCircle, Download } from "lucide-react";
import { Progress } from "@/components/ui/progress";

function formatDuration(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
}

export default function LikesPage() {
  const [loading, setLoading] = useState(false);
  const [profile, setProfile] = useState<UserProfileType | null>(null);
  const [likes, setLikes] = useState<VideoItem[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [downloadedCount, setDownloadedCount] = useState(0);
  const [downloadProgress, setDownloadProgress] = useState(0);

  // 分页状态
  const [currentUrl, setCurrentUrl] = useState("");
  const [cursor, setCursor] = useState(0);
  const [hasMore, setHasMore] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);
  const sentinelRef = useRef<HTMLDivElement>(null);

  const handleParse = useCallback(async (url: string) => {
    setLoading(true);
    setProfile(null);
    setLikes([]);
    setError(null);
    setCurrentUrl(url);
    setHasMore(false);

    const profileRes = await getUserProfile(url);
    if (profileRes.success && profileRes.data?.profile) {
      setProfile(profileRes.data.profile as unknown as UserProfileType);
    } else {
      setError(profileRes.error || "获取用户信息失败");
      setLoading(false);
      return;
    }

    const likesRes = await getUserLikes(url, 0, 20);
    if (likesRes.success && likesRes.data?.videos) {
      setLikes(likesRes.data.videos);
      setCursor(likesRes.data.next_cursor ?? 0);
      setHasMore(likesRes.data.has_more ?? false);
    }

    setLoading(false);
  }, []);

  // 加载更多
  const loadMore = useCallback(async () => {
    if (!hasMore || loadingMore || !currentUrl) return;
    setLoadingMore(true);
    const res = await getUserLikes(currentUrl, cursor, 20);
    if (res.success && res.data?.videos) {
      setLikes(prev => [...prev, ...res.data!.videos!]);
      setCursor(res.data.next_cursor ?? 0);
      setHasMore(res.data.has_more ?? false);
    }
    setLoadingMore(false);
  }, [currentUrl, cursor, hasMore, loadingMore]);

  // IntersectionObserver 自动加载
  useEffect(() => {
    const el = sentinelRef.current;
    if (!el) return;

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && hasMore && !loadingMore) {
          loadMore();
        }
      },
      { threshold: 0.1 }
    );

    observer.observe(el);
    return () => observer.disconnect();
  }, [hasMore, loadingMore, loadMore]);

  const handleDownloadAll = async () => {
    setDownloading(true);
    setDownloadProgress(0);
    setDownloadedCount(0);

    const res = await downloadUserLikes(profile?.sec_user_id ? `https://www.douyin.com/user/${profile.sec_user_id}` : "");
    if (res.success) {
      setDownloadedCount(likes.length);
      setDownloadProgress(100);
    } else {
      setError(res.error || "下载失败");
    }
    setDownloading(false);
  };

  return (
    <>
      <AnimateEntry>
        <Header title="用户点赞" description="查看用户的点赞列表" parent={{ label: "首页", path: "/douyin" }}>
          {likes.length > 0 && (
            <Button variant="capsule" size="sm" onClick={handleDownloadAll} disabled={downloading}>
              {downloading ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <Download className="h-4 w-4 mr-1" />}
              {downloading ? `下载中 ${downloadedCount}/${likes.length}` : "全部下载"}
            </Button>
          )}
        </Header>
      </AnimateEntry>

      <div className="space-y-6">
        <UrlInput onSubmit={handleParse} loading={loading} placeholder="粘贴用户主页链接..." allowedTypes={["user"]} />

        {error && (
          <div className="flex items-center gap-2 p-4 rounded-2xl bg-destructive/[0.06] ring-1 ring-destructive/20 text-destructive text-sm">
            <AlertCircle className="h-4 w-4 shrink-0" />
            <span>{error}</span>
          </div>
        )}

        {loading && (
          <div className="flex items-center justify-center py-16">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        )}

        {downloading && (
          <Bezel radius="xl">
            <div className="p-5">
              <div className="space-y-1">
                <Progress value={downloadProgress} />
                <p className="text-xs text-muted-foreground tracking-wide text-right">{downloadedCount} / {likes.length}</p>
              </div>
            </div>
          </Bezel>
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
                  duration={formatDuration(video.duration)}
                  diggCount={video.digg_count}
                  commentCount={video.comment_count}
                  shareCount={video.share_count}
                />
              ))}
            </div>
            {/* 无限滚动 sentinel */}
            <div ref={sentinelRef} className="h-4" />
            {loadingMore && (
              <div className="flex justify-center py-4">
                <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
              </div>
            )}
            {!hasMore && likes.length > 0 && (
              <p className="text-center text-xs text-muted-foreground py-4">已加载全部 {likes.length} 个点赞</p>
            )}
          </>
        )}
      </div>
    </>
  );
}
