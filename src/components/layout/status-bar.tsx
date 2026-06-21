import { useEffect } from "react";
import { Circle } from "lucide-react";
import { useAppStore } from "@/stores/app-store";
import { healthCheck } from "@/lib/api";

export function StatusBar() {
  const { connected, downloadCount, setConnected } = useAppStore();

  useEffect(() => {
    const check = async () => {
      const res = await healthCheck();
      setConnected(res.success);
    };
    check();
    const timer = setInterval(check, 10000);
    return () => clearInterval(timer);
  }, [setConnected]);

  return (
    <div className="h-8 border-t bg-muted/50 flex items-center px-4 text-xs text-muted-foreground gap-4">
      <div className="flex items-center gap-1.5">
        <Circle
          className={`h-2 w-2 fill-current ${
            connected ? "text-emerald-500" : "text-muted-foreground/50"
          }`}
        />
        <span>{connected ? "已连接" : "未连接"}</span>
      </div>
      <div className="flex-1" />
      <span>已下载 {downloadCount} 个文件</span>
    </div>
  );
}
