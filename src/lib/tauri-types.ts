// ============================================================
// 此文件由 scripts/gen_tauri_types.py 自动生成
// 源头：src-tauri/src/db.rs + src-tauri/src/database/models.rs
//
// 类型来源：
//   - 12 个类型 re-export 自 bindings.ts（specta 编译期生成，权威来源）
//   - VideoInfo 本地定义（62 字段完整 DB 模型，bindings.ts 仅 18 字段）
//
// 修改后请运行: python scripts/gen_tauri_types.py
// ============================================================

// Re-export 与 bindings.ts 一致的类型（单一来源）
export type {
  DayStat,
  UserStats,
  VideoTypeStat,
  UserInfo,
  DownloadRecord,
  VideoStats,
  MusicCollection,
  TypeStat,
  LiveRecord,
  DownloadStats,
  TrendPoint,
  AuthorStat,
  StorageStat,
  DbHealth,
} from "./bindings";

// VideoInfo：完整 DB 模型（61 字段）
// bindings.ts 仅有 18 个核心字段（用于 task service），此处保留完整版本供 library 页面和 DB 查询使用
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
