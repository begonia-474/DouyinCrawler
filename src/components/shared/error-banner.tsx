import { AlertCircle, RefreshCw, Settings } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import type { ErrorCode } from "@/lib/api-types";
import { isRetryable, needsSettingsRedirect } from "@/lib/api-types";

interface ErrorBannerProps {
  message: string | null;
  className?: string;
  /** 错误码（对齐 Rust ErrorCode），用于差异化 UI 建议 */
  errorCode?: ErrorCode;
  /** 重试回调 */
  onRetry?: () => void;
  /** 跳转设置回调 */
  onGoToSettings?: () => void;
}

/** 统一错误提示横幅 — 支持按错误码差异化操作建议 */
export function ErrorBanner({ message, className, errorCode, onRetry, onGoToSettings }: ErrorBannerProps) {
  if (!message) return null;

  const showRetry = errorCode && isRetryable(errorCode) && onRetry;
  const showSettings = errorCode && needsSettingsRedirect(errorCode) && onGoToSettings;

  return (
    <div
      className={cn(
        "flex items-start gap-3 p-4 rounded-2xl bg-destructive/[0.06] ring-1 ring-destructive/20 text-destructive text-sm",
        className
      )}
    >
      <AlertCircle className="h-4 w-4 shrink-0 mt-0.5" />
      <div className="flex-1 min-w-0 space-y-2">
        <span className="block">{message}</span>
        {(showRetry || showSettings) && (
          <div className="flex gap-2">
            {showRetry && (
              <Button variant="outline" size="sm" onClick={onRetry}>
                <RefreshCw className="h-3 w-3 mr-1" />
                重试
              </Button>
            )}
            {showSettings && (
              <Button variant="outline" size="sm" onClick={onGoToSettings}>
                <Settings className="h-3 w-3 mr-1" />
                去设置
              </Button>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
