import { Suspense } from "react";
import { Outlet } from "react-router-dom";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Sidebar } from "@/components/layout/sidebar";
import { StatusBar } from "@/components/layout/status-bar";
import { Loading } from "@/shared/components/loading";
import { ErrorBoundary } from "@/components/shared/error-boundary";

export function MainLayout() {
  return (
    <TooltipProvider>
      <div className="flex h-screen overflow-hidden ">
        <Sidebar />
        <main className="flex-1 flex flex-col overflow-hidden">
          <div className="flex-1 overflow-auto px-10 py-8">
            <ErrorBoundary>
              <Suspense fallback={<Loading />}>
                <Outlet />
              </Suspense>
            </ErrorBoundary>
          </div>
          <StatusBar />
        </main>
      </div>
    </TooltipProvider>
  );
}
