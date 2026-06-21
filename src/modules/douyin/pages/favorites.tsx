import { Header } from "@/components/layout/header";
import { Card, CardContent } from "@/components/ui/card";
import { Heart } from "lucide-react";

export default function FavoritesPage() {
  return (
    <>
      <Header title="收藏" description="我的收藏" />

      <Card>
        <CardContent className="p-8 text-center">
          <Heart className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
          <h3 className="text-lg font-semibold mb-2">收藏功能</h3>
          <p className="text-muted-foreground">
            此功能正在开发中，敬请期待
          </p>
        </CardContent>
      </Card>
    </>
  );
}
