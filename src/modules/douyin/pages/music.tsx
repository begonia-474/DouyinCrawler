import { useState, useCallback } from "react";
import { Header } from "@/components/layout/header";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { getMusicCollection } from "@/lib/api";
import type { MusicItem } from "@/lib/api-types";
import {
  Play,
  Download,
  Music,
  Loader2,
  Heart,
  Clock,
  AlertCircle,
} from "lucide-react";

function formatCount(count: number): string {
  if (count >= 10000) return `${(count / 10000).toFixed(1)}w`;
  return count.toString();
}

function formatDuration(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
}

export default function MusicPage() {
  const [loading, setLoading] = useState(false);
  const [musicList, setMusicList] = useState<MusicItem[]>([]);
  const [hasMore, setHasMore] = useState(false);
  const [cursor, setCursor] = useState(0);
  const [error, setError] = useState<string | null>(null);

  const handleLoad = useCallback(async () => {
    setLoading(true);
    setError(null);
    const res = await getMusicCollection(0);
    if (res.success && res.data) {
      setMusicList(res.data.music_list || []);
      setHasMore(res.data.has_more || false);
      setCursor(res.data.music_list?.length || 0);
    } else {
      setError(res.error || "加载失败");
    }
    setLoading(false);
  }, []);

  const handleLoadMore = async () => {
    setLoading(true);
    const res = await getMusicCollection(cursor);
    if (res.success && res.data) {
      setMusicList((prev) => [...prev, ...(res.data?.music_list || [])]);
      setHasMore(res.data?.has_more || false);
      setCursor((prev) => prev + (res.data?.music_list?.length || 0));
    }
    setLoading(false);
  };

  return (
    <>
      <Header title="音乐收藏" description="查看收藏的音乐">
        <Button onClick={handleLoad} disabled={loading} variant="outline" size="sm">
          {loading ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <Music className="h-4 w-4 mr-1" />}
          加载收藏
        </Button>
      </Header>

      <div className="space-y-6">
        {error && (
          <div className="flex items-center gap-2 p-3 rounded-lg bg-destructive/10 text-destructive text-sm">
            <AlertCircle className="h-4 w-4 shrink-0" />
            <span>{error}</span>
          </div>
        )}

        <div>
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-sm font-medium text-muted-foreground">
              我的收藏
              {musicList.length > 0 && (
                <Badge variant="secondary" className="ml-1.5">{musicList.length}</Badge>
              )}
            </h3>
          </div>

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
            <div className="space-y-3">
              {musicList.map((item) => (
                <Card key={item.id}>
                  <CardContent className="p-4 flex items-center gap-4">
                    <Avatar className="h-10 w-10 shrink-0">
                      <AvatarImage src={item.avatar} />
                      <AvatarFallback><Music className="h-4 w-4" /></AvatarFallback>
                    </Avatar>
                    <div className="flex-1 min-w-0">
                      <h4 className="text-sm font-medium truncate">{item.title}</h4>
                      <div className="flex items-center gap-2 text-xs text-muted-foreground mt-0.5">
                        <span>{item.author}</span>
                        <span>·</span>
                        <span className="flex items-center gap-1">
                          <Clock className="h-3 w-3" />
                          {formatDuration(item.duration)}
                        </span>
                        <span>·</span>
                        <span>{formatCount(item.use_count)} 次使用</span>
                      </div>
                    </div>
                    <div className="flex items-center gap-1">
                      <Button variant="ghost" size="icon" className="h-8 w-8">
                        <Heart className="h-4 w-4 text-muted-foreground" />
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
                <Button variant="outline" className="w-full" onClick={handleLoadMore} disabled={loading}>
                  {loading ? <Loader2 className="h-4 w-4 mr-2 animate-spin" /> : null}
                  加载更多
                </Button>
              )}
            </div>
          )}
        </div>
      </div>
    </>
  );
}
