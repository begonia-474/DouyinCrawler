import { Progress } from "@/components/ui/progress";
import { Bezel } from "@/components/shared/bezel";

interface DownloadProgressOverlayProps {
  progress: number;
  current: number;
  total: number;
  className?: string;
}

/** 统一下载进度遮罩 — 供 likes/user/mix/favorites 页面复用 */
export function DownloadProgressOverlay({
  progress,
  current,
  total,
  className = "",
}: DownloadProgressOverlayProps) {
  return (
    <Bezel radius="xl" className={className}>
      <div className="p-5">
        <div className="space-y-1">
          <Progress value={progress} />
          <p className="text-xs text-muted-foreground tracking-wide text-right">
            {current} / {total}
          </p>
        </div>
      </div>
    </Bezel>
  );
}
