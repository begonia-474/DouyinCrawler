import { useState, useCallback } from "react";
import { Header } from "@/components/layout/header";
import { UrlInput } from "@/components/shared/url-input";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Badge } from "@/components/ui/badge";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { Bezel } from "@/components/shared/bezel";
import { ErrorBanner } from "@/components/shared/error-banner";
import { getPostDetail, startDownload } from "@/lib/api";
import {
  Download,
  CheckCircle2,
  ThumbsUp,
  MessageSquare,
  Share2,
  Bookmark,
  BarChart3,
  ArrowRight,
} from "lucide-react";
import { formatCount } from "@/lib/utils";

interface ParsedInfo {
  type: string;
  title?: string;
  author?: string;
  duration?: string;
  awemeId?: string;
}

export default function VideoPage() {
  const [loading, setLoading] = useState(false);
  const [parsed, setParsed] = useState<ParsedInfo | null>(null);
  const [stats, setStats] = useState<{
    digg_count: number;
    comment_count: number;
    share_count: number;
    collect_count: number;
  } | null>(null);
  const [downloadUrl, setDownloadUrl] = useState("");
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [downloading, setDownloading] = useState(false);
  const [downloaded, setDownloaded] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleParse = useCallback(async (url: string) => {
    setLoading(true);
    setParsed(null);
    setStats(null);
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

      if (data?.detail) {
        const detail = data.detail;
        setParsed({
          type: detail.type || "video",
          title: detail.desc || detail.title,
          author: detail.author,
          awemeId: detail.aweme_id || detail.awemeId,
        });
        setStats({
          digg_count: detail.digg_count ?? 0,
          comment_count: detail.comment_count ?? 0,
          share_count: detail.share_count ?? 0,
          collect_count: detail.collect_count ?? 0,
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

    const res = await startDownload("one", downloadUrl);

    if (res.success) {
      setDownloaded(true);
      setDownloadProgress(100);
    } else {
      setError(res.error || "下载失败");
    }
    setDownloading(false);
  }, [downloadUrl]);

  return (
    <>
      <AnimateEntry>
        <Header title="单视频下载" description="粘贴视频或图文链接，解析后下载" parent={{ label: "首页", path: "/douyin" }} />
      </AnimateEntry>

      <div className="space-y-6">
        <AnimateEntry delay={50}>
          <UrlInput onSubmit={handleParse} loading={loading} allowedTypes={["video"]} autoDetect />
        </AnimateEntry>

        {error && (
          <AnimateEntry>
            <ErrorBanner message={error} />
          </AnimateEntry>
        )}

        {parsed && (
          <AnimateEntry delay={100}>
            <Bezel radius="xl">
              <div className="p-7">
                <div className="flex items-start justify-between">
                  <div className="space-y-2">
                    <div className="flex items-center gap-2">
                      <Badge variant={parsed.type === "video" ? "default" : "secondary"} className="text-[11px] tracking-wide rounded-full">
                        {parsed.type === "video" ? "视频" : "图文"}
                      </Badge>
                      <h3 className="font-heading text-xl font-semibold tracking-tight">{parsed.title}</h3>
                    </div>
                    <p className="text-sm text-muted-foreground tracking-wide">
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
                        下载
                        <span className="ml-2 inline-flex items-center justify-center w-6 h-6 rounded-full bg-foreground/10 group-hover/button:translate-x-0.5 transition-transform duration-300">
                          <ArrowRight className="h-3 w-3" />
                        </span>
                      </>
                    )}
                  </Button>
                </div>
                {downloading && (
                  <div className="mt-5 space-y-1.5">
                    <Progress value={downloadProgress} />
                    <p className="text-xs text-muted-foreground text-right font-mono tabular-nums">
                      {downloadProgress}%
                    </p>
                  </div>
                )}
              </div>
            </Bezel>
          </AnimateEntry>
        )}

        {stats && (
          <AnimateEntry delay={150}>
            <Bezel radius="xl">
              <div className="p-7">
                <div className="flex items-center gap-2 mb-6">
                  <BarChart3 className="h-3.5 w-3.5 text-muted-foreground" />
                  <span className="text-xs uppercase tracking-[0.15em] font-medium text-muted-foreground">
                    作品统计
                  </span>
                </div>
                <div className="grid grid-cols-4 gap-6">
                  <div className="text-center">
                    <ThumbsUp className="h-4 w-4 text-muted-foreground mx-auto mb-2" />
                    <p className="text-xl font-heading font-bold tabular-nums">{formatCount(stats.digg_count)}</p>
                    <p className="text-xs text-muted-foreground mt-0.5 tracking-wide">点赞</p>
                  </div>
                  <div className="text-center">
                    <MessageSquare className="h-4 w-4 text-muted-foreground mx-auto mb-2" />
                    <p className="text-xl font-heading font-bold tabular-nums">{formatCount(stats.comment_count)}</p>
                    <p className="text-xs text-muted-foreground mt-0.5 tracking-wide">评论</p>
                  </div>
                  <div className="text-center">
                    <Share2 className="h-4 w-4 text-muted-foreground mx-auto mb-2" />
                    <p className="text-xl font-heading font-bold tabular-nums">{formatCount(stats.share_count)}</p>
                    <p className="text-xs text-muted-foreground mt-0.5 tracking-wide">分享</p>
                  </div>
                  <div className="text-center">
                    <Bookmark className="h-4 w-4 text-muted-foreground mx-auto mb-2" />
                    <p className="text-xl font-heading font-bold tabular-nums">{formatCount(stats.collect_count)}</p>
                    <p className="text-xs text-muted-foreground mt-0.5 tracking-wide">收藏</p>
                  </div>
                </div>
              </div>
            </Bezel>
          </AnimateEntry>
        )}
      </div>
    </>
  );
}
