// ============================================================
// 此文件由 scripts/gen_tauri_types.py 自动生成
// 源头：src-tauri/src/db.rs
// 修改后请运行: python scripts/gen_tauri_types.py
// ============================================================

/** DayStat（对齐 Rust DayStat 结构体） */
export interface DayStat {
  day: string;
  cnt: number;
}

/** UserStats（对齐 Rust UserStats 结构体） */
export interface UserStats {
  total_count: number;
  total_follower: number;
  total_aweme: number;
}

/** VideoTypeStat（对齐 Rust VideoTypeStat 结构体） */
export interface VideoTypeStat {
  aweme_type: number;
  cnt: number;
}

/** UserInfo（对齐 Rust UserInfo 结构体） */
export interface UserInfo {
  sec_user_id: string;
  nickname: string | null;
  uid: string | null;
  avatar_url: string | null;
  unique_id: string | null;
  signature: string | null;
  aweme_count: number;
  follower_count: number;
  following_count: number;
  total_favorited: number;
  ip_location: string | null;
  live_status: number;
  room_id: string | null;
  city: string | null;
  country: string | null;
  favoriting_count: number;
  gender: number;
  is_ban: number;
  is_block: number;
  is_blocked: number;
  is_star: number;
  mix_count: number;
  mplatform_followers_count: number;
  nickname_raw: string | null;
  school_name: string | null;
  short_id: string | null;
  signature_raw: string | null;
  user_age: number;
  custom_verify: string | null;
  updated_at: number;
}

/** DownloadRecord（对齐 Rust DownloadRecord 结构体） */
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

/** VideoStats（对齐 Rust VideoStats 结构体） */
export interface VideoStats {
  total_count: number;
  total_digg: number;
  total_comment: number;
  total_share: number;
  total_collect: number;
  by_type: VideoTypeStat[];
}

/** MusicCollection（对齐 Rust MusicCollection 结构体） */
export interface MusicCollection {
  music_id: string;
  mid: string | null;
  title: string | null;
  author: string | null;
  owner_nickname: string | null;
  duration: number;
  cover: string | null;
  play_url: string | null;
  file_path: string | null;
  status: string;
  created_at: number;
}

/** TypeStat（对齐 Rust TypeStat 结构体） */
export interface TypeStat {
  download_type: string;
  cnt: number;
  size: number;
}

/** NewMusicCollection（对齐 Rust NewMusicCollection 结构体） */
export interface NewMusicCollection {
  music_id: string;
  mid: string | null;
  title: string | null;
  author: string | null;
  owner_nickname: string | null;
  duration: number;
  cover: string | null;
  play_url: string | null;
}

/** LiveRecord（对齐 Rust LiveRecord 结构体） */
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
  cover_url: string | null;
}

/** DownloadStats（对齐 Rust DownloadStats 结构体） */
export interface DownloadStats {
  total_count: number;
  total_size: number;
  by_type: TypeStat[];
  by_day: DayStat[];
}

/** TrendPoint（对齐 Rust TrendPoint 结构体） */
export interface TrendPoint {
  day: string;
  cnt: number;
  size: number;
}

/** AuthorStat（对齐 Rust AuthorStat 结构体） */
export interface AuthorStat {
  author_nickname: string;
  cnt: number;
  total_size: number;
}

/** StorageStat（对齐 Rust StorageStat 结构体） */
export interface StorageStat {
  download_type: string;
  cnt: number;
  total_size: number;
}

/** DbHealth（对齐 Rust DbHealth 结构体） */
export interface DbHealth {
  download_count: number;
  video_count: number;
  user_count: number;
  live_count: number;
  music_count: number;
  task_count: number;
  db_size_bytes: number;
}

/** VideoInfo（对齐 Rust VideoInfo 结构体） */
export interface VideoInfo {
  aweme_id: string;
  desc: string | null;
  aweme_type: number;
  author_nickname: string | null;
  author_sec_uid: string | null;
  author_uid: string | null;
  create_time: number | null;
  duration: number;
  video_url: string | null;
  cover_url: string | null;
  music_title: string | null;
  digg_count: number;
  comment_count: number;
  share_count: number;
  collect_count: number;
  mix_id: string | null;
  mix_name: string | null;
  author_nickname_raw: string | null;
  author_short_id: string | null;
  author_unique_id: string | null;
  desc_raw: string | null;
  is_ads: number;
  is_story: number;
  is_top: number;
  is_long_video: number;
  video_bit_rate: string | null;
  animated_cover: string | null;
  private_status: number;
  is_delete: number;
  music_author: string | null;
  music_author_raw: string | null;
  music_duration: number;
  music_id: string | null;
  music_mid: string | null;
  pgc_author: string | null;
  pgc_author_title: string | null;
  pgc_music_type: number;
  music_status: number;
  music_owner_handle: string | null;
  music_owner_id: string | null;
  music_owner_nickname: string | null;
  music_play_url: string | null;
  is_commerce_music: number;
  mix_desc: string | null;
  mix_create_time: number;
  mix_pic_type: number;
  mix_type: number;
  mix_share_url: string | null;
  can_comment: number;
  can_forward: number;
  can_share: number;
  download_setting: number;
  allow_douplus: number;
  allow_share: number;
  admire_count: number;
  hashtag_ids: string | null;
  hashtag_names: string | null;
  images: string | null;
  region: string | null;
  is_prohibited: number;
  updated_at: number;
}
