import { Skeleton } from "@/components/ui/skeleton";

interface LoadingSpinnerProps {
  size?: number;
  className?: string;
  text?: string;
}

/** 统一加载占位 — 骨架屏风格 */
export function LoadingSpinner({ className = "", text }: LoadingSpinnerProps) {
  return (
    <div className={`flex flex-col items-center justify-center gap-4 py-12 ${className}`}>
      <div className="flex gap-2">
        <Skeleton className="h-2 w-2 rounded-full animate-pulse [animation-delay:0ms]" />
        <Skeleton className="h-2 w-2 rounded-full animate-pulse [animation-delay:150ms]" />
        <Skeleton className="h-2 w-2 rounded-full animate-pulse [animation-delay:300ms]" />
      </div>
      {text && <p className="text-sm text-muted-foreground tracking-wide">{text}</p>}
    </div>
  );
}
