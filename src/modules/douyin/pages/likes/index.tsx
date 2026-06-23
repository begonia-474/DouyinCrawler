import { useState, useCallback } from "react";
import { Header } from "@/components/layout/header";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { UrlInput } from "@/components/shared/url-input";
import { VideoCard } from "@/components/shared/video-card";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
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

  const handleParse = useCallback(async (url: string) => {
    setLoading(true);
    setProfile(null);
    setLikes([]);
    setError(null);

    const profileRes = await getUserProfile(url);
    if (profileRes.success && profileRes.data?.profile) {
      setProfile(profileRes.data.profile as unknown as UserProfileType);
    } else {
      setError(profileRes.error || "获取用户信息失败");
      setLoading(false);
      return;
    }

    const likesRes = await getUserLikes(url);
    if (likesRes.success && likesRes.data?.videos) {
      setLikes(likesRes.data.videos);
    }

    setLoading(false);
  }, []);

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
            <Button variant="outline" size="sm" onClick={handleDownloadAll} disabled={downloading}>
              {downloading ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <Download className="h-4 w-4 mr-1" />}
              {downloading ? `下载中 ${downloadedCount}/${likes.length}` : "全部下载"}
            </Button>
          )}
        </Header>
      </AnimateEntry>

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
          <Card className="border-border/40 bg-card/60">
            <CardContent className="p-4">
              <div className="space-y-1">
                <Progress value={downloadProgress} />
                <p className="text-xs text-muted-foreground tracking-wide text-right">{downloadedCount} / {likes.length}</p>
              </div>
            </CardContent>
          </Card>
        )}

        {profile && !loading && (
          <>
            <Card className="border-border/40 bg-card/60">
              <CardContent className="p-4 flex items-center gap-4">
                <Avatar className="h-12 w-12">
                  <AvatarImage src={profile.avatar} />
                  <AvatarFallback>{profile.nickname?.[0] || "?"}</AvatarFallback>
                </Avatar>
                <div>
                  <h3 className="font-semibold">{profile.nickname}</h3>
                  <p className="text-sm text-muted-foreground tracking-wide">{likes.length} 个点赞</p>
                </div>
              </CardContent>
            </Card>

            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
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
          </>
        )}
      </div>
    </>
  );
}
