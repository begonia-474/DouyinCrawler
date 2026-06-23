import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Bezel } from "@/components/shared/bezel";
import { Image, Music, Loader2, Search, Clock, Heart, MessageCircle, Share2, Bookmark } from "lucide-react";
import { getVideos, getVideoCount } from "@/lib/api";
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
  const navigate = useNavigate();
  const [items, setItems] = useState<VideoInfo[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);
  const [page, setPage] = useState(0);
  const [search, setSearch] = useState("");
  const pageSize = 20;

  const Icon = typeIcons[postType] || Image;

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [data, count] = await Promise.all([
        getVideos({
          limit: pageSize,
          offset: page * pageSize,
          keyword: search || undefined,
          post_type: postType,
          sort_by: "create_time",
          sort_order: "desc",
        }),
        getVideoCount({
          keyword: search || undefined,
          post_type: postType,
        }),
      ]);
      setItems(data);
      setTotal(count);
    } catch (err) {
      console.error("加载失败:", err);
    }
    setLoading(false);
  }, [postType, page, search]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const totalPages = Math.ceil(total / pageSize);

  return (
    <div className="flex flex-col h-full">
      <div className="pb-6">
        <div className="relative">
          <Search className="absolute left-4 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            value={search}
            onChange={(e) => { setSearch(e.target.value); setPage(0); }}
            placeholder="搜索..."
            className="h-11 rounded-xl pl-10 border-foreground/[0.08] bg-foreground/[0.03]"
          />
        </div>
      </div>

      <div className="pb-4 flex items-center gap-2 text-sm">
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
        <span className="text-muted-foreground/40">/</span>
        <span className="font-medium">{title}</span>
        <span className="text-xs text-muted-foreground ml-auto">{total} 条记录</span>
      </div>

      <div className="flex-1 overflow-auto">
        {loading ? (
          <div className="flex justify-center py-16">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : items.length === 0 ? (
          <Bezel radius="xl">
            <div className="p-12 text-center text-muted-foreground">
              <Icon className="h-10 w-10 mx-auto mb-3 opacity-30" />
              <p className="tracking-wide">暂无{title}记录</p>
            </div>
          </Bezel>
        ) : (
          <div className="space-y-2">
            {items.map((item) => (
              <Bezel key={item.aweme_id} radius="lg" padding="sm">
                <div className="flex items-center gap-4 p-3 bg-card hover:bg-foreground/[0.02] transition-colors">
                  {item.cover_url ? (
                    <img
                      src={item.cover_url}
                      alt=""
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
                </div>
              </Bezel>
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
    </div>
  );
}
