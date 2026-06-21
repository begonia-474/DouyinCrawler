import { useNavigate } from "react-router-dom";
import { Header } from "@/components/layout/header";
import { Card, CardContent } from "@/components/ui/card";
import {
  Video,
  User,
  Radio,
  Heart,
  FolderOpen,
  Layers,
} from "lucide-react";

const features = [
  {
    id: "video",
    title: "单视频下载",
    description: "下载单个视频或图文",
    icon: Video,
    path: "/douyin/video",
    color: "text-blue-500",
    bgColor: "bg-blue-500/10",
  },
  {
    id: "user",
    title: "用户主页",
    description: "查看用户作品、喜欢、关注",
    icon: User,
    path: "/douyin/user",
    color: "text-green-500",
    bgColor: "bg-green-500/10",
  },
  {
    id: "live",
    title: "直播",
    description: "获取直播信息和流地址",
    icon: Radio,
    path: "/douyin/live",
    color: "text-red-500",
    bgColor: "bg-red-500/10",
  },
  {
    id: "likes",
    title: "用户点赞",
    description: "查看用户的点赞列表",
    icon: Heart,
    path: "/douyin/likes",
    color: "text-pink-500",
    bgColor: "bg-pink-500/10",
  },
  {
    id: "collects",
    title: "用户收藏",
    description: "查看用户的收藏夹",
    icon: FolderOpen,
    path: "/douyin/collects",
    color: "text-purple-500",
    bgColor: "bg-purple-500/10",
  },
  {
    id: "mix",
    title: "合集",
    description: "下载整个合集/播放列表",
    icon: Layers,
    path: "/douyin/mix",
    color: "text-orange-500",
    bgColor: "bg-orange-500/10",
  },
];

export default function DouyinIndex() {
  const navigate = useNavigate();

  return (
    <>
      <Header title="抖音" description="选择功能开始使用" />

      <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
        {features.map((feature) => (
          <Card
            key={feature.id}
            className="hover:bg-accent/50 transition-colors cursor-pointer"
            onClick={() => navigate(feature.path)}
          >
            <CardContent className="p-6">
              <div className="flex items-start gap-4">
                <div className={`h-12 w-12 rounded-xl ${feature.bgColor} flex items-center justify-center shrink-0`}>
                  <feature.icon className={`h-6 w-6 ${feature.color}`} />
                </div>
                <div className="min-w-0">
                  <h3 className="font-semibold">{feature.title}</h3>
                  <p className="text-sm text-muted-foreground mt-1">
                    {feature.description}
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </>
  );
}
