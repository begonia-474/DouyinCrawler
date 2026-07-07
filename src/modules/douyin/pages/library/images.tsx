import { toast } from "sonner";
import { VideoList } from "./_shared";
import { getDownloadDirByAwemeId, openFolder } from "@/lib/api";
import type { VideoInfo } from "@/lib/tauri-types";

export default function LibraryImagesPage() {
  const handleOpenFolder = async (item: VideoInfo) => {
    try {
      const dir = await getDownloadDirByAwemeId(item.aweme_id);
      if (dir) {
        await openFolder(dir);
      } else {
        toast.error("未找到下载文件");
      }
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "打开文件夹失败");
    }
  };

  return <VideoList postType="images" title="图集" onOpenFolder={handleOpenFolder} />;
}
