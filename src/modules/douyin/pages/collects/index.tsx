import { useState, useCallback } from "react";
import { Header } from "@/components/layout/header";
import { UrlInput } from "@/components/shared/url-input";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { getUserProfile, getUserCollects } from "@/lib/api";
import type { UserProfile as UserProfileType, CollectsFolder } from "@/lib/api-types";
import { FolderOpen, ChevronRight, Loader2, AlertCircle, Download } from "lucide-react";

export default function CollectsPage() {
  const [loading, setLoading] = useState(false);
  const [profile, setProfile] = useState<UserProfileType | null>(null);
  const [collects, setCollects] = useState<CollectsFolder[]>([]);
  const [error, setError] = useState<string | null>(null);

  const handleParse = useCallback(async (url: string) => {
    setLoading(true);
    setProfile(null);
    setCollects([]);
    setError(null);

    const profileRes = await getUserProfile(url);
    if (profileRes.success && profileRes.data?.profile) {
      setProfile(profileRes.data.profile as unknown as UserProfileType);
    } else {
      setError(profileRes.error || "获取用户信息失败");
      setLoading(false);
      return;
    }

    const collectsRes = await getUserCollects();
    if (collectsRes.success && collectsRes.data?.collects) {
      setCollects(collectsRes.data.collects as unknown as CollectsFolder[]);
    }

    setLoading(false);
  }, []);

  return (
    <>
      <Header title="用户收藏" description="查看用户的收藏夹">
        {collects.length > 0 && (
          <Button variant="outline" size="sm">
            <Download className="h-4 w-4 mr-1" />
            全部下载
          </Button>
        )}
      </Header>

      <div className="space-y-6">
        <UrlInput onSubmit={handleParse} loading={loading} placeholder="粘贴用户主页链接..." />

        {error && (
          <div className="flex items-center gap-2 p-3 rounded-lg bg-destructive/10 text-destructive text-sm">
            <AlertCircle className="h-4 w-4 shrink-0" />
            <span>{error}</span>
          </div>
        )}

        {loading && (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        )}

        {profile && !loading && (
          <>
            <Card>
              <CardContent className="p-4 flex items-center gap-4">
                <Avatar className="h-12 w-12">
                  <AvatarImage src={profile.avatar} />
                  <AvatarFallback>{profile.nickname?.[0] || "?"}</AvatarFallback>
                </Avatar>
                <div>
                  <h3 className="font-semibold">{profile.nickname}</h3>
                  <p className="text-sm text-muted-foreground">{collects.length} 个收藏夹</p>
                </div>
              </CardContent>
            </Card>

            <div className="space-y-3">
              {collects.map((folder) => (
                <Card key={folder.id} className="hover:bg-accent/50 transition-colors cursor-pointer">
                  <CardContent className="p-4 flex items-center gap-4">
                    <div className="h-10 w-10 rounded-lg bg-primary/10 flex items-center justify-center">
                      <FolderOpen className="h-5 w-5 text-primary" />
                    </div>
                    <div className="flex-1">
                      <h4 className="text-sm font-medium">{folder.name}</h4>
                      <p className="text-xs text-muted-foreground">{folder.count} 个视频</p>
                    </div>
                    <Badge variant="secondary">{folder.count}</Badge>
                    <ChevronRight className="h-4 w-4 text-muted-foreground" />
                  </CardContent>
                </Card>
              ))}
            </div>
          </>
        )}
      </div>
    </>
  );
}
