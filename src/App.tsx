import { BrowserRouter, Routes, Route } from "react-router-dom";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Sidebar } from "@/components/layout/sidebar";
import { StatusBar } from "@/components/layout/status-bar";
import { HomePage } from "@/pages/home";
import { UserPage } from "@/pages/user";
import { MixPage } from "@/pages/mix";
import { SearchPage } from "@/pages/search";
import { CommentsPage } from "@/pages/comments";
import { LivePage } from "@/pages/live";
import { FeedPage } from "@/pages/feed";
import { MusicPage } from "@/pages/music";
import { DownloadsPage } from "@/pages/downloads";
import { SettingsPage } from "@/pages/settings";

function App() {
  return (
    <TooltipProvider>
      <BrowserRouter>
        <div className="flex h-screen overflow-hidden">
          <Sidebar />
          <main className="flex-1 flex flex-col overflow-hidden">
            <div className="flex-1 overflow-auto p-6">
              <Routes>
                <Route path="/" element={<HomePage />} />
                <Route path="/user" element={<UserPage />} />
                <Route path="/mix" element={<MixPage />} />
                <Route path="/search" element={<SearchPage />} />
                <Route path="/comments" element={<CommentsPage />} />
                <Route path="/live" element={<LivePage />} />
                <Route path="/feed" element={<FeedPage />} />
                <Route path="/music" element={<MusicPage />} />
                <Route path="/downloads" element={<DownloadsPage />} />
                <Route path="/settings" element={<SettingsPage />} />
              </Routes>
            </div>
            <StatusBar />
          </main>
        </div>
      </BrowserRouter>
    </TooltipProvider>
  );
}

export default App;
