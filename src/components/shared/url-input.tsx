import { useState, useCallback } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Link, Loader2 } from "lucide-react";

interface UrlInputProps {
  onSubmit: (url: string) => void;
  loading?: boolean;
  placeholder?: string;
}

type UrlType = "video" | "user" | "mix" | "live" | "unknown";

function detectUrlType(url: string): UrlType {
  if (!url) return "unknown";
  if (url.includes("/user/") || url.includes("sec_user_id")) return "user";
  if (url.includes("/collection/") || url.includes("mix_id")) return "mix";
  if (url.includes("live.douyin.com") || url.includes("/live/")) return "live";
  if (url.includes("/video/") || url.includes("/note/") || url.includes("modal_id"))
    return "video";
  return "unknown";
}

const typeLabels: Record<UrlType, string> = {
  video: "视频",
  user: "用户",
  mix: "合集",
  live: "直播",
  unknown: "未知",
};

export function UrlInput({ onSubmit, loading, placeholder }: UrlInputProps) {
  const [url, setUrl] = useState("");
  const urlType = detectUrlType(url);

  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      if (url.trim()) {
        onSubmit(url.trim());
      }
    },
    [url, onSubmit]
  );

  const handlePaste = useCallback(() => {
    navigator.clipboard.readText().then((text) => {
      if (text) setUrl(text);
    });
  }, []);

  return (
    <form onSubmit={handleSubmit} className="flex gap-2 items-start">
      <div className="flex-1 relative">
        <Input
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          placeholder={placeholder || "粘贴抖音链接..."}
          className="pr-20"
        />
        {url && urlType !== "unknown" && (
          <Badge
            variant="secondary"
            className="absolute right-2 top-1/2 -translate-y-1/2 text-xs"
          >
            {typeLabels[urlType]}
          </Badge>
        )}
      </div>
      <Button type="button" variant="outline" size="icon" onClick={handlePaste}>
        <Link className="h-4 w-4" />
      </Button>
      <Button type="submit" disabled={!url.trim() || loading}>
        {loading ? (
          <Loader2 className="h-4 w-4 animate-spin" />
        ) : (
          "解析"
        )}
      </Button>
    </form>
  );
}
