import { useState, useCallback, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { Header } from "@/components/layout/header";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { AnimateEntry } from "@/components/shared/animate-entry";
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
      <AnimateEntry>
        <Header title="关注直播" description="查看关注用户的直播状态" />
      </AnimateEntry>

      <div className="space-y-5">
        <div className="flex items-center justify-between">
          <p className="text-sm text-muted-foreground tracking-wide">
            {lives.length > 0 ? `${lives.length} 位主播正在直播` : ""}
          </p>
          <Button
            variant="outline"
            size="sm"
            className="rounded-lg border-border/60"
            onClick={fetchLives}
            disabled={loading}
          >
            {loading ? (
              <Loader2 className="h-4 w-4 mr-1.5 animate-spin" />
            ) : (
              <RefreshCw className="h-4 w-4 mr-1.5" />
            )}
            刷新
          </Button>
        </div>

        {error && (
          <div className="flex items-center gap-2.5 p-4 rounded-xl bg-destructive/[0.06] border border-destructive/20 text-destructive text-sm">
            <AlertCircle className="h-4 w-4 shrink-0" />
            <span>{error}</span>
          </div>
        )}

        {loading && lives.length === 0 && (
          <div className="flex items-center justify-center py-16">
            <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
          </div>
        )}

        {!loading && lives.length === 0 && !error && (
          <Card className="border-border/40 bg-card/60">
            <CardContent className="p-12 text-center">
              <Radio className="h-10 w-10 text-muted-foreground/30 mx-auto mb-4" />
              <h3 className="font-heading text-lg font-semibold mb-2">暂无直播</h3>
              <p className="text-muted-foreground text-sm tracking-wide">
                你关注的用户当前没有在直播
              </p>
            </CardContent>
          </Card>
        )}

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
          {lives.map((live, i) => (
            <AnimateEntry key={live.room_id} delay={i * 50}>
              <Card
                className="overflow-hidden border-border/40 bg-card/60 hover:bg-card hover:border-border/60 hover:-translate-y-1 transition-all duration-500 cursor-pointer group"
                style={{ transitionTimingFunction: "cubic-bezier(0.32, 0.72, 0, 1)" }}
                onClick={() => navigate("/douyin/live", { state: { url: `https://live.douyin.com/${live.web_rid}` } })}
              >
                <div className="relative aspect-video bg-foreground/[0.03]">
                  {live.cover ? (
                    <img
                      src={live.cover}
                      alt={live.title}
                      className="w-full h-full object-cover"
                    />
                  ) : (
                    <div className="w-full h-full flex items-center justify-center">
                      <Radio className="h-8 w-8 text-muted-foreground/30" />
                    </div>
                  )}
                  <Badge
                    variant="destructive"
                    className="absolute top-3 left-3 animate-pulse text-[10px] tracking-wide"
                  >
                    <Circle className="h-1.5 w-1.5 fill-current mr-1" />
                    直播中
                  </Badge>
                  {live.user_count > 0 && (
                    <Badge
                      variant="secondary"
                      className="absolute top-3 right-3 bg-foreground/60 text-background backdrop-blur-sm text-[10px]"
                    >
                      <Users className="h-3 w-3 mr-1" />
                      {formatCount(live.user_count)}
                    </Badge>
                  )}
                </div>

                <CardContent className="p-4">
                  <div className="flex items-start gap-3">
                    <div className="w-10 h-10 rounded-xl overflow-hidden bg-foreground/[0.04] shrink-0">
                      {live.avatar ? (
                        <img
                          src={live.avatar}
                          alt={live.nickname}
                          className="w-full h-full object-cover"
                        />
                      ) : (
                        <div className="w-full h-full flex items-center justify-center text-muted-foreground text-sm font-medium">
                          {live.nickname.charAt(0)}
                        </div>
                      )}
                    </div>

                    <div className="flex-1 min-w-0">
                      <h4 className="font-medium text-sm line-clamp-2 leading-relaxed">
                        {live.title || "无标题"}
                      </h4>
                      <p className="text-xs text-muted-foreground mt-1 truncate tracking-wide">
                        {live.nickname}
                      </p>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </AnimateEntry>
          ))}
        </div>
      </div>
    </>
  );
}

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
