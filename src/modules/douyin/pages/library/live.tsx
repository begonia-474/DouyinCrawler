import { useState } from "react";
import { toast } from "sonner";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Header } from "@/components/layout/header";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { Bezel } from "@/components/shared/bezel";
import { Pagination } from "@/components/shared/pagination";
import { Radio, Loader2, FolderOpen, Clock, Search, Trash2 } from "lucide-react";
import { useDeleteLiveRecord } from "@/lib/mutations";
import { useLiveRecordsQuery } from "@/lib/queries";
import { usePagination } from "@/hooks/use-pagination";
import type { LiveRecord } from "@/lib/tauri-types";
import { formatFileSize, formatTimestamp, formatDuration } from "@/lib/utils";

export default function LibraryLivePage() {
  const { page, pageSize, setPage, offset } = usePagination();
  const [search, setSearch] = useState("");
  const deleteLive = useDeleteLiveRecord();
  const liveRecordsQuery = useLiveRecordsQuery({ limit: pageSize, offset });
  const items = liveRecordsQuery.data ?? [];
  const loading = liveRecordsQuery.isLoading;

  const filtered = search
    ? items.filter(
        (item) =>
          (item.title || "").toLowerCase().includes(search.toLowerCase()) ||
          (item.nickname || "").toLowerCase().includes(search.toLowerCase())
      )
    : items;

  const handleDelete = (item: LiveRecord) => {
    if (!window.confirm("确定删除这条直播录制记录？")) return;
    const deleteFile = item.file_path
      ? window.confirm("是否同时删除这条记录对应的本地文件？\n\n取消则只删除记录。")
      : false;
    deleteLive.mutate({ id: item.id, deleteFile }, {
      onError: (err) => toast.error(err instanceof Error ? err.message : "删除失败"),
    });
  };

  return (
    <>
      <AnimateEntry>
        <Header title="直播" description={`${items.length} 条记录`} parent={{ label: "资料库", path: "/douyin/library" }} />
      </AnimateEntry>

      <div className="space-y-6">
        <AnimateEntry delay={50}>
          <div className="relative">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="搜索直播标题或主播..."
              className="h-11 rounded-xl pl-10 border-foreground/[0.08] bg-foreground/[0.03]"
            />
          </div>
        </AnimateEntry>

        {loading ? (
          <div className="flex justify-center py-16">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : filtered.length === 0 ? (
          <AnimateEntry>
            <Bezel radius="xl">
              <div className="p-12 text-center text-muted-foreground">
                <Radio className="h-10 w-10 mx-auto mb-4 opacity-30" />
                <p className="text-sm tracking-wide">暂无直播录制记录</p>
              </div>
            </Bezel>
          </AnimateEntry>
        ) : (
          <div className="space-y-2">
            {filtered.map((item, i) => (
              <AnimateEntry key={item.id} delay={i * 30}>
                <Bezel radius="lg" padding="sm">
                  <div className="flex items-center gap-4 p-4 bg-card hover:bg-foreground/[0.02] transition-all duration-300">
                    {item.cover_url ? (
                      <img
                        src={item.cover_url}
                        alt={item.title || "直播封面"}
                        className="h-12 w-20 object-cover rounded-lg shrink-0 ring-1 ring-foreground/[0.06]"
                        onError={(e) => {
                          (e.target as HTMLImageElement).style.display = "none";
                        }}
                      />
                    ) : (
                      <div className="h-12 w-20 rounded-lg bg-destructive/[0.06] ring-1 ring-destructive/10 flex items-center justify-center shrink-0">
                        <Radio className="h-5 w-5 text-destructive/70" />
                      </div>
                    )}
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium truncate">{item.title || "直播录制"}</p>
                      <div className="flex items-center gap-3 mt-1">
                        {item.nickname && (
                          <span className="text-xs text-muted-foreground">{item.nickname}</span>
                        )}
                        {item.started_at && (
                          <span className="text-xs text-muted-foreground flex items-center gap-1">
                            <Clock className="h-3 w-3" />
                            {formatTimestamp(item.started_at)}
                          </span>
                        )}
                        {item.duration_sec > 0 && (
                          <span className="text-xs text-muted-foreground">
                            {formatDuration(item.duration_sec)}
                          </span>
                        )}
                        {item.file_size > 0 && (
                          <span className="text-xs text-muted-foreground font-mono tabular-nums">
                            {formatFileSize(item.file_size)}
                          </span>
                        )}
                      </div>
                    </div>
                    <div className="flex items-center gap-2 shrink-0">
                      <Badge variant={item.status === "completed" ? "default" : "destructive"} className="text-[11px] rounded-full">
                        {item.status === "completed" ? "已完成" : item.status}
                      </Badge>
                      {item.file_path && (
                        <Button variant="ghost" size="icon-sm" title="打开文件所在文件夹">
                          <FolderOpen className="h-4 w-4" />
                        </Button>
                      )}
                      <Button variant="ghost" size="icon-sm" title="删除记录" onClick={() => handleDelete(item)}>
                        <Trash2 className="h-4 w-4 text-destructive" />
                      </Button>
                    </div>
                  </div>
                </Bezel>
              </AnimateEntry>
            ))}

            <Pagination page={page} totalPages={items.length < pageSize ? page + 1 : page + 2} onPageChange={setPage} />
          </div>
        )}
      </div>
    </>
  );
}
