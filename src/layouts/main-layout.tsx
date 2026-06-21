import { Suspense } from "react";
import { Outlet } from "react-router-dom";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Sidebar } from "@/components/layout/sidebar";
import { StatusBar } from "@/components/layout/status-bar";
import { Loading } from "@/shared/components/loading";

export function MainLayout() {
  return (
    <TooltipProvider>
      <div className="flex h-screen overflow-hidden">
        <Sidebar />
        <main className="flex-1 flex flex-col overflow-hidden">
          <div className="flex-1 overflow-auto p-6">
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
