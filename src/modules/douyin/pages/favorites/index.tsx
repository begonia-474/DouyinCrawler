import { useState, useCallback, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { Header } from "@/components/layout/header";
import { Card, CardContent } from "@/components/ui/card";
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
      <Header title="我的收藏" description="当前账号的收藏夹" />

      <div className="space-y-6">
        {error && (
          <div className="flex items-center gap-2 p-3 rounded-lg bg-destructive/10 text-destructive text-sm">
            <AlertCircle className="h-4 w-4 shrink-0" />
            <span>{error}</span>
          </div>
        )}

        {loading && (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        )}

        {!loading && collects.length === 0 && !error && (
          <Card>
            <CardContent className="p-8 text-center">
              <Heart className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
              <h3 className="text-lg font-semibold mb-2">暂无收藏夹</h3>
              <p className="text-muted-foreground">
                请先在设置中配置 Cookie
              </p>
            </CardContent>
          </Card>
        )}

        {!loading && collects.length > 0 && (
          <div className="space-y-3">
            {collects.map((folder) => (
              <Card
                key={folder.id}
                className="hover:bg-accent/50 transition-colors cursor-pointer"
                onClick={() => navigate(`/douyin/favorites/${folder.id}`)}
              >
                <CardContent className="p-4 flex items-center gap-4">
                  <div className="h-10 w-10 rounded-lg bg-primary/10 flex items-center justify-center">
                    <FolderOpen className="h-5 w-5 text-primary" />
                  </div>
                  <div className="flex-1">
                    <h4 className="text-sm font-medium">{folder.name}</h4>
                    <p className="text-xs text-muted-foreground">{folder.count} 个视频</p>
                  </div>
                  <Badge variant="secondary">{folder.count}</Badge>
                  <ChevronRight className="h-4 w-4 text-muted-foreground" />
                </CardContent>
              </Card>
            ))}
          </div>
        )}
      </div>
    </>
  );
}
