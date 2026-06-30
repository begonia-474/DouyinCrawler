import { useState } from "react";
import { toast } from "sonner";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Header } from "@/components/layout/header";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { Bezel } from "@/components/shared/bezel";
import { SortDropdown } from "@/components/shared/sort-dropdown";
import { Pagination } from "@/components/shared/pagination";
import {
  Film, Loader2, Search, Clock, Heart, MessageCircle,
  Share2, Bookmark, Trash2,
} from "lucide-react";
import { useDeleteVideoInfo } from "@/lib/mutations";
import { useVideoCountQuery, useVideosQuery } from "@/lib/queries";
import { usePagination } from "@/hooks/use-pagination";
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
  const deleteVideo = useDeleteVideoInfo();

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

  const handleSearch = (value: string) => {
    setSearch(value);
    resetPage();
  };

  const handleDelete = (item: VideoInfo) => {
    if (!window.confirm("确定删除这条视频记录？")) return;
    deleteVideo.mutate(item.aweme_id, {
      onError: (err) => toast.error(err instanceof Error ? err.message : "删除失败"),
    });
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
              onSortByChange={(v) => { setSortBy(v); resetPage(); }}
              onSortOrderChange={(v) => { setSortOrder(v); resetPage(); }}
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
                    <Button variant="ghost" size="icon-sm" title="删除记录" onClick={() => handleDelete(item)}>
                      <Trash2 className="h-4 w-4 text-destructive" />
                    </Button>
                  </div>
                </Bezel>
              </AnimateEntry>
            ))}

            <Pagination page={page} totalPages={Math.ceil(total / pageSize)} onPageChange={setPage} />
          </div>
        )}
      </div>
    </>
  );
}
