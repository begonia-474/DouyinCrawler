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
import {
  getUserProfile, getUserPosts,
  getUserFollowing, getUserFollowers, downloadUserPosts,
} from "@/lib/api";
import type { UserProfile as UserProfileType, VideoItem, FollowItem } from "@/lib/api-types";
import {
  Download, Users, Heart, Video, Loader2,
  UserPlus, UserCheck, ThumbsUp, AlertCircle,
} from "lucide-react";
import { Progress } from "@/components/ui/progress";

function formatCount(n: number): string {
  if (n >= 10000) return `${(n / 10000).toFixed(1)}w`;
  return n.toLocaleString();
}

function formatDuration(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
}

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
  const [videos, setVideos] = useState<VideoItem[]>([]);
  const [following, setFollowing] = useState<FollowItem[]>([]);
  const [followers, setFollowers] = useState<FollowItem[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [downloadedCount, setDownloadedCount] = useState(0);
  const [downloadProgress, setDownloadProgress] = useState(0);

  const handleParse = useCallback(async (url: string) => {
    setLoading(true);
    setProfile(null);
    setVideos([]);
    setFollowing([]);
    setFollowers([]);
    setError(null);

    const profileRes = await getUserProfile(url);
    if (profileRes.success && profileRes.data?.profile) {
      setProfile(profileRes.data.profile as unknown as UserProfileType);
    } else {
      setError(profileRes.error || "获取用户信息失败");
      setLoading(false);
      return;
    }

    const [postsRes, followingRes, followersRes] = await Promise.all([
      getUserPosts(url),
      getUserFollowing(url),
      getUserFollowers(url),
    ]);

    if (postsRes.success && postsRes.data?.videos) setVideos(postsRes.data.videos);
    if (followingRes.success && followingRes.data?.followings) setFollowing(followingRes.data.followings);
    if (followersRes.success && followersRes.data?.followers) setFollowers(followersRes.data.followers);

    setLoading(false);
  }, []);

  const handleDownloadAll = async () => {
    setDownloading(true);
    setDownloadProgress(0);
    setDownloadedCount(0);

    const res = await downloadUserPosts(profile?.sec_user_id ? `https://www.douyin.com/user/${profile.sec_user_id}` : "");
    if (res.success) {
      setDownloadedCount(videos.length);
      setDownloadProgress(100);
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
            <Button onClick={handleDownloadAll} disabled={downloading}>
              {downloading ? <Loader2 className="h-4 w-4 mr-2 animate-spin" /> : <Download className="h-4 w-4 mr-2" />}
              {downloading ? `下载中 ${downloadedCount}/${videos.length}` : "全部下载"}
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
                <p className="text-xs text-muted-foreground tracking-wide text-right">{downloadedCount} / {videos.length}</p>
              </div>
            </div>
          </Bezel>
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
                <TabsTrigger value="posts">作品<Badge variant="secondary" className="ml-1.5">{videos.length}</Badge></TabsTrigger>
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
                      duration={formatDuration(video.duration)}
                      diggCount={video.digg_count}
                      commentCount={video.comment_count}
                      shareCount={video.share_count}
                    />
                  ))}
                </div>
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
    </>
  );
}
