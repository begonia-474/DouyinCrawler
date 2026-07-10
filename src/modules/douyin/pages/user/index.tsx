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
import {
  getUserProfile, getUserPosts,
  getUserFollowing, getUserFollowers, downloadUserPosts,
} from "@/lib/api";
import { useActiveTask } from "@/hooks/use-active-task";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
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
import { CommentDialog } from "@/components/shared/comment-dialog";

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
  const [loading, setLoading] = useState(false);
  const [profile, setProfile] = useState<UserProfileType | null>(null);
  const [following, setFollowing] = useState<FollowItem[]>([]);
  const [followers, setFollowers] = useState<FollowItem[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [downloadedCount, setDownloadedCount] = useState(0);
  const [activeTaskId, setActiveTaskId] = useState<string | null>(null);
  const [currentUrl, setCurrentUrl] = useState("");
  const [commentAwemeId, setCommentAwemeId] = useState<string | null>(null);
  const batchTask = useActiveTask(activeTaskId);
  const downloadProgress = batchTask ? ((batchTask.total ?? 0) > 0 ? Math.round(((batchTask.completed ?? 0) / (batchTask.total ?? 1)) * 100) : 0) : 0;

  const { items: videos, setItems: setVideos, hasMore, loadingMore, sentinelRef, reset } = useInfiniteScroll<VideoItem>({
    fetchPage: useCallback(async (cursor: number) => {
      if (!currentUrl) return null;
      const res = await getUserPosts(currentUrl, cursor, 20);
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
    setVideos([]);
    setFollowing([]);
    setFollowers([]);
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

    const [postsRes, followingRes, followersRes] = await Promise.all([
      getUserPosts(url, 0, 20),
      getUserFollowing(url),
      getUserFollowers(url),
    ]);

    if (postsRes.success && postsRes.data?.videos) {
      reset(async () => ({
        items: postsRes.data!.videos!,
        nextCursor: postsRes.data!.next_cursor ?? 0,
        hasMore: postsRes.data!.has_more ?? false,
      }));
    }
    if (followingRes.success && followingRes.data?.followings) setFollowing(followingRes.data.followings);
    if (followersRes.success && followersRes.data?.followers) setFollowers(followersRes.data.followers);

    setLoading(false);
  }, [reset, setVideos]);

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

        <ErrorBanner message={error} />

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
                      onClick={() => setCommentAwemeId(video.aweme_id)}
                    />
                  ))}
                </div>
                <InfiniteScrollSentinel
                  sentinelRef={sentinelRef}
                  loadingMore={loadingMore}
                  hasMore={hasMore}
                  total={videos.length}
                  label="作品"
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

      <CommentDialog
        awemeId={commentAwemeId ?? ""}
        open={!!commentAwemeId}
        onOpenChange={(open) => !open && setCommentAwemeId(null)}
      />
    </>
  );
}
