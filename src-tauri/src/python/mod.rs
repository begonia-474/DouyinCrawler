//! Python 桥接模块
//!
//! 使用 PyO3 实现 Rust 与 Python 的直接调用

pub mod bridge;
pub mod config;
pub mod db_bridge;
pub mod emit;
pub mod handler;

// 重新导出主要接口
pub use bridge::PythonBridge;
pub use config::init_config;
pub use db_bridge::register_db_bridge;
pub use emit::register_app_handle;
pub use handler::{
    parse_video, download_video, get_live_info, start_batch_download,
    get_user_profile, get_user_posts, search_videos, get_mix_info,
    get_collects_list, get_collects_video_list, get_following_list,
    get_follower_list, get_music_collection, download_music, get_following_live,
    get_comments, get_comment_replies, get_tab_feed, get_follow_feed,
    get_friend_feed, get_user_likes, get_post_stats, start_live_record,
    stop_live_record, get_live_status, test_emit,
};
