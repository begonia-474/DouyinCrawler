import { useNavigate } from "react-router-dom";
import { Header } from "@/components/layout/header";
import { Card, CardContent } from "@/components/ui/card";
import { AnimateEntry } from "@/components/shared/animate-entry";
import {
  Video,
  User,
  Radio,
  Heart,
  Layers,
} from "lucide-react";

const features = [
  {
    id: "video",
    title: "单视频下载",
    description: "下载单个视频或图文内容",
    icon: Video,
    path: "/douyin/video",
    span: "col-span-2 row-span-1",
  },
  {
    id: "user",
    title: "用户主页",
    description: "查看用户作品、喜欢、关注",
    icon: User,
    path: "/douyin/user",
    span: "col-span-1 row-span-1",
  },
  {
    id: "live",
    title: "直播",
    description: "获取直播信息和流地址",
    icon: Radio,
    path: "/douyin/live",
    span: "col-span-1 row-span-1",
  },
  {
    id: "likes",
    title: "用户点赞",
    description: "查看用户的点赞列表",
    icon: Heart,
    path: "/douyin/likes",
    span: "col-span-1 row-span-1",
  },
  {
    id: "mix",
    title: "合集",
    description: "下载整个合集/播放列表",
    icon: Layers,
    path: "/douyin/mix",
    span: "col-span-2 row-span-1",
  },
];

export default function DouyinIndex() {
  const navigate = useNavigate();

  return (
    <>
      <AnimateEntry>
        <Header title="抖音" description="选择功能开始使用" />
      </AnimateEntry>

      <div className="grid grid-cols-3 gap-4">
        {features.map((feature, i) => (
          <AnimateEntry key={feature.id} delay={i * 60}>
            <Card
              className={`group cursor-pointer border-border/40 bg-card/60 backdrop-blur-sm hover:bg-card hover:border-border/60 hover:-translate-y-1 transition-all duration-500 ${feature.span}`}
              style={{ transitionTimingFunction: "cubic-bezier(0.32, 0.72, 0, 1)" }}
              onClick={() => navigate(feature.path)}
            >
              <CardContent className="p-6">
                <div className="flex items-start gap-5">
                  <div className="h-12 w-12 rounded-2xl bg-foreground/[0.04] ring-1 ring-foreground/[0.06] flex items-center justify-center shrink-0 group-hover:bg-brand/[0.08] group-hover:ring-brand/20 transition-all duration-500">
                    <feature.icon className="h-5 w-5 text-muted-foreground group-hover:text-brand transition-colors duration-500" />
                  </div>
                  <div className="min-w-0">
                    <h3 className="font-heading text-lg font-semibold tracking-tight">
                      {feature.title}
                    </h3>
                    <p className="text-sm text-muted-foreground mt-1 leading-relaxed">
                      {feature.description}
                    </p>
                  </div>
                </div>
              </CardContent>
            </Card>
          </AnimateEntry>
        ))}
      </div>
    </>
  );
}
