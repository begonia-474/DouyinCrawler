import { useState } from "react";
import { Header } from "@/components/layout/header";
import { Separator } from "@/components/ui/separator";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Button } from "@/components/ui/button";
import { AnimateEntry } from "@/components/shared/animate-entry";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { setConfig } from "@/lib/api";
import {
  Cookie,
  FolderOpen,
  FileText,
  Shield,
  Globe,
  Zap,
  Clock,
  RefreshCw,
  Save,
  RotateCcw,
  User,
  Download,
  Settings,
} from "lucide-react";

function SettingItem({
  icon: Icon,
  title,
  description,
  children,
}: {
  icon: React.ComponentType<{ className?: string }>;
  title: string;
  description?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between py-5">
      <div className="flex items-center gap-3.5 min-w-0">
        <div className="h-8 w-8 rounded-lg bg-foreground/[0.04] flex items-center justify-center shrink-0">
          <Icon className="h-4 w-4 text-muted-foreground" />
        </div>
        <div className="min-w-0">
          <p className="text-sm font-medium">{title}</p>
          {description && (
            <p className="text-xs text-muted-foreground mt-0.5 tracking-wide">{description}</p>
          )}
        </div>
      </div>
      <div className="shrink-0 ml-6">{children}</div>
    </div>
  );
}

export default function SettingsPage() {
  const [cookie, setCookie] = useState("");
  const [downloadPath, setDownloadPath] = useState("Download");
  const [naming, setNaming] = useState("{create}_{desc}");
  const [encryption, setEncryption] = useState("ab");
  const [proxy, setProxy] = useState("");
  const [maxConnections, setMaxConnections] = useState("5");
  const [reqTimeout, setReqTimeout] = useState("10");
  const [maxRetries, setMaxRetries] = useState("5");

  const handleSelectFolder = async () => {};

  const [saving, setSaving] = useState(false);
  const [saveMsg, setSaveMsg] = useState<string | null>(null);

  const handleSave = async () => {
    setSaving(true);
    setSaveMsg(null);
    try {
      await setConfig({
        cookie,
        download_path: downloadPath,
        naming,
        encryption,
        proxy,
      });
      setSaveMsg("保存成功");
    } catch (e) {
      setSaveMsg(e instanceof Error ? e.message : "保存失败");
    }
    setSaving(false);
    window.setTimeout(() => setSaveMsg(null), 2000);
  };

  const handleReset = () => {
    setCookie("");
    setDownloadPath("Download");
    setNaming("{create}_{desc}");
    setEncryption("ab");
    setProxy("");
    setMaxConnections("5");
    setReqTimeout("10");
    setMaxRetries("5");
  };

  return (
    <>
      <AnimateEntry>
        <Header title="设置" description="配置 Cookie、下载路径、代理等参数">
          <div className="flex items-center gap-2">
            {saveMsg && (
              <span className="text-xs text-muted-foreground tracking-wide">{saveMsg}</span>
            )}
            <Button variant="outline" size="sm" className="rounded-lg border-border/60" onClick={handleReset}>
              <RotateCcw className="h-3.5 w-3.5 mr-1.5" />
              重置
            </Button>
            <Button size="sm" className="rounded-lg" onClick={handleSave} disabled={saving}>
              <Save className="h-3.5 w-3.5 mr-1.5" />
              {saving ? "保存中..." : "保存"}
            </Button>
          </div>
        </Header>
      </AnimateEntry>

      <div className="max-w-2xl space-y-10">
        <AnimateEntry delay={50}>
          <div>
            <h3 className="text-xs uppercase tracking-[0.15em] font-medium text-muted-foreground mb-2 flex items-center gap-2">
              <User className="h-3.5 w-3.5" />
              账号设置
            </h3>
            <SettingItem
              icon={Cookie}
              title="Cookie"
              description="从浏览器开发者工具中复制"
            >
              <div className="w-80">
                <Textarea
                  value={cookie}
                  onChange={(e) => setCookie(e.target.value)}
                  placeholder="粘贴浏览器 Cookie..."
                  rows={3}
                  className="text-xs rounded-xl border-border/60"
                />
              </div>
            </SettingItem>
          </div>
        </AnimateEntry>

        <Separator className="bg-border/40" />

        <AnimateEntry delay={100}>
          <div>
            <h3 className="text-xs uppercase tracking-[0.15em] font-medium text-muted-foreground mb-2 flex items-center gap-2">
              <Download className="h-3.5 w-3.5" />
              下载设置
            </h3>
            <SettingItem
              icon={FolderOpen}
              title="下载路径"
              description="视频保存的本地目录"
            >
              <div className="flex gap-2 w-80">
                <Input
                  value={downloadPath}
                  onChange={(e) => setDownloadPath(e.target.value)}
                  className="flex-1 h-9 text-sm rounded-xl border-border/60"
                />
                <Button variant="outline" size="icon" className="h-9 w-9 rounded-xl border-border/60 shrink-0" onClick={handleSelectFolder}>
                  <FolderOpen className="h-3.5 w-3.5" />
                </Button>
              </div>
            </SettingItem>
            <Separator className="bg-border/40" />
            <SettingItem
              icon={FileText}
              title="文件命名规则"
              description="支持变量: {create} {desc} {nickname} {aweme_id} {uid}"
            >
              <Input
                value={naming}
                onChange={(e) => setNaming(e.target.value)}
                className="w-80 h-9 text-sm rounded-xl border-border/60"
              />
            </SettingItem>
          </div>
        </AnimateEntry>

        <Separator className="bg-border/40" />

        <AnimateEntry delay={150}>
          <div>
            <h3 className="text-xs uppercase tracking-[0.15em] font-medium text-muted-foreground mb-2 flex items-center gap-2">
              <Globe className="h-3.5 w-3.5" />
              网络设置
            </h3>
            <SettingItem
              icon={Shield}
              title="加密模式"
              description="请求签名加密算法"
            >
              <Select value={encryption} onValueChange={(v) => setEncryption(v ?? "ab")}>
                <SelectTrigger className="w-80 h-9 text-sm rounded-xl border-border/60">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="ab">ABogus</SelectItem>
                  <SelectItem value="xb">XBogus</SelectItem>
                </SelectContent>
              </Select>
            </SettingItem>
            <Separator className="bg-border/40" />
            <SettingItem
              icon={Globe}
              title="代理地址"
              description="HTTP 代理，留空则不使用"
            >
              <Input
                value={proxy}
                onChange={(e) => setProxy(e.target.value)}
                placeholder="http://127.0.0.1:7890"
                className="w-80 h-9 text-sm rounded-xl border-border/60"
              />
            </SettingItem>
          </div>
        </AnimateEntry>

        <Separator className="bg-border/40" />

        <AnimateEntry delay={200}>
          <div>
            <h3 className="text-xs uppercase tracking-[0.15em] font-medium text-muted-foreground mb-2 flex items-center gap-2">
              <Settings className="h-3.5 w-3.5" />
              高级设置
            </h3>
            <SettingItem
              icon={Zap}
              title="最大连接数"
              description="同时下载的任务数量"
            >
              <Input
                type="number"
                value={maxConnections}
                onChange={(e) => setMaxConnections(e.target.value)}
                min="1"
                max="20"
                className="w-28 h-9 text-sm rounded-xl border-border/60 font-mono tabular-nums"
              />
            </SettingItem>
            <Separator className="bg-border/40" />
            <SettingItem
              icon={Clock}
              title="超时 (秒)"
              description="请求超时时间"
            >
              <Input
                type="number"
                value={reqTimeout}
                onChange={(e) => setReqTimeout(e.target.value)}
                min="1"
                max="60"
                className="w-28 h-9 text-sm rounded-xl border-border/60 font-mono tabular-nums"
              />
            </SettingItem>
            <Separator className="bg-border/40" />
            <SettingItem
              icon={RefreshCw}
              title="最大重试"
              description="失败后自动重试次数"
            >
              <Input
                type="number"
                value={maxRetries}
                onChange={(e) => setMaxRetries(e.target.value)}
                min="0"
                max="10"
                className="w-28 h-9 text-sm rounded-xl border-border/60 font-mono tabular-nums"
              />
            </SettingItem>
          </div>
        </AnimateEntry>
      </div>
    </>
  );
}
