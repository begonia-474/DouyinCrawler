import { useState } from "react";
import { Header } from "@/components/layout/header";
import { useMounted } from "@/hooks/use-safe-timer";
import { UrlInput } from "@/components/shared/url-input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Radio,
  Users,
  Copy,
  ExternalLink,
  CheckCircle2,
  Circle,
} from "lucide-react";

interface LiveInfo {
  title: string;
  nickname: string;
  isLive: boolean;
  userCount: number;
  roomId: string;
  m3u8Urls: string[];
  flvUrls: string[];
}

export function LivePage() {
  const [loading, setLoading] = useState(false);
  const [liveInfo, setLiveInfo] = useState<LiveInfo | null>(null);
  const [copied, setCopied] = useState<string | null>(null);
  const mountedRef = useMounted();

  const handleParse = async (_url: string) => {
    setLoading(true);
    setTimeout(() => {
      if (!mountedRef.current) return;
      setLiveInfo({
        title: "示例直播间标题",
        nickname: "主播昵称",
        isLive: true,
        userCount: 1234,
        roomId: "123456789",
        m3u8Urls: [
          "https://pull-flv-l1.douyincdn.com/third/stream-xxx.flv",
        ],
        flvUrls: [
          "https://pull-flv-l1.douyincdn.com/third/stream-xxx.flv",
        ],
      });
      setLoading(false);
    }, 1000);
  };

  const handleCopy = (text: string, type: string) => {
    navigator.clipboard.writeText(text);
    setCopied(type);
    setTimeout(() => {
      if (mountedRef.current) setCopied(null);
    }, 2000);
  };

  return (
    <>
      <Header title="直播" description="获取直播信息和流地址" />

      <div className="space-y-6">
        <UrlInput
          onSubmit={handleParse}
          loading={loading}
          placeholder="粘贴直播间链接..."
        />

        {liveInfo && (
          <div className="space-y-4">
            <Card>
              <CardHeader>
                <div className="flex items-center justify-between">
                  <CardTitle className="text-lg">{liveInfo.title}</CardTitle>
                  <Badge variant={liveInfo.isLive ? "default" : "secondary"}>
                    {liveInfo.isLive ? (
                      <>
                        <Circle className="h-2 w-2 fill-current mr-1" />
                        直播中
                      </>
                    ) : (
                      "未开播"
                    )}
                  </Badge>
                </div>
              </CardHeader>
              <CardContent>
                <div className="grid grid-cols-3 gap-4 text-sm">
                  <div>
                    <p className="text-muted-foreground">主播</p>
                    <p className="font-medium">{liveInfo.nickname}</p>
                  </div>
                  <div>
                    <p className="text-muted-foreground">观看人数</p>
                    <p className="font-medium flex items-center gap-1">
                      <Users className="h-4 w-4" />
                      {liveInfo.userCount}
                    </p>
                  </div>
                  <div>
                    <p className="text-muted-foreground">房间号</p>
                    <p className="font-medium">{liveInfo.roomId}</p>
                  </div>
                </div>
              </CardContent>
            </Card>

            {liveInfo.isLive && (
              <>
                <Card>
                  <CardHeader>
                    <CardTitle className="text-base flex items-center gap-2">
                      <Radio className="h-4 w-4" />
                      流地址
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    {liveInfo.m3u8Urls.length > 0 && (
                      <div className="space-y-2">
                        <Label>M3U8</Label>
                        {liveInfo.m3u8Urls.map((url, i) => (
                          <div key={i} className="flex gap-2">
                            <Input value={url} readOnly className="flex-1 font-mono text-xs" />
                            <Button
                              variant="outline"
                              size="icon"
                              onClick={() => handleCopy(url, `m3u8-${i}`)}
                            >
                              {copied === `m3u8-${i}` ? (
                                <CheckCircle2 className="h-4 w-4 text-green-600" />
                              ) : (
                                <Copy className="h-4 w-4" />
                              )}
                            </Button>
                          </div>
                        ))}
                      </div>
                    )}

                    {liveInfo.flvUrls.length > 0 && (
                      <div className="space-y-2">
                        <Label>FLV</Label>
                        {liveInfo.flvUrls.map((url, i) => (
                          <div key={i} className="flex gap-2">
                            <Input value={url} readOnly className="flex-1 font-mono text-xs" />
                            <Button
                              variant="outline"
                              size="icon"
                              onClick={() => handleCopy(url, `flv-${i}`)}
                            >
                              {copied === `flv-${i}` ? (
                                <CheckCircle2 className="h-4 w-4 text-green-600" />
                              ) : (
                                <Copy className="h-4 w-4" />
                              )}
                            </Button>
                          </div>
                        ))}
                      </div>
                    )}
                  </CardContent>
                </Card>

                <div className="flex gap-2">
                  <Button variant="outline" className="flex-1">
                    <ExternalLink className="h-4 w-4 mr-2" />
                    用播放器打开
                  </Button>
                </div>
              </>
            )}
          </div>
        )}
      </div>
    </>
  );
}
