import { Suspense } from "react";
import { Outlet } from "react-router-dom";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Sidebar } from "@/components/layout/sidebar";
import { StatusBar } from "@/components/layout/status-bar";
import { Loading } from "@/shared/components/loading";

export function MainLayout() {
  return (
    <TooltipProvider>
      <div className="flex h-screen overflow-hidden grain-overlay bg-background">
        <Sidebar />
        <main className="flex-1 flex flex-col overflow-hidden">
          <div className="flex-1 overflow-auto px-10 py-8">
            <Suspense fallback={<Loading />}>
              <Outlet />
            </Suspense>
          </div>
          <StatusBar />
        </main>
      </div>
    </TooltipProvider>
  );
}
