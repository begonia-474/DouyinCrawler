import { useNavigate } from "react-router-dom";
import { ChevronLeft } from "lucide-react";
import { cn } from "@/lib/utils";

interface HeaderProps {
  title: string;
  description?: string;
  parent?: { label: string; path: string };
  eyebrow?: string;
  children?: React.ReactNode;
}

export function Header({ title, description, parent, eyebrow, children }: HeaderProps) {
  const navigate = useNavigate();

  return (
    <div className="pb-10">
      <div className="flex items-center justify-between">
        <div>
          {parent ? (
            <div className="flex items-center gap-2 text-base mb-2">
              <button
                className="inline-flex items-center justify-center h-8 w-8 rounded-full text-muted-foreground hover:text-foreground hover:bg-foreground/[0.05] transition-all duration-300"
                onClick={() => navigate(parent.path)}
              >
                <ChevronLeft className="h-4 w-4" />
              </button>
              <button
                className="text-muted-foreground hover:text-foreground transition-colors duration-200"
                onClick={() => navigate(parent.path)}
              >
                {parent.label}
              </button>
              <span className="text-muted-foreground/40">/</span>
              <span className="font-medium">{title}</span>
            </div>
          ) : (
            <>
              {eyebrow && (
                <span className="inline-block rounded-full px-3 py-1 text-[10px] uppercase tracking-[0.2em] font-medium bg-foreground/[0.05] text-muted-foreground mb-3">
                  {eyebrow}
                </span>
              )}
              <h2 className="font-heading text-[2rem] font-semibold tracking-[-0.02em] text-foreground leading-tight">
                {title}
              </h2>
            </>
          )}
          {description && (
            <p className={cn("text-sm text-muted-foreground tracking-wide", parent ? "" : "mt-2")}>
              {description}
            </p>
          )}
        </div>
        {children && <div className="flex items-center gap-2">{children}</div>}
      </div>
    </div>
  );
}
