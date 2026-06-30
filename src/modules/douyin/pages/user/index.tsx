import { useState, useCallback } from "react";
import { Header } from "@/components/layout/header";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { UrlInput } from "@/components/shared/url-input";
import { VideoCard } from "@/components/shared/video-card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Bezel } from "@/components/shared/bezel";
import { DownloadStatusCard } from "@/components/shared/download-status-card";
import { useUserProfileQuery, useUserPostsInfiniteQuery, useUserFollowingQuery, useUserFollowersQuery } from "@/lib/queries";
import { downloadUserPosts } from "@/lib/api";
import { useTaskStore } from "@/stores/task-store";
import type { UserProfile as UserProfileType, VideoItem, FollowItem } from "@/lib/api-types";
import {
  Users, Heart, Video, UserPlus,
  UserCheck, ThumbsUp,
} from "lucide-react";
import { DownloadAllButton } from "@/components/shared/download-all-button";
import { DownloadProgressOverlay } from "@/components/shared/download-progress-overlay";
import { InfiniteScrollSentinel } from "@/components/shared/infinite-scroll-sentinel";
import { LoadingSpinner } from "@/components/shared/loading-spinner";
import { ErrorBanner } from "@/components/shared/error-banner";
import { formatCount, formatDurationSec } from "@/lib/utils";

function FollowItemCard({ item, type }: { item: FollowItem; type: "following" | "follower" }) {
  return (
    <Bezel radius="lg" padding="sm">
      <div className="flex items-center gap-4 p-4 bg-card">
        <Avatar className="h-10 w-10">
          <AvatarImage src={item.avatar} />
          <AvatarFallback>{item.nickname?.[0] || "?"}</AvatarFallback>
        </Avatar>
        <div className="flex-1 min-w-0">
          <h4 className="text-sm font-medium">{item.nickname}</h4>
          {item.signature && <p className="text-xs text-muted-foreground tracking-wide truncate">{item.signature}</p>}
          <p className="text-xs text-muted-foreground tracking-wide">{formatCount(item.follower_count)} 粉丝</p>
        </div>
        <Button variant="capsule" size="sm">
          {type === "following" ? <><UserCheck className="h-3.5 w-3.5 mr-1" />已关注</> : <><UserPlus className="h-3.5 w-3.5 mr-1" />关注</>}
        </Button>
      </div>
    </Bezel>
  );
}

export default function UserPage() {
  const [currentUrl, setCurrentUrl] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [downloadedCount, setDownloadedCount] = useState(0);
  const [activeTaskId, setActiveTaskId] = useState<string | null>(null);
  const batchTask = useTaskStore((s) => activeTaskId ? s.tasks[activeTaskId] : null);
  const downloadProgress = batchTask ? ((batchTask.total ?? 0) > 0 ? Math.round(((batchTask.completed ?? 0) / (batchTask.total ?? 1)) * 100) : 0) : 0;

  const profileQuery = useUserProfileQuery(currentUrl || null);
  const postsQuery = useUserPostsInfiniteQuery(currentUrl || null);
  const followingQuery = useUserFollowingQuery(currentUrl || null);
  const followersQuery = useUserFollowersQuery(currentUrl || null);

  const profile = (profileQuery.data?.data?.profile as unknown as UserProfileType) || null;
  const videos: VideoItem[] = postsQuery.data?.pages.flatMap((p) => p.data?.videos ?? []) ?? [];
  const hasMore = postsQuery.hasNextPage;
  const loadingMore = postsQuery.isFetchingNextPage;
  const following: FollowItem[] = (followingQuery.data?.data?.followings ?? []) as unknown as FollowItem[];
  const followers: FollowItem[] = (followersQuery.data?.data?.followers ?? []) as unknown as FollowItem[];
  const loading = profileQuery.isLoading;

  const fetchNextPage = useCallback(() => {
    if (postsQuery.hasNextPage && !postsQuery.isFetchingNextPage) {
      postsQuery.fetchNextPage();
    }
  }, [postsQuery]);

  const handleParse = useCallback((url: string) => {
    setError(null);
    setCurrentUrl(url);
  }, []);

  const handleDownloadAll = async () => {
    setDownloading(true);
    setDownloadedCount(0);
    setActiveTaskId(null);

    const res = await downloadUserPosts(profile?.sec_user_id ? `https://www.douyin.com/user/${profile.sec_user_id}` : "");
    if (res.success && res.task_id) {
      setActiveTaskId(res.task_id);
    } else if (res.success) {
      setDownloadedCount(videos.length);
    } else {
      setError(res.error || "下载失败");
    }
    setDownloading(false);
  };

  const queryError = profileQuery.error?.message || postsQuery.error?.message
    || (!profileQuery.data?.success ? (profileQuery.data?.error ?? null) : null)
    || error;

  return (
    <>
      <AnimateEntry>
        <Header title="用户主页" description="查看用户资料和作品" parent={{ label: "首页", path: "/douyin" }}>
          {profile && (
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
        <UrlInput onSubmit={handleParse} loading={loading} placeholder="粘贴用户主页链接..." allowedTypes={["user"]} autoDetect />

        <ErrorBanner message={queryError} />

        {loading && <LoadingSpinner size={24} className="py-16" />}

        {downloading && (
          <DownloadProgressOverlay
            progress={downloadProgress}
            current={downloadedCount}
            total={videos.length}
          />
        )}

        {profile && !loading && (
          <>
            <AnimateEntry>
              <Bezel radius="xl">
                <div className="p-7">
                  <div className="flex items-start gap-5">
                    <Avatar className="h-16 w-16">
                      <AvatarImage src={profile.avatar} />
                      <AvatarFallback>{profile.nickname?.[0] || "?"}</AvatarFallback>
                    </Avatar>
                    <div className="flex-1">
                      <h3 className="text-lg font-semibold">{profile.nickname}</h3>
                      <p className="text-sm text-muted-foreground tracking-wide mt-1">{profile.signature}</p>
                      <div className="flex items-center gap-6 mt-4">
                        <div className="flex items-center gap-1.5 text-sm">
                          <Video className="h-4 w-4 text-muted-foreground" />
                          <span className="font-heading font-semibold tabular-nums">{profile.aweme_count}</span>
                          <span className="text-muted-foreground">作品</span>
                        </div>
                        <div className="flex items-center gap-1.5 text-sm">
                          <Users className="h-4 w-4 text-muted-foreground" />
                          <span className="font-heading font-semibold tabular-nums">{formatCount(profile.follower_count)}</span>
                          <span className="text-muted-foreground">粉丝</span>
                        </div>
                        <div className="flex items-center gap-1.5 text-sm">
                          <Heart className="h-4 w-4 text-muted-foreground" />
                          <span className="font-heading font-semibold tabular-nums">{formatCount(profile.following_count)}</span>
                          <span className="text-muted-foreground">关注</span>
                        </div>
                        <div className="flex items-center gap-1.5 text-sm">
                          <ThumbsUp className="h-4 w-4 text-muted-foreground" />
                          <span className="font-heading font-semibold tabular-nums">{formatCount(profile.total_favorited)}</span>
                          <span className="text-muted-foreground">获赞</span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </Bezel>
            </AnimateEntry>

            <Tabs defaultValue="posts">
              <TabsList>
                <TabsTrigger value="posts">作品<Badge variant="secondary" className="ml-1.5">{videos.length}{hasMore ? "+" : ""}</Badge></TabsTrigger>
                <TabsTrigger value="following">关注<Badge variant="secondary" className="ml-1.5">{following.length}</Badge></TabsTrigger>
                <TabsTrigger value="followers">粉丝<Badge variant="secondary" className="ml-1.5">{followers.length}</Badge></TabsTrigger>
              </TabsList>

              <TabsContent value="posts" className="mt-6">
                <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-5">
                  {videos.map((video) => (
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
                  total={videos.length}
                  label="作品"
                  onVisible={fetchNextPage}
                />
              </TabsContent>

              <TabsContent value="following" className="mt-6 space-y-2">
                {following.map((item) => (
                  <FollowItemCard key={item.uid} item={item} type="following" />
                ))}
              </TabsContent>

              <TabsContent value="followers" className="mt-6 space-y-2">
                {followers.map((item) => (
                  <FollowItemCard key={item.uid} item={item} type="follower" />
                ))}
              </TabsContent>
            </Tabs>
          </>
        )}
      </div>

      <DownloadStatusCard />
    </>
  );
}
