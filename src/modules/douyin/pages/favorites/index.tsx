import { useState, useCallback, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { Header } from "@/components/layout/header";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { Bezel } from "@/components/shared/bezel";
import { Badge } from "@/components/ui/badge";
import { getUserCollects } from "@/lib/api";
import type { CollectsFolder } from "@/lib/api-types";
import { FolderOpen, ChevronRight, Loader2, AlertCircle, Heart } from "lucide-react";

export default function FavoritesPage() {
  const navigate = useNavigate();
  const [loading, setLoading] = useState(false);
  const [collects, setCollects] = useState<CollectsFolder[]>([]);
  const [error, setError] = useState<string | null>(null);

  const fetchCollects = useCallback(async () => {
    setLoading(true);
    setError(null);

    const res = await getUserCollects();
    if (res.success && res.data?.collects) {
      setCollects(res.data.collects as unknown as CollectsFolder[]);
    } else {
      setError(res.error || "获取收藏夹失败");
    }

    setLoading(false);
  }, []);

  useEffect(() => {
    fetchCollects();
  }, [fetchCollects]);

  return (
    <>
      <AnimateEntry>
        <Header title="我的收藏" description="当前账号的收藏夹" />
      </AnimateEntry>

      <div className="space-y-5">
        {error && (
          <div className="flex items-center gap-2 p-4 rounded-2xl bg-destructive/[0.06] ring-1 ring-destructive/20 text-destructive text-sm">
            <AlertCircle className="h-4 w-4 shrink-0" />
            <span>{error}</span>
          </div>
        )}

        {loading && (
          <div className="flex items-center justify-center py-16">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        )}

        {!loading && collects.length === 0 && !error && (
          <Bezel radius="xl">
            <div className="p-12 text-center">
              <Heart className="h-12 w-12 text-muted-foreground/30 mx-auto mb-4" />
              <h3 className="text-lg font-semibold mb-2">暂无收藏夹</h3>
              <p className="text-muted-foreground tracking-wide">
                请先在设置中配置 Cookie
              </p>
            </div>
          </Bezel>
        )}

        {!loading && collects.length > 0 && (
          <div className="space-y-3">
            {collects.map((folder, i) => (
              <AnimateEntry key={folder.id} delay={i * 40}>
                <Bezel radius="xl" padding="sm">
                  <button
                    className="w-full flex items-center gap-4 p-5 bg-card hover:bg-foreground/[0.02] transition-all duration-500 cursor-pointer"
                    onClick={() => navigate(`/douyin/favorites/${folder.id}`)}
                  >
                    <div className="h-11 w-11 rounded-2xl bg-primary/10 ring-1 ring-primary/15 flex items-center justify-center shrink-0">
                      <FolderOpen className="h-5 w-5 text-primary" />
                    </div>
                    <div className="flex-1 text-left">
                      <h4 className="text-sm font-medium">{folder.name}</h4>
                      <p className="text-xs text-muted-foreground tracking-wide">{folder.count} 个视频</p>
                    </div>
                    <Badge variant="secondary" className="rounded-full">{folder.count}</Badge>
                    <ChevronRight className="h-4 w-4 text-muted-foreground group-hover:translate-x-0.5 transition-transform" />
                  </button>
                </Bezel>
              </AnimateEntry>
            ))}
          </div>
        )}
      </div>
    </>
  );
}
