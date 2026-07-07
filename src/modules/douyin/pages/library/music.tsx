import { useState } from "react";
import { toast } from "sonner";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Header } from "@/components/layout/header";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { Bezel } from "@/components/shared/bezel";
import { Pagination } from "@/components/shared/pagination";
import {
  AlertDialog, AlertDialogContent, AlertDialogHeader, AlertDialogFooter,
  AlertDialogTitle, AlertDialogDescription, AlertDialogAction, AlertDialogCancel,
} from "@/components/ui/alert-dialog";
import { Checkbox } from "@/components/ui/checkbox";
import { Music, Loader2, Search, Clock, Download, CheckCircle2, Trash2, FolderOpen } from "lucide-react";
import { useDeleteMusicCollection, useDownloadMusic, useUpdateMusicFilePath } from "@/lib/mutations";
import { getDownloadDirByMusicId, openFolder } from "@/lib/api";
import { useMusicCollectionQuery, useMusicCountQuery } from "@/lib/queries";
import { usePagination } from "@/hooks/use-pagination";
import type { MusicCollectionItem } from "@/lib/api";
import { formatDurationSec } from "@/lib/utils";

export default function LibraryMusicPage() {
  const { page, pageSize, setPage, offset, resetPage } = usePagination();
  const [search, setSearch] = useState("");
  const [downloadingId, setDownloadingId] = useState<string | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<MusicCollectionItem | null>(null);
  const [deleteFile, setDeleteFile] = useState(false);
  const deleteMusic = useDeleteMusicCollection();
  const dlMusic = useDownloadMusic();
  const updatePath = useUpdateMusicFilePath();
  const itemsQuery = useMusicCollectionQuery({
    limit: pageSize,
    offset,
    keyword: search || undefined,
    status: "downloaded",
  });
  const countQuery = useMusicCountQuery({ status: "downloaded" });
  const items = itemsQuery.data ?? [];
  const total = countQuery.data ?? 0;
  const loading = itemsQuery.isLoading || countQuery.isLoading;

  const handleDownload = (item: MusicCollectionItem) => {
    if (!item.play_url) return;
    setDownloadingId(item.music_id);
    dlMusic.mutate(
      { play_url: item.play_url, title: item.title || "", author: item.author || "" },
      {
        onSuccess: (res) => {
          if (res?.success && res.data?.path) {
            updatePath.mutate({ musicId: item.music_id, filePath: res.data.path });
          }
        },
        onSettled: () => setDownloadingId(null),
      },
    );
  };

  const handleConfirmDelete = () => {
    if (!deleteTarget) return;
    deleteMusic.mutate({ musicId: deleteTarget.music_id, deleteFile }, {
      onError: (err) => toast.error(err instanceof Error ? err.message : "删除失败"),
    });
    setDeleteTarget(null);
    setDeleteFile(false);
  };

  const handleOpenFolder = async (item: MusicCollectionItem) => {
    try {
      const dir = await getDownloadDirByMusicId(item.music_id);
      if (dir) {
        await openFolder(dir);
      } else {
        toast.error("未找到下载文件");
      }
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "打开文件夹失败");
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
              onChange={(e) => { setSearch(e.target.value); resetPage(); }}
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
                          {formatDurationSec(item.duration)}
                        </span>
                        {item.status === "downloaded" && (
                          <span className="text-xs text-success flex items-center gap-1">
                            <CheckCircle2 className="h-3 w-3" />
                            已下载
                          </span>
                        )}
                      </div>
                    </div>
                    {item.file_path && (
                      <Button
                        variant="ghost"
                        size="icon-sm"
                        className="shrink-0"
                        title="打开文件所在文件夹"
                        onClick={() => handleOpenFolder(item)}
                      >
                        <FolderOpen className="h-4 w-4" />
                      </Button>
                    )}
                    <Button
                      variant="ghost"
                      size="icon-sm"
                      className="shrink-0"
                      title="删除记录"
                      onClick={() => setDeleteTarget(item)}
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

            <Pagination page={page} totalPages={totalPages} onPageChange={setPage} />
          </div>
        )}
      </div>

      <AlertDialog open={!!deleteTarget} onOpenChange={(open) => !open && setDeleteTarget(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认删除</AlertDialogTitle>
            <AlertDialogDescription>
              确定删除这条音乐记录？此操作不可撤销。
            </AlertDialogDescription>
          </AlertDialogHeader>
          {deleteTarget?.file_path && (
            <label className="flex items-center gap-2 text-sm cursor-pointer">
              <Checkbox checked={deleteFile} onCheckedChange={(checked) => setDeleteFile(checked === true)} />
              同时删除本地文件
            </label>
          )}
          <AlertDialogFooter>
            <AlertDialogCancel onClick={() => setDeleteFile(false)}>取消</AlertDialogCancel>
            <AlertDialogAction variant="destructive" onClick={handleConfirmDelete}>删除</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
