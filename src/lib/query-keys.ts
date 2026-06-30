type QueryParams = Record<string, unknown>;

export const queryKeys = {
  downloads: (params: QueryParams = {}) => ["downloads", params] as const,
  downloadStats: () => ["download-stats"] as const,
  liveRecords: (params: QueryParams = {}) => ["live-records", params] as const,
  liveRecordCount: () => ["live-record-count"] as const,
  videoStats: () => ["video-stats"] as const,
  userStats: () => ["user-stats"] as const,
  videos: (params: QueryParams = {}) => ["videos", params] as const,
  videoCount: (params: QueryParams = {}) => ["video-count", params] as const,
  musicCollection: (params: QueryParams = {}) => ["music-collection", params] as const,
  musicCount: (params: QueryParams = {}) => ["music-count", params] as const,
  users: (params: QueryParams = {}) => ["users", params] as const,
  userCount: (params: QueryParams = {}) => ["user-count", params] as const,
  downloadTrend: (range: string) => ["download-trend", range] as const,
  topAuthors: (limit: number) => ["top-authors", limit] as const,
  storageAnalysis: () => ["storage-analysis"] as const,
  dbHealth: () => ["db-health"] as const,
  downloadTasks: (params: QueryParams = {}) => ["download-tasks", params] as const,
  downloadTaskDetail: (taskId: string) => ["download-task-detail", taskId] as const,
  downloadTaskItems: (taskId: string, status?: string) => ["download-task-items", taskId, status] as const,
  downloadTaskItemCounts: (taskId: string) => ["download-task-item-counts", taskId] as const,
  videoParse: (url: string) => ["video-parse", url] as const,
  // Feed
  tabFeed: (params: QueryParams = {}) => ["tab-feed", params] as const,
  followFeed: (params: QueryParams = {}) => ["follow-feed", params] as const,
  friendFeed: (params: QueryParams = {}) => ["friend-feed", params] as const,
  // 收藏夹
  collectsList: () => ["collects-list"] as const,
  collectsVideos: (id: string, params: QueryParams = {}) => ["collects-videos", id, params] as const,
  // 关注直播
  followingLive: () => ["following-live"] as const,
  // 用户
  userProfile: (url: string) => ["user-profile", url] as const,
  userPosts: (url: string, params: QueryParams = {}) => ["user-posts", url, params] as const,
  userFollowing: (url: string) => ["user-following", url] as const,
  userFollowers: (url: string) => ["user-followers", url] as const,
  userLikes: (url: string, params: QueryParams = {}) => ["user-likes", url, params] as const,
  // 合集
  mixInfo: (url: string, params: QueryParams = {}) => ["mix-info", url, params] as const,
  // 直播
  liveInfo: (url: string) => ["live-info", url] as const,
  liveStatus: () => ["live-status"] as const,
};
