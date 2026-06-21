import { NavLink, useLocation } from "react-router-dom";
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
  Video,
} from "lucide-react";
import { cn } from "@/lib/utils";

// 平台切换
const platformItems = [
  { id: "douyin", icon: Video, label: "抖音" },
  // { id: "kuaishou", icon: ..., label: "快手" },
  // { id: "bilibili", icon: ..., label: "B站" },
];

// 抖音导航项
const douyinNavItems = [
  { to: "/douyin", icon: Download, label: "快速下载" },
  { to: "/douyin/user", icon: User, label: "用户主页" },
  { to: "/douyin/mix", icon: Layers, label: "合集" },
  { to: "/douyin/search", icon: Search, label: "搜索" },
  { to: "/douyin/comments", icon: MessageSquare, label: "评论" },
  { to: "/douyin/live", icon: Radio, label: "直播" },
  { to: "/douyin/feed", icon: Rss, label: "Feed" },
  { to: "/douyin/music", icon: ListMusic, label: "音乐收藏" },
];

// 底部固定项
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
      end={to === "/douyin"}
      className={({ isActive }) =>
        cn(
          "flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors relative",
          "hover:bg-accent hover:text-accent-foreground",
          isActive
            ? "bg-accent text-accent-foreground font-medium before:absolute before:left-0 before:top-1/2 before:-translate-y-1/2 before:h-5 before:w-[3px] before:rounded-full before:bg-primary"
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
  const location = useLocation();
  const currentPlatform = "douyin"; // 后续可以根据路径动态判断

  return (
    <aside className="w-[220px] border-r bg-card flex flex-col h-full">
      <div className="p-4 pb-2">
        <h1 className="text-lg font-semibold tracking-tight">DouyinCrawler</h1>
      </div>

      {/* 平台切换 */}
      <div className="px-3 pb-2">
        <div className="flex gap-1 p-1 bg-muted rounded-lg">
          {platformItems.map((platform) => (
            <button
              key={platform.id}
              className={cn(
                "flex-1 flex items-center justify-center gap-1.5 px-2 py-1.5 rounded-md text-xs font-medium transition-colors",
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

      {/* 主导航 */}
      <nav className="flex-1 px-3 py-2 space-y-1 overflow-auto">
        {douyinNavItems.map((item) => (
          <SidebarLink key={item.to} {...item} />
        ))}
      </nav>

      {/* 底部固定项 */}
      <div className="px-3 py-3 border-t space-y-1">
        {bottomItems.map((item) => (
          <SidebarLink key={item.to} {...item} />
        ))}
      </div>
    </aside>
  );
}
