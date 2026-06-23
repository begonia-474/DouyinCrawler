import { useState, useEffect, useCallback } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Header } from "@/components/layout/header";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { Bezel } from "@/components/shared/bezel";
import { Music, Loader2, Search, Clock, Download, CheckCircle2, Trash2 } from "lucide-react";
import { deleteMusicCollection, getMusicCollectionFromDB, getMusicCollectionCountFromDB, downloadMusic } from "@/lib/api";
import type { MusicCollectionItem } from "@/lib/api";

function formatDuration(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
}

export default function LibraryMusicPage() {
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
      loadData();
    }

    setDownloadingId(null);
  };

  const handleDelete = async (item: MusicCollectionItem) => {
    if (!window.confirm("确定删除这条音乐记录？")) return;
    const deleteFile = item.file_path
      ? window.confirm("是否同时删除这条记录对应的本地文件？\n\n取消则只删除记录。")
      : false;
    try {
      await deleteMusicCollection(item.music_id, deleteFile);
      await loadData();
    } catch (err) {
      window.alert(err instanceof Error ? err.message : "删除失败");
    }
  };

  const totalPages = Math.ceil(total / pageSize);

  return (
    <>
      <AnimateEntry>
        <Header title="音乐" description={`${total} 条记录`} parent={{ label: "资料库", path: "/douyin/library" }} />
      </AnimateEntry>

      <div className="space-y-6">
        <AnimateEntry delay={50}>
          <div className="relative">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              value={search}
              onChange={(e) => { setSearch(e.target.value); setPage(0); }}
              placeholder="搜索音乐..."
              className="h-11 rounded-xl pl-10 border-foreground/[0.08] bg-foreground/[0.03]"
            />
          </div>
        </AnimateEntry>

        {loading ? (
          <div className="flex justify-center py-16">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : items.length === 0 ? (
          <AnimateEntry>
            <Bezel radius="xl">
              <div className="p-12 text-center text-muted-foreground">
                <Music className="h-10 w-10 mx-auto mb-4 opacity-30" />
                <p className="text-sm tracking-wide">暂无音乐收藏</p>
                <p className="text-xs mt-1 tracking-wide text-muted-foreground/60">请先在「音乐」页面收藏音乐</p>
              </div>
            </Bezel>
          </AnimateEntry>
        ) : (
          <div className="space-y-2">
            {items.map((item, i) => (
              <AnimateEntry key={item.music_id} delay={i * 30}>
                <Bezel radius="lg" padding="sm">
                  <div className="flex items-center gap-4 p-4 bg-card hover:bg-foreground/[0.02] transition-all duration-300">
                    {item.cover ? (
                      <img
                        src={item.cover}
                        alt={item.title || "音乐封面"}
                        className="h-12 w-12 rounded-lg object-cover shrink-0 ring-1 ring-foreground/[0.06]"
                      />
                    ) : (
                      <div className="h-12 w-12 rounded-lg bg-brand/[0.08] ring-1 ring-brand/15 flex items-center justify-center shrink-0">
                        <Music className="h-5 w-5 text-brand" />
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
                          <span className="text-xs text-success flex items-center gap-1">
                            <CheckCircle2 className="h-3 w-3" />
                            已下载
                          </span>
                        )}
                      </div>
                    </div>
                    <Button
                      variant="ghost"
                      size="icon-sm"
                      className="shrink-0"
                      title="删除记录"
                      onClick={() => handleDelete(item)}
                    >
                      <Trash2 className="h-4 w-4 text-destructive" />
                    </Button>
                    <Button
                      variant="capsule"
                      size="icon-sm"
                      className="shrink-0"
                      onClick={() => handleDownload(item)}
                      disabled={!item.play_url || downloadingId === item.music_id || item.status === "downloaded"}
                    >
                      {item.status === "downloaded" ? (
                        <CheckCircle2 className="h-4 w-4 text-success" />
                      ) : downloadingId === item.music_id ? (
                        <Loader2 className="h-4 w-4 animate-spin" />
                      ) : (
                        <Download className="h-4 w-4" />
                      )}
                    </Button>
                  </div>
                </Bezel>
              </AnimateEntry>
            ))}

            <div className="flex justify-between items-center pt-4">
              <Button
                variant="capsule"
                size="sm"
                disabled={page === 0}
                onClick={() => setPage((p) => p - 1)}
              >
                上一页
              </Button>
              <span className="text-sm text-muted-foreground">
                第 {page + 1} / {totalPages || 1} 页
              </span>
              <Button
                variant="capsule"
                size="sm"
                disabled={page + 1 >= totalPages}
                onClick={() => setPage((p) => p + 1)}
              >
                下一页
              </Button>
            </div>
          </div>
        )}
      </div>
    </>
  );
}
