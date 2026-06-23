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
        <main className="flex-1 flex flex-col overflow-hidden py-3 pr-3">
          <div className="flex-1 overflow-auto rounded-[1.5rem] bg-card/40 ring-1 ring-foreground/[0.04] px-10 py-10">
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
