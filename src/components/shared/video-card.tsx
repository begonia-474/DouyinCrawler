import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Download, Play, Heart, MessageSquare, Share2 } from "lucide-react";

interface VideoCardProps {
  title: string;
  author: string;
  cover?: string;
  duration?: string;
  diggCount?: number;
  commentCount?: number;
  shareCount?: number;
  onDownload?: () => void;
}

export function VideoCard({
  title,
  author,
  duration,
  diggCount,
  commentCount,
  shareCount,
  onDownload,
}: VideoCardProps) {
  return (
    <Card className="group overflow-hidden border-border/40 bg-card/60 hover:bg-card hover:border-border/60 hover:-translate-y-1 transition-all duration-500"
      style={{ transitionTimingFunction: "cubic-bezier(0.32, 0.72, 0, 1)" }}
    >
      <div className="aspect-video bg-foreground/[0.03] relative overflow-hidden">
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="h-14 w-14 rounded-full bg-foreground/[0.06] backdrop-blur-sm flex items-center justify-center">
            <Play className="h-6 w-6 text-muted-foreground/60" />
          </div>
        </div>
        {duration && (
          <Badge
            variant="secondary"
            className="absolute bottom-3 right-3 text-[11px] font-mono tabular-nums bg-foreground/60 text-background border-0 backdrop-blur-sm"
          >
            {duration}
          </Badge>
        )}
        <div className="absolute inset-0 bg-foreground/0 group-hover:bg-foreground/20 transition-all duration-500 flex items-center justify-center opacity-0 group-hover:opacity-100">
          <Button
            size="sm"
            onClick={onDownload}
            className="rounded-full px-5 shadow-lg bg-background text-foreground hover:bg-background/90"
          >
            <Download className="h-4 w-4 mr-1.5" />
            下载
          </Button>
        </div>
      </div>
      <CardContent className="p-4">
        <h4 className="text-sm font-medium line-clamp-2 leading-relaxed">{title}</h4>
        <p className="text-xs text-muted-foreground mt-1.5 tracking-wide">{author}</p>
        <div className="flex items-center gap-4 mt-3 text-xs text-muted-foreground">
          {diggCount !== undefined && (
            <span className="flex items-center gap-1 font-mono text-[11px] tabular-nums">
              <Heart className="h-3 w-3" />
              {diggCount > 10000
                ? `${(diggCount / 10000).toFixed(1)}w`
                : diggCount}
            </span>
          )}
          {commentCount !== undefined && (
            <span className="flex items-center gap-1 font-mono text-[11px] tabular-nums">
              <MessageSquare className="h-3 w-3" />
              {commentCount}
            </span>
          )}
          {shareCount !== undefined && (
            <span className="flex items-center gap-1 font-mono text-[11px] tabular-nums">
              <Share2 className="h-3 w-3" />
              {shareCount}
            </span>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
