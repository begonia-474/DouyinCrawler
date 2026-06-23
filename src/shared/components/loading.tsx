export function Loading() {
  return (
    <div className="flex flex-col items-center justify-center h-full gap-4">
      <div className="relative h-12 w-12">
        <div className="absolute inset-0 rounded-full border-2 border-foreground/[0.06]" />
        <div className="absolute inset-0 rounded-full border-2 border-transparent border-t-brand animate-spin" />
      </div>
      <span className="text-xs text-muted-foreground tracking-widest uppercase">加载中</span>
    </div>
  );
}
