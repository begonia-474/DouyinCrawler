import { useNavigate } from "react-router-dom";
import { Header } from "@/components/layout/header";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { Bezel } from "@/components/shared/bezel";
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
import {
  useLiveRecordCountQuery,
  useMusicCountQuery,
  useUserStatsQuery,
  useVideoCountQuery,
  useVideoStatsQuery,
  useDownloadTrendQuery,
  useTopAuthorsQuery,
  useStorageAnalysisQuery,
  useDbHealthQuery,
} from "@/lib/queries";
import { formatFileSize } from "@/lib/utils";

const categories = [
  { key: "video_info", title: "视频库", icon: Film, path: "/douyin/library/video-info", span: "col-span-8" },
  { key: "user_info", title: "用户库", icon: Users, path: "/douyin/library/user-info", span: "col-span-4" },
  { key: "images", title: "图集", icon: Image, path: "/douyin/library/images", span: "col-span-4" },
  { key: "music", title: "音乐", icon: Music, path: "/douyin/library/music", span: "col-span-4" },
  { key: "live", title: "直播", icon: Radio, path: "/douyin/library/live", span: "col-span-4" },
];

export default function LibraryPage() {
  const navigate = useNavigate();
  const liveRecordCountQuery = useLiveRecordCountQuery();
  const videoStatsQuery = useVideoStatsQuery();
  const userStatsQuery = useUserStatsQuery();
  const videoCountQuery = useVideoCountQuery({ post_type: "video" });
  const imageCountQuery = useVideoCountQuery({ post_type: "images" });
  const musicCountQuery = useMusicCountQuery({ status: "downloaded" });
  const trendQuery = useDownloadTrendQuery("day");
  const topAuthorsQuery = useTopAuthorsQuery(8);
  const storageQuery = useStorageAnalysisQuery();
  const healthQuery = useDbHealthQuery();

  const videoStats = videoStatsQuery.data ?? null;
  const userStats = userStatsQuery.data ?? null;
  const trend = trendQuery.data ?? [];
  const topAuthors = topAuthorsQuery.data ?? [];
  const storage = storageQuery.data ?? [];
  const health = healthQuery.data ?? null;
  const typeCounts: Record<string, number> = {
    video_info: videoCountQuery.data ?? 0,
    user_info: userStats?.total_count ?? 0,
    images: imageCountQuery.data ?? 0,
    music: musicCountQuery.data ?? 0,
    live: liveRecordCountQuery.data ?? 0,
  };

  return (
    <>
      <AnimateEntry>
        <Header title="资料库" description="数据管理中心" eyebrow="Library" />
      </AnimateEntry>

      <div className="space-y-8">
        <div className="grid grid-cols-12 gap-5">
          {categories.map((cat, i) => (
            <AnimateEntry key={cat.key} delay={i * 60} className={cat.span}>
              <Bezel radius="xl">
                <button
                  className="w-full text-left p-7 group cursor-pointer transition-all duration-500 hover:bg-foreground/[0.02]"
                  onClick={() => navigate(cat.path)}
                >
                  <div className="flex items-start gap-5">
                    <div className="h-12 w-12 rounded-2xl bg-foreground/[0.04] ring-1 ring-foreground/[0.07] flex items-center justify-center shrink-0 group-hover:bg-brand/[0.1] group-hover:ring-brand/25 transition-all duration-500">
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
                </button>
              </Bezel>
            </AnimateEntry>
          ))}
        </div>

        {(videoStats || userStats) && (
          <AnimateEntry delay={300}>
            <Bezel radius="xl">
              <div className="p-8">
                <span className="inline-block rounded-full px-3 py-1 text-[10px] uppercase tracking-[0.2em] font-medium bg-foreground/[0.05] text-muted-foreground mb-6">
                  <span className="flex items-center gap-1.5"><TrendingUp className="h-3 w-3" />总览</span>
                </span>
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
              </div>
            </Bezel>
          </AnimateEntry>
        )}

        {/* 下载趋势 + 存储分析 */}
        <div className="grid grid-cols-12 gap-5">
          {/* 下载趋势（最近 30 天） */}
          {trend.length > 0 && (
            <AnimateEntry delay={400} className="col-span-7">
              <Bezel radius="xl">
                <div className="p-8">
                  <span className="inline-block rounded-full px-3 py-1 text-[10px] uppercase tracking-[0.2em] font-medium bg-foreground/[0.05] text-muted-foreground mb-6">
                    <span className="flex items-center gap-1.5"><TrendingUp className="h-3 w-3" />下载趋势</span>
                  </span>
                  <div className="space-y-2">
                    {trend.slice().reverse().map((point) => {
                      const maxCnt = Math.max(...trend.map((p) => p.cnt), 1);
                      const pct = (point.cnt / maxCnt) * 100;
                      return (
                        <div key={point.day} className="flex items-center gap-3">
                          <span className="text-[11px] text-muted-foreground font-mono tabular-nums w-20 shrink-0">
                            {point.day.slice(5)}
                          </span>
                          <div className="flex-1 h-5 rounded-full bg-foreground/[0.04] overflow-hidden">
                            <div
                              className="h-full rounded-full bg-brand/60 transition-all duration-500"
                              style={{ width: `${Math.max(pct, 2)}%` }}
                            />
                          </div>
                          <span className="text-[11px] font-mono tabular-nums text-muted-foreground w-10 text-right shrink-0">
                            {point.cnt}
                          </span>
                        </div>
                      );
                    })}
                  </div>
                </div>
              </Bezel>
            </AnimateEntry>
          )}

          {/* 存储占用分析 */}
          {storage.length > 0 && (
            <AnimateEntry delay={450} className="col-span-5">
              <Bezel radius="xl">
                <div className="p-8">
                  <span className="inline-block rounded-full px-3 py-1 text-[10px] uppercase tracking-[0.2em] font-medium bg-foreground/[0.05] text-muted-foreground mb-6">
                    存储分析
                  </span>
                  <div className="space-y-3">
                    {storage.map((s) => {
                      const maxSize = Math.max(...storage.map((x) => x.total_size), 1);
                      const pct = (s.total_size / maxSize) * 100;
                      return (
                        <div key={s.download_type}>
                          <div className="flex items-center justify-between mb-1">
                            <span className="text-xs font-medium">{s.download_type}</span>
                            <span className="text-[11px] text-muted-foreground font-mono tabular-nums">
                              {formatFileSize(s.total_size)} · {s.cnt} 条
                            </span>
                          </div>
                          <div className="h-2 rounded-full bg-foreground/[0.04] overflow-hidden">
                            <div
                              className="h-full rounded-full bg-brand/50 transition-all duration-500"
                              style={{ width: `${Math.max(pct, 2)}%` }}
                            />
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>
              </Bezel>
            </AnimateEntry>
          )}
        </div>

        {/* Top 作者 + 数据库健康 */}
        <div className="grid grid-cols-12 gap-5">
          {/* 下载量 Top 作者 */}
          {topAuthors.length > 0 && (
            <AnimateEntry delay={500} className="col-span-7">
              <Bezel radius="xl">
                <div className="p-8">
                  <span className="inline-block rounded-full px-3 py-1 text-[10px] uppercase tracking-[0.2em] font-medium bg-foreground/[0.05] text-muted-foreground mb-6">
                    热门作者
                  </span>
                  <div className="space-y-2">
                    {topAuthors.map((a, i) => (
                      <div key={a.author_nickname} className="flex items-center gap-3">
                        <span className="text-[11px] text-muted-foreground font-mono tabular-nums w-5 text-right shrink-0">
                          {i + 1}
                        </span>
                        <span className="text-sm truncate flex-1 min-w-0">{a.author_nickname}</span>
                        <span className="text-[11px] text-muted-foreground font-mono tabular-nums shrink-0">
                          {a.cnt} 条
                        </span>
                        <span className="text-[11px] text-muted-foreground font-mono tabular-nums shrink-0">
                          {formatFileSize(a.total_size)}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              </Bezel>
            </AnimateEntry>
          )}

          {/* 数据库健康 */}
          {health && (
            <AnimateEntry delay={550} className="col-span-5">
              <Bezel radius="xl">
                <div className="p-8">
                  <span className="inline-block rounded-full px-3 py-1 text-[10px] uppercase tracking-[0.2em] font-medium bg-foreground/[0.05] text-muted-foreground mb-6">
                    数据库健康
                  </span>
                  <div className="grid grid-cols-2 gap-x-6 gap-y-3">
                    {[
                      { label: "下载记录", value: health.download_count },
                      { label: "视频", value: health.video_count },
                      { label: "用户", value: health.user_count },
                      { label: "直播录制", value: health.live_count },
                      { label: "音乐", value: health.music_count },
                      { label: "下载任务", value: health.task_count },
                    ].map((item) => (
                      <div key={item.label} className="flex items-center justify-between">
                        <span className="text-xs text-muted-foreground">{item.label}</span>
                        <span className="text-sm font-mono tabular-nums">{item.value.toLocaleString()}</span>
                      </div>
                    ))}
                  </div>
                  <div className="mt-4 pt-3 border-t border-foreground/[0.06]">
                    <div className="flex items-center justify-between">
                      <span className="text-xs text-muted-foreground">数据库大小</span>
                      <span className="text-sm font-mono tabular-nums">{formatFileSize(health.db_size_bytes)}</span>
                    </div>
                  </div>
                </div>
              </Bezel>
            </AnimateEntry>
          )}
        </div>
      </div>
    </>
  );
}
