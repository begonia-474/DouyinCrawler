import { Component, type ErrorInfo, type ReactNode } from "react";
import { AlertCircle, RotateCcw } from "lucide-react";
import { Button } from "@/components/ui/button";

interface ErrorBoundaryProps {
  children: ReactNode;
  /** 自定义降级 UI，不传则使用默认样式 */
  fallback?: ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

/**
 * React Error Boundary — 捕获子组件渲染错误，显示降级 UI 而非白屏。
 * 仅捕获渲染期间的同步错误；异步错误和事件处理器中的错误需用 try/catch。
 */
export class ErrorBoundary extends Component<
  ErrorBoundaryProps,
  ErrorBoundaryState
> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    console.error("[ErrorBoundary] 捕获渲染错误:", error, errorInfo);
  }

  private handleReset = (): void => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div className="flex flex-col items-center justify-center gap-4 p-8 m-4 rounded-2xl bg-destructive/[0.04] ring-1 ring-destructive/15">
          <AlertCircle className="h-10 w-10 text-destructive/60" />
          <div className="text-center space-y-1">
            <p className="text-sm font-medium text-foreground">
              页面渲染出错
            </p>
            <p className="text-xs text-muted-foreground max-w-md">
              {this.state.error?.message || "未知错误"}
            </p>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={this.handleReset}
            className="gap-1.5"
          >
            <RotateCcw className="h-3.5 w-3.5" />
            重试
          </Button>
        </div>
      );
    }

    return this.props.children;
  }
}
