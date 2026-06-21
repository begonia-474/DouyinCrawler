import { useState } from "react";
import { Header } from "@/components/layout/header";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { FolderOpen, Save, RotateCcw } from "lucide-react";

export function SettingsPage() {
  const [cookie, setCookie] = useState("");
  const [downloadPath, setDownloadPath] = useState("Download");
  const [naming, setNaming] = useState("{create}_{desc}");
  const [encryption, setEncryption] = useState("ab");
  const [proxy, setProxy] = useState("");
  const [maxConnections, setMaxConnections] = useState("5");
  const [timeout, setTimeout] = useState("10");
  const [maxRetries, setMaxRetries] = useState("5");

  const handleSelectFolder = async () => {
    // 实际会调用 Tauri dialog API
    // const selected = await open({ directory: true });
    // if (selected) setDownloadPath(selected);
  };

  const handleSave = () => {
    // 实际会调用 Tauri invoke 保存配置
    console.log("Saving config...");
  };

  const handleReset = () => {
    setCookie("");
    setDownloadPath("Download");
    setNaming("{create}_{desc}");
    setEncryption("ab");
    setProxy("");
    setMaxConnections("5");
    setTimeout("10");
    setMaxRetries("5");
  };

  return (
    <>
      <Header title="设置" description="配置 Cookie、下载路径、代理等参数">
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={handleReset}>
            <RotateCcw className="h-4 w-4 mr-1" />
            重置
          </Button>
          <Button size="sm" onClick={handleSave}>
            <Save className="h-4 w-4 mr-1" />
            保存
          </Button>
        </div>
      </Header>

      <div className="space-y-6 max-w-2xl">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Cookie</CardTitle>
          </CardHeader>
          <CardContent>
            <Textarea
              value={cookie}
              onChange={(e) => setCookie(e.target.value)}
              placeholder="粘贴浏览器 Cookie..."
              rows={4}
            />
            <p className="text-xs text-muted-foreground mt-2">
              从浏览器开发者工具中复制 Cookie
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">下载设置</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label>下载路径</Label>
              <div className="flex gap-2">
                <Input
                  value={downloadPath}
                  onChange={(e) => setDownloadPath(e.target.value)}
                  className="flex-1"
                />
                <Button variant="outline" onClick={handleSelectFolder}>
                  <FolderOpen className="h-4 w-4" />
                </Button>
              </div>
            </div>

            <div className="space-y-2">
              <Label>文件命名规则</Label>
              <Input
                value={naming}
                onChange={(e) => setNaming(e.target.value)}
              />
              <p className="text-xs text-muted-foreground">
                支持变量: {"{create}"} {"{desc}"} {"{nickname}"} {"{aweme_id}"}{" "}
                {"{uid}"}
              </p>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">网络设置</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label>加密模式</Label>
              <Select value={encryption} onValueChange={(v) => setEncryption(v ?? "ab")}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="ab">ABogus</SelectItem>
                  <SelectItem value="xb">XBogus</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <Label>代理地址</Label>
              <Input
                value={proxy}
                onChange={(e) => setProxy(e.target.value)}
                placeholder="http://127.0.0.1:7890"
              />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">高级设置</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-3 gap-4">
              <div className="space-y-2">
                <Label>最大连接数</Label>
                <Input
                  type="number"
                  value={maxConnections}
                  onChange={(e) => setMaxConnections(e.target.value)}
                  min="1"
                  max="20"
                />
              </div>
              <div className="space-y-2">
                <Label>超时 (秒)</Label>
                <Input
                  type="number"
                  value={timeout}
                  onChange={(e) => setTimeout(e.target.value)}
                  min="1"
                  max="60"
                />
              </div>
              <div className="space-y-2">
                <Label>最大重试</Label>
                <Input
                  type="number"
                  value={maxRetries}
                  onChange={(e) => setMaxRetries(e.target.value)}
                  min="0"
                  max="10"
                />
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </>
  );
}
