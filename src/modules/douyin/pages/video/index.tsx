import { useState, useCallback, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { Header } from "@/components/layout/header";
import { UrlInput } from "@/components/shared/url-input";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Badge } from "@/components/ui/badge";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { Bezel } from "@/components/shared/bezel";
import { ErrorBanner } from "@/components/shared/error-banner";
import { LoadingSpinner } from "@/components/shared/loading-spinner";
import { startDownload } from "@/lib/api";
import { useVideoParseQuery } from "@/lib/queries";
import {
  Download,
  CheckCircle2,
  ThumbsUp,
  MessageSquare,
  Share2,
  Bookmark,
  BarChart3,
  ArrowRight,
  Compass,
} from "lucide-react";
import { formatCount } from "@/lib/utils";
import { CommentDialog } from "@/components/shared/comment-dialog";

interface ParsedInfo {
  type: string;
  title?: string;
  author?: string;
  duration?: string;
  awemeId?: string;
}

export default function VideoPage() {
  const navigate = useNavigate();
  const [submittedUrl, setSubmittedUrl] = useState<string | null>(null);
  const [downloadUrl, setDownloadUrl] = useState("");
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [downloading, setDownloading] = useState(false);
  const [downloaded, setDownloaded] = useState(false);
  const [commentAwemeId, setCommentAwemeId] = useState<string | null>(null);

  // React Query: 解析结果按 URL 缓存，路由切换不丢失
  const { data: parseResult, isLoading, error: parseError } = useVideoParseQuery(submittedUrl);

  // 从 React Query 结果派生 ParsedInfo + stats
  const { parsed, stats } = useMemo(() => {
    if (!parseResult?.success || !parseResult.data?.detail) {
      return { parsed: null, stats: null };
    }
    const detail = parseResult.data.detail;
    return {
      parsed: {
        type: detail.type || "video",
        title: detail.desc || detail.title,
        author: detail.author,
        awemeId: detail.aweme_id || detail.awemeId,
      } as ParsedInfo,
      stats: {
        digg_count: detail.digg_count ?? 0,
        comment_count: detail.comment_count ?? 0,
        share_count: detail.share_count ?? 0,
        collect_count: detail.collect_count ?? 0,
      },
    };
  }, [parseResult]);

  const handleParse = useCallback(async (url: string) => {
    setSubmittedUrl(null);  // 重置以触发新请求
    setDownloadUrl(url);
    setDownloaded(false);
    // 使用 setTimeout 确保 React 状态已重置，然后触发新查询
    setTimeout(() => setSubmittedUrl(url), 0);
  }, []);

  const error = parseResult && !parseResult.success
    ? (parseResult.error || "解析失败")
    : parseError
      ? (parseError instanceof Error ? parseError.message : "请求异常")
      : null;

  const handleDownload = useCallback(async () => {
    if (!downloadUrl) return;
    setDownloading(true);
    setDownloadProgress(0);

    const res = await startDownload("one", downloadUrl);

    if (res.success) {
      setDownloaded(true);
      setDownloadProgress(100);
    } else {
      // download error — shown inline, not replacing parse result
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
          <UrlInput onSubmit={handleParse} loading={isLoading} allowedTypes={["video"]} autoDetect />
        </AnimateEntry>

        {error && (
          <AnimateEntry>
            <ErrorBanner message={error} />
          </AnimateEntry>
        )}

        {isLoading && <LoadingSpinner text="解析中…" />}

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
                  <div
                    className="text-center cursor-pointer hover:bg-foreground/[0.03] rounded-lg py-1 -my-1 transition-colors"
                    onClick={() => parsed?.awemeId && setCommentAwemeId(parsed.awemeId)}
                  >
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

        {parsed?.awemeId && (
          <AnimateEntry delay={200}>
            <Bezel radius="xl">
              <div
                className="p-7 flex items-center justify-between cursor-pointer hover:bg-foreground/[0.02] transition-colors"
                onClick={() => {
                  const videoUrl = `https://www.douyin.com/video/${parsed.awemeId}`;
                  navigate(`/douyin/related?url=${encodeURIComponent(videoUrl)}&aweme_id=${parsed.awemeId}`);
                }}
              >
                <div className="flex items-center gap-3">
                  <Compass className="h-5 w-5 text-muted-foreground" />
                  <div>
                    <h4 className="text-sm font-medium">相关推荐</h4>
                    <p className="text-xs text-muted-foreground tracking-wide">查看基于此视频推荐的相似作品</p>
                  </div>
                </div>
                <ArrowRight className="h-4 w-4 text-muted-foreground" />
              </div>
            </Bezel>
          </AnimateEntry>
        )}
      </div>

      <CommentDialog
        awemeId={commentAwemeId ?? ""}
        open={!!commentAwemeId}
        onOpenChange={(open) => !open && setCommentAwemeId(null)}
      />
    </>
  );
}
