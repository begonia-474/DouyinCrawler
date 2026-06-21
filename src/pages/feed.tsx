import { useState } from "react";
import { Header } from "@/components/layout/header";
import { VideoCard } from "@/components/shared/video-card";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { RefreshCw, Loader2 } from "lucide-react";

interface FeedVideo {
  id: string;
  title: string;
  author: string;
  duration: string;
  diggCount: number;
  commentCount: number;
  shareCount: number;
}

export function FeedPage() {
  const [loading, setLoading] = useState(false);
  const [tabVideos, setTabVideos] = useState<FeedVideo[]>([]);
  const [followVideos, setFollowVideos] = useState<FeedVideo[]>([]);
  const [friendVideos, setFriendVideos] = useState<FeedVideo[]>([]);

  const handleRefresh = async (type: string) => {
    setLoading(true);
    // 模拟加载
    setTimeout(() => {
      const mockVideo: FeedVideo = {
        id: Date.now().toString(),
        title: `${type}推荐视频`,
        author: "推荐用户",
        duration: "00:30",
        diggCount: 5200,
        commentCount: 320,
        shareCount: 120,
      };

      if (type === "推荐") {
        setTabVideos([mockVideo]);
      } else if (type === "关注") {
        setFollowVideos([mockVideo]);
      } else {
        setFriendVideos([mockVideo]);
      }
      setLoading(false);
    }, 1000);
  };

  const renderVideoGrid = (videos: FeedVideo[], type: string) => {
    if (videos.length === 0) {
      return (
        <Card>
          <CardContent className="p-8 text-center">
            <p className="text-muted-foreground mb-4">暂无{type}内容</p>
            <Button
              variant="outline"
              onClick={() => handleRefresh(type)}
              disabled={loading}
            >
              {loading ? (
                <Loader2 className="h-4 w-4 animate-spin mr-2" />
              ) : (
                <RefreshCw className="h-4 w-4 mr-2" />
              )}
              刷新
            </Button>
          </CardContent>
        </Card>
      );
    }

    return (
      <div className="space-y-4">
        <div className="flex justify-end">
          <Button
            variant="outline"
            size="sm"
            onClick={() => handleRefresh(type)}
            disabled={loading}
          >
            {loading ? (
              <Loader2 className="h-4 w-4 animate-spin mr-1" />
            ) : (
              <RefreshCw className="h-4 w-4 mr-1" />
            )}
            刷新
          </Button>
        </div>
        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
          {videos.map((video) => (
            <VideoCard
              key={video.id}
              title={video.title}
              author={video.author}
              duration={video.duration}
              diggCount={video.diggCount}
              commentCount={video.commentCount}
              shareCount={video.shareCount}
            />
          ))}
        </div>
      </div>
    );
  };

  return (
    <>
      <Header title="Feed" description="推荐、关注、朋友动态" />

      <Tabs defaultValue="tab">
        <TabsList>
          <TabsTrigger value="tab">推荐</TabsTrigger>
          <TabsTrigger value="follow">关注</TabsTrigger>
          <TabsTrigger value="friend">朋友</TabsTrigger>
        </TabsList>

        <TabsContent value="tab" className="mt-4">
          {renderVideoGrid(tabVideos, "推荐")}
        </TabsContent>

        <TabsContent value="follow" className="mt-4">
          {renderVideoGrid(followVideos, "关注")}
        </TabsContent>

        <TabsContent value="friend" className="mt-4">
          {renderVideoGrid(friendVideos, "朋友")}
        </TabsContent>
      </Tabs>
    </>
  );
}
