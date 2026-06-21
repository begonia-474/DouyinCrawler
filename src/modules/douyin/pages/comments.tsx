import { useState, useCallback } from "react";
import { Header } from "@/components/layout/header";
import { UrlInput } from "@/components/shared/url-input";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { getComments, getPostDetail } from "@/lib/api";
import type { CommentItem } from "@/lib/api-types";
import {
  Heart,
  MessageSquare,
  ChevronDown,
  ChevronUp,
  Loader2,
  AlertCircle,
} from "lucide-react";

function formatCount(n: number): string {
  if (n >= 10000) return `${(n / 10000).toFixed(1)}w`;
  return n.toLocaleString();
}

function formatTime(ts: number): string {
  if (!ts) return "";
  const d = new Date(ts * 1000);
  const now = Date.now();
  const diff = now - d.getTime();
  if (diff < 60000) return "刚刚";
  if (diff < 3600000) return `${Math.floor(diff / 60000)}分钟前`;
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}小时前`;
  return `${Math.floor(diff / 86400000)}天前`;
}

function CommentCard({ comment }: { comment: CommentItem }) {
  const [showReplies, setShowReplies] = useState(false);

  return (
    <div className="space-y-3">
      <div className="flex gap-3">
        <Avatar className="h-8 w-8 shrink-0">
          <AvatarImage src={comment.user?.avatar} />
          <AvatarFallback>{comment.user?.nickname?.[0] || "?"}</AvatarFallback>
        </Avatar>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium">{comment.user?.nickname}</span>
            <span className="text-xs text-muted-foreground">{formatTime(comment.create_time)}</span>
          </div>
          <p className="text-sm mt-1">{comment.text}</p>
          <div className="flex items-center gap-4 mt-2">
            <Button variant="ghost" size="sm" className="h-auto px-1.5 py-0.5 text-xs text-muted-foreground">
              <Heart className="h-3.5 w-3.5" />
              {formatCount(comment.digg_count)}
            </Button>
            {comment.reply_comment_total > 0 && (
              <Button
                variant="ghost"
                size="sm"
                className="h-auto px-1.5 py-0.5 text-xs text-muted-foreground"
                onClick={() => setShowReplies(!showReplies)}
              >
                <MessageSquare className="h-3.5 w-3.5" />
                {comment.reply_comment_total} 回复
                {showReplies ? <ChevronUp className="h-3 w-3" /> : <ChevronDown className="h-3 w-3" />}
              </Button>
            )}
          </div>
        </div>
      </div>

      {showReplies && comment.replies && comment.replies.length > 0 && (
        <div className="ml-11 space-y-3 border-l-2 pl-4">
          {comment.replies.map((reply) => (
            <CommentCard key={reply.cid} comment={reply} />
          ))}
        </div>
      )}
    </div>
  );
}

export default function CommentsPage() {
  const [loading, setLoading] = useState(false);
  const [comments, setComments] = useState<CommentItem[]>([]);
  const [hasMore, setHasMore] = useState(false);
  const [cursor, setCursor] = useState(0);
  const [postUrl, setPostUrl] = useState("");
  const [postTitle, setPostTitle] = useState("");
  const [error, setError] = useState<string | null>(null);

  const handleParse = useCallback(async (url: string) => {
    setLoading(true);
    setComments([]);
    setPostUrl(url);
    setPostTitle("");
    setError(null);

    const [commentRes, detailRes] = await Promise.all([
      getComments(url),
      getPostDetail(url),
    ]);

    if (detailRes.success && detailRes.data?.detail) {
      setPostTitle(detailRes.data.detail.desc || "");
    }

    if (commentRes.success && commentRes.data) {
      setComments(commentRes.data.comments || []);
      setHasMore(commentRes.data.has_more || false);
      setCursor(commentRes.data.cursor || 0);
    } else {
      setError(commentRes.error || "获取评论失败");
    }
    setLoading(false);
  }, []);

  const handleLoadMore = async () => {
    if (!postUrl) return;
    setLoading(true);
    const res = await getComments(postUrl, cursor);
    if (res.success && res.data) {
      setComments((prev) => [...prev, ...(res.data?.comments || [])]);
      setHasMore(res.data?.has_more || false);
      setCursor(res.data?.cursor || 0);
    }
    setLoading(false);
  };

  return (
    <>
      <Header title="评论" description="查看视频评论" />

      <div className="space-y-6">
        <UrlInput
          onSubmit={handleParse}
          loading={loading}
          placeholder="粘贴视频链接查看评论..."
        />

        {error && (
          <div className="flex items-center gap-2 p-3 rounded-lg bg-destructive/10 text-destructive text-sm">
            <AlertCircle className="h-4 w-4 shrink-0" />
            <span>{error}</span>
          </div>
        )}

        {loading && comments.length === 0 && (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        )}

        {postTitle && (
          <p className="text-sm text-muted-foreground">视频：{postTitle}</p>
        )}

        {comments.length > 0 ? (
          <div className="space-y-4">
            <h3 className="text-sm font-medium text-muted-foreground flex items-center gap-2">
              <MessageSquare className="h-4 w-4" />
              评论 ({comments.length})
            </h3>

            {comments.map((comment) => (
              <Card key={comment.cid}>
                <CardContent className="p-4">
                  <CommentCard comment={comment} />
                </CardContent>
              </Card>
            ))}

            {hasMore && (
              <Button
                variant="outline"
                className="w-full"
                onClick={handleLoadMore}
                disabled={loading}
              >
                {loading ? <Loader2 className="h-4 w-4 mr-2 animate-spin" /> : null}
                加载更多评论
              </Button>
            )}
          </div>
        ) : (
          !loading && (
            <Card>
              <CardContent className="p-8 text-center text-muted-foreground">
                <MessageSquare className="h-10 w-10 mx-auto mb-3" />
                <p>输入视频链接查看评论</p>
              </CardContent>
            </Card>
          )
        )}
      </div>
    </>
  );
}
