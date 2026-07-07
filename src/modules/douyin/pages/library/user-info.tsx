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
  AlertDialog, AlertDialogContent, AlertDialogHeader, AlertDialogFooter,
  AlertDialogTitle, AlertDialogDescription, AlertDialogAction, AlertDialogCancel,
} from "@/components/ui/alert-dialog";
import { Checkbox } from "@/components/ui/checkbox";
import { Users, Loader2, Search, Trash2, FolderOpen } from "lucide-react";
import { useDeleteUserInfo } from "@/lib/mutations";
import { useUsersQuery, useUserCountQuery } from "@/lib/queries";
import { getUserDownloadDir, openFolder } from "@/lib/api";
import { usePagination } from "@/hooks/use-pagination";
import type { UserInfo } from "@/lib/tauri-types";
import { formatCount } from "@/lib/utils";

const SORT_OPTIONS = [
  { value: "updated_at", label: "更新时间" },
  { value: "follower_count", label: "粉丝数" },
  { value: "aweme_count", label: "作品数" },
  { value: "following_count", label: "关注数" },
  { value: "total_favorited", label: "获赞数" },
];

export default function LibraryUserInfoPage() {
  const { page, pageSize, setPage, offset, resetPage } = usePagination();
  const [search, setSearch] = useState("");
  const [sortBy, setSortBy] = useState("updated_at");
  const [sortOrder, setSortOrder] = useState<"asc" | "desc">("desc");
  const [deleteTarget, setDeleteTarget] = useState<UserInfo | null>(null);
  const [deleteFile, setDeleteFile] = useState(false);
  const deleteUser = useDeleteUserInfo();

  const itemsQuery = useUsersQuery({
    limit: pageSize,
    offset,
    keyword: search || undefined,
    sort_by: sortBy,
    sort_order: sortOrder,
  });
  const countQuery = useUserCountQuery({ keyword: search || undefined });
  const items = itemsQuery.data ?? [];
  const total = countQuery.data ?? 0;
  const loading = itemsQuery.isLoading || countQuery.isLoading;

  const handleSearch = (value: string) => {
    setSearch(value);
    resetPage();
  };

  const handleConfirmDelete = () => {
    if (!deleteTarget) return;
    deleteUser.mutate({ secUserId: deleteTarget.sec_user_id, deleteFile }, {
      onError: (err) => toast.error(err instanceof Error ? err.message : "删除失败"),
    });
    setDeleteTarget(null);
    setDeleteFile(false);
  };

  const handleOpenFolder = async (item: UserInfo) => {
    try {
      const dir = await getUserDownloadDir(item.sec_user_id);
      if (dir) {
        await openFolder(dir);
      } else {
        toast.error("未找到下载文件");
      }
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "打开文件夹失败");
    }
  };

  return (
    <>
      <AnimateEntry>
        <Header title="用户库" description={`${total} 条记录`} parent={{ label: "资料库", path: "/douyin/library" }} />
      </AnimateEntry>

      <div className="space-y-6">
        <AnimateEntry delay={50} className="relative z-20">
          <div className="flex gap-2">
            <div className="relative flex-1">
              <Search className="absolute left-4 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                value={search}
                onChange={(e) => handleSearch(e.target.value)}
                placeholder="搜索昵称、抖音号或签名..."
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
                <Users className="h-10 w-10 mx-auto mb-4 opacity-30" />
                <p className="text-sm tracking-wide">暂无用户记录</p>
              </div>
            </Bezel>
          </AnimateEntry>
        ) : (
          <div className="space-y-2">
            {items.map((item, i) => (
              <AnimateEntry key={item.sec_user_id} delay={i * 30}>
                <Bezel radius="lg" padding="sm">
                  <div className="flex items-center gap-4 p-4 bg-card hover:bg-foreground/[0.02] transition-all duration-300">
                    {item.avatar_url ? (
                      <img
                        src={item.avatar_url}
                        alt={item.nickname || "用户头像"}
                        className="h-12 w-12 rounded-full object-cover shrink-0 ring-2 ring-foreground/[0.06]"
                      />
                    ) : (
                      <div className="h-12 w-12 rounded-full bg-foreground/[0.04] ring-1 ring-foreground/[0.06] flex items-center justify-center shrink-0">
                        <Users className="h-5 w-5 text-muted-foreground" />
                      </div>
                    )}

                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <p className="text-sm font-medium truncate">
                          {item.nickname || "未知用户"}
                        </p>
                        {item.unique_id && (
                          <span className="text-xs text-muted-foreground">
                            @{item.unique_id}
                          </span>
                        )}
                        {item.custom_verify && (
                          <span className="text-xs text-brand bg-brand-muted px-1.5 py-0.5 rounded">
                            {item.custom_verify}
                          </span>
                        )}
                      </div>
                      {item.signature && (
                        <p className="text-xs text-muted-foreground truncate mt-0.5">
                          {item.signature}
                        </p>
                      )}
                      <div className="flex items-center gap-4 mt-1">
                        <span className="text-xs text-muted-foreground">
                          粉丝 <span className="font-medium text-foreground">{formatCount(item.follower_count)}</span>
                        </span>
                        <span className="text-xs text-muted-foreground">
                          关注 <span className="font-medium text-foreground">{formatCount(item.following_count)}</span>
                        </span>
                        <span className="text-xs text-muted-foreground">
                          作品 <span className="font-medium text-foreground">{formatCount(item.aweme_count)}</span>
                        </span>
                        <span className="text-xs text-muted-foreground">
                          获赞 <span className="font-medium text-foreground">{formatCount(item.total_favorited)}</span>
                        </span>
                        {item.ip_location && (
                          <span className="text-xs text-muted-foreground">
                            IP: {item.ip_location}
                          </span>
                        )}
                      </div>
                    </div>
                    <Button variant="ghost" size="icon-sm" title="打开用户下载目录" onClick={() => handleOpenFolder(item)}>
                      <FolderOpen className="h-4 w-4" />
                    </Button>
                    <Button variant="ghost" size="icon-sm" title="删除记录" onClick={() => setDeleteTarget(item)}>
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

      <AlertDialog open={!!deleteTarget} onOpenChange={(open) => !open && setDeleteTarget(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认删除</AlertDialogTitle>
            <AlertDialogDescription>
              确定删除这条用户记录？将同时清理该用户的视频/图片元数据和直播录制记录。此操作不可撤销。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <label className="flex items-center gap-2 text-sm cursor-pointer">
            <Checkbox checked={deleteFile} onCheckedChange={(checked) => setDeleteFile(checked === true)} />
            同时删除本地文件
          </label>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={() => setDeleteFile(false)}>取消</AlertDialogCancel>
            <AlertDialogAction variant="destructive" onClick={handleConfirmDelete}>删除</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
