import { useState } from "react";
import { toast } from "sonner";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Header } from "@/components/layout/header";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { Bezel } from "@/components/shared/bezel";
import {
  AlertDialog, AlertDialogContent, AlertDialogHeader, AlertDialogFooter,
  AlertDialogTitle, AlertDialogDescription, AlertDialogAction, AlertDialogCancel,
} from "@/components/ui/alert-dialog";
import { Image, Music, Loader2, Search, Clock, Heart, MessageCircle, Share2, Bookmark, Trash2 } from "lucide-react";
import { useDeleteVideoInfo } from "@/lib/mutations";
import { useVideosQuery, useVideoCountQuery } from "@/lib/queries";
import type { VideoInfo } from "@/lib/tauri-types";
import { formatTimestamp } from "@/lib/utils";

const typeIcons: Record<string, typeof Image> = {
  images: Image,
  music: Music,
};

interface VideoListProps {
  postType: "images" | "music";
  title: string;
}

export function VideoList({ postType, title }: VideoListProps) {
  const [page, setPage] = useState(0);
  const [search, setSearch] = useState("");
  const [deleteTarget, setDeleteTarget] = useState<VideoInfo | null>(null);
  const pageSize = 20;
  const deleteVideo = useDeleteVideoInfo();

  const Icon = typeIcons[postType] || Image;

  const handleConfirmDelete = () => {
    if (!deleteTarget) return;
    deleteVideo.mutate(deleteTarget.aweme_id, {
      onError: (err) => toast.error(err instanceof Error ? err.message : "删除失败"),
    });
    setDeleteTarget(null);
  };

  const itemsQuery = useVideosQuery({
    limit: pageSize,
    offset: page * pageSize,
    keyword: search || undefined,
    post_type: postType,
    sort_by: "create_time",
    sort_order: "desc",
  });
  const countQuery = useVideoCountQuery({
    keyword: search || undefined,
    post_type: postType,
  });
  const items: VideoInfo[] = itemsQuery.data ?? [];
  const total = countQuery.data ?? 0;
  const loading = itemsQuery.isLoading || countQuery.isLoading;

  const totalPages = Math.ceil(total / pageSize);

  return (
    <>
      <AnimateEntry>
        <Header title={title} description={`${total} 条记录`} parent={{ label: "资料库", path: "/douyin/library" }} />
      </AnimateEntry>

      <div className="space-y-6">
        <AnimateEntry delay={50}>
          <div className="relative">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              value={search}
              onChange={(e) => { setSearch(e.target.value); setPage(0); }}
              placeholder="搜索..."
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
                <Icon className="h-10 w-10 mx-auto mb-4 opacity-30" />
                <p className="text-sm tracking-wide">暂无{title}记录</p>
              </div>
            </Bezel>
          </AnimateEntry>
        ) : (
          <div className="space-y-2">
            {items.map((item, i) => (
              <AnimateEntry key={item.aweme_id} delay={i * 30}>
                <Bezel radius="lg" padding="sm">
                  <div className="flex items-center gap-4 p-4 bg-card hover:bg-foreground/[0.02] transition-all duration-300">
                    {item.cover_url ? (
                      <img
                        src={item.cover_url}
                        alt={item.desc || "内容封面"}
                        className="h-12 w-20 object-cover rounded-lg shrink-0 ring-1 ring-foreground/[0.06]"
                      />
                    ) : (
                      <div className="h-12 w-20 bg-foreground/[0.04] rounded-lg ring-1 ring-foreground/[0.06] flex items-center justify-center shrink-0">
                        <Icon className="h-5 w-5 text-muted-foreground" />
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
                        {item.create_time && (
                          <span className="text-xs text-muted-foreground flex items-center gap-1">
                            <Clock className="h-3 w-3" />
                            {formatTimestamp(item.create_time)}
                          </span>
                        )}
                        <span className="text-xs text-muted-foreground flex items-center gap-1">
                          <Heart className="h-3 w-3" />
                          {item.digg_count.toLocaleString()}
                        </span>
                        <span className="text-xs text-muted-foreground flex items-center gap-1">
                          <MessageCircle className="h-3 w-3" />
                          {item.comment_count.toLocaleString()}
                        </span>
                        <span className="text-xs text-muted-foreground flex items-center gap-1">
                          <Share2 className="h-3 w-3" />
                          {item.share_count.toLocaleString()}
                        </span>
                        <span className="text-xs text-muted-foreground flex items-center gap-1">
                          <Bookmark className="h-3 w-3" />
                          {item.collect_count.toLocaleString()}
                        </span>
                        {item.mix_name && (
                          <span className="text-xs text-brand">
                            合集: {item.mix_name}
                          </span>
                        )}
                      </div>
                    </div>
                    <Button variant="ghost" size="icon-sm" title="删除记录" onClick={() => setDeleteTarget(item)}>
                      <Trash2 className="h-4 w-4 text-destructive" />
                    </Button>
                  </div>
                </Bezel>
              </AnimateEntry>
            ))}

            <div className="flex justify-between items-center pt-4">
              <Button variant="capsule" size="sm" disabled={page === 0} onClick={() => setPage((p) => p - 1)}>
                上一页
              </Button>
              <span className="text-sm text-muted-foreground">
                第 {page + 1} / {totalPages || 1} 页
              </span>
              <Button variant="capsule" size="sm" disabled={page + 1 >= totalPages} onClick={() => setPage((p) => p + 1)}>
                下一页
              </Button>
            </div>
          </div>
        )}
      </div>

      <AlertDialog open={!!deleteTarget} onOpenChange={(open) => !open && setDeleteTarget(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认删除</AlertDialogTitle>
            <AlertDialogDescription>
              确定删除这条记录？此操作不可撤销。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction variant="destructive" onClick={handleConfirmDelete}>删除</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
