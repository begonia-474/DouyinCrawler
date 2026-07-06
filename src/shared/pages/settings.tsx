import { useEffect, useState } from "react";
import { useForm, Controller } from "react-hook-form";
import { z } from "zod/v4";
import { zodResolver } from "@hookform/resolvers/zod";
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
import { setConfig, getConfig, openFolder } from "@/lib/api";
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

const _settingsSchema = z.object({
  cookie: z.string(),
  downloadPath: z.string(),
  naming: z.string(),
  encryption: z.string(),
  proxy: z.string(),
  maxConnections: z.string(),
  reqTimeout: z.string(),
  maxRetries: z.string(),
  pageCounts: z.string(),
  maxCounts: z.string(),
  maxTasks: z.string(),
  appName: z.string(),
  folderize: z.boolean(),
  music: z.boolean(),
  cover: z.boolean(),
  desc: z.boolean(),
  interval: z.string(),
});

type SettingsForm = z.infer<typeof _settingsSchema>;

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
      <div className="min-w-0 ml-6">{children}</div>
    </div>
  );
}

export default function SettingsPage() {
  const { register, handleSubmit, control, reset, getValues, formState: { isSubmitting } } = useForm<SettingsForm>({
    // eslint-disable-next-line @typescript-eslint/no-explicit-any -- zodResolver 类型签名尚未完全兼容 zod v4
    resolver: zodResolver(_settingsSchema as any),
    defaultValues: {
      cookie: "",
      downloadPath: "Download",
      naming: "{create}_{desc}",
      encryption: "ab",
      proxy: "",
      maxConnections: "5",
      reqTimeout: "5",
      maxRetries: "5",
      pageCounts: "20",
      maxCounts: "0",
      maxTasks: "10",
      appName: "douyin",
      folderize: false,
      music: false,
      cover: false,
      desc: false,
      interval: "",
    },
  });

  // 加载当前配置
  useEffect(() => {
    getConfig().then((cfg) => {
      reset({
        cookie: cfg.cookie || "",
        downloadPath: cfg.download_path || "Download",
        naming: cfg.naming || "{create}_{desc}",
        encryption: cfg.encryption || "ab",
        proxy: cfg.proxy || "",
        maxConnections: String(cfg.max_connections ?? 5),
        reqTimeout: String(cfg.timeout ?? 5),
        maxRetries: String(cfg.max_retries ?? 5),
        pageCounts: String(cfg.page_counts ?? 20),
        maxCounts: String(cfg.max_counts ?? 0),
        maxTasks: String(cfg.max_tasks ?? 10),
        appName: cfg.app_name || "douyin",
        folderize: cfg.folderize ?? false,
        music: cfg.music ?? false,
        cover: cfg.cover ?? false,
        desc: cfg.desc ?? false,
        interval: cfg.interval || "",
      });
    }).catch(() => {});
  }, [reset]);

  const [saveMsg, setSaveMsg] = useState<string | null>(null);

  const onSubmit = async (data: SettingsForm) => {
    setSaveMsg(null);
    try {
      await setConfig({
        cookie: data.cookie,
        download_path: data.downloadPath,
        naming: data.naming,
        encryption: data.encryption,
        proxy: data.proxy,
        app_name: data.appName,
        folderize: data.folderize.toString(),
        music: data.music.toString(),
        cover: data.cover.toString(),
        desc: data.desc.toString(),
        interval: data.interval,
        page_counts: data.pageCounts,
        max_counts: data.maxCounts,
        timeout: data.reqTimeout,
        max_connections: data.maxConnections,
        max_retries: data.maxRetries,
        max_tasks: data.maxTasks,
      });
      setSaveMsg("保存成功");
    } catch (e) {
      setSaveMsg(e instanceof Error ? e.message : "保存失败");
    }
    window.setTimeout(() => setSaveMsg(null), 2000);
  };

  const handleReset = () => {
    reset({
      cookie: "",
      downloadPath: "Download",
      naming: "{create}_{desc}",
      encryption: "ab",
      proxy: "",
      maxConnections: "5",
      reqTimeout: "5",
      maxRetries: "5",
      pageCounts: "20",
      maxCounts: "0",
      maxTasks: "10",
      appName: "douyin",
      folderize: false,
      music: false,
      cover: false,
      desc: false,
      interval: "",
    });
  };

  return (
    <div className="max-w-3xl mx-auto">
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
            <Button size="sm" onClick={handleSubmit(onSubmit)} disabled={isSubmitting}>
              <Save className="h-3.5 w-3.5 mr-1.5" />
              {isSubmitting ? "保存中..." : "保存"}
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
                  <div className="w-96 max-w-full">
                    <Textarea
                      {...register("cookie")}
                      placeholder="粘贴浏览器 Cookie..."
                      rows={3}
                      className="field-sizing-fixed text-xs rounded-xl border-foreground/[0.08] bg-foreground/[0.03]"
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
                      {...register("downloadPath")}
                      className="flex-1 h-9 text-sm rounded-xl border-foreground/[0.08] bg-foreground/[0.03]"
                    />
                    <Button
                      variant="capsule"
                      size="icon"
                      className="h-9 w-9 shrink-0"
                      onClick={() => openFolder(getValues("downloadPath"))}
                    >
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
                    {...register("naming")}
                    className="w-full max-w-sm h-9 text-sm rounded-xl border-foreground/[0.08] bg-foreground/[0.03]"
                  />
                </SettingItem>
                <Separator className="bg-foreground/[0.06]" />
                <SettingItem
                  icon={Folder}
                  title="作品独立文件夹"
                  description="为每个作品创建单独的子文件夹"
                >
                  <Controller
                    name="folderize"
                    control={control}
                    render={({ field }) => (
                      <Switch checked={field.value} onCheckedChange={field.onChange} />
                    )}
                  />
                </SettingItem>
                <Separator className="bg-foreground/[0.06]" />
                <SettingItem
                  icon={Calendar}
                  title="日期区间过滤"
                  description="格式: YYYY-MM-DD|YYYY-MM-DD，留空下载全部"
                >
                  <Input
                    {...register("interval")}
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
                  <Controller
                    name="music"
                    control={control}
                    render={({ field }) => (
                      <Switch checked={field.value} onCheckedChange={field.onChange} />
                    )}
                  />
                </SettingItem>
                <Separator className="bg-foreground/[0.06]" />
                <SettingItem
                  icon={Image}
                  title="下载封面"
                  description="同时保存视频封面图片"
                >
                  <Controller
                    name="cover"
                    control={control}
                    render={({ field }) => (
                      <Switch checked={field.value} onCheckedChange={field.onChange} />
                    )}
                  />
                </SettingItem>
                <Separator className="bg-foreground/[0.06]" />
                <SettingItem
                  icon={DescIcon}
                  title="下载文案"
                  description="同时保存视频描述文案"
                >
                  <Controller
                    name="desc"
                    control={control}
                    render={({ field }) => (
                      <Switch checked={field.value} onCheckedChange={field.onChange} />
                    )}
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
                  <Controller
                    name="encryption"
                    control={control}
                    render={({ field }) => (
                      <Select value={field.value} onValueChange={field.onChange}>
                        <SelectTrigger className="w-full max-w-sm h-9 text-sm rounded-xl border-foreground/[0.08] bg-foreground/[0.03]">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="ab">ABogus</SelectItem>
                          <SelectItem value="xb">XBogus</SelectItem>
                        </SelectContent>
                      </Select>
                    )}
                  />
                </SettingItem>
                <Separator className="bg-foreground/[0.06]" />
                <SettingItem
                  icon={Globe}
                  title="代理地址"
                  description="HTTP 代理，留空则不使用"
                >
                  <Input
                    {...register("proxy")}
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
                    {...register("maxConnections")}
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
                    {...register("reqTimeout")}
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
                    {...register("maxRetries")}
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
                    {...register("pageCounts")}
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
                    {...register("maxCounts")}
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
                    {...register("maxTasks")}
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
    </div>
  );
}
