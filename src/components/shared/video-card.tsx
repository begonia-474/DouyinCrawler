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
    <Card className="overflow-hidden group">
      <div className="aspect-video bg-muted relative">
        <div className="absolute inset-0 flex items-center justify-center">
          <Play className="h-10 w-10 text-muted-foreground/50" />
        </div>
        {duration && (
          <Badge
            variant="secondary"
            className="absolute bottom-2 right-2 text-xs"
          >
            {duration}
          </Badge>
        )}
        <div className="absolute inset-0 bg-black/0 group-hover:bg-black/20 transition-colors flex items-center justify-center opacity-0 group-hover:opacity-100">
          <Button
            size="sm"
            onClick={onDownload}
            className="shadow-lg"
          >
            <Download className="h-4 w-4 mr-1" />
            下载
          </Button>
        </div>
      </div>
      <CardContent className="p-3">
        <h4 className="text-sm font-medium line-clamp-2">{title}</h4>
        <p className="text-xs text-muted-foreground mt-1">{author}</p>
        <div className="flex items-center gap-3 mt-2 text-xs text-muted-foreground">
          {diggCount !== undefined && (
            <span className="flex items-center gap-1">
              <Heart className="h-3 w-3" />
              {diggCount > 10000
                ? `${(diggCount / 10000).toFixed(1)}w`
                : diggCount}
            </span>
          )}
          {commentCount !== undefined && (
            <span className="flex items-center gap-1">
              <MessageSquare className="h-3 w-3" />
              {commentCount}
            </span>
          )}
          {shareCount !== undefined && (
            <span className="flex items-center gap-1">
              <Share2 className="h-3 w-3" />
              {shareCount}
            </span>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
