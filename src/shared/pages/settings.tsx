import { useState, useEffect } from "react";
import { Header } from "@/components/layout/header";
import { Separator } from "@/components/ui/separator";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { AnimateEntry } from "@/components/shared/animate-entry";
import { Bezel } from "@/components/shared/bezel";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { setConfig, getConfig } from "@/lib/api";
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
  Music,
  Image,
  FileText as DescIcon,
  Folder,
  Calendar,
  List,
  Hash,
  Activity,
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
      <div className="flex items-center gap-4 min-w-0">
        <div className="h-9 w-9 rounded-xl bg-foreground/[0.04] ring-1 ring-foreground/[0.06] flex items-center justify-center shrink-0">
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
  const [reqTimeout, setReqTimeout] = useState("5");
  const [maxRetries, setMaxRetries] = useState("5");
  const [pageCounts, setPageCounts] = useState("20");
  const [maxCounts, setMaxCounts] = useState("0");
  const [maxTasks, setMaxTasks] = useState("10");
  const [appName, setAppName] = useState("douyin");
  const [folderize, setFolderize] = useState(false);
  const [music, setMusic] = useState(false);
  const [cover, setCover] = useState(false);
  const [desc, setDesc] = useState(false);
  const [interval, setInterval] = useState("");

  // 加载当前配置
  useEffect(() => {
    getConfig().then((cfg) => {
      setCookie(cfg.cookie || "");
      setDownloadPath(cfg.download_path || "Download");
      setNaming(cfg.naming || "{create}_{desc}");
      setEncryption(cfg.encryption || "ab");
      setProxy(cfg.proxy || "");
      setMaxConnections(String(cfg.max_connections ?? 5));
      setReqTimeout(String(cfg.timeout ?? 5));
      setMaxRetries(String(cfg.max_retries ?? 5));
      setPageCounts(String(cfg.page_counts ?? 20));
      setMaxCounts(String(cfg.max_counts ?? 0));
      setMaxTasks(String(cfg.max_tasks ?? 10));
      setAppName(cfg.app_name || "douyin");
      setFolderize(cfg.folderize ?? false);
      setMusic(cfg.music ?? false);
      setCover(cfg.cover ?? false);
      setDesc(cfg.desc ?? false);
      setInterval(cfg.interval || "");
    }).catch(() => {});
  }, []);

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
        app_name: appName,
        folderize: folderize.toString(),
        music: music.toString(),
        cover: cover.toString(),
        desc: desc.toString(),
        interval,
        page_counts: pageCounts,
        max_counts: maxCounts,
        timeout: reqTimeout,
        max_connections: maxConnections,
        max_retries: maxRetries,
        max_tasks: maxTasks,
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
    setReqTimeout("5");
    setMaxRetries("5");
    setPageCounts("20");
    setMaxCounts("0");
    setMaxTasks("10");
    setAppName("douyin");
    setFolderize(false);
    setMusic(false);
    setCover(false);
    setDesc(false);
    setInterval("");
  };

  return (
    <>
      <AnimateEntry>
        <Header title="设置" description="配置 Cookie、下载路径、代理等参数">
          <div className="flex items-center gap-2">
            {saveMsg && (
              <span className="text-xs text-muted-foreground tracking-wide">{saveMsg}</span>
            )}
            <Button variant="capsule" size="sm" onClick={handleReset}>
              <RotateCcw className="h-3.5 w-3.5 mr-1.5" />
              重置
            </Button>
            <Button size="sm" onClick={handleSave} disabled={saving}>
              <Save className="h-3.5 w-3.5 mr-1.5" />
              {saving ? "保存中..." : "保存"}
            </Button>
          </div>
        </Header>
      </AnimateEntry>

      <div className="space-y-12">
        <AnimateEntry delay={50}>
          <div>
            <span className="inline-block rounded-full px-3 py-1 text-[10px] uppercase tracking-[0.2em] font-medium bg-foreground/[0.05] text-muted-foreground mb-4">
              <span className="flex items-center gap-1.5"><User className="h-3 w-3" />账号</span>
            </span>
            <Bezel radius="xl">
              <div className="p-6">
                <SettingItem
                  icon={Cookie}
                  title="Cookie"
                  description="从浏览器开发者工具中复制"
                >
                  <div className="w-full max-w-sm">
                    <Textarea
                      value={cookie}
                      onChange={(e) => setCookie(e.target.value)}
                      placeholder="粘贴浏览器 Cookie..."
                      rows={3}
                      className="text-xs rounded-xl border-foreground/[0.08] bg-foreground/[0.03]"
                    />
                  </div>
                </SettingItem>
              </div>
            </Bezel>
          </div>
        </AnimateEntry>

        <AnimateEntry delay={100}>
          <div>
            <span className="inline-block rounded-full px-3 py-1 text-[10px] uppercase tracking-[0.2em] font-medium bg-foreground/[0.05] text-muted-foreground mb-4">
              <span className="flex items-center gap-1.5"><Download className="h-3 w-3" />下载</span>
            </span>
            <Bezel radius="xl">
              <div className="p-6">
                <SettingItem
                  icon={FolderOpen}
                  title="下载路径"
                  description="视频保存的本地目录"
                >
                  <div className="flex gap-2 w-full max-w-sm">
                    <Input
                      value={downloadPath}
                      onChange={(e) => setDownloadPath(e.target.value)}
                      className="flex-1 h-9 text-sm rounded-xl border-foreground/[0.08] bg-foreground/[0.03]"
                    />
                    <Button variant="capsule" size="icon" className="h-9 w-9 shrink-0" onClick={handleSelectFolder}>
                      <FolderOpen className="h-3.5 w-3.5" />
                    </Button>
                  </div>
                </SettingItem>
                <Separator className="bg-foreground/[0.06]" />
                <SettingItem
                  icon={FileText}
                  title="文件命名规则"
                  description="支持变量: {create} {desc} {caption} {nickname} {aweme_id} {uid}"
                >
                  <Input
                    value={naming}
                    onChange={(e) => setNaming(e.target.value)}
                    className="w-full max-w-sm h-9 text-sm rounded-xl border-foreground/[0.08] bg-foreground/[0.03]"
                  />
                </SettingItem>
                <Separator className="bg-foreground/[0.06]" />
                <SettingItem
                  icon={Folder}
                  title="作品独立文件夹"
                  description="为每个作品创建单独的子文件夹"
                >
                  <Switch
                    checked={folderize}
                    onCheckedChange={setFolderize}
                  />
                </SettingItem>
                <Separator className="bg-foreground/[0.06]" />
                <SettingItem
                  icon={Calendar}
                  title="日期区间过滤"
                  description="格式: YYYY-MM-DD|YYYY-MM-DD，留空下载全部"
                >
                  <Input
                    value={interval}
                    onChange={(e) => setInterval(e.target.value)}
                    placeholder="2024-01-01|2024-12-31"
                    className="w-full max-w-sm h-9 text-sm rounded-xl border-foreground/[0.08] bg-foreground/[0.03]"
                  />
                </SettingItem>
              </div>
            </Bezel>
          </div>
        </AnimateEntry>

        <AnimateEntry delay={125}>
          <div>
            <span className="inline-block rounded-full px-3 py-1 text-[10px] uppercase tracking-[0.2em] font-medium bg-foreground/[0.05] text-muted-foreground mb-4">
              <span className="flex items-center gap-1.5"><Download className="h-3 w-3" />附属文件</span>
            </span>
            <Bezel radius="xl">
              <div className="p-6">
                <SettingItem
                  icon={Music}
                  title="下载原声"
                  description="同时保存视频的背景音乐"
                >
                  <Switch
                    checked={music}
                    onCheckedChange={setMusic}
                  />
                </SettingItem>
                <Separator className="bg-foreground/[0.06]" />
                <SettingItem
                  icon={Image}
                  title="下载封面"
                  description="同时保存视频封面图片"
                >
                  <Switch
                    checked={cover}
                    onCheckedChange={setCover}
                  />
                </SettingItem>
                <Separator className="bg-foreground/[0.06]" />
                <SettingItem
                  icon={DescIcon}
                  title="下载文案"
                  description="同时保存视频描述文案"
                >
                  <Switch
                    checked={desc}
                    onCheckedChange={setDesc}
                  />
                </SettingItem>
              </div>
            </Bezel>
          </div>
        </AnimateEntry>

        <AnimateEntry delay={150}>
          <div>
            <span className="inline-block rounded-full px-3 py-1 text-[10px] uppercase tracking-[0.2em] font-medium bg-foreground/[0.05] text-muted-foreground mb-4">
              <span className="flex items-center gap-1.5"><Globe className="h-3 w-3" />网络</span>
            </span>
            <Bezel radius="xl">
              <div className="p-6">
                <SettingItem
                  icon={Shield}
                  title="加密模式"
                  description="请求签名加密算法"
                >
                  <Select value={encryption} onValueChange={(v) => setEncryption(v ?? "ab")}>
                    <SelectTrigger className="w-full max-w-sm h-9 text-sm rounded-xl border-foreground/[0.08] bg-foreground/[0.03]">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="ab">ABogus</SelectItem>
                      <SelectItem value="xb">XBogus</SelectItem>
                    </SelectContent>
                  </Select>
                </SettingItem>
                <Separator className="bg-foreground/[0.06]" />
                <SettingItem
                  icon={Globe}
                  title="代理地址"
                  description="HTTP 代理，留空则不使用"
                >
                  <Input
                    value={proxy}
                    onChange={(e) => setProxy(e.target.value)}
                    placeholder="http://127.0.0.1:7890"
                    className="w-full max-w-sm h-9 text-sm rounded-xl border-foreground/[0.08] bg-foreground/[0.03]"
                  />
                </SettingItem>
              </div>
            </Bezel>
          </div>
        </AnimateEntry>

        <AnimateEntry delay={200}>
          <div>
            <span className="inline-block rounded-full px-3 py-1 text-[10px] uppercase tracking-[0.2em] font-medium bg-foreground/[0.05] text-muted-foreground mb-4">
              <span className="flex items-center gap-1.5"><Settings className="h-3 w-3" />高级</span>
            </span>
            <Bezel radius="xl">
              <div className="p-6">
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
                    className="w-28 h-9 text-sm rounded-xl border-foreground/[0.08] bg-foreground/[0.03] font-mono tabular-nums"
                  />
                </SettingItem>
                <Separator className="bg-foreground/[0.06]" />
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
                    className="w-28 h-9 text-sm rounded-xl border-foreground/[0.08] bg-foreground/[0.03] font-mono tabular-nums"
                  />
                </SettingItem>
                <Separator className="bg-foreground/[0.06]" />
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
                    className="w-28 h-9 text-sm rounded-xl border-foreground/[0.08] bg-foreground/[0.03] font-mono tabular-nums"
                  />
                </SettingItem>
                <Separator className="bg-foreground/[0.06]" />
                <SettingItem
                  icon={List}
                  title="每页数量"
                  description="列表预览每次加载的条目数"
                >
                  <Input
                    type="number"
                    value={pageCounts}
                    onChange={(e) => setPageCounts(e.target.value)}
                    min="1"
                    max="50"
                    className="w-28 h-9 text-sm rounded-xl border-foreground/[0.08] bg-foreground/[0.03] font-mono tabular-nums"
                  />
                </SettingItem>
                <Separator className="bg-foreground/[0.06]" />
                <SettingItem
                  icon={Hash}
                  title="最大解析数量"
                  description="0 表示无限制，下载全部作品"
                >
                  <Input
                    type="number"
                    value={maxCounts}
                    onChange={(e) => setMaxCounts(e.target.value)}
                    min="0"
                    className="w-28 h-9 text-sm rounded-xl border-foreground/[0.08] bg-foreground/[0.03] font-mono tabular-nums"
                  />
                </SettingItem>
                <Separator className="bg-foreground/[0.06]" />
                <SettingItem
                  icon={Activity}
                  title="最大并发任务"
                  description="批量下载时同时进行的任务数"
                >
                  <Input
                    type="number"
                    value={maxTasks}
                    onChange={(e) => setMaxTasks(e.target.value)}
                    min="1"
                    max="50"
                    className="w-28 h-9 text-sm rounded-xl border-foreground/[0.08] bg-foreground/[0.03] font-mono tabular-nums"
                  />
                </SettingItem>
              </div>
            </Bezel>
          </div>
        </AnimateEntry>
      </div>
    </>
  );
}
