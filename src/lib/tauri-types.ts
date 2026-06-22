/** 下载记录 */
export interface DownloadRecord {
  id: number;
  aweme_id: string | null;
  download_type: string;
  title: string | null;
  author_nickname: string | null;
  author_sec_uid: string | null;
  file_path: string | null;
  file_size: number;
  cover_url: string | null;
  status: string;
  error_msg: string | null;
  created_at: number;
}

/** 下载统计 */
export interface DownloadStats {
  total_count: number;
  total_size: number;
  by_type: TypeStat[];
  by_day: DayStat[];
}

export interface TypeStat {
  download_type: string;
  cnt: number;
  size: number;
}

export interface DayStat {
  day: string;
  cnt: number;
}

/** 直播录制记录 */
export interface LiveRecord {
  id: number;
  room_id: string | null;
  web_rid: string | null;
  title: string | null;
  nickname: string | null;
  sec_user_id: string | null;
  file_path: string | null;
  file_size: number;
  duration_sec: number;
  status: string;
  started_at: number | null;
  ended_at: number | null;
}
