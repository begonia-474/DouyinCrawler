import { useNavigate } from "react-router-dom";

interface HeaderProps {
  title: string;
  description?: string;
  parent?: { label: string; path: string };
  children?: React.ReactNode;
}

export function Header({ title, description, parent, children }: HeaderProps) {
  const navigate = useNavigate();

  return (
    <div className="pb-6">
      <div className="flex items-center justify-between">
        <div>
          {parent ? (
            <div className="flex items-center gap-2 text-sm">
              <button
                className="text-muted-foreground hover:text-foreground transition-colors"
                onClick={() => navigate(parent.path)}
              >
                &lt;
              </button>
              <button
                className="text-muted-foreground hover:text-foreground transition-colors"
                onClick={() => navigate(parent.path)}
              >
                {parent.label}
              </button>
              <span className="text-muted-foreground">/</span>
              <span className="font-medium">{title}</span>
            </div>
          ) : (
            <h2 className="text-2xl font-semibold tracking-tight">{title}</h2>
          )}
          {description && (
            <p className="text-sm text-muted-foreground mt-1">{description}</p>
          )}
        </div>
        {children && <div className="flex items-center gap-2">{children}</div>}
      </div>
    </div>
  );
}
