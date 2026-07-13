//! Python 桥接模块
//!
//! 使用 PyO3 实现 Rust 与 Python 的直接调用

pub mod bridge;
pub mod config;
pub mod db_bridge;
pub mod emit;
pub mod handler;
pub mod responses;
pub mod runtime;

// 重新导出主要接口
pub use bridge::PythonBridge;
#[allow(unused_imports)]
pub use responses::*;
pub use config::init_config;
pub use db_bridge::register_db_bridge;
#[allow(unused_imports)]
pub use handler::{
    parse_video, get_live_info, resolve_live,
    resolve_single, resolve_music_urls,
    get_user_profile, get_user_posts, search_videos, get_mix_info,
    get_collects_list, get_collects_video_list, get_following_list,
    get_follower_list, get_music_collection, download_music, get_following_live,
    get_comments, get_comment_replies, get_tab_feed, get_follow_feed,
    get_friend_feed, get_user_likes, get_post_stats,
};
