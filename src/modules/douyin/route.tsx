import React from "react";
import { RouteObject } from "react-router-dom";

const DouyinIndex = React.lazy(() => import("./pages/index"));
const VideoPage = React.lazy(() => import("./pages/video"));
const UserPage = React.lazy(() => import("./pages/user"));
const LivePage = React.lazy(() => import("./pages/live"));
const FollowingLivePage = React.lazy(() => import("./pages/following-live"));
const LikesPage = React.lazy(() => import("./pages/likes"));
const MixPage = React.lazy(() => import("./pages/mix"));
const FeedPage = React.lazy(() => import("./pages/feed"));
const FavoritesPage = React.lazy(() => import("./pages/favorites"));
const FavoritesDetailPage = React.lazy(() => import("./pages/favorites/[id]"));
const MusicPage = React.lazy(() => import("./pages/music"));
const LibraryPage = React.lazy(() => import("./pages/library"));
const LibraryImagesPage = React.lazy(() => import("./pages/library/images"));
const LibraryLivePage = React.lazy(() => import("./pages/library/live"));
const LibraryMusicPage = React.lazy(() => import("./pages/library/music"));
const LibraryVideoInfoPage = React.lazy(() => import("./pages/library/video-info"));
const LibraryUserInfoPage = React.lazy(() => import("./pages/library/user-info"));

export const douyinRoutes: RouteObject[] = [
  { index: true, element: <DouyinIndex /> },
  { path: "video", element: <VideoPage /> },
  { path: "user", element: <UserPage /> },
  { path: "live", element: <LivePage /> },
  { path: "following-live", element: <FollowingLivePage /> },
  { path: "likes", element: <LikesPage /> },
  { path: "mix", element: <MixPage /> },
  { path: "feed", element: <FeedPage /> },
  { path: "favorites", element: <FavoritesPage /> },
  { path: "favorites/:id", element: <FavoritesDetailPage /> },
  { path: "music", element: <MusicPage /> },
  { path: "library", element: <LibraryPage /> },
  { path: "library/images", element: <LibraryImagesPage /> },
  { path: "library/live", element: <LibraryLivePage /> },
  { path: "library/music", element: <LibraryMusicPage /> },
  { path: "library/video-info", element: <LibraryVideoInfoPage /> },
  { path: "library/user-info", element: <LibraryUserInfoPage /> },
];
