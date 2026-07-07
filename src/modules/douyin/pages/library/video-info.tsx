import { useState } from "react";
import { toast } from "sonner";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Header } from "@/components/layout/header";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { Bezel } from "@/components/shared/bezel";
import { SortDropdown } from "@/components/shared/sort-dropdown";
import { Pagination } from "@/components/shared/pagination";
import { Checkbox } from "@/components/ui/checkbox";
import {
  AlertDialog, AlertDialogContent, AlertDialogHeader, AlertDialogFooter,
  AlertDialogTitle, AlertDialogDescription, AlertDialogAction, AlertDialogCancel,
} from "@/components/ui/alert-dialog";
import {
  Film, Loader2, Search, Clock, Heart, MessageCircle,
  Share2, Bookmark, Trash2, FolderOpen,
} from "lucide-react";
import { useDeleteVideoInfo, useDeleteVideoInfoBatch } from "@/lib/mutations";
import { getDownloadDirByAwemeId, openFolder } from "@/lib/api";
import { useVideoCountQuery, useVideosQuery } from "@/lib/queries";
import { usePagination } from "@/hooks/use-pagination";
import { useSelection } from "@/hooks/use-selection";
import type { VideoInfo } from "@/lib/tauri-types";
import { formatDuration, formatTimestamp, formatCount } from "@/lib/utils";

const SORT_OPTIONS = [
  { value: "updated_at", label: "更新时间" },
  { value: "create_time", label: "创建时间" },
  { value: "digg_count", label: "点赞数" },
  { value: "comment_count", label: "评论数" },
  { value: "share_count", label: "分享数" },
  { value: "collect_count", label: "收藏数" },
];

export default function LibraryVideoInfoPage() {
  const { page, pageSize, setPage, offset, resetPage } = usePagination();
  const [search, setSearch] = useState("");
  const [sortBy, setSortBy] = useState("updated_at");
  const [sortOrder, setSortOrder] = useState<"asc" | "desc">("desc");
  const [deleteTarget, setDeleteTarget] = useState<VideoInfo | null>(null);
  const [batchDeleteOpen, setBatchDeleteOpen] = useState(false);
  const deleteVideo = useDeleteVideoInfo();
  const deleteVideoBatch = useDeleteVideoInfoBatch();
  const selection = useSelection();

  const itemsQuery = useVideosQuery({
    limit: pageSize,
    offset,
    keyword: search || undefined,
    sort_by: sortBy,
    sort_order: sortOrder,
    post_type: "video",
  });
  const countQuery = useVideoCountQuery({ keyword: search || undefined, post_type: "video" });
  const items = itemsQuery.data ?? [];
  const total = countQuery.data ?? 0;
  const loading = itemsQuery.isLoading || countQuery.isLoading;
  const itemIds = items.map((item) => item.aweme_id);

  const handleSearch = (value: string) => {
    setSearch(value);
    resetPage();
    selection.clearSelection();
  };

  const handleConfirmDelete = () => {
    if (!deleteTarget) return;
    deleteVideo.mutate(deleteTarget.aweme_id, {
      onError: (err) => toast.error(err instanceof Error ? err.message : "删除失败"),
    });
    setDeleteTarget(null);
  };

  const handleOpenFolder = async (item: VideoInfo) => {
    try {
      const dir = await getDownloadDirByAwemeId(item.aweme_id);
      if (dir) {
        await openFolder(dir);
      } else {
        toast.error("未找到下载文件");
      }
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "打开文件夹失败");
    }
  };

  const handleBatchDelete = () => {
    const ids = Array.from(selection.selected);
    deleteVideoBatch.mutate(ids, {
      onSuccess: () => {
        toast.success(`已删除 ${ids.length} 条记录`);
        selection.clearSelection();
      },
      onError: (err) => toast.error(err instanceof Error ? err.message : "批量删除失败"),
    });
    setBatchDeleteOpen(false);
  };

  return (
    <>
      <AnimateEntry>
        <Header title="视频库" description={`${total} 条记录`} parent={{ label: "资料库", path: "/douyin/library" }} />
      </AnimateEntry>

      <div className="space-y-6">
        <AnimateEntry delay={50} className="relative z-20">
          <div className="flex gap-2">
            <div className="relative flex-1">
              <Search className="absolute left-4 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                value={search}
                onChange={(e) => handleSearch(e.target.value)}
                placeholder="搜索视频标题或作者..."
                className="h-11 rounded-xl pl-10 border-foreground/[0.08] bg-foreground/[0.03]"
              />
            </div>
            <SortDropdown
              options={SORT_OPTIONS}
              sortBy={sortBy}
              sortOrder={sortOrder}
              onSortByChange={(v) => { setSortBy(v); resetPage(); selection.clearSelection(); }}
              onSortOrderChange={(v) => { setSortOrder(v); resetPage(); selection.clearSelection(); }}
            />
          </div>
        </AnimateEntry>

        {selection.selectedCount > 0 && (
          <AnimateEntry>
            <div className="flex items-center gap-3 p-3 rounded-xl bg-foreground/[0.04] ring-1 ring-foreground/[0.08]">
              <Checkbox
                checked={selection.isAllSelected(itemIds)}
                indeterminate={selection.isIndeterminate(itemIds)}
                onCheckedChange={(checked) => {
                  if (checked) selection.selectAll(itemIds);
                  else selection.clearSelection();
                }}
              />
              <span className="text-sm text-muted-foreground">
                已选 {selection.selectedCount} 项
              </span>
              <Button variant="destructive" size="sm" onClick={() => setBatchDeleteOpen(true)}>
                <Trash2 className="h-3.5 w-3.5 mr-1" />
                删除选中
              </Button>
            </div>
          </AnimateEntry>
        )}

        {loading ? (
          <div className="flex justify-center py-16">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : items.length === 0 ? (
          <AnimateEntry>
            <Bezel radius="xl">
              <div className="p-12 text-center text-muted-foreground">
                <Film className="h-10 w-10 mx-auto mb-4 opacity-30" />
                <p className="text-sm tracking-wide">暂无视频记录</p>
              </div>
            </Bezel>
          </AnimateEntry>
        ) : (
          <div className="space-y-2">
            {items.map((item, i) => (
              <AnimateEntry key={item.aweme_id} delay={i * 30}>
                <Bezel radius="lg" padding="sm">
                  <div className="flex items-center gap-4 p-4 bg-card hover:bg-foreground/[0.02] transition-all duration-300">
                    <Checkbox
                      checked={selection.isSelected(item.aweme_id)}
                      onCheckedChange={() => selection.toggle(item.aweme_id)}
                    />
                    {item.cover_url ? (
                      <img
                        src={item.cover_url}
                        alt={item.desc || "视频封面"}
                        className="h-16 w-28 object-cover rounded-lg shrink-0 ring-1 ring-foreground/[0.06]"
                      />
                    ) : (
                      <div className="h-16 w-28 rounded-lg bg-foreground/[0.04] ring-1 ring-foreground/[0.06] flex items-center justify-center shrink-0">
                        <Film className="h-6 w-6 text-muted-foreground" />
                      </div>
                    )}

                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium truncate">
                        {item.desc || "无标题"}
                      </p>
                      <div className="flex items-center gap-3 mt-1 flex-wrap">
                        {item.author_nickname && (
                          <span className="text-xs text-muted-foreground">
                            {item.author_nickname}
                          </span>
                        )}
                        {item.duration > 0 && (
                          <span className="text-xs text-muted-foreground flex items-center gap-1">
                            <Clock className="h-3 w-3" />
                            {formatDuration(Math.floor(item.duration / 1000))}
                          </span>
                        )}
                        {item.create_time && (
                          <span className="text-xs text-muted-foreground">
                            {formatTimestamp(item.create_time)}
                          </span>
                        )}
                        {item.mix_name && (
                          <span className="text-xs text-brand">
                            合集: {item.mix_name}
                          </span>
                        )}
                      </div>
                      <div className="flex items-center gap-4 mt-1">
                        <span className="text-xs text-muted-foreground flex items-center gap-1">
                          <Heart className="h-3 w-3" /> {formatCount(item.digg_count)}
                        </span>
                        <span className="text-xs text-muted-foreground flex items-center gap-1">
                          <MessageCircle className="h-3 w-3" /> {formatCount(item.comment_count)}
                        </span>
                        <span className="text-xs text-muted-foreground flex items-center gap-1">
                          <Share2 className="h-3 w-3" /> {formatCount(item.share_count)}
                        </span>
                        <span className="text-xs text-muted-foreground flex items-center gap-1">
                          <Bookmark className="h-3 w-3" /> {formatCount(item.collect_count)}
                        </span>
                      </div>
                    </div>
                    <Button variant="ghost" size="icon-sm" title="打开文件所在文件夹" onClick={() => handleOpenFolder(item)}>
                      <FolderOpen className="h-4 w-4" />
                    </Button>
                    <Button variant="ghost" size="icon-sm" title="删除记录" onClick={() => setDeleteTarget(item)}>
                      <Trash2 className="h-4 w-4 text-destructive" />
                    </Button>
                  </div>
                </Bezel>
              </AnimateEntry>
            ))}

            <Pagination page={page} totalPages={Math.ceil(total / pageSize)} onPageChange={(p) => { setPage(p); selection.clearSelection(); }} />
          </div>
        )}
      </div>

      {/* 单条删除确认 */}
      <AlertDialog open={!!deleteTarget} onOpenChange={(open) => !open && setDeleteTarget(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认删除</AlertDialogTitle>
            <AlertDialogDescription>
              确定删除这条视频记录？此操作不可撤销。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction variant="destructive" onClick={handleConfirmDelete}>删除</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* 批量删除确认 */}
      <AlertDialog open={batchDeleteOpen} onOpenChange={setBatchDeleteOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认批量删除</AlertDialogTitle>
            <AlertDialogDescription>
              确定删除选中的 {selection.selectedCount} 条视频记录？此操作不可撤销。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction variant="destructive" onClick={handleBatchDelete}>删除</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
