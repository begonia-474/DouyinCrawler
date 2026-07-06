type QueryParams = Record<string, unknown>;

export const queryKeys = {
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
  // 关注直播
  followingLive: () => ["following-live"] as const,
};
