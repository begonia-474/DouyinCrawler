import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Video, Image, Music, Loader2, FolderOpen, Search } from "lucide-react";
import { getDownloads } from "@/lib/tauri-api";
import type { DownloadRecord } from "@/lib/tauri-types";
import { formatFileSize, formatTimestamp } from "@/lib/utils";

const typeIcons: Record<string, typeof Video> = {
  video: Video,
  images: Image,
  music: Music,
};

interface DownloadListProps {
  type: "video" | "images" | "music";
  title: string;
}

export function DownloadList({ type, title }: DownloadListProps) {
  const navigate = useNavigate();
  const [items, setItems] = useState<DownloadRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [page, setPage] = useState(0);
  const [search, setSearch] = useState("");
  const pageSize = 20;

  const Icon = typeIcons[type] || Video;

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const data = await getDownloads({
        limit: pageSize,
        offset: page * pageSize,
        download_type: type,
      });
      setItems(data);
    } catch (err) {
      console.error("加载失败:", err);
    }
    setLoading(false);
  }, [type, page]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const filtered = search
    ? items.filter(
        (item) =>
          (item.title || "").toLowerCase().includes(search.toLowerCase()) ||
          (item.author_nickname || "").toLowerCase().includes(search.toLowerCase())
      )
    : items;

  return (
    <div className="flex flex-col h-full">
      {/* 搜索栏 */}
      <div className="p-4 border-b">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="搜索..."
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
        <span className="font-medium">{title}</span>
      </div>

      {/* 内容区 */}
      <div className="flex-1 overflow-auto p-4">
        {loading ? (
          <div className="flex justify-center py-12">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : filtered.length === 0 ? (
          <Card>
            <CardContent className="p-8 text-center text-muted-foreground">
              <Icon className="h-10 w-10 mx-auto mb-3" />
              <p>暂无{title}记录</p>
            </CardContent>
          </Card>
        ) : (
          <div className="space-y-2">
            {filtered.map((item) => (
              <div
                key={item.id}
                className="flex items-center gap-4 p-3 border rounded-lg hover:bg-muted/50 transition-colors"
              >
                <Icon className="h-5 w-5 text-muted-foreground shrink-0" />
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium truncate">
                    {item.title || item.file_path || "未知"}
                  </p>
                  <div className="flex items-center gap-3 mt-1">
                    {item.author_nickname && (
                      <span className="text-xs text-muted-foreground">
                        {item.author_nickname}
                      </span>
                    )}
                    <span className="text-xs text-muted-foreground">
                      {formatTimestamp(item.created_at)}
                    </span>
                    {item.file_size > 0 && (
                      <span className="text-xs text-muted-foreground">
                        {formatFileSize(item.file_size)}
                      </span>
                    )}
                  </div>
                </div>
                {item.file_path && (
                  <Button variant="ghost" size="icon" title="打开文件所在文件夹">
                    <FolderOpen className="h-4 w-4" />
                  </Button>
                )}
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
              <span className="text-sm text-muted-foreground">第 {page + 1} 页</span>
              <Button
                variant="outline" size="sm"
                disabled={items.length < pageSize}
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
