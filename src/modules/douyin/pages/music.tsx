import { useState, useCallback, useEffect } from "react";
import { Header } from "@/components/layout/header";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { Bezel } from "@/components/shared/bezel";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { getMusicCollection, downloadMusic, saveMusicCollectionBatch } from "@/lib/api";
import type { MusicItem } from "@/lib/api-types";
import { Loader2, AlertCircle, Download, Music, CheckCircle2 } from "lucide-react";

function formatDuration(ms: number): string {
  const seconds = Math.floor(ms / 1000);
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
}

export default function MusicPage() {
  const [loading, setLoading] = useState(false);
  const [musicList, setMusicList] = useState<MusicItem[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [downloadingId, setDownloadingId] = useState<string | null>(null);
  const [downloadedIds, setDownloadedIds] = useState<Set<string>>(new Set());

  const fetchMusic = useCallback(async () => {
    setLoading(true);
    setError(null);

    const res = await getMusicCollection();
    if (res.success && res.data?.music_list) {
      setMusicList(res.data.music_list);
      try {
        await saveMusicCollectionBatch(
          res.data.music_list.map((item) => ({
            music_id: item.music_id,
            mid: item.mid,
            title: item.title,
            author: item.author,
            owner_nickname: item.owner_nickname,
            duration: Math.floor(item.duration / 1000),
            cover: item.cover,
            play_url: item.play_url,
          }))
        );
      } catch (e) {
        console.error("保存音乐收藏到数据库失败:", e);
      }
    } else {
      setError(res.error || "获取音乐收藏失败");
    }

    setLoading(false);
  }, []);

  useEffect(() => {
    fetchMusic();
  }, [fetchMusic]);

  const handleDownload = async (item: MusicItem) => {
    if (!item.play_url) return;
    setDownloadingId(item.music_id);

    const res = await downloadMusic(item.play_url, item.title, item.author);
    if (res.success) {
      setDownloadedIds((prev) => new Set(prev).add(item.music_id));
    }

    setDownloadingId(null);
  };

  const handleDownloadAll = async () => {
    for (const item of musicList) {
      if (item.play_url && !downloadedIds.has(item.music_id)) {
        await handleDownload(item);
      }
    }
  };

  return (
    <>
      <AnimateEntry>
        <Header title="我的音乐" description="当前账号的音乐收藏">
          {musicList.length > 0 && (
            <Button variant="capsule" size="sm" onClick={handleDownloadAll}>
              <Download className="h-4 w-4 mr-1" />
              全部下载
            </Button>
          )}
        </Header>
      </AnimateEntry>

      <div className="space-y-6">
        {error && (
          <div className="flex items-center gap-2 p-4 rounded-2xl bg-destructive/[0.06] ring-1 ring-destructive/20 text-destructive text-sm">
            <AlertCircle className="h-4 w-4 shrink-0" />
            <span>{error}</span>
          </div>
        )}

        {loading && (
          <div className="flex items-center justify-center py-16">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        )}

        {!loading && musicList.length === 0 && !error && (
          <Bezel radius="xl">
            <div className="p-12 text-center">
              <Music className="h-12 w-12 text-muted-foreground/30 mx-auto mb-4" />
              <h3 className="text-lg font-semibold mb-2">暂无音乐收藏</h3>
              <p className="text-muted-foreground tracking-wide">
                请先在设置中配置 Cookie
              </p>
            </div>
          </Bezel>
        )}

        {!loading && musicList.length > 0 && (
          <div className="space-y-2">
            {musicList.map((item, i) => (
              <AnimateEntry key={item.music_id} delay={i * 25}>
                <Bezel radius="lg" padding="sm">
                  <div className="flex items-center gap-4 p-4 bg-card hover:bg-foreground/[0.02] transition-all duration-300">
                    {item.cover ? (
                      <img src={item.cover} alt={item.title} className="h-12 w-12 rounded-xl object-cover ring-1 ring-foreground/[0.06]" />
                    ) : (
                      <div className="h-12 w-12 rounded-xl bg-primary/10 ring-1 ring-primary/15 flex items-center justify-center">
                        <Music className="h-5 w-5 text-primary" />
                      </div>
                    )}
                    <div className="flex-1 min-w-0">
                      <h4 className="text-sm font-medium truncate">{item.title}</h4>
                      <p className="text-xs text-muted-foreground tracking-wide truncate">
                        {item.author || item.owner_nickname}
                      </p>
                    </div>
                    <Badge variant="secondary" className="text-xs rounded-full font-mono tabular-nums">
                      {formatDuration(item.duration)}
                    </Badge>
                    <Button
                      variant="capsule"
                      size="icon"
                      className="h-8 w-8 shrink-0"
                      onClick={() => handleDownload(item)}
                      disabled={downloadingId === item.music_id || downloadedIds.has(item.music_id)}
                    >
                      {downloadedIds.has(item.music_id) ? (
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
          </div>
        )}
      </div>
    </>
  );
}
