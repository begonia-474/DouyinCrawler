import { useState, useEffect, useCallback } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { LoadingSpinner } from "@/components/shared/loading-spinner";
import { ErrorBanner } from "@/components/shared/error-banner";
import { getComments, getCommentReplies } from "@/lib/api";
import { formatTimestamp } from "@/lib/utils";
import type { CommentItem } from "@/lib/api-types";
import { Heart, MessageSquare, ChevronDown, Loader2 } from "lucide-react";

interface CommentDialogProps {
  awemeId: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
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
                <div key={reply.cid} className="flex gap-2">
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

export function CommentDialog({ awemeId, open, onOpenChange }: CommentDialogProps) {
  const [comments, setComments] = useState<CommentItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [cursor, setCursor] = useState(0);
  const [hasMore, setHasMore] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);

  const url = awemeId ? `https://www.douyin.com/video/${awemeId}` : "";

  // 打开时加载评论
  useEffect(() => {
    if (!open || !awemeId) {
      // 关闭时重置状态
      if (!open) {
        setComments([]);
        setCursor(0);
        setHasMore(false);
        setError(null);
      }
      return;
    }

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
  }, [open, awemeId, url]);

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

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg max-h-[80vh] flex flex-col">
        <DialogHeader>
          <DialogTitle>评论</DialogTitle>
        </DialogHeader>

        <div className="flex-1 overflow-y-auto -mx-4 px-4 min-h-0">
          {loading && (
            <div className="py-12">
              <LoadingSpinner text="加载评论中…" />
            </div>
          )}

          {error && <ErrorBanner message={error} />}

          {!loading && !error && comments.length === 0 && (
            <div className="py-12 text-center">
              <MessageSquare className="h-10 w-10 text-muted-foreground/30 mx-auto mb-3" />
              <p className="text-sm text-muted-foreground">暂无评论</p>
            </div>
          )}

          {comments.map((comment) => (
            <CommentItemView key={comment.cid} comment={comment} url={url} />
          ))}

          {hasMore && (
            <div className="py-3 text-center">
              <Button
                variant="capsule"
                size="sm"
                onClick={handleLoadMore}
                disabled={loadingMore}
              >
                {loadingMore ? (
                  <><Loader2 className="h-3.5 w-3.5 animate-spin mr-1" />加载中…</>
                ) : (
                  <><ChevronDown className="h-3.5 w-3.5 mr-1" />加载更多</>
                )}
              </Button>
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
