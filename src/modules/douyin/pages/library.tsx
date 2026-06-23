import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { Header } from "@/components/layout/header";
import { Card, CardContent } from "@/components/ui/card";
import { AnimateEntry } from "@/components/shared/animate-entry";
import {
  Radio,
  TrendingUp,
  Film,
  Users,
  Heart,
  UserCheck,
  Image,
  Music,
} from "lucide-react";
import { getLiveRecords, getVideoStats, getUserStats, getVideoCount, getMusicCollectionCountFromDB } from "@/lib/api";
import type { VideoStats, UserStats } from "@/lib/tauri-types";

const categories = [
  { key: "video_info", title: "视频库", icon: Film, path: "/douyin/library/video-info", span: "col-span-2 row-span-1" },
  { key: "user_info", title: "用户库", icon: Users, path: "/douyin/library/user-info", span: "col-span-1 row-span-1" },
  { key: "images", title: "图集", icon: Image, path: "/douyin/library/images", span: "col-span-1 row-span-1" },
  { key: "music", title: "音乐", icon: Music, path: "/douyin/library/music", span: "col-span-1 row-span-1" },
  { key: "live", title: "直播", icon: Radio, path: "/douyin/library/live", span: "col-span-2 row-span-1" },
];

export default function LibraryPage() {
  const navigate = useNavigate();
  const [videoStats, setVideoStats] = useState<VideoStats | null>(null);
  const [userStats, setUserStats] = useState<UserStats | null>(null);
  const [typeCounts, setTypeCounts] = useState<Record<string, number>>({});

  const loadStats = useCallback(async () => {
    try {
      const [liveData, vs, us, videoCount, imgCount, musicCount] = await Promise.all([
        getLiveRecords({ limit: 1 }),
        getVideoStats().catch(() => null),
        getUserStats().catch(() => null),
        getVideoCount({ post_type: "video" }).catch(() => 0),
        getVideoCount({ post_type: "images" }).catch(() => 0),
        getMusicCollectionCountFromDB(undefined, "downloaded").catch(() => 0),
      ]);
      setVideoStats(vs);
      setUserStats(us);

      const counts: Record<string, number> = {};
      counts.video_info = videoCount;
      if (us) counts.user_info = us.total_count;
      counts.images = imgCount;
      counts.music = musicCount;
      counts.live = liveData.length;
      setTypeCounts(counts);
    } catch (err) {
      console.error("加载统计失败:", err);
    }
  }, []);

  useEffect(() => {
    loadStats();
  }, [loadStats]);

  return (
    <>
      <AnimateEntry>
        <Header title="资料库" description="数据管理中心" />
      </AnimateEntry>

      <div className="space-y-6">
        <div className="grid grid-cols-3 gap-4">
          {categories.map((cat, i) => (
            <AnimateEntry key={cat.key} delay={i * 60}>
              <Card
                className={`group cursor-pointer border-border/40 bg-card/60 backdrop-blur-sm hover:bg-card hover:border-border/60 hover:-translate-y-1 transition-all duration-500 ${cat.span}`}
                style={{ transitionTimingFunction: "cubic-bezier(0.32, 0.72, 0, 1)" }}
                onClick={() => navigate(cat.path)}
              >
                <CardContent className="p-6">
                  <div className="flex items-start gap-5">
                    <div className="h-12 w-12 rounded-2xl bg-foreground/[0.04] ring-1 ring-foreground/[0.06] flex items-center justify-center shrink-0 group-hover:bg-brand/[0.08] group-hover:ring-brand/20 transition-all duration-500">
                      <cat.icon className="h-5 w-5 text-muted-foreground group-hover:text-brand transition-colors duration-500" />
                    </div>
                    <div className="min-w-0">
                      <h3 className="font-heading text-lg font-semibold tracking-tight">{cat.title}</h3>
                      <p className="text-3xl font-heading font-bold mt-1 tabular-nums">
                        {typeCounts[cat.key] ?? 0}
                      </p>
                      <p className="text-xs text-muted-foreground mt-0.5 tracking-wide">
                        {cat.key === "live" ? "次录制" : "条记录"}
                      </p>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </AnimateEntry>
          ))}
        </div>

        {(videoStats || userStats) && (
          <AnimateEntry delay={300}>
            <Card className="border-border/40 bg-card/60">
              <CardContent className="p-8">
                <h3 className="text-xs uppercase tracking-[0.15em] font-medium text-muted-foreground mb-6 flex items-center gap-2">
                  <TrendingUp className="h-3.5 w-3.5" />
                  总览
                </h3>
                <div className="grid grid-cols-2 md:grid-cols-4 gap-8">
                  {videoStats && (
                    <>
                      <div>
                        <p className="text-3xl font-heading font-bold tabular-nums">{videoStats.total_count}</p>
                        <p className="text-xs text-muted-foreground mt-1 tracking-wide">视频总数</p>
                      </div>
                      <div>
                        <p className="text-3xl font-heading font-bold tabular-nums flex items-center gap-2">
                          <Heart className="h-5 w-5 text-destructive/70" />
                          {videoStats.total_digg.toLocaleString()}
                        </p>
                        <p className="text-xs text-muted-foreground mt-1 tracking-wide">总点赞</p>
                      </div>
                    </>
                  )}
                  {userStats && (
                    <>
                      <div>
                        <p className="text-3xl font-heading font-bold tabular-nums">{userStats.total_count}</p>
                        <p className="text-xs text-muted-foreground mt-1 tracking-wide">用户总数</p>
                      </div>
                      <div>
                        <p className="text-3xl font-heading font-bold tabular-nums flex items-center gap-2">
                          <UserCheck className="h-5 w-5 text-brand/70" />
                          {userStats.total_follower.toLocaleString()}
                        </p>
                        <p className="text-xs text-muted-foreground mt-1 tracking-wide">总粉丝</p>
                      </div>
                    </>
                  )}
                </div>
              </CardContent>
            </Card>
          </AnimateEntry>
        )}
      </div>
    </>
  );
}
