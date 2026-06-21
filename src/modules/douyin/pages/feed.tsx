import { useState, useCallback } from "react";
import { Header } from "@/components/layout/header";
import { VideoCard } from "@/components/shared/video-card";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { getTabFeed, getFollowFeed, getFriendFeed } from "@/lib/api";
import type { VideoItem } from "@/lib/api-types";
import { RefreshCw, Loader2, Rss, AlertCircle } from "lucide-react";

export default function FeedPage() {
  const [loading, setLoading] = useState(false);
  const [tabVideos, setTabVideos] = useState<VideoItem[]>([]);
  const [followVideos, setFollowVideos] = useState<VideoItem[]>([]);
  const [friendVideos, setFriendVideos] = useState<VideoItem[]>([]);
  const [error, setError] = useState<string | null>(null);

  const handleRefresh = useCallback(async (type: "tab" | "follow" | "friend") => {
    setLoading(true);
    setError(null);

    const apiMap = {
      tab: getTabFeed,
      follow: getFollowFeed,
      friend: getFriendFeed,
    };
    const setterMap = {
      tab: setTabVideos,
      follow: setFollowVideos,
      friend: setFriendVideos,
    };

    const res = await apiMap[type]();
    if (res.success && res.data?.videos) {
      setterMap[type](res.data.videos);
    } else {
      setError(res.error || "加载失败");
    }
    setLoading(false);
  }, []);

  const renderVideoGrid = (videos: VideoItem[], type: string, typeKey: "tab" | "follow" | "friend") => {
    if (videos.length === 0) {
      return (
        <Card>
          <CardContent className="p-8 text-center">
            <Rss className="h-10 w-10 text-muted-foreground mx-auto mb-3" />
            <p className="text-muted-foreground mb-4">暂无{type}内容</p>
            <Button variant="outline" onClick={() => handleRefresh(typeKey)} disabled={loading}>
              {loading ? <Loader2 className="h-4 w-4 animate-spin mr-2" /> : <RefreshCw className="h-4 w-4 mr-2" />}
              刷新
            </Button>
          </CardContent>
        </Card>
      );
    }

    return (
      <div className="space-y-4">
        <div className="flex justify-end">
          <Button variant="outline" size="sm" onClick={() => handleRefresh(typeKey)} disabled={loading}>
            {loading ? <Loader2 className="h-4 w-4 animate-spin mr-1" /> : <RefreshCw className="h-4 w-4 mr-1" />}
            刷新
          </Button>
        </div>
        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
          {videos.map((video) => (
            <VideoCard
              key={video.aweme_id}
              title={video.desc}
              author={video.author}
              duration={String(video.duration)}
              diggCount={video.digg_count}
              commentCount={video.comment_count}
              shareCount={video.share_count}
            />
          ))}
        </div>
      </div>
    );
  };

  return (
    <>
      <Header title="Feed" description="推荐、关注、朋友动态" />

      {error && (
        <div className="flex items-center gap-2 p-3 rounded-lg bg-destructive/10 text-destructive text-sm mb-4">
          <AlertCircle className="h-4 w-4 shrink-0" />
          <span>{error}</span>
        </div>
      )}

      <Tabs defaultValue="tab">
        <TabsList>
          <TabsTrigger value="tab">推荐</TabsTrigger>
          <TabsTrigger value="follow">关注</TabsTrigger>
          <TabsTrigger value="friend">朋友</TabsTrigger>
        </TabsList>

        <TabsContent value="tab" className="mt-4">
          {renderVideoGrid(tabVideos, "推荐", "tab")}
        </TabsContent>

        <TabsContent value="follow" className="mt-4">
          {renderVideoGrid(followVideos, "关注", "follow")}
        </TabsContent>

        <TabsContent value="friend" className="mt-4">
          {renderVideoGrid(friendVideos, "朋友", "friend")}
        </TabsContent>
      </Tabs>
    </>
  );
}
