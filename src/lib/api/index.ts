// 统一 re-export — 保持 `import { ... } from "@/lib/api"` 兼容
export { wrap, pyCall } from "./core";
export type { BackendResponse } from "./core";

export { getConfig, setConfig } from "./config";
export type { AppConfig } from "./config";

export { getPostDetail, getPostStats } from "./video";

export { downloadOne, startBatchDownload, startDownload, downloadUserPosts, downloadUserLikes, downloadMix, downloadCollectsVideo, getMixInfo } from "./download";

export { getDownloadTasks, getDownloadTaskDetail, getDownloadTaskItems, getDownloadTaskItemCounts, deleteDownloadTask } from "./download-task";

export { getUserProfile, getUserPosts, getUserLikes, getUserCollects, getCollectsVideoList, getUserFollowing, getUserFollowers } from "./user";

export { getLiveInfo, startLiveRecord, stopLiveRecord, getLiveStatus, getFollowingLive } from "./live";

export { getTabFeed, getFollowFeed, getFriendFeed } from "./feed";

export { getMusicCollection, downloadMusic, getMusicCollectionFromDB, getMusicCollectionCountFromDB, saveMusicCollection, saveMusicCollectionBatch, updateMusicFilePath, deleteMusicCollection } from "./music";
export type { MusicCollectionItem, NewMusicCollectionItem } from "./music";

export { getComments, getCommentReplies } from "./comment";

export { search } from "./search";

export { getDownloads, getDownloadStats, getLiveRecords, getLiveRecordCount, getVideos, getVideoCount, getUsers, getUserCount, getUserBySecUid, getVideoStats, getUserStats, getDownloadTrend, getTopAuthors, getStorageAnalysis, dbHealthCheck, getDbPath, isVideoDownloaded } from "./db-query";

export { openFolder, exportData } from "./file";

export { deleteDownloadRecord, deleteLiveRecord, deleteVideoInfo, deleteUserInfo } from "./delete";
