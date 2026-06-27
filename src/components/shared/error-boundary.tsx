import { ErrorBoundary as ReactErrorBoundary, type FallbackProps } from "react-error-boundary";
import { AlertCircle, RotateCcw } from "lucide-react";
import { Button } from "@/components/ui/button";

function DefaultFallback({ error, resetErrorBoundary }: FallbackProps) {
  const msg = error instanceof Error ? error.message : String(error ?? "未知错误");
  return (
    <div className="flex flex-col items-center justify-center gap-4 p-8 m-4 rounded-2xl bg-destructive/[0.04] ring-1 ring-destructive/15">
      <AlertCircle className="h-10 w-10 text-destructive/60" />
      <div className="text-center space-y-1">
        <p className="text-sm font-medium text-foreground">页面渲染出错</p>
        <p className="text-xs text-muted-foreground max-w-md">{msg}</p>
      </div>
      <Button
        variant="outline"
        size="sm"
        onClick={resetErrorBoundary}
        className="gap-1.5"
      >
        <RotateCcw className="h-3.5 w-3.5" />
        重试
      </Button>
    </div>
  );
}

/**
 * React Error Boundary — 捕获子组件渲染错误，显示降级 UI 而非白屏。
 * 基于 react-error-boundary，保留原有默认降级 UI。
 */
export function ErrorBoundary({
  children,
  fallback,
}: {
  children: React.ReactNode;
  fallback?: React.ReactNode;
}) {
  if (fallback) {
    return (
      <ReactErrorBoundary
        fallback={fallback}
        onError={(error, info) =>
          console.error("[ErrorBoundary] 捕获渲染错误:", error, info.componentStack)
        }
      >
        {children}
      </ReactErrorBoundary>
    );
  }
  return (
    <ReactErrorBoundary
      fallbackRender={DefaultFallback}
      onError={(error, info) =>
        console.error("[ErrorBoundary] 捕获渲染错误:", error, info.componentStack)
      }
    >
      {children}
    </ReactErrorBoundary>
  );
}
