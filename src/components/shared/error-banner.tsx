import { AlertCircle } from "lucide-react";
import { cn } from "@/lib/utils";

interface ErrorBannerProps {
  message: string | null;
  className?: string;
}

/** 统一错误提示横幅 */
export function ErrorBanner({ message, className }: ErrorBannerProps) {
  if (!message) return null;

  return (
    <div
      className={cn(
        "flex items-center gap-2 p-4 rounded-2xl bg-destructive/[0.06] ring-1 ring-destructive/20 text-destructive text-sm",
        className
      )}
    >
      <AlertCircle className="h-4 w-4 shrink-0" />
      <span>{message}</span>
    </div>
  );
}
