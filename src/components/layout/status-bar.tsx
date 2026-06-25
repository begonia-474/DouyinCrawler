import { Circle } from "lucide-react";
import { useAppStore } from "@/stores/app-store";

export function StatusBar() {
  const { downloadCount } = useAppStore();

  return (
    <div className="h-8 px-10 flex items-center text-xs text-muted-foreground gap-4">
      <div className="flex items-center gap-1.5">
        <Circle className="h-1.5 w-1.5 fill-current text-success" />
        <span className="text-success">已就绪</span>
      </div>
      <div className="flex-1" />
      <span className="font-mono text-[11px] tabular-nums tracking-tight">{downloadCount}</span>
      <span className="text-[11px]">已下载</span>
    </div>
  );
}
