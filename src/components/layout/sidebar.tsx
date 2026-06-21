import { NavLink } from "react-router-dom";
import {
  Download,
  User,
  Search,
  MessageSquare,
  Radio,
  Rss,
  ListMusic,
  Layers,
  FolderDown,
  Settings,
} from "lucide-react";
import { cn } from "@/lib/utils";

const navItems = [
  { to: "/", icon: Download, label: "快速下载" },
  { to: "/user", icon: User, label: "用户主页" },
  { to: "/mix", icon: Layers, label: "合集" },
  { to: "/search", icon: Search, label: "搜索" },
  { to: "/comments", icon: MessageSquare, label: "评论" },
  { to: "/live", icon: Radio, label: "直播" },
  { to: "/feed", icon: Rss, label: "Feed" },
  { to: "/music", icon: ListMusic, label: "音乐收藏" },
];

const bottomItems = [
  { to: "/downloads", icon: FolderDown, label: "下载管理" },
  { to: "/settings", icon: Settings, label: "设置" },
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
      end={to === "/"}
      className={({ isActive }) =>
        cn(
          "flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors",
          "hover:bg-accent hover:text-accent-foreground",
          isActive
            ? "bg-accent text-accent-foreground font-medium"
            : "text-muted-foreground"
        )
      }
    >
      <Icon className="h-4 w-4 shrink-0" />
      <span>{label}</span>
    </NavLink>
  );
}

export function Sidebar() {
  return (
    <aside className="w-[220px] border-r bg-card flex flex-col h-full">
      <div className="p-4 pb-2">
        <h1 className="text-lg font-semibold tracking-tight">DouyinCrawler</h1>
      </div>

      <nav className="flex-1 px-3 py-2 space-y-1">
        {navItems.map((item) => (
          <SidebarLink key={item.to} {...item} />
        ))}
      </nav>

      <div className="px-3 py-3 border-t space-y-1">
        {bottomItems.map((item) => (
          <SidebarLink key={item.to} {...item} />
        ))}
      </div>
    </aside>
  );
}
