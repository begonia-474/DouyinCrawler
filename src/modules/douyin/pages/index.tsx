import { useNavigate } from "react-router-dom";
import { Header } from "@/components/layout/header";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { Bezel } from "@/components/shared/bezel";
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
    span: "col-span-8",
  },
  {
    id: "user",
    title: "用户主页",
    description: "查看用户作品、喜欢、关注",
    icon: User,
    path: "/douyin/user",
    span: "col-span-4",
  },
  {
    id: "live",
    title: "直播",
    description: "获取直播信息和流地址",
    icon: Radio,
    path: "/douyin/live",
    span: "col-span-4",
  },
  {
    id: "likes",
    title: "用户点赞",
    description: "查看用户的点赞列表",
    icon: Heart,
    path: "/douyin/likes",
    span: "col-span-4",
  },
  {
    id: "mix",
    title: "合集",
    description: "下载整个合集/播放列表",
    icon: Layers,
    path: "/douyin/mix",
    span: "col-span-4",
  },
];

export default function DouyinIndex() {
  const navigate = useNavigate();

  return (
    <>
      <AnimateEntry>
        <Header title="抖音" description="选择功能开始使用" eyebrow="Platform" />
      </AnimateEntry>

      <div className="grid grid-cols-12 auto-rows-fr gap-5">
        {features.map((feature, i) => (
          <AnimateEntry key={feature.id} delay={i * 60} className={`${feature.span} h-full`}>
            <Bezel radius="xl" lift className="h-full">
              <button
                className="w-full h-full text-left p-7 group cursor-pointer transition-all duration-500 hover:bg-foreground/[0.02]"
                onClick={() => navigate(feature.path)}
              >
                <div className="flex items-start gap-5">
                  <div className="h-12 w-12 rounded-2xl bg-foreground/[0.04] ring-1 ring-foreground/[0.07] flex items-center justify-center shrink-0 group-hover:bg-brand/[0.1] group-hover:ring-brand/25 transition-all duration-500">
                    <feature.icon className="h-5 w-5 text-muted-foreground group-hover:text-brand transition-colors duration-500" />
                  </div>
                  <div className="min-w-0">
                    <h3 className="font-heading text-lg font-semibold tracking-tight">
                      {feature.title}
                    </h3>
                    <p className="text-sm text-muted-foreground mt-1.5 leading-relaxed">
                      {feature.description}
                    </p>
                  </div>
                </div>
              </button>
            </Bezel>
          </AnimateEntry>
        ))}
      </div>
    </>
  );
}
