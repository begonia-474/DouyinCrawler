import { useState, useCallback, useEffect, useRef } from "react";
import { useParams, useLocation, useNavigate } from "react-router-dom";
import { Header } from "@/components/layout/header";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { VideoCard } from "@/components/shared/video-card";
import { InfiniteScrollSentinel } from "@/components/shared/infinite-scroll-sentinel";
import { LoadingSpinner } from "@/components/shared/loading-spinner";
import { ErrorBanner } from "@/components/shared/error-banner";
import { Bezel } from "@/components/shared/bezel";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";
import { getPostDetail } from "@/lib/api/video";
import { getComments, getCommentReplies } from "@/lib/api/comment";
import { getRelated } from "@/lib/api/related";
import { startDownload } from "@/lib/api/download";
import { formatCount, formatDurationSec, formatTimestamp } from "@/lib/utils";
import type { PostDetail, VideoItem, CommentItem } from "@/lib/api-types";
import {
  Download, CheckCircle2, ThumbsUp, MessageSquare,
  Share2, Bookmark, BarChart3, ArrowRight, Heart,
  ChevronDown, Loader2, Compass,
} from "lucide-react";

interface LocationState {
  from?: string;
  fromPath?: string;
}

// ============================================================
// 评论回复组件
// ============================================================

function ReplyItem({ reply }: { reply: CommentItem }) {
  return (
    <div className="flex gap-2">
      <Avatar className="h-6 w-6 shrink-0">
        <AvatarImage src={reply.user?.avatar} />
        <AvatarFallback className="text-[10px]">{reply.user?.nickname?.[0] || "?"}</AvatarFallback>
      </Avatar>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-[11px] font-medium">{reply.user?.nickname || "匿名"}</span>
          <span className="text-[10px] text-muted-foreground">{formatTimestamp(reply.create_time)}</span>
        </div>
        <p className="text-xs mt-0.5 leading-relaxed whitespace-pre-wrap break-words">{reply.text}</p>
        {reply.digg_count > 0 && (
          <span className="flex items-center gap-1 text-[10px] text-muted-foreground mt-1">
            <Heart className="h-2.5 w-2.5" />
            {reply.digg_count}
          </span>
        )}
      </div>
    </div>
  );
}

function CommentItemView({ comment, url }: { comment: CommentItem; url: string }) {
  const [replies, setReplies] = useState<CommentItem[]>([]);
  const [showReplies, setShowReplies] = useState(false);
  const [loadingReplies, setLoadingReplies] = useState(false);
  const [replyCursor, setReplyCursor] = useState(0);
  const [hasMoreReplies, setHasMoreReplies] = useState(false);

  const loadReplies = useCallback(async () => {
    if (replies.length > 0) {
      setShowReplies(!showReplies);
      return;
    }
    setLoadingReplies(true);
    const res = await getCommentReplies(url, comment.cid, 0, 10);
    if (res.success && res.data) {
      setReplies(res.data.comments ?? []);
      setReplyCursor(res.data.cursor ?? 0);
      setHasMoreReplies(res.data.has_more ?? false);
      setShowReplies(true);
    }
    setLoadingReplies(false);
  }, [url, comment.cid, replies.length, showReplies]);

  const loadMoreReplies = useCallback(async () => {
    setLoadingReplies(true);
    const res = await getCommentReplies(url, comment.cid, replyCursor, 10);
    if (res.success && res.data) {
      setReplies((prev) => [...prev, ...(res.data!.comments ?? [])]);
      setReplyCursor(res.data.cursor ?? 0);
      setHasMoreReplies(res.data.has_more ?? false);
    }
    setLoadingReplies(false);
  }, [url, comment.cid, replyCursor]);

  return (
    <div className="py-3">
      <div className="flex gap-3">
        <Avatar className="h-8 w-8 shrink-0">
          <AvatarImage src={comment.user?.avatar} />
          <AvatarFallback>{comment.user?.nickname?.[0] || "?"}</AvatarFallback>
        </Avatar>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="text-xs font-medium">{comment.user?.nickname || "匿名"}</span>
            <span className="text-[11px] text-muted-foreground">{formatTimestamp(comment.create_time)}</span>
          </div>
          <p className="text-sm mt-1 leading-relaxed whitespace-pre-wrap break-words">{comment.text}</p>
          <div className="flex items-center gap-4 mt-1.5">
            {comment.digg_count > 0 && (
              <span className="flex items-center gap-1 text-[11px] text-muted-foreground">
                <Heart className="h-3 w-3" />
                {comment.digg_count}
              </span>
            )}
            {comment.reply_comment_total > 0 && (
              <button
                className="flex items-center gap-1 text-[11px] text-muted-foreground hover:text-foreground transition-colors"
                onClick={loadReplies}
              >
                <MessageSquare className="h-3 w-3" />
                {comment.reply_comment_total} 回复
                {loadingReplies && <Loader2 className="h-3 w-3 animate-spin" />}
              </button>
            )}
          </div>

          {showReplies && replies.length > 0 && (
            <div className="mt-2 ml-1 pl-3 border-l-2 border-foreground/[0.06] space-y-2">
              {replies.map((reply) => (
                <ReplyItem key={reply.cid} reply={reply} />
              ))}
              {hasMoreReplies && (
                <button
                  className="flex items-center gap-1 text-[11px] text-primary hover:underline"
                  onClick={loadMoreReplies}
                  disabled={loadingReplies}
                >
                  {loadingReplies ? <Loader2 className="h-3 w-3 animate-spin" /> : <ChevronDown className="h-3 w-3" />}
                  展开更多回复
                </button>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

// ============================================================
// 评论 Tab
// ============================================================

function CommentsSection({ url }: { url: string }) {
  const [comments, setComments] = useState<CommentItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [cursor, setCursor] = useState(0);
  const [hasMore, setHasMore] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);

  useEffect(() => {
    if (!url) return;
    let cancelled = false;
    (async () => {
      setLoading(true);
      setError(null);
      const res = await getComments(url, 0, 20);
      if (cancelled) return;
      if (res.success && res.data) {
        setComments(res.data.comments ?? []);
        setCursor(res.data.cursor ?? 0);
        setHasMore(res.data.has_more ?? false);
      } else {
        setError(res.error || "获取评论失败");
      }
      setLoading(false);
    })();
    return () => { cancelled = true; };
  }, [url]);

  const handleLoadMore = useCallback(async () => {
    setLoadingMore(true);
    const res = await getComments(url, cursor, 20);
    if (res.success && res.data) {
      setComments((prev) => [...prev, ...(res.data!.comments ?? [])]);
      setCursor(res.data.cursor ?? 0);
      setHasMore(res.data.has_more ?? false);
    }
    setLoadingMore(false);
  }, [url, cursor]);

  if (loading) {
    return <LoadingSpinner text="加载评论中…" />;
  }

  if (error) {
    return <ErrorBanner message={error} />;
  }

  if (comments.length === 0) {
    return (
      <div className="py-12 text-center">
        <MessageSquare className="h-10 w-10 text-muted-foreground/30 mx-auto mb-3" />
        <p className="text-sm text-muted-foreground">暂无评论</p>
      </div>
    );
  }

  return (
    <div>
      {comments.map((comment) => (
        <CommentItemView key={comment.cid} comment={comment} url={url} />
      ))}
      {hasMore && (
        <div className="py-3 text-center">
          <Button variant="capsule" size="sm" onClick={handleLoadMore} disabled={loadingMore}>
            {loadingMore ? (
              <><Loader2 className="h-3.5 w-3.5 animate-spin mr-1" />加载中…</>
            ) : (
              <><ChevronDown className="h-3.5 w-3.5 mr-1" />加载更多</>
            )}
          </Button>
        </div>
      )}
    </div>
  );
}

// ============================================================
// 相关推荐 Tab
// ============================================================

function RelatedSection({ awemeId, onVideoClick }: { awemeId: string; onVideoClick: (video: VideoItem) => void }) {
  const url = `https://www.douyin.com/video/${awemeId}`;
  const seenIdsRef = useRef<Set<string>>(new Set());
  const [error, setError] = useState<string | null>(null);

  const { items: videos, hasMore, loadingMore, initialLoading, sentinelRef, reset } = useInfiniteScroll<VideoItem>({
    fetchPage: useCallback(async () => {
      const filterGids = seenIdsRef.current.size > 0 ? [...seenIdsRef.current].join(",") + "," : "";
      const res = await getRelated(url, 20, filterGids);
      if (res.success && res.data) {
        const newIds = (res.data.videos as unknown as VideoItem[])
          .map((v) => v.aweme_id)
          .filter(Boolean);
        newIds.forEach((id) => seenIdsRef.current.add(id));
        return {
          items: res.data.videos as unknown as VideoItem[],
          nextCursor: 0,
          hasMore: res.data.has_more ?? false,
        };
      }
      setError(res.error || "获取相关推荐失败");
      return null;
    }, [url]),
    enabled: true,
  });

  useEffect(() => {
    seenIdsRef.current = new Set();
    setError(null);
    reset();
  }, [awemeId, reset]);

  if (initialLoading) {
    return <LoadingSpinner text="正在加载相关推荐…" />;
  }

  if (error) {
    return <ErrorBanner message={error} />;
  }

  if (videos.length === 0) {
    return (
      <div className="py-12 text-center">
        <Compass className="h-10 w-10 text-muted-foreground/30 mx-auto mb-3" />
        <p className="text-sm text-muted-foreground">暂无相关推荐</p>
      </div>
    );
  }

  return (
    <div>
      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-5">
        {videos.map((video) => (
          <VideoCard
            key={video.aweme_id}
            title={video.desc}
            author={video.author}
            cover={video.cover_url}
            duration={formatDurationSec(video.duration)}
            diggCount={video.digg_count}
            commentCount={video.comment_count}
            shareCount={video.share_count}
            onClick={() => onVideoClick(video)}
            onDownload={() => startDownload("one", `https://www.douyin.com/video/${video.aweme_id}`)}
          />
        ))}
      </div>
      <InfiniteScrollSentinel
        sentinelRef={sentinelRef}
        loadingMore={loadingMore}
        hasMore={hasMore}
        total={videos.length}
        label="相关推荐"
      />
    </div>
  );
}

// ============================================================
// 视频详情页主组件
// ============================================================

export default function VideoDetailPage() {
  const { awemeId } = useParams<{ awemeId: string }>();
  const location = useLocation();
  const navigate = useNavigate();
  const state = (location.state ?? {}) as LocationState;

  const [detail, setDetail] = useState<PostDetail | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [downloaded, setDownloaded] = useState(false);

  const url = awemeId ? `https://www.douyin.com/video/${awemeId}` : "";
  const parentLabel = state.from ?? "首页";
  const parentPath = state.fromPath ?? "/douyin";

  // 加载视频信息
  useEffect(() => {
    if (!url) return;
    let cancelled = false;
    (async () => {
      setLoading(true);
      setError(null);
      const res = await getPostDetail(url);
      if (cancelled) return;
      if (res.success && res.data?.detail) {
        setDetail(res.data.detail);
      } else {
        setError(res.error || "获取视频信息失败");
      }
      setLoading(false);
    })();
    return () => { cancelled = true; };
  }, [url]);

  // 下载
  const handleDownload = useCallback(async () => {
    if (!url) return;
    setDownloading(true);
    const res = await startDownload("one", url);
    if (res.success) {
      setDownloaded(true);
    }
    setDownloading(false);
  }, [url]);

  // 相关推荐点击 → 导航到新详情页
  const handleRelatedClick = useCallback(
    (video: VideoItem) => {
      navigate(`/douyin/video/${video.aweme_id}`, {
        state: { from: "视频详情", fromPath: `/douyin/video/${awemeId}` },
      });
    },
    [navigate, awemeId]
  );

  if (!awemeId) {
    return (
      <>
        <AnimateEntry>
          <Header title="视频详情" parent={{ label: parentLabel, path: parentPath }} />
        </AnimateEntry>
        <div className="py-16 text-center text-muted-foreground text-sm">
          无效的视频 ID
        </div>
      </>
    );
  }

  return (
    <>
      <AnimateEntry>
        <Header title="视频详情" parent={{ label: parentLabel, path: parentPath }} />
      </AnimateEntry>

      <div className="space-y-6">
        {error && <ErrorBanner message={error} />}

        {loading && <LoadingSpinner text="加载视频信息…" />}

        {detail && (
          <AnimateEntry>
            <Bezel radius="xl">
              <div className="p-7">
                <div className="flex items-start justify-between">
                  <div className="space-y-2">
                    <div className="flex items-center gap-2">
                      <Badge variant={detail.type === "video" ? "default" : "secondary"} className="text-[11px] tracking-wide rounded-full">
                        {detail.type === "video" ? "视频" : "图文"}
                      </Badge>
                      <h3 className="font-heading text-xl font-semibold tracking-tight">{detail.desc || detail.title}</h3>
                    </div>
                    <p className="text-sm text-muted-foreground tracking-wide">
                      {detail.author}{detail.duration ? ` · ${formatDurationSec(detail.duration)}` : ""}
                      {detail.aweme_id && ` · ID: ${detail.aweme_id}`}
                    </p>
                  </div>
                  <Button onClick={handleDownload} disabled={downloading || downloaded} size="lg">
                    {downloaded ? (
                      <><CheckCircle2 className="h-4 w-4 mr-2" />已完成</>
                    ) : downloading ? (
                      <><Download className="h-4 w-4 mr-2 animate-pulse" />下载中...</>
                    ) : (
                      <>下载<span className="ml-2 inline-flex items-center justify-center w-6 h-6 rounded-full bg-foreground/10 group-hover/button:translate-x-0.5 transition-transform duration-300"><ArrowRight className="h-3 w-3" /></span></>
                    )}
                  </Button>
                </div>
              </div>
            </Bezel>
          </AnimateEntry>
        )}

        {/* 统计信息 */}
        {detail && (
          <AnimateEntry delay={50}>
            <Bezel radius="xl">
              <div className="p-7">
                <div className="flex items-center gap-2 mb-6">
                  <BarChart3 className="h-3.5 w-3.5 text-muted-foreground" />
                  <span className="text-xs uppercase tracking-[0.15em] font-medium text-muted-foreground">作品统计</span>
                </div>
                <div className="grid grid-cols-4 gap-6">
                  <div className="text-center">
                    <ThumbsUp className="h-4 w-4 text-muted-foreground mx-auto mb-2" />
                    <p className="text-xl font-heading font-bold tabular-nums">{formatCount(detail.digg_count ?? 0)}</p>
                    <p className="text-xs text-muted-foreground mt-0.5 tracking-wide">点赞</p>
                  </div>
                  <div className="text-center">
                    <MessageSquare className="h-4 w-4 text-muted-foreground mx-auto mb-2" />
                    <p className="text-xl font-heading font-bold tabular-nums">{formatCount(detail.comment_count ?? 0)}</p>
                    <p className="text-xs text-muted-foreground mt-0.5 tracking-wide">评论</p>
                  </div>
                  <div className="text-center">
                    <Share2 className="h-4 w-4 text-muted-foreground mx-auto mb-2" />
                    <p className="text-xl font-heading font-bold tabular-nums">{formatCount(detail.share_count ?? 0)}</p>
                    <p className="text-xs text-muted-foreground mt-0.5 tracking-wide">分享</p>
                  </div>
                  <div className="text-center">
                    <Bookmark className="h-4 w-4 text-muted-foreground mx-auto mb-2" />
                    <p className="text-xl font-heading font-bold tabular-nums">{formatCount(detail.collect_count ?? 0)}</p>
                    <p className="text-xs text-muted-foreground mt-0.5 tracking-wide">收藏</p>
                  </div>
                </div>
              </div>
            </Bezel>
          </AnimateEntry>
        )}

        {/* Tabs：评论 + 相关推荐 */}
        {detail && (
          <AnimateEntry delay={100}>
            <Tabs defaultValue="comments">
              <TabsList>
                <TabsTrigger value="comments">
                  评论<Badge variant="secondary" className="ml-1.5">{detail.comment_count ?? 0}</Badge>
                </TabsTrigger>
                <TabsTrigger value="related">相关推荐</TabsTrigger>
              </TabsList>

              <TabsContent value="comments" className="mt-6">
                <CommentsSection url={url} />
              </TabsContent>

              <TabsContent value="related" className="mt-6">
                <RelatedSection awemeId={awemeId} onVideoClick={handleRelatedClick} />
              </TabsContent>
            </Tabs>
          </AnimateEntry>
        )}
      </div>
    </>
  );
}
