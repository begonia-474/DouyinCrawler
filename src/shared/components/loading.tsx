import { Skeleton } from "@/components/ui/skeleton";

export function Loading() {
  return (
    <div className="space-y-8 animate-in fade-in duration-300">
      {/* Header skeleton */}
      <div className="space-y-3">
        <Skeleton className="h-3 w-16 rounded-full" />
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-4 w-64" />
      </div>

      {/* Content skeleton — generic card grid */}
      <div className="grid grid-cols-12 gap-5">
        <div className="col-span-8 space-y-3">
          <Skeleton className="h-32 rounded-[1.5rem]" />
        </div>
        <div className="col-span-4 space-y-3">
          <Skeleton className="h-32 rounded-[1.5rem]" />
        </div>
        <div className="col-span-4 space-y-3">
          <Skeleton className="h-28 rounded-[1.5rem]" />
        </div>
        <div className="col-span-4 space-y-3">
          <Skeleton className="h-28 rounded-[1.5rem]" />
        </div>
        <div className="col-span-4 space-y-3">
          <Skeleton className="h-28 rounded-[1.5rem]" />
        </div>
      </div>
    </div>
  );
}
