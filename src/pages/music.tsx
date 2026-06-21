import { useState } from "react";
import { Header } from "@/components/layout/header";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Input } from "@/components/ui/input";
import {
  Play,
  Download,
  Music,
  ExternalLink,
  Loader2,
  Heart,
  Clock,
} from "lucide-react";

interface MusicItem {
  id: string;
  title: string;
  artist: string;
  avatar: string;
  duration: string;
  useCount: number;
  liked?: boolean;
}

const mockMusic: MusicItem[] = [
  { id: "1", title: "热门BGM 1", artist: "音乐人A", avatar: "", duration: "00:30", useCount: 125000 },
  { id: "2", title: "旅行必备音乐", artist: "音乐人B", avatar: "", duration: "01:45", useCount: 89000 },
  { id: "3", title: "搞笑配乐", artist: "音乐人C", avatar: "", duration: "00:15", useCount: 230000 },
  { id: "4", title: "美食视频BGM", artist: "音乐人D", avatar: "", duration: "02:00", useCount: 56000 },
  { id: "5", title: "舞蹈热门音乐", artist: "音乐人E", avatar: "", duration: "00:45", useCount: 340000 },
];

function formatCount(count: number): string {
  if (count >= 10000) return `${(count / 10000).toFixed(1)}w`;
  return count.toString();
}

export function MusicPage() {
  const [loading, setLoading] = useState(false);
  const [musicList, setMusicList] = useState<MusicItem[]>([]);
  const [hasMore, setHasMore] = useState(false);
  const [searchKeyword, setSearchKeyword] = useState("");

  const handleLoad = async () => {
    setLoading(true);
    setTimeout(() => {
      setMusicList(mockMusic);
      setHasMore(true);
      setLoading(false);
    }, 1000);
  };

  const handleLoadMore = async () => {
    setLoading(true);
    setTimeout(() => {
      setMusicList((prev) => [
        ...prev,
        {
          id: Date.now().toString(),
          title: "更多音乐...",
          artist: "音乐人F",
          avatar: "",
          duration: "01:20",
          useCount: 42000,
        },
      ]);
      setHasMore(false);
      setLoading(false);
    }, 800);
  };

  const handleToggleLike = (id: string) => {
    setMusicList((prev) =>
      prev.map((m) => (m.id === id ? { ...m, liked: !m.liked } : m))
    );
  };

  return (
    <>
      <Header title="音乐收藏" description="查看和管理收藏的音乐">
        <Button onClick={handleLoad} disabled={loading} variant="outline" size="sm">
          {loading ? (
            <Loader2 className="h-4 w-4 mr-1 animate-spin" />
          ) : (
            <Music className="h-4 w-4 mr-1" />
          )}
          加载收藏
        </Button>
      </Header>

      <div className="space-y-6">
        <Tabs defaultValue="collection">
          <TabsList>
            <TabsTrigger value="collection">
              我的收藏
              {musicList.length > 0 && (
                <Badge variant="secondary" className="ml-1.5">
                  {musicList.length}
                </Badge>
              )}
            </TabsTrigger>
            <TabsTrigger value="search">搜索音乐</TabsTrigger>
          </TabsList>

          <TabsContent value="collection" className="mt-4 space-y-3">
            {musicList.length === 0 ? (
              <Card>
                <CardContent className="p-8 text-center">
                  <Music className="h-10 w-10 text-muted-foreground mx-auto mb-3" />
                  <p className="text-muted-foreground mb-4">
                    {loading ? "加载中..." : "点击「加载收藏」获取音乐列表"}
                  </p>
                </CardContent>
              </Card>
            ) : (
              <>
                {musicList.map((item) => (
                  <Card key={item.id}>
                    <CardContent className="p-4 flex items-center gap-4">
                      <Avatar className="h-10 w-10 shrink-0">
                        <AvatarImage src={item.avatar} />
                        <AvatarFallback>
                          <Music className="h-4 w-4" />
                        </AvatarFallback>
                      </Avatar>
                      <div className="flex-1 min-w-0">
                        <h4 className="text-sm font-medium truncate">
                          {item.title}
                        </h4>
                        <div className="flex items-center gap-2 text-xs text-muted-foreground mt-0.5">
                          <span>{item.artist}</span>
                          <span>·</span>
                          <span className="flex items-center gap-1">
                            <Clock className="h-3 w-3" />
                            {item.duration}
                          </span>
                          <span>·</span>
                          <span>{formatCount(item.useCount)} 次使用</span>
                        </div>
                      </div>
                      <div className="flex items-center gap-1">
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8"
                          onClick={() => handleToggleLike(item.id)}
                        >
                          <Heart
                            className={`h-4 w-4 ${
                              item.liked
                                ? "fill-red-500 text-red-500"
                                : "text-muted-foreground"
                            }`}
                          />
                        </Button>
                        <Button variant="ghost" size="icon" className="h-8 w-8">
                          <Play className="h-4 w-4" />
                        </Button>
                        <Button variant="ghost" size="icon" className="h-8 w-8">
                          <Download className="h-4 w-4" />
                        </Button>
                      </div>
                    </CardContent>
                  </Card>
                ))}

                {hasMore && (
                  <Button
                    variant="outline"
                    className="w-full"
                    onClick={handleLoadMore}
                    disabled={loading}
                  >
                    {loading ? (
                      <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                    ) : null}
                    加载更多
                  </Button>
                )}
              </>
            )}
          </TabsContent>

          <TabsContent value="search" className="mt-4 space-y-4">
            <div className="flex gap-2">
              <Input
                value={searchKeyword}
                onChange={(e) => setSearchKeyword(e.target.value)}
                placeholder="搜索音乐..."
                className="flex-1"
              />
              <Button variant="outline">
                <ExternalLink className="h-4 w-4" />
              </Button>
            </div>
            <Card>
              <CardContent className="p-8 text-center text-muted-foreground">
                输入关键词搜索音乐
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </>
  );
}
