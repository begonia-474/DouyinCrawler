import { Download, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";

interface DownloadAllButtonProps {
  downloading: boolean;
  downloadedCount: number;
  total: number;
  onClick: () => void;
  variant?: "capsule" | "default";
  size?: "sm" | "default";
  className?: string;
}

/** 统一下载全部按钮 — 供 likes/user/mix/favorites/music 页面复用 */
export function DownloadAllButton({
  downloading,
  downloadedCount,
  total,
  onClick,
  variant = "default",
  size = "default",
  className = "",
}: DownloadAllButtonProps) {
  return (
    <Button
      variant={variant}
      size={size}
      onClick={onClick}
      disabled={downloading}
      className={className}
    >
      {downloading ? (
        <Loader2 className="h-4 w-4 mr-2 animate-spin" />
      ) : (
        <Download className="h-4 w-4 mr-2" />
      )}
      {downloading ? `下载中 ${downloadedCount}/${total}` : "全部下载"}
    </Button>
  );
}
