import { useState, useCallback, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { Header } from "@/components/layout/header";
import { VideoCard } from "@/components/shared/video-card";
import { Button } from "@/components/ui/button";
import { Bezel } from "@/components/shared/bezel";
import { getCollectsVideoList, downloadCollectsVideo } from "@/lib/api";
import type { VideoItem } from "@/lib/api-types";
import { Loader2, AlertCircle, Download, ArrowLeft } from "lucide-react";
import { Progress } from "@/components/ui/progress";

function formatDuration(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
}

export default function CollectsDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [loading, setLoading] = useState(false);
  const [videos, setVideos] = useState<VideoItem[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [downloadedCount, setDownloadedCount] = useState(0);
  const [downloadProgress, setDownloadProgress] = useState(0);

  const fetchVideos = useCallback(async () => {
    if (!id) return;
    setLoading(true);
    setError(null);

    const res = await getCollectsVideoList(id);
    if (res.success && res.data?.videos) {
      setVideos(res.data.videos);
    } else {
      setError(res.error || "获取收藏夹视频失败");
    }

    setLoading(false);
  }, [id]);

  useEffect(() => {
    fetchVideos();
  }, [fetchVideos]);

  const handleDownloadAll = async () => {
    if (!id) return;
    setDownloading(true);
    setDownloadProgress(0);
    setDownloadedCount(0);

    const res = await downloadCollectsVideo(id);
    if (res.success) {
      setDownloadedCount(videos.length);
      setDownloadProgress(100);
    } else {
      setError(res.error || "下载失败");
    }
    setDownloading(false);
  };

  return (
    <>
      <Header title="收藏夹详情" description={`共 ${videos.length} 个视频`} parent={{ label: "我的收藏", path: "/douyin/favorites" }}>
        <div className="flex gap-2">
          <Button variant="capsule" size="sm" onClick={() => navigate("/douyin/favorites")}>
            <ArrowLeft className="h-4 w-4 mr-1" />
            返回
          </Button>
          {videos.length > 0 && (
            <Button size="sm" onClick={handleDownloadAll} disabled={downloading}>
              {downloading ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <Download className="h-4 w-4 mr-1" />}
              {downloading ? `下载中 ${downloadedCount}/${videos.length}` : "全部下载"}
            </Button>
          )}
        </div>
      </Header>

      <div className="space-y-6">
        {error && (
          <div className="flex items-center gap-2 p-4 rounded-2xl bg-destructive/[0.06] ring-1 ring-destructive/20 text-destructive text-sm">
            <AlertCircle className="h-4 w-4 shrink-0" />
            <span>{error}</span>
          </div>
        )}

        {loading && (
          <div className="flex items-center justify-center py-16">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        )}

        {downloading && (
          <Bezel radius="xl">
            <div className="p-5">
              <div className="space-y-1">
                <Progress value={downloadProgress} />
                <p className="text-xs text-muted-foreground text-right">{downloadedCount} / {videos.length}</p>
              </div>
            </div>
          </Bezel>
        )}

        {!loading && videos.length === 0 && !error && (
          <Bezel radius="xl">
            <div className="p-12 text-center">
              <p className="text-muted-foreground">暂无视频</p>
            </div>
          </Bezel>
        )}

        {!loading && videos.length > 0 && (
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-5">
            {videos.map((video) => (
              <VideoCard
                key={video.aweme_id}
                title={video.desc}
                author={video.author}
                duration={formatDuration(video.duration)}
                diggCount={video.digg_count}
                commentCount={video.comment_count}
                shareCount={video.share_count}
              />
            ))}
          </div>
        )}
      </div>
    </>
  );
}
