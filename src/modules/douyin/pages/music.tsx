import { useState, useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { Header } from "@/components/layout/header";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { Bezel } from "@/components/shared/bezel";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { getMusicCollection, downloadMusic, saveMusicCollectionBatch, updateMusicFilePath } from "@/lib/api";
import { useMusicCollectionQuery } from "@/lib/queries";
import { queryKeys } from "@/lib/query-keys";
import type { MusicItem } from "@/lib/api-types";
import { Loader2, Download, Music, CheckCircle2, RefreshCw } from "lucide-react";
import { ErrorBanner } from "@/components/shared/error-banner";
import { useAsyncAction } from "@/hooks/use-async-action";
import { formatDurationMs } from "@/lib/utils";

export default function MusicPage() {
  const queryClient = useQueryClient();
  // 从 DB 读取已缓存的音乐列表
  const { data: dbMusicList, isLoading: dbLoading } = useMusicCollectionQuery({});
  const [apiMusicList, setApiMusicList] = useState<MusicItem[] | null>(null);
  const [downloadingIds, setDownloadingIds] = useState<Set<string>>(new Set());
  const [downloadedIds, setDownloadedIds] = useState<Set<string>>(new Set());

  // API 数据优先，DB 缓存兜底（DB 字段可空，需补默认值）
  const dbAsMusicItems: MusicItem[] | undefined = dbMusicList?.map((item) => ({
    music_id: item.music_id,
    mid: item.mid ?? "",
    title: item.title ?? "",
    author: item.author ?? "",
    owner_nickname: item.owner_nickname ?? "",
    duration: item.duration,
    cover: item.cover ?? "",
    play_url: item.play_url ?? "",
  }));
  const musicList: MusicItem[] = apiMusicList ?? dbAsMusicItems ?? [];

  const { run: fetchFromApi, loading: fetching, error } = useAsyncAction({
    action: useCallback(async () => {
      const res = await getMusicCollection();
      if (res.success && res.data?.music_list) {
        setApiMusicList(res.data.music_list);
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
        queryClient.invalidateQueries({ queryKey: queryKeys.musicCollection({}) });
      } else {
        throw new Error(res.error || "获取音乐收藏失败");
      }
    }, [queryClient]),
  });

  const CONCURRENT_LIMIT = 3;

  const handleDownload = async (item: MusicItem) => {
    if (!item.play_url) return;
    setDownloadingIds((prev) => new Set(prev).add(item.music_id));

    const res = await downloadMusic(item.play_url, item.title, item.author);
    if (res.success) {
      setDownloadedIds((prev) => new Set(prev).add(item.music_id));
      // 更新 music_collection 的 file_path 和 status
      if (res.data?.path) {
        try {
          await updateMusicFilePath(item.music_id, res.data.path);
          queryClient.invalidateQueries({ queryKey: queryKeys.musicCollection({}) });
        } catch (e) {
          console.error("更新音乐文件路径失败:", e);
        }
      }
    }

    setDownloadingIds((prev) => {
      const next = new Set(prev);
      next.delete(item.music_id);
      return next;
    });
  };

  const handleDownloadAll = async () => {
    const pending = musicList.filter(
      (item) => item.play_url && !downloadedIds.has(item.music_id)
    );
    // 按 CONCURRENT_LIMIT 分批并发
    for (let i = 0; i < pending.length; i += CONCURRENT_LIMIT) {
      const batch = pending.slice(i, i + CONCURRENT_LIMIT);
      await Promise.all(batch.map((item) => handleDownload(item)));
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
          <Button variant="capsule" size="sm" onClick={fetchFromApi} disabled={fetching}>
            {fetching ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <RefreshCw className="h-4 w-4 mr-1" />}
            刷新
          </Button>
        </Header>
      </AnimateEntry>

      <div className="space-y-6">
        <ErrorBanner message={error} />

        {(dbLoading || fetching) && (
          <div className="flex items-center justify-center py-16">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        )}

        {!dbLoading && !fetching && musicList.length === 0 && !error && (
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

        {!dbLoading && musicList.length > 0 && (
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
                      {formatDurationMs(item.duration)}
                    </Badge>
                    <Button
                      variant="capsule"
                      size="icon"
                      className="h-8 w-8 shrink-0"
                      onClick={() => handleDownload(item)}
                      disabled={downloadingIds.has(item.music_id) || downloadedIds.has(item.music_id)}
                    >
                      {downloadedIds.has(item.music_id) ? (
                        <CheckCircle2 className="h-4 w-4 text-success" />
                      ) : downloadingIds.has(item.music_id) ? (
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
