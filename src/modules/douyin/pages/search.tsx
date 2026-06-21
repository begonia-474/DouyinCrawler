import { useState } from "react";
import { Header } from "@/components/layout/header";
import { VideoCard } from "@/components/shared/video-card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { search } from "@/lib/api";
import type { VideoItem } from "@/lib/api-types";
import { Search, Loader2, AlertCircle } from "lucide-react";

export default function SearchPage() {
  const [keyword, setKeyword] = useState("");
  const [loading, setLoading] = useState(false);
  const [results, setResults] = useState<VideoItem[]>([]);
  const [searched, setSearched] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!keyword.trim()) return;

    setLoading(true);
    setSearched(true);
    setError(null);

    const res = await search(keyword);
    if (res.success && res.data?.videos) {
      setResults(res.data.videos);
    } else {
      setResults([]);
      if (!res.success) setError(res.error || "搜索失败");
    }
    setLoading(false);
  };

  return (
    <>
      <Header title="搜索" description="搜索抖音视频" />

      <div className="space-y-6">
        <form onSubmit={handleSearch} className="flex gap-2">
          <Input
            value={keyword}
            onChange={(e) => setKeyword(e.target.value)}
            placeholder="输入关键词搜索..."
            className="flex-1"
          />
          <Button type="submit" disabled={!keyword.trim() || loading}>
            {loading ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <Search className="h-4 w-4" />
            )}
          </Button>
        </form>

        {error && (
          <div className="flex items-center gap-2 p-3 rounded-lg bg-destructive/10 text-destructive text-sm">
            <AlertCircle className="h-4 w-4 shrink-0" />
            <span>{error}</span>
          </div>
        )}

        {searched && (
          <>
            {results.length === 0 ? (
              <Card>
                <CardContent className="p-8 text-center text-muted-foreground">
                  <Search className="h-10 w-10 mx-auto mb-3" />
                  <p>{loading ? "搜索中..." : "没有找到相关结果"}</p>
                </CardContent>
              </Card>
            ) : (
              <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
                {results.map((item) => (
                  <VideoCard
                    key={item.aweme_id}
                    title={item.desc}
                    author={item.author}
                    duration={String(item.duration)}
                    diggCount={item.digg_count}
                    commentCount={item.comment_count}
                    shareCount={item.share_count}
                  />
                ))}
              </div>
            )}
          </>
        )}
      </div>
    </>
  );
}
