import { useCallback } from "react";
import { Header } from "@/components/layout/header";
import { VideoCard } from "@/components/shared/video-card";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { Bezel } from "@/components/shared/bezel";
import { ErrorBanner } from "@/components/shared/error-banner";
import { useTabFeedQuery, useFollowFeedQuery, useFriendFeedQuery } from "@/lib/queries";
import type { VideoItem } from "@/lib/api-types";
import { RefreshCw, Loader2, Rss } from "lucide-react";

export default function FeedPage() {
  const tabFeed = useTabFeedQuery(false);
  const followFeed = useFollowFeedQuery(false);
  const friendFeed = useFriendFeedQuery(false);

  const handleRefresh = useCallback((type: "tab" | "follow" | "friend") => {
    const feedMap = { tab: tabFeed, follow: followFeed, friend: friendFeed };
    feedMap[type].refetch();
  }, [tabFeed, followFeed, friendFeed]);

  const renderVideoGrid = (
    videos: VideoItem[],
    type: string,
    typeKey: "tab" | "follow" | "friend",
    isLoading: boolean,
  ) => {
    if (videos.length === 0) {
      return (
        <Bezel radius="xl">
          <div className="p-14 text-center">
            <Rss className="h-10 w-10 text-muted-foreground/30 mx-auto mb-4" />
            <p className="text-muted-foreground text-sm tracking-wide mb-6">暂无{type}内容</p>
            <Button variant="capsule" onClick={() => handleRefresh(typeKey)} disabled={isLoading}>
              {isLoading ? <Loader2 className="h-4 w-4 animate-spin mr-1.5" /> : <RefreshCw className="h-4 w-4 mr-1.5" />}
              刷新
            </Button>
          </div>
        </Bezel>
      );
    }

    return (
      <div className="space-y-6">
        <div className="flex justify-end">
          <Button variant="capsule" size="sm" onClick={() => handleRefresh(typeKey)} disabled={isLoading}>
            {isLoading ? <Loader2 className="h-4 w-4 animate-spin mr-1" /> : <RefreshCw className="h-4 w-4 mr-1" />}
            刷新
          </Button>
        </div>
        <div className="grid grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-5">
          {videos.map((video, i) => (
            <AnimateEntry key={video.aweme_id} delay={i * 40}>
              <VideoCard
                title={video.desc}
                author={video.author}
                duration={String(video.duration)}
                diggCount={video.digg_count}
                commentCount={video.comment_count}
                shareCount={video.share_count}
              />
            </AnimateEntry>
          ))}
        </div>
      </div>
    );
  };

  const tabVideos = (tabFeed.data?.data?.videos ?? []) as unknown as VideoItem[];
  const followVideos = (followFeed.data?.data?.videos ?? []) as unknown as VideoItem[];
  const friendVideos = (friendFeed.data?.data?.videos ?? []) as unknown as VideoItem[];

  return (
    <>
      <AnimateEntry>
        <Header title="Feed" description="推荐、关注、朋友动态" />
      </AnimateEntry>

      <ErrorBanner message={tabFeed.error?.message ?? null} className="mb-6" />

      <Tabs defaultValue="tab">
        <TabsList>
          <TabsTrigger value="tab">推荐</TabsTrigger>
          <TabsTrigger value="follow">关注</TabsTrigger>
          <TabsTrigger value="friend">朋友</TabsTrigger>
        </TabsList>

        <TabsContent value="tab" className="mt-8">
          {renderVideoGrid(tabVideos, "推荐", "tab", tabFeed.isFetching)}
        </TabsContent>

        <TabsContent value="follow" className="mt-8">
          {renderVideoGrid(followVideos, "关注", "follow", followFeed.isFetching)}
        </TabsContent>

        <TabsContent value="friend" className="mt-8">
          {renderVideoGrid(friendVideos, "朋友", "friend", friendFeed.isFetching)}
        </TabsContent>
      </Tabs>
    </>
  );
}
