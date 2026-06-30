import { useCallback, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Header } from "@/components/layout/header";
import { UrlInput } from "@/components/shared/url-input";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Bezel } from "@/components/shared/bezel";
import { useUserProfileQuery, useCollectsListQuery } from "@/lib/queries";
import type { UserProfile as UserProfileType, CollectsFolder } from "@/lib/api-types";
import { FolderOpen, ChevronRight } from "lucide-react";
import { ErrorBanner } from "@/components/shared/error-banner";
import { LoadingSpinner } from "@/components/shared/loading-spinner";

export default function CollectsPage() {
  const navigate = useNavigate();
  const [currentUrl, setCurrentUrl] = useState("");

  const profileQuery = useUserProfileQuery(currentUrl || null);
  const collectsQuery = useCollectsListQuery();

  const profile = (profileQuery.data?.data?.profile as unknown as UserProfileType) || null;
  const collects = (collectsQuery.data?.data?.collects ?? []) as unknown as CollectsFolder[];
  const loading = profileQuery.isLoading;
  const error = profileQuery.error?.message
    || collectsQuery.error?.message
    || (!profileQuery.data?.success ? (profileQuery.data?.error ?? null) : null);

  const handleParse = useCallback((url: string) => {
    setCurrentUrl(url);
  }, []);

  return (
    <>
      <Header title="用户收藏" description="查看用户的收藏夹，点击进入下载" parent={{ label: "首页", path: "/douyin" }} />

      <div className="space-y-6">
        <UrlInput onSubmit={handleParse} loading={loading} placeholder="粘贴用户主页链接..." allowedTypes={["user"]} autoDetect />

        <ErrorBanner message={error} />

        {loading && <LoadingSpinner size={24} className="py-16" />}

        {profile && !loading && (
          <>
            <Bezel radius="xl" padding="sm">
              <div className="flex items-center gap-4 p-5 bg-card">
                <Avatar className="h-12 w-12">
                  <AvatarImage src={profile.avatar} />
                  <AvatarFallback>{profile.nickname?.[0] || "?"}</AvatarFallback>
                </Avatar>
                <div>
                  <h3 className="font-semibold">{profile.nickname}</h3>
                  <p className="text-sm text-muted-foreground">{collects.length} 个收藏夹</p>
                </div>
              </div>
            </Bezel>

            <div className="space-y-3">
              {collects.map((folder) => (
                <Bezel key={folder.id} radius="xl" padding="sm">
                  <button
                    className="w-full flex items-center gap-4 p-5 bg-card hover:bg-foreground/[0.02] transition-all duration-300 cursor-pointer"
                    onClick={() => navigate(`/douyin/favorites/${folder.id}`)}
                  >
                    <div className="h-11 w-11 rounded-2xl bg-primary/10 ring-1 ring-primary/15 flex items-center justify-center shrink-0">
                      <FolderOpen className="h-5 w-5 text-primary" />
                    </div>
                    <div className="flex-1 text-left">
                      <h4 className="text-sm font-medium">{folder.name}</h4>
                      <p className="text-xs text-muted-foreground">{folder.count} 个视频</p>
                    </div>
                    <Badge variant="secondary" className="rounded-full">{folder.count}</Badge>
                    <ChevronRight className="h-4 w-4 text-muted-foreground" />
                  </button>
                </Bezel>
              ))}
            </div>
          </>
        )}
      </div>
    </>
  );
}
