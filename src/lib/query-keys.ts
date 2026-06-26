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
};
