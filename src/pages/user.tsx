import { useState } from "react";
import { Header } from "@/components/layout/header";
import { useMounted } from "@/hooks/use-safe-timer";
import { UrlInput } from "@/components/shared/url-input";
import { VideoCard } from "@/components/shared/video-card";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import {
  Download,
  Users,
  Heart,
  Video,
  FolderOpen,
  Loader2,
  ChevronRight,
  UserPlus,
  UserCheck,
  BarChart3,
  ThumbsUp,
  MessageSquare,
  Share2,
  Bookmark,
  Layers,
} from "lucide-react";

// --- 数据类型 ---

interface UserProfile {
  nickname: string;
  avatar: string;
  followerCount: number;
  followingCount: number;
  awemeCount: number;
  totalFavorited: number;
  signature: string;
  secUserId: string;
}

interface VideoItem {
  id: string;
  title: string;
  duration: string;
  diggCount: number;
  commentCount: number;
  shareCount: number;
  collectCount: number;
  cover?: string;
  createTime: string;
}

interface CollectsFolder {
  id: string;
  name: string;
  count: number;
}

interface FollowItem {
  id: string;
  nickname: string;
  avatar: string;
  signature: string;
  followerCount: number;
}

interface PostStats {
  diggCount: number;
  commentCount: number;
  shareCount: number;
  collectCount: number;
  playCount: number;
}

// --- 模拟数据 ---

const mockVideos: VideoItem[] = [
  { id: "1", title: "示例作品 1 - 旅行日记", duration: "02:30", diggCount: 12000, commentCount: 450, shareCount: 230, collectCount: 890, createTime: "2024-01-15" },
  { id: "2", title: "示例作品 2 - 美食分享", duration: "01:45", diggCount: 8500, commentCount: 320, shareCount: 150, collectCount: 560, createTime: "2024-01-10" },
  { id: "3", title: "示例作品 3 - 搞笑日常", duration: "00:30", diggCount: 25000, commentCount: 1200, shareCount: 890, collectCount: 2100, createTime: "2024-01-05" },
];

const mockCollects: CollectsFolder[] = [
  { id: "c1", name: "美食收藏", count: 15 },
  { id: "c2", name: "旅行攻略", count: 8 },
  { id: "c3", name: "学习资料", count: 23 },
];

const mockFollowing: FollowItem[] = [
  { id: "f1", nickname: "美食达人", avatar: "", signature: "分享每日美食", followerCount: 520000 },
  { id: "f2", nickname: "旅行博主", avatar: "", signature: "环游世界中", followerCount: 310000 },
  { id: "f3", nickname: "搞笑博主", avatar: "", signature: "每天开心一点", followerCount: 890000 },
];

const mockFollowers: FollowItem[] = [
  { id: "g1", nickname: "用户A", avatar: "", signature: "热爱生活", followerCount: 1200 },
  { id: "g2", nickname: "用户B", avatar: "", signature: "", followerCount: 450 },
];

const mockStats: PostStats = {
  diggCount: 45500,
  commentCount: 1970,
  shareCount: 1270,
  collectCount: 3550,
  playCount: 892000,
};

// --- 工具函数 ---

function formatCount(n: number): string {
  if (n >= 10000) return `${(n / 10000).toFixed(1)}w`;
  return n.toLocaleString();
}

// --- 组件 ---

function StatsCard({ stats }: { stats: PostStats }) {
  const items = [
    { label: "总播放", value: stats.playCount, icon: Video },
    { label: "总点赞", value: stats.diggCount, icon: ThumbsUp },
    { label: "总评论", value: stats.commentCount, icon: MessageSquare },
    { label: "总分享", value: stats.shareCount, icon: Share2 },
    { label: "总收藏", value: stats.collectCount, icon: Bookmark },
  ];

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base flex items-center gap-2">
          <BarChart3 className="h-4 w-4" />
          作品统计
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="grid grid-cols-5 gap-4">
          {items.map((item) => (
            <div key={item.label} className="text-center">
              <item.icon className="h-4 w-4 text-muted-foreground mx-auto mb-1" />
              <p className="text-lg font-semibold">{formatCount(item.value)}</p>
              <p className="text-xs text-muted-foreground">{item.label}</p>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}

function FollowItemCard({ item, type }: { item: FollowItem; type: "following" | "follower" }) {
  return (
    <div className="flex items-center gap-3 p-3 rounded-lg hover:bg-accent/50 transition-colors">
      <Avatar className="h-10 w-10">
        <AvatarImage src={item.avatar} />
        <AvatarFallback>{item.nickname[0]}</AvatarFallback>
      </Avatar>
      <div className="flex-1 min-w-0">
        <h4 className="text-sm font-medium">{item.nickname}</h4>
        {item.signature && (
          <p className="text-xs text-muted-foreground truncate">
            {item.signature}
          </p>
        )}
        <p className="text-xs text-muted-foreground">
          {formatCount(item.followerCount)} 粉丝
        </p>
      </div>
      {type === "following" ? (
        <Button variant="outline" size="sm">
          <UserCheck className="h-3.5 w-3.5 mr-1" />
          已关注
        </Button>
      ) : (
        <Button variant="outline" size="sm">
          <UserPlus className="h-3.5 w-3.5 mr-1" />
          关注
        </Button>
      )}
    </div>
  );
}

function CollectsFolderCard({ folder }: { folder: CollectsFolder }) {
  return (
    <Card className="hover:bg-accent/50 transition-colors cursor-pointer">
      <CardContent className="p-4 flex items-center gap-3">
        <div className="h-10 w-10 rounded-lg bg-primary/10 flex items-center justify-center">
          <FolderOpen className="h-5 w-5 text-primary" />
        </div>
        <div className="flex-1">
          <h4 className="text-sm font-medium">{folder.name}</h4>
          <p className="text-xs text-muted-foreground">{folder.count} 个视频</p>
        </div>
        <div className="flex items-center gap-1">
          <Button variant="ghost" size="sm">
            <Download className="h-3.5 w-3.5 mr-1" />
            下载
          </Button>
          <ChevronRight className="h-4 w-4 text-muted-foreground" />
        </div>
      </CardContent>
    </Card>
  );
}

// --- 主页面 ---

export function UserPage() {
  const [loading, setLoading] = useState(false);
  const [profile, setProfile] = useState<UserProfile | null>(null);
  const [videos] = useState<VideoItem[]>(mockVideos);
  const [likes] = useState<VideoItem[]>(mockVideos.slice(0, 2));
  const [collects] = useState<CollectsFolder[]>(mockCollects);
  const [following] = useState<FollowItem[]>(mockFollowing);
  const [followers] = useState<FollowItem[]>(mockFollowers);
  const [stats] = useState<PostStats>(mockStats);
  const mountedRef = useMounted();

  const handleParse = async (_url: string) => {
    setLoading(true);
    setTimeout(() => {
      if (!mountedRef.current) return;
      setProfile({
        nickname: "示例用户",
        avatar: "",
        followerCount: 12345,
        followingCount: 678,
        awemeCount: 42,
        totalFavorited: 89000,
        signature: "分享生活的美好瞬间 | 合作私信",
        secUserId: "MS4wLjABAAAA",
      });
      setLoading(false);
    }, 1000);
  };

  return (
    <>
      <Header title="用户主页" description="查看用户作品、喜欢、收藏、关注">
        {profile && (
          <Button>
            <Download className="h-4 w-4 mr-2" />
            全部下载
          </Button>
        )}
      </Header>

      <div className="space-y-6">
        <UrlInput
          onSubmit={handleParse}
          loading={loading}
          placeholder="粘贴用户主页链接..."
        />

        {loading && (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        )}

        {profile && !loading && (
          <>
            {/* 用户信息卡片 */}
            <Card>
              <CardContent className="p-6">
                <div className="flex items-start gap-4">
                  <Avatar className="h-16 w-16">
                    <AvatarImage src={profile.avatar} />
                    <AvatarFallback>{profile.nickname[0]}</AvatarFallback>
                  </Avatar>
                  <div className="flex-1">
                    <h3 className="text-lg font-semibold">{profile.nickname}</h3>
                    <p className="text-sm text-muted-foreground mt-1">
                      {profile.signature}
                    </p>
                    <div className="flex items-center gap-5 mt-3">
                      <div className="flex items-center gap-1.5 text-sm">
                        <Video className="h-4 w-4 text-muted-foreground" />
                        <span className="font-semibold">{profile.awemeCount}</span>
                        <span className="text-muted-foreground">作品</span>
                      </div>
                      <div className="flex items-center gap-1.5 text-sm">
                        <Users className="h-4 w-4 text-muted-foreground" />
                        <span className="font-semibold">
                          {formatCount(profile.followerCount)}
                        </span>
                        <span className="text-muted-foreground">粉丝</span>
                      </div>
                      <div className="flex items-center gap-1.5 text-sm">
                        <Heart className="h-4 w-4 text-muted-foreground" />
                        <span className="font-semibold">
                          {formatCount(profile.followingCount)}
                        </span>
                        <span className="text-muted-foreground">关注</span>
                      </div>
                      <div className="flex items-center gap-1.5 text-sm">
                        <ThumbsUp className="h-4 w-4 text-muted-foreground" />
                        <span className="font-semibold">
                          {formatCount(profile.totalFavorited)}
                        </span>
                        <span className="text-muted-foreground">获赞</span>
                      </div>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* 作品统计 */}
            <StatsCard stats={stats} />

            {/* 功能 Tab */}
            <Tabs defaultValue="posts">
              <TabsList>
                <TabsTrigger value="posts">
                  作品
                  <Badge variant="secondary" className="ml-1.5">
                    {videos.length}
                  </Badge>
                </TabsTrigger>
                <TabsTrigger value="likes">
                  喜欢
                  <Badge variant="secondary" className="ml-1.5">
                    {likes.length}
                  </Badge>
                </TabsTrigger>
                <TabsTrigger value="collects">
                  收藏夹
                  <Badge variant="secondary" className="ml-1.5">
                    {collects.length}
                  </Badge>
                </TabsTrigger>
                <TabsTrigger value="following">
                  关注
                  <Badge variant="secondary" className="ml-1.5">
                    {following.length}
                  </Badge>
                </TabsTrigger>
                <TabsTrigger value="followers">
                  粉丝
                  <Badge variant="secondary" className="ml-1.5">
                    {followers.length}
                  </Badge>
                </TabsTrigger>
              </TabsList>

              {/* 作品列表 */}
              <TabsContent value="posts" className="mt-4">
                <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
                  {videos.map((video) => (
                    <VideoCard
                      key={video.id}
                      title={video.title}
                      author={profile.nickname}
                      duration={video.duration}
                      diggCount={video.diggCount}
                      commentCount={video.commentCount}
                      shareCount={video.shareCount}
                    />
                  ))}
                </div>
              </TabsContent>

              {/* 喜欢列表 */}
              <TabsContent value="likes" className="mt-4">
                <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
                  {likes.map((video) => (
                    <VideoCard
                      key={video.id}
                      title={video.title}
                      author={profile.nickname}
                      duration={video.duration}
                      diggCount={video.diggCount}
                      commentCount={video.commentCount}
                      shareCount={video.shareCount}
                    />
                  ))}
                </div>
              </TabsContent>

              {/* 收藏夹 */}
              <TabsContent value="collects" className="mt-4 space-y-3">
                <div className="flex items-center justify-between">
                  <h3 className="text-sm font-medium text-muted-foreground">
                    收藏夹列表
                  </h3>
                  <Button variant="outline" size="sm">
                    <Layers className="h-3.5 w-3.5 mr-1" />
                    全部下载
                  </Button>
                </div>
                {collects.map((folder) => (
                  <CollectsFolderCard key={folder.id} folder={folder} />
                ))}
              </TabsContent>

              {/* 关注列表 */}
              <TabsContent value="following" className="mt-4 space-y-1">
                {following.map((item) => (
                  <FollowItemCard key={item.id} item={item} type="following" />
                ))}
              </TabsContent>

              {/* 粉丝列表 */}
              <TabsContent value="followers" className="mt-4 space-y-1">
                {followers.map((item) => (
                  <FollowItemCard key={item.id} item={item} type="follower" />
                ))}
              </TabsContent>
            </Tabs>
          </>
        )}
      </div>
    </>
  );
}
