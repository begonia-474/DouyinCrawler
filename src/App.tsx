import { BrowserRouter, useRoutes } from "react-router-dom";
import { Toaster } from "sonner";
import { routes } from "@/routes";

function AppRoutes() {
  return useRoutes(routes);
}

function App() {
  return (
    <BrowserRouter>
      <AppRoutes />
      <Toaster position="top-right" richColors />
    </BrowserRouter>
  );
}

export default App;
