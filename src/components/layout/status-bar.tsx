import { useEffect } from "react";
import { Circle } from "lucide-react";
import { useAppStore } from "@/stores/app-store";
import { healthCheck } from "@/lib/api";
import { cn } from "@/lib/utils";

export function StatusBar() {
  const { connected, downloadCount, setConnected } = useAppStore();

  useEffect(() => {
    const check = async () => {
      try {
        const ok = await healthCheck();
        setConnected(ok);
      } catch {
        setConnected(false);
      }
    };
    check();
    const timer = setInterval(check, 10000);
    return () => clearInterval(timer);
  }, [setConnected]);

  return (
    <div className="h-8 border-t border-border/60 bg-foreground/[0.02] flex items-center px-6 text-xs text-muted-foreground gap-4">
      <div className="flex items-center gap-1.5">
        <Circle
          className={`h-1.5 w-1.5 fill-current transition-colors duration-500 ${
            connected ? "text-success" : "text-muted-foreground/40"
          }`}
        />
        <span className={cn("transition-colors duration-500", connected && "text-success")}>
          {connected ? "已连接" : "未连接"}
        </span>
      </div>
      <div className="flex-1" />
      <span className="font-mono text-[11px] tabular-nums tracking-tight">{downloadCount}</span>
      <span className="text-[11px]">已下载</span>
    </div>
  );
}
