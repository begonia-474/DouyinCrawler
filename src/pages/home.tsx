import { useState, useCallback } from "react";
import { Header } from "@/components/layout/header";
import { useMounted } from "@/hooks/use-safe-timer";
import { UrlInput } from "@/components/shared/url-input";
import { VideoCard } from "@/components/shared/video-card";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Badge } from "@/components/ui/badge";
import {
  Download,
  CheckCircle2,
  AlertCircle,
  Clock,
  ThumbsUp,
  MessageSquare,
  Share2,
  Bookmark,
  Play,
  BarChart3,
  Sparkles,
} from "lucide-react";

interface ParsedInfo {
  type: string;
  title?: string;
  author?: string;
  duration?: string;
  awemeId?: string;
}

interface PostStats {
  diggCount: number;
  commentCount: number;
  shareCount: number;
  collectCount: number;
  playCount: number;
}

interface RelatedVideo {
  id: string;
  title: string;
  author: string;
  duration: string;
  diggCount: number;
  commentCount: number;
  shareCount: number;
}

interface DownloadRecord {
  id: string;
  url: string;
  title: string;
  status: "completed" | "error" | "downloading";
  progress?: number;
}

function formatCount(n: number): string {
  if (n >= 10000) return `${(n / 10000).toFixed(1)}w`;
  return n.toLocaleString();
}

export function HomePage() {
  const [loading, setLoading] = useState(false);
  const [parsed, setParsed] = useState<ParsedInfo | null>(null);
  const [stats, setStats] = useState<PostStats | null>(null);
  const [related, setRelated] = useState<RelatedVideo[]>([]);
  const [downloadUrl, setDownloadUrl] = useState("");
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [downloading, setDownloading] = useState(false);
  const [downloaded, setDownloaded] = useState(false);
  const [records] = useState<DownloadRecord[]>([]);
  const mountedRef = useMounted();

  const handleParse = useCallback(async (url: string) => {
    setLoading(true);
    setParsed(null);
    setStats(null);
    setRelated([]);
    setDownloadUrl(url);
    setDownloaded(false);

    // 模拟解析
    setTimeout(() => {
      if (!mountedRef.current) return;
      setParsed({
        type: "video",
        title: "示例视频标题 - 旅行中的美好瞬间",
        author: "旅行达人",
        duration: "00:30",
        awemeId: "1234567890",
      });
      setStats({
        diggCount: 52000,
        commentCount: 3200,
        shareCount: 1800,
        collectCount: 8900,
        playCount: 892000,
      });
      setRelated([
        { id: "r1", title: "相关视频 1 - 风景合集", author: "摄影师A", duration: "01:20", diggCount: 18000, commentCount: 890, shareCount: 450 },
        { id: "r2", title: "相关视频 2 - 美食探店", author: "美食家B", duration: "02:15", diggCount: 9500, commentCount: 560, shareCount: 230 },
        { id: "r3", title: "相关视频 3 - 搞笑日常", author: "段子手C", duration: "00:45", diggCount: 35000, commentCount: 2100, shareCount: 1200 },
      ]);
      setLoading(false);
    }, 1000);
  }, []);

  const handleDownload = useCallback(async () => {
    if (!downloadUrl) return;
    setDownloading(true);
    setDownloadProgress(0);

    const interval = setInterval(() => {
      setDownloadProgress((prev) => {
        if (prev >= 100) {
          clearInterval(interval);
          if (mountedRef.current) {
            setDownloading(false);
            setDownloaded(true);
          }
          return 100;
        }
        return prev + 10;
      });
    }, 300);
  }, [downloadUrl, mountedRef]);

  return (
    <>
      <Header title="快速下载" description="粘贴抖音链接，一键下载视频/图文" />

      <div className="space-y-6">
        <UrlInput onSubmit={handleParse} loading={loading} />

        {/* 解析结果 */}
        {parsed && (
          <Card>
            <CardContent className="p-5">
              <div className="flex items-start justify-between">
                <div className="space-y-2">
                  <div className="flex items-center gap-2">
                    <Badge variant={parsed.type === "video" ? "default" : "secondary"}>
                      {parsed.type === "video" ? "视频" : "图文"}
                    </Badge>
                    <h3 className="font-medium text-lg">{parsed.title}</h3>
                  </div>
                  <p className="text-sm text-muted-foreground">
                    {parsed.author} · {parsed.duration}
                    {parsed.awemeId && ` · ID: ${parsed.awemeId}`}
                  </p>
                </div>
                <Button
                  onClick={handleDownload}
                  disabled={downloading || downloaded}
                  size="lg"
                >
                  {downloaded ? (
                    <>
                      <CheckCircle2 className="h-4 w-4 mr-2" />
                      已完成
                    </>
                  ) : downloading ? (
                    <>
                      <Download className="h-4 w-4 mr-2 animate-pulse" />
                      下载中...
                    </>
                  ) : (
                    <>
                      <Download className="h-4 w-4 mr-2" />
                      下载
                    </>
                  )}
                </Button>
              </div>
              {downloading && (
                <div className="mt-4 space-y-1">
                  <Progress value={downloadProgress} />
                  <p className="text-xs text-muted-foreground text-right">
                    {downloadProgress}%
                  </p>
                </div>
              )}
            </CardContent>
          </Card>
        )}

        {/* 作品统计 */}
        {stats && (
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-sm flex items-center gap-2 text-muted-foreground">
                <BarChart3 className="h-4 w-4" />
                作品统计
              </CardTitle>
            </CardHeader>
            <CardContent className="pt-0">
              <div className="grid grid-cols-5 gap-4">
                <div className="text-center">
                  <Play className="h-4 w-4 text-muted-foreground mx-auto mb-1" />
                  <p className="text-base font-semibold">{formatCount(stats.playCount)}</p>
                  <p className="text-xs text-muted-foreground">播放</p>
                </div>
                <div className="text-center">
                  <ThumbsUp className="h-4 w-4 text-muted-foreground mx-auto mb-1" />
                  <p className="text-base font-semibold">{formatCount(stats.diggCount)}</p>
                  <p className="text-xs text-muted-foreground">点赞</p>
                </div>
                <div className="text-center">
                  <MessageSquare className="h-4 w-4 text-muted-foreground mx-auto mb-1" />
                  <p className="text-base font-semibold">{formatCount(stats.commentCount)}</p>
                  <p className="text-xs text-muted-foreground">评论</p>
                </div>
                <div className="text-center">
                  <Share2 className="h-4 w-4 text-muted-foreground mx-auto mb-1" />
                  <p className="text-base font-semibold">{formatCount(stats.shareCount)}</p>
                  <p className="text-xs text-muted-foreground">分享</p>
                </div>
                <div className="text-center">
                  <Bookmark className="h-4 w-4 text-muted-foreground mx-auto mb-1" />
                  <p className="text-base font-semibold">{formatCount(stats.collectCount)}</p>
                  <p className="text-xs text-muted-foreground">收藏</p>
                </div>
              </div>
            </CardContent>
          </Card>
        )}

        {/* 相关视频推荐 */}
        {related.length > 0 && (
          <div className="space-y-3">
            <h3 className="text-sm font-medium text-muted-foreground flex items-center gap-2">
              <Sparkles className="h-4 w-4" />
              相关推荐
            </h3>
            <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
              {related.map((video) => (
                <VideoCard
                  key={video.id}
                  title={video.title}
                  author={video.author}
                  duration={video.duration}
                  diggCount={video.diggCount}
                  commentCount={video.commentCount}
                  shareCount={video.shareCount}
                />
              ))}
            </div>
          </div>
        )}

        {/* 最近下载记录 */}
        {records.length > 0 && (
          <div className="space-y-3">
            <h3 className="text-sm font-medium text-muted-foreground">最近下载</h3>
            {records.map((record) => (
              <div
                key={record.id}
                className="flex items-center gap-3 text-sm p-3 rounded-lg bg-muted/50"
              >
                {record.status === "completed" && (
                  <CheckCircle2 className="h-4 w-4 text-emerald-500 shrink-0" />
                )}
                {record.status === "error" && (
                  <AlertCircle className="h-4 w-4 text-destructive shrink-0" />
                )}
                {record.status === "downloading" && (
                  <Clock className="h-4 w-4 text-muted-foreground shrink-0" />
                )}
                <span className="flex-1 truncate">{record.title}</span>
                <span className="text-muted-foreground text-xs shrink-0">
                  {record.status === "completed"
                    ? "已完成"
                    : record.status === "error"
                    ? "失败"
                    : `${record.progress}%`}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </>
  );
}
