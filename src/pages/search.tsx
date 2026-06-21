import { useState } from "react";
import { Header } from "@/components/layout/header";
import { useMounted } from "@/hooks/use-safe-timer";
import { VideoCard } from "@/components/shared/video-card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Search, Loader2 } from "lucide-react";

interface SearchResult {
  id: string;
  title: string;
  author: string;
  duration: string;
  diggCount: number;
  commentCount: number;
}

export function SearchPage() {
  const [keyword, setKeyword] = useState("");
  const [loading, setLoading] = useState(false);
  const [results, setResults] = useState<SearchResult[]>([]);
  const [searched, setSearched] = useState(false);
  const mountedRef = useMounted();

  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!keyword.trim()) return;

    setLoading(true);
    setSearched(true);

    // 模拟搜索
    setTimeout(() => {
      if (!mountedRef.current) return;
      setResults([
        {
          id: "1",
          title: "搜索结果示例 1",
          author: "用户A",
          duration: "00:15",
          diggCount: 5200,
          commentCount: 320,
        },
        {
          id: "2",
          title: "搜索结果示例 2",
          author: "用户B",
          duration: "01:20",
          diggCount: 1800,
          commentCount: 95,
        },
      ]);
      setLoading(false);
    }, 1000);
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

        {searched && (
          <>
            {results.length === 0 ? (
              <Card>
                <CardContent className="p-8 text-center text-muted-foreground">
                  {loading ? "搜索中..." : "没有找到相关结果"}
                </CardContent>
              </Card>
            ) : (
              <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
                {results.map((item) => (
                  <VideoCard
                    key={item.id}
                    title={item.title}
                    author={item.author}
                    duration={item.duration}
                    diggCount={item.diggCount}
                    commentCount={item.commentCount}
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
