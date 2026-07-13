import { type ReactNode } from "react";
import { cn } from "@/lib/utils";

interface BezelProps {
  children: ReactNode;
  className?: string;
  radius?: "md" | "lg" | "xl" | "2xl";
  ghost?: boolean;
  /** 启用悬浮提升效果 */
  lift?: boolean;
  /** @deprecated No longer used — kept for backward compatibility */
  padding?: "sm" | "md" | "lg";
}

const radiusMap = { md: "rounded-xl", lg: "rounded-2xl", xl: "rounded-[1.5rem]", "2xl": "rounded-[2rem]" };

export function Bezel({
  children,
  className,
  radius = "xl",
  ghost = false,
  lift = false,
  padding: _padding,
}: BezelProps) {
  return (
    <div
      className={cn(
        "relative overflow-hidden",
        radiusMap[radius],
        ghost ? "bg-transparent" : "bg-card",
        !ghost && "noise",
        lift && "card-lift",
        className,
      )}
    >
      {children}
    </div>
  );
}
