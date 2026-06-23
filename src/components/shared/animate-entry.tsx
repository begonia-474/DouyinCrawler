import { type ReactNode } from "react";
import { useInView } from "@/lib/use-in-view";
import { cn } from "@/lib/utils";

interface AnimateEntryProps {
  children: ReactNode;
  className?: string;
  delay?: number;
}

export function AnimateEntry({ children, className, delay = 0 }: AnimateEntryProps) {
  const { ref, inView } = useInView();

  return (
    <div
      ref={ref}
      className={cn(
        "transition-all duration-700",
        inView
          ? "opacity-100 translate-y-0 blur-0"
          : "opacity-0 translate-y-6 blur-sm",
        className
      )}
      style={{
        transitionTimingFunction: "cubic-bezier(0.32, 0.72, 0, 1)",
        transitionDelay: `${delay}ms`,
      }}
    >
      {children}
    </div>
  );
}
