import { useState, useCallback, useEffect, useRef } from "react";
import { useNavigate } from "react-router-dom";
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
import { usePersistedUrl } from "@/hooks/use-persisted-url";
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
  const navigate = useNavigate();
  const [loading, setLoading] = useState(false);
  const [profile, setProfile] = useState<UserProfileType | null>(null);
  const [following, setFollowing] = useState<FollowItem[]>([]);
  const [followers, setFollowers] = useState<FollowItem[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [downloadedCount, setDownloadedCount] = useState(0);
  const [activeTaskId, setActiveTaskId] = useState<string | null>(null);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [currentUrl, setCurrentUrl] = usePersistedUrl("user");
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
  }, [reset, setVideos, setCurrentUrl]);

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

    const url = profile?.sec_user_id ? `https://www.douyin.com/user/${profile.sec_user_id}` : "";
    const res = await downloadUserPosts(url, Array.from(selectedIds));
    if (res.success && res.task_id) {
      setActiveTaskId(res.task_id);
      setSelectedIds(new Set());
    } else {
      setError(res.error || "下载失败");
    }
    setDownloading(false);
  };

  const handleCardDownload = useCallback((video: VideoItem) => {
    // 单个下载：选中后触发选择下载
    const url = profile?.sec_user_id ? `https://www.douyin.com/user/${profile.sec_user_id}` : "";
    downloadUserPosts(url, [video.aweme_id]);
  }, [profile]);

  return (
    <>
      <AnimateEntry>
        <Header title="用户主页" description="查看用户资料和作品" parent={{ label: "首页", path: "/douyin" }}>
          {profile && (
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
                {videos.length > 0 && (
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
                )}
                <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-5">
                  {videos.map((video) => (
                    <VideoCard
                      key={video.aweme_id}
                      title={video.desc}
                      author={profile.nickname}
                      cover={video.cover_url}
                      duration={formatDurationSec(video.duration)}
                      diggCount={video.digg_count}
                      commentCount={video.comment_count}
                      shareCount={video.share_count}
                      onClick={() => navigate(`/douyin/video/${video.aweme_id}`, { state: { from: "用户主页", fromPath: "/douyin/user" } })}
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
    </>
  );
}
