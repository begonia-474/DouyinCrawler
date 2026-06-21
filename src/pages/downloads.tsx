import { Header } from "@/components/layout/header";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Download,
  CheckCircle2,
  AlertCircle,
  Loader2,
  Trash2,
  FolderOpen,
  X,
} from "lucide-react";

interface Task {
  id: string;
  filename: string;
  size: string;
  progress: number;
  status: "pending" | "downloading" | "completed" | "error";
  speed?: string;
  error?: string;
}

const mockTasks: Task[] = [
  {
    id: "1",
    filename: "2024-01-15_120000_旅行日记.mp4",
    size: "45.2 MB",
    progress: 100,
    status: "completed",
  },
  {
    id: "2",
    filename: "2024-01-15_110000_美食分享.mp4",
    size: "32.1 MB",
    progress: 67,
    status: "downloading",
    speed: "2.3 MB/s",
  },
  {
    id: "3",
    filename: "2024-01-15_100000_搞笑日常.mp4",
    size: "28.5 MB",
    progress: 0,
    status: "error",
    error: "网络连接失败",
  },
];

function StatusIcon({ status }: { status: Task["status"] }) {
  switch (status) {
    case "completed":
      return <CheckCircle2 className="h-4 w-4 text-emerald-500" />;
    case "downloading":
      return <Loader2 className="h-4 w-4 text-blue-500 animate-spin" />;
    case "error":
      return <AlertCircle className="h-4 w-4 text-destructive" />;
    default:
      return <Download className="h-4 w-4 text-muted-foreground" />;
  }
}

function TaskItem({ task }: { task: Task }) {
  return (
    <div className="flex items-center gap-4 p-4 border rounded-lg">
      <StatusIcon status={task.status} />
      <div className="flex-1 min-w-0">
        <p className="text-sm font-medium truncate">{task.filename}</p>
        <div className="flex items-center gap-2 mt-1">
          <span className="text-xs text-muted-foreground">{task.size}</span>
          {task.speed && (
            <span className="text-xs text-blue-500">{task.speed}</span>
          )}
          {task.error && (
            <span className="text-xs text-destructive">{task.error}</span>
          )}
        </div>
        {task.status === "downloading" && (
          <Progress value={task.progress} className="mt-2 h-1.5" />
        )}
      </div>
      <div className="flex items-center gap-2">
        {task.status === "completed" && (
          <Button variant="ghost" size="icon">
            <FolderOpen className="h-4 w-4" />
          </Button>
        )}
        {task.status === "downloading" && (
          <Button variant="ghost" size="icon">
            <X className="h-4 w-4" />
          </Button>
        )}
        <Button variant="ghost" size="icon">
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}

export function DownloadsPage() {
  const activeTasks = mockTasks.filter(
    (t) => t.status === "downloading" || t.status === "pending"
  );
  const completedTasks = mockTasks.filter((t) => t.status === "completed");
  const errorTasks = mockTasks.filter((t) => t.status === "error");

  return (
    <>
      <Header title="下载管理" description="查看下载队列和历史记录">
        <Button variant="outline" size="sm">
          <FolderOpen className="h-4 w-4 mr-1" />
          打开文件夹
        </Button>
      </Header>

      <Tabs defaultValue="active">
        <TabsList>
          <TabsTrigger value="active">
            进行中
            {activeTasks.length > 0 && (
              <Badge variant="secondary" className="ml-1.5">
                {activeTasks.length}
              </Badge>
            )}
          </TabsTrigger>
          <TabsTrigger value="completed">
            已完成
            {completedTasks.length > 0 && (
              <Badge variant="secondary" className="ml-1.5">
                {completedTasks.length}
              </Badge>
            )}
          </TabsTrigger>
          <TabsTrigger value="error">
            失败
            {errorTasks.length > 0 && (
              <Badge variant="destructive" className="ml-1.5">
                {errorTasks.length}
              </Badge>
            )}
          </TabsTrigger>
        </TabsList>

        <TabsContent value="active" className="mt-4 space-y-3">
          {activeTasks.length === 0 ? (
            <Card>
              <CardContent className="p-8 text-center text-muted-foreground">
                没有进行中的下载任务
              </CardContent>
            </Card>
          ) : (
            activeTasks.map((task) => <TaskItem key={task.id} task={task} />)
          )}
        </TabsContent>

        <TabsContent value="completed" className="mt-4 space-y-3">
          {completedTasks.length === 0 ? (
            <Card>
              <CardContent className="p-8 text-center text-muted-foreground">
                没有已完成的下载
              </CardContent>
            </Card>
          ) : (
            completedTasks.map((task) => <TaskItem key={task.id} task={task} />)
          )}
        </TabsContent>

        <TabsContent value="error" className="mt-4 space-y-3">
          {errorTasks.length === 0 ? (
            <Card>
              <CardContent className="p-8 text-center text-muted-foreground">
                没有失败的下载
              </CardContent>
            </Card>
          ) : (
            errorTasks.map((task) => <TaskItem key={task.id} task={task} />)
          )}
        </TabsContent>
      </Tabs>
    </>
  );
}
