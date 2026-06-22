import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import {
  Film, Loader2, Search, Clock, Heart, MessageCircle,
  Share2, Bookmark, ChevronDown,
} from "lucide-react";
import { getVideos, getVideoCount } from "@/lib/api";
import type { VideoInfo } from "@/lib/tauri-types";
import { formatDuration, formatTimestamp } from "@/lib/utils";

const SORT_OPTIONS = [
  { value: "updated_at", label: "更新时间" },
  { value: "create_time", label: "创建时间" },
  { value: "digg_count", label: "点赞数" },
  { value: "comment_count", label: "评论数" },
  { value: "share_count", label: "分享数" },
  { value: "collect_count", label: "收藏数" },
];

function formatCount(n: number): string {
  if (n >= 10000) return (n / 10000).toFixed(1) + "w";
  return String(n);
}

export default function LibraryVideoInfoPage() {
  const navigate = useNavigate();
  const [items, setItems] = useState<VideoInfo[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);
  const [page, setPage] = useState(0);
  const [search, setSearch] = useState("");
  const [sortBy, setSortBy] = useState("updated_at");
  const [sortOrder, setSortOrder] = useState<"asc" | "desc">("desc");
  const [showSort, setShowSort] = useState(false);
  const pageSize = 20;

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [data, count] = await Promise.all([
        getVideos({
          limit: pageSize,
          offset: page * pageSize,
          keyword: search || undefined,
          sort_by: sortBy,
          sort_order: sortOrder,
          post_type: "video",
        }),
        getVideoCount({ keyword: search || undefined, post_type: "video" }),
      ]);
      setItems(data);
      setTotal(count);
    } catch (err) {
      console.error("加载失败:", err);
    }
    setLoading(false);
  }, [page, search, sortBy, sortOrder]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const handleSearch = (value: string) => {
    setSearch(value);
    setPage(0);
  };

  return (
    <div className="flex flex-col h-full">
      {/* 搜索栏 + 排序 */}
      <div className="p-4 border-b flex gap-2">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            value={search}
            onChange={(e) => handleSearch(e.target.value)}
            placeholder="搜索视频标题或作者..."
            className="pl-9"
          />
        </div>
        <div className="relative">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setShowSort(!showSort)}
            className="gap-1"
          >
            {SORT_OPTIONS.find((o) => o.value === sortBy)?.label}
            <ChevronDown className="h-3 w-3" />
          </Button>
          {showSort && (
            <div className="absolute right-0 top-full mt-1 z-10 bg-popover border rounded-md shadow-md py-1 min-w-[140px]">
              {SORT_OPTIONS.map((opt) => (
                <button
                  key={opt.value}
                  className={`w-full text-left px-3 py-1.5 text-sm hover:bg-accent transition-colors ${
                    sortBy === opt.value ? "font-medium" : ""
                  }`}
                  onClick={() => {
                    setSortBy(opt.value);
                    setShowSort(false);
                    setPage(0);
                  }}
                >
                  {opt.label}
                </button>
              ))}
              <div className="border-t mt-1 pt-1">
                <button
                  className="w-full text-left px-3 py-1.5 text-sm hover:bg-accent"
                  onClick={() => {
                    setSortOrder(sortOrder === "desc" ? "asc" : "desc");
                    setShowSort(false);
                    setPage(0);
                  }}
                >
                  {sortOrder === "desc" ? "降序 ↓" : "升序 ↑"}
                </button>
              </div>
            </div>
          )}
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
        <span className="font-medium">视频库</span>
        <span className="text-muted-foreground ml-auto text-xs">{total} 条记录</span>
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
              <Film className="h-10 w-10 mx-auto mb-3" />
              <p>暂无视频记录</p>
            </CardContent>
          </Card>
        ) : (
          <div className="space-y-2">
            {items.map((item) => (
              <div
                key={item.aweme_id}
                className="flex items-center gap-4 p-3 border rounded-lg hover:bg-muted/50 transition-colors"
              >
                {/* 封面 */}
                {item.cover_url ? (
                  <img
                    src={item.cover_url}
                    alt=""
                    className="h-16 w-28 object-cover rounded shrink-0 bg-muted"
                  />
                ) : (
                  <div className="h-16 w-28 rounded bg-muted flex items-center justify-center shrink-0">
                    <Film className="h-6 w-6 text-muted-foreground" />
                  </div>
                )}

                {/* 信息 */}
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
                      <span className="text-xs text-blue-500">
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
              </div>
            ))}

            {/* 分页 */}
            <div className="flex justify-between items-center pt-4">
              <Button
                variant="outline"
                size="sm"
                disabled={page === 0}
                onClick={() => setPage((p) => p - 1)}
              >
                上一页
              </Button>
              <span className="text-sm text-muted-foreground">
                第 {page + 1} 页 / 共 {Math.ceil(total / pageSize)} 页
              </span>
              <Button
                variant="outline"
                size="sm"
                disabled={(page + 1) * pageSize >= total}
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
