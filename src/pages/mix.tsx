import { useState } from "react";
import { Header } from "@/components/layout/header";
import { UrlInput } from "@/components/shared/url-input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import {
  Download,
  Layers,
  Loader2,
  CheckCircle2,
  ListVideo,
} from "lucide-react";

interface MixInfo {
  mixName: string;
  totalCount: number;
  videos: MixVideo[];
}

interface MixVideo {
  id: string;
  title: string;
  author: string;
  duration: string;
  diggCount: number;
  commentCount: number;
  index: number;
  downloaded?: boolean;
}

export function MixPage() {
  const [loading, setLoading] = useState(false);
  const [downloading, setDownloading] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [downloadedCount, setDownloadedCount] = useState(0);
  const [mixInfo, setMixInfo] = useState<MixInfo | null>(null);

  const handleParse = async (_url: string) => {
    setLoading(true);
    setTimeout(() => {
      setMixInfo({
        mixName: "旅行日记合集",
        totalCount: 5,
        videos: [
          { id: "1", title: "旅行日记 EP01 - 出发", author: "旅行达人", duration: "02:30", diggCount: 12000, commentCount: 450, index: 1 },
          { id: "2", title: "旅行日记 EP02 - 第一站", author: "旅行达人", duration: "03:15", diggCount: 9800, commentCount: 320, index: 2 },
          { id: "3", title: "旅行日记 EP03 - 美食", author: "旅行达人", duration: "01:45", diggCount: 15000, commentCount: 680, index: 3 },
          { id: "4", title: "旅行日记 EP04 - 风景", author: "旅行达人", duration: "04:00", diggCount: 8500, commentCount: 290, index: 4 },
          { id: "5", title: "旅行日记 EP05 - 回程", author: "旅行达人", duration: "02:10", diggCount: 7200, commentCount: 210, index: 5 },
        ],
      });
      setLoading(false);
    }, 1000);
  };

  const handleDownloadAll = async () => {
    if (!mixInfo) return;
    setDownloading(true);
    setDownloadProgress(0);
    setDownloadedCount(0);

    // 模拟逐个下载
    for (let i = 0; i < mixInfo.videos.length; i++) {
      await new Promise((r) => setTimeout(r, 800));
      setDownloadedCount(i + 1);
      setDownloadProgress(((i + 1) / mixInfo.videos.length) * 100);
      setMixInfo((prev) => {
        if (!prev) return prev;
        const videos = [...prev.videos];
        videos[i] = { ...videos[i], downloaded: true };
        return { ...prev, videos };
      });
    }
    setDownloading(false);
  };

  const handleDownloadOne = (videoId: string) => {
    setMixInfo((prev) => {
      if (!prev) return prev;
      const videos = prev.videos.map((v) =>
        v.id === videoId ? { ...v, downloaded: true } : v
      );
      return { ...prev, videos };
    });
  };

  return (
    <>
      <Header title="合集" description="下载整个合集/播放列表">
        {mixInfo && (
          <Button onClick={handleDownloadAll} disabled={downloading}>
            {downloading ? (
              <Loader2 className="h-4 w-4 mr-2 animate-spin" />
            ) : (
              <Download className="h-4 w-4 mr-2" />
            )}
            {downloading
              ? `下载中 ${downloadedCount}/${mixInfo.totalCount}`
              : "全部下载"}
          </Button>
        )}
      </Header>

      <div className="space-y-6">
        <UrlInput
          onSubmit={handleParse}
          loading={loading}
          placeholder="粘贴合集链接..."
        />

        {mixInfo && (
          <>
            <Card>
              <CardHeader>
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <div className="h-10 w-10 rounded-lg bg-primary/10 flex items-center justify-center">
                      <Layers className="h-5 w-5 text-primary" />
                    </div>
                    <div>
                      <CardTitle>{mixInfo.mixName}</CardTitle>
                      <p className="text-sm text-muted-foreground">
                        {mixInfo.totalCount} 个视频
                      </p>
                    </div>
                  </div>
                  <Badge variant="secondary">
                    <ListVideo className="h-3 w-3 mr-1" />
                    合集
                  </Badge>
                </div>
              </CardHeader>
              {downloading && (
                <CardContent className="pt-0">
                  <div className="space-y-1">
                    <Progress value={downloadProgress} />
                    <p className="text-xs text-muted-foreground text-right">
                      {downloadedCount} / {mixInfo.totalCount}
                    </p>
                  </div>
                </CardContent>
              )}
            </Card>

            <div className="space-y-3">
              {mixInfo.videos.map((video) => (
                <Card
                  key={video.id}
                  className={video.downloaded ? "border-emerald-200" : ""}
                >
                  <CardContent className="p-4 flex items-center gap-4">
                    <div className="h-8 w-8 rounded-full bg-muted flex items-center justify-center text-sm font-medium shrink-0">
                      {video.downloaded ? (
                        <CheckCircle2 className="h-4 w-4 text-emerald-500" />
                      ) : (
                        video.index
                      )}
                    </div>
                    <div className="flex-1 min-w-0">
                      <h4 className="text-sm font-medium truncate">
                        {video.title}
                      </h4>
                      <p className="text-xs text-muted-foreground">
                        {video.duration} · {video.diggCount > 10000
                          ? `${(video.diggCount / 10000).toFixed(1)}w`
                          : video.diggCount}{" "}
                        赞 · {video.commentCount} 评论
                      </p>
                    </div>
                    <Button
                      variant={video.downloaded ? "outline" : "default"}
                      size="sm"
                      onClick={() => handleDownloadOne(video.id)}
                      disabled={video.downloaded}
                    >
                      {video.downloaded ? (
                        <>
                          <CheckCircle2 className="h-3.5 w-3.5 mr-1" />
                          已下载
                        </>
                      ) : (
                        <>
                          <Download className="h-3.5 w-3.5 mr-1" />
                          下载
                        </>
                      )}
                    </Button>
                  </CardContent>
                </Card>
              ))}
            </div>
          </>
        )}
      </div>
    </>
  );
}
