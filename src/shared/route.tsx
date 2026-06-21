import React from "react";
import { RouteObject } from "react-router-dom";

const DownloadsPage = React.lazy(() => import("./pages/downloads"));
const SettingsPage = React.lazy(() => import("./pages/settings"));

export const sharedRoutes: RouteObject[] = [
  { path: "downloads", element: <DownloadsPage /> },
  { path: "settings", element: <SettingsPage /> },
];
