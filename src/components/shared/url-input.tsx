import { useState, useCallback, useEffect } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Bezel } from "@/components/shared/bezel";
import { Link, Loader2, ArrowRight, Clipboard, X } from "lucide-react";

interface UrlInputProps {
  onSubmit: (url: string) => void;
  loading?: boolean;
  placeholder?: string;
  allowedTypes?: UrlType[];
  defaultValue?: string;
  autoSubmit?: boolean;
  /** 启用剪贴板自动检测抖音链接 */
  autoDetect?: boolean;
}

type UrlType = "video" | "user" | "mix" | "live" | "unknown";

/** 从分享口令或纯文本中提取 URL，去掉尾部粘连标点 */
function extractUrl(text: string): string {
  const match = text.match(/https?:\/\/\S+/);
  if (match) {
    return match[0].replace(/[,，。.!！?？)）\]】}>」』\s]+$/, "");
  }
  return text.trim();
}

function detectUrlType(input: string): UrlType {
  const url = extractUrl(input);
  if (!url) return "unknown";
  if (url.includes("/user/") || url.includes("sec_user_id")) return "user";
  if (url.includes("/collection/") || url.includes("mix_id")) return "mix";
  if (url.includes("live.douyin.com") || url.includes("/live/") || url.includes("webcast.amemv.com")) return "live";
  if (url.includes("/video/") || url.includes("/note/") || url.includes("modal_id") || url.includes("iesdouyin.com"))
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

export function UrlInput({ onSubmit, loading, placeholder, allowedTypes, defaultValue, autoSubmit, autoDetect }: UrlInputProps) {
  const [url, setUrl] = useState(defaultValue || "");
  const [typeError, setTypeError] = useState<string | null>(null);
  const [clipboardHint, setClipboardHint] = useState<string | null>(null);
  const urlType = detectUrlType(url);

  useEffect(() => {
    if (defaultValue && autoSubmit) {
      onSubmit(defaultValue);
    }
  }, [defaultValue, autoSubmit, onSubmit]);

  // 剪贴板自动检测：监听 paste 事件，仅在用户粘贴时读取剪贴板
  useEffect(() => {
    if (!autoDetect) return;

    const handlePasteEvent = (e: ClipboardEvent) => {
      // 如果焦点在输入框内，让输入框自然处理粘贴
      if (document.activeElement?.tagName === "INPUT" || document.activeElement?.tagName === "TEXTAREA") {
        return;
      }

      const text = e.clipboardData?.getData("text") ?? "";
      if (!text) return;

      const extracted = extractUrl(text);
      const type = detectUrlType(extracted);
      if (type !== "unknown" && extracted !== url) {
        setClipboardHint(extracted);
      }
    };

    document.addEventListener("paste", handlePasteEvent);
    return () => document.removeEventListener("paste", handlePasteEvent);
  }, [autoDetect, url]);

  const handleAcceptClipboard = useCallback(() => {
    if (clipboardHint) {
      setUrl(clipboardHint);
      setClipboardHint(null);
    }
  }, [clipboardHint]);

  const handleDismissClipboard = useCallback(() => {
    setClipboardHint(null);
  }, []);

  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      setTypeError(null);
      if (!url.trim()) return;

      if (allowedTypes && allowedTypes.length > 0 && urlType !== "unknown" && !allowedTypes.includes(urlType)) {
        setTypeError(`当前页面不支持「${typeLabels[urlType]}」类型链接，请前往「${typeLabels[allowedTypes[0]]}」页面`);
        return;
      }

      // 提交前提取纯 URL（兼容分享口令）
      const cleaned = extractUrl(url);
      onSubmit(cleaned);
    },
    [url, onSubmit, allowedTypes, urlType]
  );

  const handlePaste = useCallback(() => {
    navigator.clipboard.readText().then((text) => {
      if (text) {
        // 粘贴时自动从口令中提取 URL
        setUrl(extractUrl(text));
      }
    });
  }, []);

  return (
    <div className="space-y-3">
      <form onSubmit={handleSubmit} className="flex gap-3 items-start">
        <div className="flex-1 relative">
          <Bezel radius="xl" padding="sm">
            <div className="relative">
              <Input
                value={url}
                onChange={(e) => { setUrl(e.target.value); setTypeError(null); }}
                placeholder={placeholder || "粘贴抖音链接..."}
                className="h-12 rounded-[calc(1.5rem-0.25rem)] border-0 bg-transparent pr-22 text-sm focus-visible:ring-0 focus-visible:border-0"
              />
              {url && urlType !== "unknown" && (
                <Badge
                  variant="secondary"
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-[11px] tracking-wide bg-foreground/[0.06]"
                >
                  {typeLabels[urlType]}
                </Badge>
              )}
            </div>
          </Bezel>
        </div>
        <Button type="button" variant="capsule" size="icon" className="h-12 w-12 shrink-0" onClick={handlePaste}>
          <Link className="h-4 w-4" />
        </Button>
        <Button type="submit" disabled={!url.trim() || loading} className="h-12 px-6 shrink-0">
          {loading ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <>
              解析
              <span className="ml-1.5 inline-flex items-center justify-center w-6 h-6 rounded-full bg-foreground/10 group-hover/button:translate-x-0.5 transition-transform duration-300">
                <ArrowRight className="h-3 w-3" />
              </span>
            </>
          )}
        </Button>
      </form>
      {typeError && (
        <p className="text-xs text-destructive tracking-wide">{typeError}</p>
      )}
      {clipboardHint && (
        <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-primary/[0.06] border border-primary/10 text-xs">
          <Clipboard className="h-3.5 w-3.5 text-primary/70 shrink-0" />
          <span className="text-muted-foreground truncate flex-1 min-w-0">
            检测到链接：{clipboardHint}
          </span>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="h-6 px-2 text-xs shrink-0"
            onClick={handleAcceptClipboard}
          >
            使用
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            className="h-6 w-6 shrink-0"
            onClick={handleDismissClipboard}
          >
            <X className="h-3 w-3" />
          </Button>
        </div>
      )}
    </div>
  );
}
