import { useState, useCallback } from "react";
import { Header } from "@/components/layout/header";
import { UrlInput } from "@/components/shared/url-input";
import { VideoCard } from "@/components/shared/video-card";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Badge } from "@/components/ui/badge";
import { getPostDetail, downloadOne } from "@/lib/api";
import type { VideoItem } from "@/lib/api-types";
import {
  Download,
  CheckCircle2,
  AlertCircle,
  Clock,
  ThumbsUp,
  MessageSquare,
  Share2,
  Bookmark,
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

export default function HomePage() {
  const [loading, setLoading] = useState(false);
  const [parsed, setParsed] = useState<ParsedInfo | null>(null);
  const [stats, setStats] = useState<{
    digg_count: number;
    comment_count: number;
    share_count: number;
    collect_count: number;
  } | null>(null);
  const [related, setRelated] = useState<RelatedVideo[]>([]);
  const [downloadUrl, setDownloadUrl] = useState("");
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [downloading, setDownloading] = useState(false);
  const [downloaded, setDownloaded] = useState(false);
  const [records, setRecords] = useState<DownloadRecord[]>([]);
  const [error, setError] = useState<string | null>(null);

  const handleParse = useCallback(async (url: string) => {
    setLoading(true);
    setParsed(null);
    setStats(null);
    setRelated([]);
    setDownloadUrl(url);
    setDownloaded(false);
    setError(null);

    try {
      const detailRes = await getPostDetail(url);

      if (!detailRes.success) {
        setError(detailRes.error || "解析失败");
        setLoading(false);
        return;
      }

      const data = detailRes.data;

      // 单视频：有 detail 字段
      if (data?.detail) {
        const detail = data.detail;
        setParsed({
          type: detail.type || "video",
          title: detail.desc || detail.title,
          author: detail.author,
          awemeId: detail.aweme_id || detail.awemeId,
        });
        // 从 detail 中取统计数据
        setStats({
          digg_count: detail.digg_count ?? 0,
          comment_count: detail.comment_count ?? 0,
          share_count: detail.share_count ?? 0,
          collect_count: detail.collect_count ?? 0,
        });
      }
      // 用户主页/合集：有 videos 字段
      else if (data?.videos) {
        const videos = data.videos as VideoItem[];
        const isUser = data.type === "user";
        setParsed({
          type: isUser ? "user" : "mix",
          title: `${isUser ? "用户主页" : "合集"} (${videos.length} 个视频)`,
          author: videos[0]?.author || "",
        });
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : "请求异常");
    }

    setLoading(false);
  }, []);

  const handleDownload = useCallback(async () => {
    if (!downloadUrl) return;
    setDownloading(true);
    setDownloadProgress(0);
    setError(null);

    const res = await downloadOne(downloadUrl);

    if (res.success) {
      setDownloaded(true);
      setDownloadProgress(100);
      setRecords((prev) => [
        { id: Date.now().toString(), url: downloadUrl, title: parsed?.title || downloadUrl, status: "completed" },
        ...prev,
      ]);
    } else {
      setError(res.error || "下载失败");
      setRecords((prev) => [
        { id: Date.now().toString(), url: downloadUrl, title: parsed?.title || downloadUrl, status: "error" },
        ...prev,
      ]);
    }
    setDownloading(false);
  }, [downloadUrl, parsed]);

  return (
    <>
      <Header title="快速下载" description="粘贴抖音链接，一键下载视频/图文" />

      <div className="space-y-6">
        <UrlInput onSubmit={handleParse} loading={loading} />

        {/* 错误提示 */}
        {error && (
          <div className="flex items-center gap-2 p-3 rounded-lg bg-destructive/10 text-destructive text-sm">
            <AlertCircle className="h-4 w-4 shrink-0" />
            <span>{error}</span>
          </div>
        )}

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
              <div className="grid grid-cols-4 gap-4">
                <div className="text-center">
                  <ThumbsUp className="h-4 w-4 text-muted-foreground mx-auto mb-1" />
                  <p className="text-base font-semibold">{formatCount(stats.digg_count)}</p>
                  <p className="text-xs text-muted-foreground">点赞</p>
                </div>
                <div className="text-center">
                  <MessageSquare className="h-4 w-4 text-muted-foreground mx-auto mb-1" />
                  <p className="text-base font-semibold">{formatCount(stats.comment_count)}</p>
                  <p className="text-xs text-muted-foreground">评论</p>
                </div>
                <div className="text-center">
                  <Share2 className="h-4 w-4 text-muted-foreground mx-auto mb-1" />
                  <p className="text-base font-semibold">{formatCount(stats.share_count)}</p>
                  <p className="text-xs text-muted-foreground">分享</p>
                </div>
                <div className="text-center">
                  <Bookmark className="h-4 w-4 text-muted-foreground mx-auto mb-1" />
                  <p className="text-base font-semibold">{formatCount(stats.collect_count)}</p>
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
            <h3 className="text-sm font-medium text-muted-foreground flex items-center gap-2">
              <Clock className="h-4 w-4" />
              最近下载
            </h3>
            {records.map((record) => (
              <div
                key={record.id}
                className="flex items-center gap-4 text-sm p-3 rounded-lg bg-muted/50"
              >
                {record.status === "completed" && (
                  <CheckCircle2 className="h-4 w-4 text-green-600 shrink-0" />
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
