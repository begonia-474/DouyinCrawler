import React from "react";
import { RouteObject } from "react-router-dom";

const DouyinIndex = React.lazy(() => import("./pages/index"));
const VideoPage = React.lazy(() => import("./pages/video"));
const UserPage = React.lazy(() => import("./pages/user"));
const LivePage = React.lazy(() => import("./pages/live"));
const LikesPage = React.lazy(() => import("./pages/likes"));
const CollectsPage = React.lazy(() => import("./pages/collects"));
const MixPage = React.lazy(() => import("./pages/mix"));
const FeedPage = React.lazy(() => import("./pages/feed"));
const FavoritesPage = React.lazy(() => import("./pages/favorites"));
const LibraryPage = React.lazy(() => import("./pages/library"));

export const douyinRoutes: RouteObject[] = [
  { index: true, element: <DouyinIndex /> },
  { path: "video", element: <VideoPage /> },
  { path: "user", element: <UserPage /> },
  { path: "live", element: <LivePage /> },
  { path: "likes", element: <LikesPage /> },
  { path: "collects", element: <CollectsPage /> },
  { path: "mix", element: <MixPage /> },
  { path: "feed", element: <FeedPage /> },
  { path: "favorites", element: <FavoritesPage /> },
  { path: "library", element: <LibraryPage /> },
];
