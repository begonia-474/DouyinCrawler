import { useState, useCallback, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { Header } from "@/components/layout/header";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { getFollowingLive } from "@/lib/api";
import type { FollowingLiveItem } from "@/lib/api-types";
import {
  Users,
  Radio,
  RefreshCw,
  Loader2,
  AlertCircle,
} from "lucide-react";

export default function FollowingLivePage() {
  const navigate = useNavigate();
  const [loading, setLoading] = useState(false);
  const [lives, setLives] = useState<FollowingLiveItem[]>([]);
  const [error, setError] = useState<string | null>(null);

  const fetchLives = useCallback(async () => {
    setLoading(true);
    setError(null);

    const res = await getFollowingLive();
    if (res.success && res.data?.lives) {
      setLives(res.data.lives);
    } else {
      setError(res.error || "获取关注直播列表失败");
    }

    setLoading(false);
  }, []);

  useEffect(() => {
    fetchLives();
  }, [fetchLives]);

  const formatCount = (count: number) => {
    if (count >= 10000) {
      return `${(count / 10000).toFixed(1)}万`;
    }
    return count.toString();
  };

  return (
    <>
      <Header title="关注直播" description="查看关注用户的直播状态" />

      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <p className="text-sm text-muted-foreground">
            {lives.length > 0 ? `${lives.length} 位主播正在直播` : ""}
          </p>
          <Button
            variant="outline"
            size="sm"
            onClick={fetchLives}
            disabled={loading}
          >
            {loading ? (
              <Loader2 className="h-4 w-4 mr-2 animate-spin" />
            ) : (
              <RefreshCw className="h-4 w-4 mr-2" />
            )}
            刷新
          </Button>
        </div>

        {error && (
          <div className="flex items-center gap-2 p-3 rounded-lg bg-destructive/10 text-destructive text-sm">
            <AlertCircle className="h-4 w-4 shrink-0" />
            <span>{error}</span>
          </div>
        )}

        {loading && lives.length === 0 && (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        )}

        {!loading && lives.length === 0 && !error && (
          <Card>
            <CardContent className="p-8 text-center">
              <Radio className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
              <h3 className="text-lg font-semibold mb-2">暂无直播</h3>
              <p className="text-muted-foreground">
                你关注的用户当前没有在直播
              </p>
            </CardContent>
          </Card>
        )}

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
          {lives.map((live) => (
            <Card
              key={live.room_id}
              className="overflow-hidden hover:shadow-md transition-shadow cursor-pointer"
              onClick={() => navigate("/douyin/live", { state: { url: `https://live.douyin.com/${live.web_rid}` } })}
            >
              {/* 封面 */}
              <div className="relative aspect-video bg-muted">
                {live.cover ? (
                  <img
                    src={live.cover}
                    alt={live.title}
                    className="w-full h-full object-cover"
                  />
                ) : (
                  <div className="w-full h-full flex items-center justify-center">
                    <Radio className="h-8 w-8 text-muted-foreground" />
                  </div>
                )}
                <Badge
                  variant="destructive"
                  className="absolute top-2 left-2 animate-pulse"
                >
                  <Circle className="h-2 w-2 fill-current mr-1" />
                  直播中
                </Badge>
                {live.user_count > 0 && (
                  <Badge
                    variant="secondary"
                    className="absolute top-2 right-2 bg-black/60 text-white"
                  >
                    <Users className="h-3 w-3 mr-1" />
                    {formatCount(live.user_count)}
                  </Badge>
                )}
              </div>

              {/* 信息 */}
              <CardContent className="p-3">
                <div className="flex items-start gap-3">
                  {/* 头像 */}
                  <div className="w-10 h-10 rounded-full overflow-hidden bg-muted shrink-0">
                    {live.avatar ? (
                      <img
                        src={live.avatar}
                        alt={live.nickname}
                        className="w-full h-full object-cover"
                      />
                    ) : (
                      <div className="w-full h-full flex items-center justify-center text-muted-foreground">
                        {live.nickname.charAt(0)}
                      </div>
                    )}
                  </div>

                  {/* 文字信息 */}
                  <div className="flex-1 min-w-0">
                    <h4 className="font-medium text-sm line-clamp-2">
                      {live.title || "无标题"}
                    </h4>
                    <p className="text-xs text-muted-foreground mt-1 truncate">
                      {live.nickname}
                    </p>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    </>
  );
}

// 辅助组件
function Circle({ className }: { className?: string }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      fill="currentColor"
      className={className}
    >
      <circle cx="12" cy="12" r="10" />
    </svg>
  );
}
