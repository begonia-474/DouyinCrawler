import { NavLink } from "react-router-dom";
import {
  Home,
  FolderDown,
  Rss,
  Heart,
  Music,
  Database,
  Settings,
  Video,
  Radio,
  Sun,
  Moon,
  Monitor,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/stores/app-store";

const platformItems = [
  { id: "douyin", icon: Video, label: "抖音" },
];

const navItems = [
  { to: "/douyin", icon: Home, label: "首页" },
  { to: "/downloads", icon: FolderDown, label: "下载记录" },
  { to: "/douyin/feed", icon: Rss, label: "Feed" },
  { to: "/douyin/favorites", icon: Heart, label: "收藏" },
  { to: "/douyin/following-live", icon: Radio, label: "关注直播" },
  { to: "/douyin/music", icon: Music, label: "音乐" },
  { to: "/douyin/library", icon: Database, label: "资料库" },
];

const bottomItems = [
  { to: "/settings", icon: Settings, label: "设置" },
];

const themeOptions = [
  { value: "light" as const, icon: Sun, label: "浅色" },
  { value: "dark" as const, icon: Moon, label: "深色" },
  { value: "system" as const, icon: Monitor, label: "跟随系统" },
];

function SidebarLink({
  to,
  icon: Icon,
  label,
}: {
  to: string;
  icon: React.ComponentType<{ className?: string }>;
  label: string;
}) {
  return (
    <NavLink
      to={to}
      end={to === "/douyin"}
      className={({ isActive }) =>
        cn(
          "flex items-center gap-3 px-3 py-2 rounded-xl text-sm transition-all duration-300 relative group",
          "hover:bg-foreground/[0.04]",
          isActive
            ? "bg-foreground/[0.06] text-foreground font-medium"
            : "text-muted-foreground"
        )
      }
    >
      {({ isActive }) => (
        <>
          <span
            className={cn(
              "absolute left-0 top-1/2 -translate-y-1/2 h-4 w-[2.5px] rounded-full transition-all duration-300",
              isActive
                ? "bg-brand opacity-100 scale-100"
                : "bg-brand opacity-0 scale-75"
            )}
          />
          <Icon
            className={cn(
              "h-4 w-4 shrink-0 transition-colors duration-300",
              isActive ? "text-brand" : "text-muted-foreground group-hover:text-foreground/70"
            )}
          />
          <span>{label}</span>
        </>
      )}
    </NavLink>
  );
}

export function Sidebar() {
  const currentPlatform = "douyin";
  const { theme, setTheme } = useAppStore();

  return (
    <aside className="w-[220px] border-r border-border/60 bg-card/80 backdrop-blur-sm flex flex-col h-full">
      <div className="p-5 pb-4">
        <h1 className="text-lg font-bold tracking-tight text-foreground flex items-center gap-2.5">
          <span className="inline-flex items-center justify-center h-8 w-8 rounded-[0.6rem] bg-brand text-brand-foreground text-[11px] font-bold tracking-wide shadow-lg shadow-brand/20">
            DC
          </span>
          <span className="font-heading text-[17px] tracking-[-0.01em]">DouyinCrawler</span>
        </h1>
      </div>

      <div className="px-3 pb-3">
        <div className="flex gap-1 p-1 bg-foreground/[0.03] rounded-xl">
          {platformItems.map((platform) => (
            <button
              key={platform.id}
              className={cn(
                "flex-1 flex items-center justify-center gap-1.5 px-2 py-1.5 rounded-lg text-xs font-medium transition-all duration-200",
                currentPlatform === platform.id
                  ? "bg-background text-foreground shadow-sm"
                  : "text-muted-foreground hover:text-foreground"
              )}
            >
              <platform.icon className="h-3.5 w-3.5" />
              {platform.label}
            </button>
          ))}
        </div>
      </div>

      <nav className="flex-1 px-3 py-1 space-y-0.5 overflow-auto">
        {navItems.map((item) => (
          <SidebarLink key={item.to} {...item} />
        ))}
      </nav>

      <div className="px-3 py-4 border-t border-border/60 space-y-2">
        <div className="flex gap-1 p-1 bg-foreground/[0.03] rounded-xl">
          {themeOptions.map((opt) => (
            <button
              key={opt.value}
              onClick={() => setTheme(opt.value)}
              className={cn(
                "flex-1 flex items-center justify-center p-1.5 rounded-lg transition-all duration-200",
                theme === opt.value
                  ? "bg-background text-foreground shadow-sm"
                  : "text-muted-foreground hover:text-foreground"
              )}
              title={opt.label}
            >
              <opt.icon className="h-3.5 w-3.5" />
            </button>
          ))}
        </div>
        {bottomItems.map((item) => (
          <SidebarLink key={item.to} {...item} />
        ))}
      </div>
    </aside>
  );
}
