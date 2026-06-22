import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { Header } from "@/components/layout/header";
import { Card, CardContent } from "@/components/ui/card";
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
import { getLiveRecords, getVideoStats, getUserStats, getVideoCount } from "@/lib/api";
import type { VideoStats, UserStats } from "@/lib/tauri-types";

const categories = [
  { key: "video_info", title: "视频库", icon: Film, color: "text-orange-500", bgColor: "bg-orange-500/10", path: "/douyin/library/video-info" },
  { key: "user_info", title: "用户库", icon: Users, color: "text-cyan-500", bgColor: "bg-cyan-500/10", path: "/douyin/library/user-info" },
  { key: "images", title: "图集", icon: Image, color: "text-green-500", bgColor: "bg-green-500/10", path: "/douyin/library/images" },
  { key: "music", title: "音乐", icon: Music, color: "text-purple-500", bgColor: "bg-purple-500/10", path: "/douyin/library/music" },
  { key: "live", title: "直播", icon: Radio, color: "text-red-500", bgColor: "bg-red-500/10", path: "/douyin/library/live" },
];

export default function LibraryPage() {
  const navigate = useNavigate();
  const [videoStats, setVideoStats] = useState<VideoStats | null>(null);
  const [userStats, setUserStats] = useState<UserStats | null>(null);
  const [typeCounts, setTypeCounts] = useState<Record<string, number>>({});

  const loadStats = useCallback(async () => {
    try {
      const [liveData, vs, us, imgCount, musicCount] = await Promise.all([
        getLiveRecords({ limit: 1 }),
        getVideoStats().catch(() => null),
        getUserStats().catch(() => null),
        getVideoCount({ post_type: "images" }).catch(() => 0),
        getVideoCount({ post_type: "music" }).catch(() => 0),
      ]);
      setVideoStats(vs);
      setUserStats(us);

      const counts: Record<string, number> = {};
      if (vs) counts.video_info = vs.total_count;
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
      <Header title="资料库" description="数据管理中心" />

      <div className="space-y-6">
        {/* 卡片网格 */}
        <div className="grid grid-cols-3 md:grid-cols-5 gap-4">
          {categories.map((cat) => (
            <Card
              key={cat.key}
              className="hover:bg-accent/50 transition-colors cursor-pointer"
              onClick={() => navigate(cat.path)}
            >
              <CardContent className="p-6">
                <div className="flex items-start gap-4">
                  <div className={`h-12 w-12 rounded-xl ${cat.bgColor} flex items-center justify-center shrink-0`}>
                    <cat.icon className={`h-6 w-6 ${cat.color}`} />
                  </div>
                  <div className="min-w-0">
                    <h3 className="font-semibold">{cat.title}</h3>
                    <p className="text-2xl font-bold mt-1">{typeCounts[cat.key] ?? 0}</p>
                    <p className="text-xs text-muted-foreground">
                      {cat.key === "live" ? "次录制" : "条记录"}
                    </p>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>

        {/* 总览 */}
        {(videoStats || userStats) && (
          <Card>
            <CardContent className="p-6">
              <h3 className="text-sm font-medium text-muted-foreground mb-4 flex items-center gap-2">
                <TrendingUp className="h-4 w-4" />
                总览
              </h3>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
                {videoStats && (
                  <>
                    <div>
                      <p className="text-2xl font-bold">{videoStats.total_count}</p>
                      <p className="text-xs text-muted-foreground">视频总数</p>
                    </div>
                    <div>
                      <p className="text-2xl font-bold flex items-center gap-1">
                        <Heart className="h-5 w-5 text-red-400" />
                        {videoStats.total_digg.toLocaleString()}
                      </p>
                      <p className="text-xs text-muted-foreground">总点赞</p>
                    </div>
                  </>
                )}
                {userStats && (
                  <>
                    <div>
                      <p className="text-2xl font-bold">{userStats.total_count}</p>
                      <p className="text-xs text-muted-foreground">用户总数</p>
                    </div>
                    <div>
                      <p className="text-2xl font-bold flex items-center gap-1">
                        <UserCheck className="h-5 w-5 text-cyan-400" />
                        {userStats.total_follower.toLocaleString()}
                      </p>
                      <p className="text-xs text-muted-foreground">总粉丝</p>
                    </div>
                  </>
                )}
              </div>
            </CardContent>
          </Card>
        )}
      </div>
    </>
  );
}
