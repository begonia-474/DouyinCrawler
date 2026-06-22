import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { Header } from "@/components/layout/header";
import { Card, CardContent } from "@/components/ui/card";
import {
  Video,
  Image,
  Radio,
  Music,
  HardDrive,
  TrendingUp,
} from "lucide-react";
import { getDownloadStats, getLiveRecords } from "@/lib/tauri-api";
import type { DownloadStats } from "@/lib/tauri-types";
import { formatFileSize } from "@/lib/utils";

const categories = [
  { key: "video", title: "视频", icon: Video, color: "text-blue-500", bgColor: "bg-blue-500/10", path: "/douyin/library/videos" },
  { key: "images", title: "图集", icon: Image, color: "text-green-500", bgColor: "bg-green-500/10", path: "/douyin/library/images" },
  { key: "live", title: "直播", icon: Radio, color: "text-red-500", bgColor: "bg-red-500/10", path: "/douyin/library/live" },
  { key: "music", title: "音乐", icon: Music, color: "text-purple-500", bgColor: "bg-purple-500/10", path: "/douyin/library/music" },
];

export default function LibraryPage() {
  const navigate = useNavigate();
  const [stats, setStats] = useState<DownloadStats | null>(null);
  const [typeCounts, setTypeCounts] = useState<Record<string, number>>({});

  const loadStats = useCallback(async () => {
    try {
      const [statsData, liveData] = await Promise.all([
        getDownloadStats(),
        getLiveRecords({ limit: 1 }),
      ]);
      setStats(statsData);

      const counts: Record<string, number> = {};
      for (const t of statsData.by_type) {
        counts[t.download_type] = t.cnt;
      }
      // 直播录制用 live_records 表的数量
      if (liveData.length > 0 && !counts.live) {
        counts.live = liveData.length;
      }
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
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
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
        {stats && (
          <Card>
            <CardContent className="p-6">
              <h3 className="text-sm font-medium text-muted-foreground mb-4 flex items-center gap-2">
                <TrendingUp className="h-4 w-4" />
                总览
              </h3>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
                <div>
                  <p className="text-2xl font-bold">{stats.total_count}</p>
                  <p className="text-xs text-muted-foreground">总下载数</p>
                </div>
                <div>
                  <p className="text-2xl font-bold flex items-center gap-1">
                    <HardDrive className="h-5 w-5 text-muted-foreground" />
                    {formatFileSize(stats.total_size)}
                  </p>
                  <p className="text-xs text-muted-foreground">总文件大小</p>
                </div>
                <div>
                  <p className="text-2xl font-bold">
                    {stats.by_day.reduce((sum, d) => sum + d.cnt, 0)}
                  </p>
                  <p className="text-xs text-muted-foreground">最近 7 天</p>
                </div>
                <div>
                  <p className="text-2xl font-bold">{stats.by_type.length}</p>
                  <p className="text-xs text-muted-foreground">下载类型</p>
                </div>
              </div>
            </CardContent>
          </Card>
        )}
      </div>
    </>
  );
}
