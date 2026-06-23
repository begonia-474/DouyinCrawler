import { type ReactNode } from "react";
import { cn } from "@/lib/utils";

interface BezelProps {
  children: ReactNode;
  className?: string;
  /** Outer shell padding */
  padding?: "sm" | "md" | "lg";
  /** Outer border radius */
  radius?: "md" | "lg" | "xl" | "2xl";
  /** Make the inner core transparent */
  ghost?: boolean;
}

const paddingMap = { sm: "p-1", md: "p-1.5", lg: "p-2" };
const radiusMap = { md: "rounded-xl", lg: "rounded-2xl", xl: "rounded-[1.5rem]", "2xl": "rounded-[2rem]" };
const innerRadiusMap = { md: "rounded-[calc(theme(borderRadius.xl)-0.25rem)]", lg: "rounded-[calc(theme(borderRadius.2xl)-0.375rem)]", xl: "rounded-[calc(1.5rem-0.375rem)]", "2xl": "rounded-[calc(2rem-0.5rem)]" };

export function Bezel({
  children,
  className,
  padding = "md",
  radius = "xl",
  ghost = false,
}: BezelProps) {
  return (
    <div
      className={cn(
        "bg-foreground/[0.03] ring-1 ring-foreground/[0.06]",
        paddingMap[padding],
        radiusMap[radius],
        className,
      )}
    >
      <div
        className={cn(
          "relative overflow-hidden",
          innerRadiusMap[radius],
          ghost
            ? "bg-transparent"
            : "bg-card shadow-[inset_0_1px_1px_oklch(1_0_0/0.1)] dark:shadow-[inset_0_1px_1px_oklch(1_0_0/0.05)]",
        )}
      >
        {children}
      </div>
    </div>
  );
}
