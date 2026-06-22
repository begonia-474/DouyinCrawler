import { useState, useCallback } from "react";
import { Header } from "@/components/layout/header";
import { UrlInput } from "@/components/shared/url-input";
import { VideoCard } from "@/components/shared/video-card";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
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
    <Card>
      <CardContent className="p-4 flex items-center gap-4">
        <Avatar className="h-10 w-10">
          <AvatarImage src={item.avatar} />
          <AvatarFallback>{item.nickname?.[0] || "?"}</AvatarFallback>
        </Avatar>
        <div className="flex-1 min-w-0">
          <h4 className="text-sm font-medium">{item.nickname}</h4>
          {item.signature && <p className="text-xs text-muted-foreground truncate">{item.signature}</p>}
          <p className="text-xs text-muted-foreground">{formatCount(item.follower_count)} 粉丝</p>
        </div>
        <Button variant="outline" size="sm">
          {type === "following" ? <><UserCheck className="h-3.5 w-3.5 mr-1" />已关注</> : <><UserPlus className="h-3.5 w-3.5 mr-1" />关注</>}
        </Button>
      </CardContent>
    </Card>
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

    // 并行加载其他数据
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
      <Header title="用户主页" description="查看用户资料和作品" parent={{ label: "首页", path: "/douyin" }}>
        {profile && (
          <Button onClick={handleDownloadAll} disabled={downloading}>
            {downloading ? <Loader2 className="h-4 w-4 mr-2 animate-spin" /> : <Download className="h-4 w-4 mr-2" />}
            {downloading ? `下载中 ${downloadedCount}/${videos.length}` : "全部下载"}
          </Button>
        )}
      </Header>

      <div className="space-y-6">
        <UrlInput onSubmit={handleParse} loading={loading} placeholder="粘贴用户主页链接..." allowedTypes={["user"]} />

        {error && (
          <div className="flex items-center gap-2 p-3 rounded-lg bg-destructive/10 text-destructive text-sm">
            <AlertCircle className="h-4 w-4 shrink-0" />
            <span>{error}</span>
          </div>
        )}

        {loading && (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        )}

        {downloading && (
          <Card>
            <CardContent className="p-4">
              <div className="space-y-1">
                <Progress value={downloadProgress} />
                <p className="text-xs text-muted-foreground text-right">{downloadedCount} / {videos.length}</p>
              </div>
            </CardContent>
          </Card>
        )}

        {profile && !loading && (
          <>
            {/* 用户信息卡片 */}
            <Card>
              <CardContent className="p-6">
                <div className="flex items-start gap-4">
                  <Avatar className="h-16 w-16">
                    <AvatarImage src={profile.avatar} />
                    <AvatarFallback>{profile.nickname?.[0] || "?"}</AvatarFallback>
                  </Avatar>
                  <div className="flex-1">
                    <h3 className="text-lg font-semibold">{profile.nickname}</h3>
                    <p className="text-sm text-muted-foreground mt-1">{profile.signature}</p>
                    <div className="flex items-center gap-5 mt-3">
                      <div className="flex items-center gap-1.5 text-sm">
                        <Video className="h-4 w-4 text-muted-foreground" />
                        <span className="font-semibold">{profile.aweme_count}</span>
                        <span className="text-muted-foreground">作品</span>
                      </div>
                      <div className="flex items-center gap-1.5 text-sm">
                        <Users className="h-4 w-4 text-muted-foreground" />
                        <span className="font-semibold">{formatCount(profile.follower_count)}</span>
                        <span className="text-muted-foreground">粉丝</span>
                      </div>
                      <div className="flex items-center gap-1.5 text-sm">
                        <Heart className="h-4 w-4 text-muted-foreground" />
                        <span className="font-semibold">{formatCount(profile.following_count)}</span>
                        <span className="text-muted-foreground">关注</span>
                      </div>
                      <div className="flex items-center gap-1.5 text-sm">
                        <ThumbsUp className="h-4 w-4 text-muted-foreground" />
                        <span className="font-semibold">{formatCount(profile.total_favorited)}</span>
                        <span className="text-muted-foreground">获赞</span>
                      </div>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* 功能 Tab */}
            <Tabs defaultValue="posts">
              <TabsList>
                <TabsTrigger value="posts">作品<Badge variant="secondary" className="ml-1.5">{videos.length}</Badge></TabsTrigger>
                <TabsTrigger value="following">关注<Badge variant="secondary" className="ml-1.5">{following.length}</Badge></TabsTrigger>
                <TabsTrigger value="followers">粉丝<Badge variant="secondary" className="ml-1.5">{followers.length}</Badge></TabsTrigger>
              </TabsList>

              <TabsContent value="posts" className="mt-4">
                <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
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

              <TabsContent value="following" className="mt-4 space-y-1">
                {following.map((item) => (
                  <FollowItemCard key={item.uid} item={item} type="following" />
                ))}
              </TabsContent>

              <TabsContent value="followers" className="mt-4 space-y-1">
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
