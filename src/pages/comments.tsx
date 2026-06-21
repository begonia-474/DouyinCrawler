import { useState } from "react";
import { Header } from "@/components/layout/header";
import { UrlInput } from "@/components/shared/url-input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import {
  Heart,
  MessageSquare,
  ChevronDown,
  ChevronUp,
  ThumbsUp,
  Share2,
  Bookmark,
  Play,
  Loader2,
} from "lucide-react";

interface Comment {
  id: string;
  user: string;
  avatar: string;
  content: string;
  likeCount: number;
  time: string;
  replyCount: number;
  replies?: Comment[];
}

interface PostStats {
  awemeId: string;
  title: string;
  author: string;
  diggCount: number;
  commentCount: number;
  shareCount: number;
  collectCount: number;
  playCount: number;
}

const mockComments: Comment[] = [
  {
    id: "1",
    user: "用户A",
    avatar: "",
    content: "这个视频太棒了！",
    likeCount: 1234,
    time: "2小时前",
    replyCount: 3,
    replies: [
      {
        id: "1-1",
        user: "用户B",
        avatar: "",
        content: "确实很棒！",
        likeCount: 56,
        time: "1小时前",
        replyCount: 0,
      },
      {
        id: "1-2",
        user: "用户C",
        avatar: "",
        content: "同感！",
        likeCount: 23,
        time: "45分钟前",
        replyCount: 0,
      },
    ],
  },
  {
    id: "2",
    user: "用户D",
    avatar: "",
    content: "学到了很多，感谢分享！",
    likeCount: 890,
    time: "3小时前",
    replyCount: 0,
  },
  {
    id: "3",
    user: "用户E",
    avatar: "",
    content: "请问这是在哪里拍的？好想去！",
    likeCount: 456,
    time: "5小时前",
    replyCount: 1,
    replies: [
      {
        id: "3-1",
        user: "作者",
        avatar: "",
        content: "在云南大理，推荐你去！",
        likeCount: 120,
        time: "4小时前",
        replyCount: 0,
      },
    ],
  },
];

function formatCount(n: number): string {
  if (n >= 10000) return `${(n / 10000).toFixed(1)}w`;
  return n.toLocaleString();
}

function CommentItem({ comment }: { comment: Comment }) {
  const [showReplies, setShowReplies] = useState(false);

  return (
    <div className="space-y-3">
      <div className="flex gap-3">
        <Avatar className="h-8 w-8 shrink-0">
          <AvatarImage src={comment.avatar} />
          <AvatarFallback>{comment.user[0]}</AvatarFallback>
        </Avatar>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium">{comment.user}</span>
            <span className="text-xs text-muted-foreground">{comment.time}</span>
          </div>
          <p className="text-sm mt-1">{comment.content}</p>
          <div className="flex items-center gap-4 mt-2">
            <button className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors">
              <Heart className="h-3.5 w-3.5" />
              {formatCount(comment.likeCount)}
            </button>
            {comment.replyCount > 0 && (
              <button
                className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors"
                onClick={() => setShowReplies(!showReplies)}
              >
                <MessageSquare className="h-3.5 w-3.5" />
                {comment.replyCount} 回复
                {showReplies ? (
                  <ChevronUp className="h-3 w-3" />
                ) : (
                  <ChevronDown className="h-3 w-3" />
                )}
              </button>
            )}
          </div>
        </div>
      </div>

      {showReplies && comment.replies && comment.replies.length > 0 && (
        <div className="ml-11 space-y-3 border-l-2 pl-4">
          {comment.replies.map((reply) => (
            <CommentItem key={reply.id} comment={reply} />
          ))}
        </div>
      )}
    </div>
  );
}

export function CommentsPage() {
  const [loading, setLoading] = useState(false);
  const [comments, setComments] = useState<Comment[]>([]);
  const [postStats, setPostStats] = useState<PostStats | null>(null);
  const [hasMore, setHasMore] = useState(true);

  const handleParse = async (_url: string) => {
    setLoading(true);
    setComments([]);
    setPostStats(null);
    setHasMore(true);

    setTimeout(() => {
      setPostStats({
        awemeId: "1234567890",
        title: "旅行中的美好瞬间",
        author: "旅行达人",
        diggCount: 52000,
        commentCount: 3200,
        shareCount: 1800,
        collectCount: 8900,
        playCount: 892000,
      });
      setComments(mockComments);
      setLoading(false);
    }, 1000);
  };

  const handleLoadMore = async () => {
    setLoading(true);
    setTimeout(() => {
      setComments((prev) => [
        ...prev,
        {
          id: `more-${Date.now()}`,
          user: "新用户",
          avatar: "",
          content: "更多评论加载中...",
          likeCount: 12,
          time: "刚刚",
          replyCount: 0,
        },
      ]);
      setHasMore(false);
      setLoading(false);
    }, 800);
  };

  return (
    <>
      <Header title="评论" description="查看视频评论、回复和作品统计" />

      <div className="space-y-6">
        <UrlInput
          onSubmit={handleParse}
          loading={loading}
          placeholder="粘贴视频链接查看评论..."
        />

        {loading && comments.length === 0 && (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        )}

        {/* 作品统计 */}
        {postStats && (
          <Card>
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <CardTitle className="text-base">{postStats.title}</CardTitle>
                <Badge variant="secondary">{postStats.author}</Badge>
              </div>
            </CardHeader>
            <CardContent className="pt-0">
              <div className="grid grid-cols-5 gap-3">
                <div className="text-center p-2 rounded-lg bg-muted/50">
                  <Play className="h-3.5 w-3.5 text-muted-foreground mx-auto mb-1" />
                  <p className="text-sm font-semibold">{formatCount(postStats.playCount)}</p>
                  <p className="text-xs text-muted-foreground">播放</p>
                </div>
                <div className="text-center p-2 rounded-lg bg-muted/50">
                  <ThumbsUp className="h-3.5 w-3.5 text-muted-foreground mx-auto mb-1" />
                  <p className="text-sm font-semibold">{formatCount(postStats.diggCount)}</p>
                  <p className="text-xs text-muted-foreground">点赞</p>
                </div>
                <div className="text-center p-2 rounded-lg bg-muted/50">
                  <MessageSquare className="h-3.5 w-3.5 text-muted-foreground mx-auto mb-1" />
                  <p className="text-sm font-semibold">{formatCount(postStats.commentCount)}</p>
                  <p className="text-xs text-muted-foreground">评论</p>
                </div>
                <div className="text-center p-2 rounded-lg bg-muted/50">
                  <Share2 className="h-3.5 w-3.5 text-muted-foreground mx-auto mb-1" />
                  <p className="text-sm font-semibold">{formatCount(postStats.shareCount)}</p>
                  <p className="text-xs text-muted-foreground">分享</p>
                </div>
                <div className="text-center p-2 rounded-lg bg-muted/50">
                  <Bookmark className="h-3.5 w-3.5 text-muted-foreground mx-auto mb-1" />
                  <p className="text-sm font-semibold">{formatCount(postStats.collectCount)}</p>
                  <p className="text-xs text-muted-foreground">收藏</p>
                </div>
              </div>
            </CardContent>
          </Card>
        )}

        {/* 评论列表 */}
        {comments.length > 0 ? (
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-medium text-muted-foreground flex items-center gap-2">
                <MessageSquare className="h-4 w-4" />
                评论 ({formatCount(postStats?.commentCount ?? comments.length)})
              </h3>
            </div>

            {comments.map((comment) => (
              <Card key={comment.id}>
                <CardContent className="p-4">
                  <CommentItem comment={comment} />
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
                {loading ? (
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                ) : null}
                加载更多评论
              </Button>
            )}
          </div>
        ) : (
          !loading && (
            <Card>
              <CardContent className="p-8 text-center text-muted-foreground">
                输入视频链接查看评论和作品统计
              </CardContent>
            </Card>
          )
        )}
      </div>
    </>
  );
}
