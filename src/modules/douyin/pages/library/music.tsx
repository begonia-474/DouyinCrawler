import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Music, Loader2, Search, Clock, Download, CheckCircle2 } from "lucide-react";
import { getMusicCollectionFromDB, getMusicCollectionCountFromDB, downloadMusic } from "@/lib/api";
import type { MusicCollectionItem } from "@/lib/api";

function formatDuration(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
}

export default function LibraryMusicPage() {
  const navigate = useNavigate();
  const [items, setItems] = useState<MusicCollectionItem[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);
  const [page, setPage] = useState(0);
  const [search, setSearch] = useState("");
  const [downloadingId, setDownloadingId] = useState<string | null>(null);
  const pageSize = 20;

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [data, count] = await Promise.all([
        getMusicCollectionFromDB({
          limit: pageSize,
          offset: page * pageSize,
          keyword: search || undefined,
          status: "downloaded",
        }),
        getMusicCollectionCountFromDB(search || undefined, "downloaded"),
      ]);
      setItems(data);
      setTotal(count);
    } catch (err) {
      console.error("加载失败:", err);
    }
    setLoading(false);
  }, [page, search]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const handleDownload = async (item: MusicCollectionItem) => {
    if (!item.play_url) return;
    setDownloadingId(item.music_id);

    const res = await downloadMusic(item.play_url, item.title || "", item.author || "");
    if (res.success) {
      loadData(); // 刷新列表以更新状态
    }

    setDownloadingId(null);
  };

  const totalPages = Math.ceil(total / pageSize);

  return (
    <div className="flex flex-col h-full">
      {/* 搜索栏 */}
      <div className="p-4 border-b">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            value={search}
            onChange={(e) => { setSearch(e.target.value); setPage(0); }}
            placeholder="搜索音乐..."
            className="pl-9"
          />
        </div>
      </div>

      {/* 面包屑 */}
      <div className="px-4 py-3 flex items-center gap-2 text-sm">
        <button
          className="text-muted-foreground hover:text-foreground transition-colors"
          onClick={() => navigate("/douyin/library")}
        >
          &lt;
        </button>
        <button
          className="text-muted-foreground hover:text-foreground transition-colors"
          onClick={() => navigate("/douyin/library")}
        >
          资料库
        </button>
        <span className="text-muted-foreground">/</span>
        <span className="font-medium">音乐</span>
        <span className="text-xs text-muted-foreground ml-auto">{total} 条记录</span>
      </div>

      {/* 内容区 */}
      <div className="flex-1 overflow-auto p-4">
        {loading ? (
          <div className="flex justify-center py-12">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : items.length === 0 ? (
          <Card>
            <CardContent className="p-8 text-center text-muted-foreground">
              <Music className="h-10 w-10 mx-auto mb-3" />
              <p>暂无音乐收藏</p>
              <p className="text-sm mt-2">请先在「音乐」页面收藏音乐</p>
            </CardContent>
          </Card>
        ) : (
          <div className="space-y-2">
            {items.map((item) => (
              <div
                key={item.music_id}
                className="flex items-center gap-4 p-3 border rounded-lg hover:bg-muted/50 transition-colors"
              >
                {item.cover ? (
                  <img
                    src={item.cover}
                    alt=""
                    className="h-12 w-12 rounded-lg object-cover shrink-0"
                  />
                ) : (
                  <div className="h-12 w-12 rounded-lg bg-primary/10 flex items-center justify-center shrink-0">
                    <Music className="h-5 w-5 text-primary" />
                  </div>
                )}
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium truncate">
                    {item.title || "未知歌曲"}
                  </p>
                  <div className="flex items-center gap-3 mt-1 flex-wrap">
                    {(item.author || item.owner_nickname) && (
                      <span className="text-xs text-muted-foreground">
                        {item.author || item.owner_nickname}
                      </span>
                    )}
                    <span className="text-xs text-muted-foreground flex items-center gap-1">
                      <Clock className="h-3 w-3" />
                      {formatDuration(item.duration)}
                    </span>
                    {item.status === "downloaded" && (
                      <span className="text-xs text-green-600 flex items-center gap-1">
                        <CheckCircle2 className="h-3 w-3" />
                        已下载
                      </span>
                    )}
                  </div>
                </div>
                <Button
                  variant="outline"
                  size="icon"
                  className="h-8 w-8 shrink-0"
                  onClick={() => handleDownload(item)}
                  disabled={!item.play_url || downloadingId === item.music_id || item.status === "downloaded"}
                >
                  {item.status === "downloaded" ? (
                    <CheckCircle2 className="h-4 w-4 text-green-600" />
                  ) : downloadingId === item.music_id ? (
                    <Loader2 className="h-4 w-4 animate-spin" />
                  ) : (
                    <Download className="h-4 w-4" />
                  )}
                </Button>
              </div>
            ))}

            {/* 分页 */}
            <div className="flex justify-between items-center pt-4">
              <Button
                variant="outline" size="sm"
                disabled={page === 0}
                onClick={() => setPage((p) => p - 1)}
              >
                上一页
              </Button>
              <span className="text-sm text-muted-foreground">
                第 {page + 1} / {totalPages || 1} 页
              </span>
              <Button
                variant="outline" size="sm"
                disabled={page + 1 >= totalPages}
                onClick={() => setPage((p) => p + 1)}
              >
                下一页
              </Button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
