import { Navigate, RouteObject } from "react-router-dom";
import { MainLayout } from "@/layouts/main-layout";
import { douyinRoutes } from "@/modules/douyin/route";
import { sharedRoutes } from "@/shared/route";

export const routes: RouteObject[] = [
  {
    path: "/",
    element: <MainLayout />,
    children: [
      // 默认重定向到抖音
      { index: true, element: <Navigate to="/douyin" replace /> },

      // 平台路由
      {
        path: "douyin",
        children: douyinRoutes,
      },
      // {
      //   path: "kuaishou",
      //   children: kuaishouRoutes,
      // },
      // {
      //   path: "bilibili",
      //   children: bilibiliRoutes,
      // },

      // 共享路由
      ...sharedRoutes,
    ],
  },
];
