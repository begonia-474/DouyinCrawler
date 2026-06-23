import { useNavigate } from "react-router-dom";
import { ChevronLeft } from "lucide-react";

interface HeaderProps {
  title: string;
  description?: string;
  parent?: { label: string; path: string };
  children?: React.ReactNode;
}

export function Header({ title, description, parent, children }: HeaderProps) {
  const navigate = useNavigate();

  return (
    <div className="pb-8">
      <div className="flex items-center justify-between">
        <div>
          {parent ? (
            <div className="flex items-center gap-2 text-sm">
              <button
                className="inline-flex items-center justify-center h-8 w-8 rounded-lg text-muted-foreground hover:text-foreground hover:bg-foreground/[0.04] transition-all duration-200"
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
            <h2 className="font-heading text-3xl font-semibold tracking-tight text-foreground">
              {title}
            </h2>
          )}
          {description && (
            <p className="text-sm text-muted-foreground mt-1.5 tracking-wide">{description}</p>
          )}
        </div>
        {children && <div className="flex items-center gap-2">{children}</div>}
      </div>
    </div>
  );
}
