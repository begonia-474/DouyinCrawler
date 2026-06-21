import { Header } from "@/components/layout/header";
import { Card, CardContent } from "@/components/ui/card";
import { Database } from "lucide-react";

export default function LibraryPage() {
  return (
    <>
      <Header title="资料库" description="数据管理" />

      <Card>
        <CardContent className="p-8 text-center">
          <Database className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
          <h3 className="text-lg font-semibold mb-2">资料库功能</h3>
          <p className="text-muted-foreground">
            此功能正在开发中，将打通数据库实现数据管理
          </p>
        </CardContent>
      </Card>
    </>
  );
}
