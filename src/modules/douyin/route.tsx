import React from "react";
import { RouteObject } from "react-router-dom";

const HomePage = React.lazy(() => import("./pages/home"));
const UserPage = React.lazy(() => import("./pages/user"));
const MixPage = React.lazy(() => import("./pages/mix"));
const SearchPage = React.lazy(() => import("./pages/search"));
const CommentsPage = React.lazy(() => import("./pages/comments"));
const LivePage = React.lazy(() => import("./pages/live"));
const FeedPage = React.lazy(() => import("./pages/feed"));
const MusicPage = React.lazy(() => import("./pages/music"));

export const douyinRoutes: RouteObject[] = [
  { index: true, element: <HomePage /> },
  { path: "user", element: <UserPage /> },
  { path: "mix", element: <MixPage /> },
  { path: "search", element: <SearchPage /> },
  { path: "comments", element: <CommentsPage /> },
  { path: "live", element: <LivePage /> },
  { path: "feed", element: <FeedPage /> },
  { path: "music", element: <MusicPage /> },
];
