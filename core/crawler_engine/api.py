"""抖音 API 端点定义

能力分级说明：
- [active]  — 当前 UI/服务层在用，crawler 方法被 service 调用
- [reserved] — 已实现但暂未接入 UI，crawler 方法存在或端点已定义但无 service 调用方
- 未标注的端点均为 active
"""

DOUYIN_DOMAIN = "https://www.douyin.com"
IESDOUYIN_DOMAIN = "https://www.iesdouyin.com"
LIVE_DOMAIN = "https://live.douyin.com"
LIVE_DOMAIN2 = "https://webcast.amemv.com"


class DouyinAPIEndpoints:
    """抖音 API 端点常量"""

    # === 用户相关 ===
    USER_SHORT_INFO = f"{DOUYIN_DOMAIN}/aweme/v1/web/im/user/info/"  # [reserved] 无 crawler 方法引用
    USER_DETAIL = f"{DOUYIN_DOMAIN}/aweme/v1/web/user/profile/other/"
    USER_POST = f"{DOUYIN_DOMAIN}/aweme/v1/web/aweme/post/"
    USER_FAVORITE = f"{DOUYIN_DOMAIN}/aweme/v1/web/aweme/favorite/"
    USER_FOLLOWING = f"{DOUYIN_DOMAIN}/aweme/v1/web/user/following/list/"
    USER_FOLLOWER = f"{DOUYIN_DOMAIN}/aweme/v1/web/user/follower/list/"
    USER_HISTORY = f"{DOUYIN_DOMAIN}/aweme/v1/web/history/read/"  # [reserved] 无 crawler 方法引用
    QUERY_USER = f"{DOUYIN_DOMAIN}/aweme/v1/web/query/user/"  # [reserved] fetch_query_user 无 service 调用

    # === 视频相关 ===
    POST_DETAIL = f"{DOUYIN_DOMAIN}/aweme/v1/web/aweme/detail/"
    POST_RELATED = f"{DOUYIN_DOMAIN}/aweme/v1/web/aweme/related/"
    POST_STATS = f"{DOUYIN_DOMAIN}/aweme/v2/web/aweme/stats/"
    LOCATE_POST = f"{DOUYIN_DOMAIN}/aweme/v1/web/locate/post/"

    # === 评论相关 ===
    POST_COMMENT = f"{DOUYIN_DOMAIN}/aweme/v1/web/comment/list/"
    POST_COMMENT_REPLY = f"{DOUYIN_DOMAIN}/aweme/v1/web/comment/list/reply/"

    # === 合集 ===
    MIX_AWEME = f"{DOUYIN_DOMAIN}/aweme/v1/web/mix/aweme/"

    # === 收藏 ===
    USER_COLLECTION = f"{DOUYIN_DOMAIN}/aweme/v1/web/aweme/listcollection/"
    USER_COLLECTS = f"{DOUYIN_DOMAIN}/aweme/v1/web/collects/list/"
    USER_COLLECTS_VIDEO = f"{DOUYIN_DOMAIN}/aweme/v1/web/collects/video/list/"

    # === 音乐 ===
    USER_MUSIC_COLLECTION = f"{DOUYIN_DOMAIN}/aweme/v1/web/music/listcollection/"

    # === 搜索 ===
    POST_SEARCH = f"{DOUYIN_DOMAIN}/aweme/v1/web/general/search/single/"
    HOME_POST_SEARCH = f"{DOUYIN_DOMAIN}/aweme/v1/web/home/search/item/"  # [reserved] fetch_home_post_search 无 service 调用
    SUGGEST_WORDS = f"{DOUYIN_DOMAIN}/aweme/v1/web/api/suggest_words/"  # [reserved] fetch_suggest_word 无 service 调用

    # === 信息流 ===
    TAB_FEED = f"{DOUYIN_DOMAIN}/aweme/v1/web/tab/feed/"
    FRIEND_FEED = f"{DOUYIN_DOMAIN}/aweme/v1/web/familiar/feed/"
    FOLLOW_FEED = f"{DOUYIN_DOMAIN}/aweme/v1/web/follow/feed/"

    # === 直播 ===
    LIVE_INFO = f"{LIVE_DOMAIN}/webcast/room/web/enter/"
    LIVE_INFO_ROOM_ID = f"{LIVE_DOMAIN2}/webcast/room/reflow/info/"  # [reserved] fetch_live_info_by_room_id 无 service 调用
    LIVE_USER_INFO = f"{LIVE_DOMAIN}/webcast/user/me/"  # [reserved] 无 crawler 方法引用
    LIVE_IM_FETCH = f"{LIVE_DOMAIN}/webcast/im/fetch/"  # [reserved] fetch_live_im 无 service 调用
    USER_LIVE_STATUS = f"{LIVE_DOMAIN}/webcast/distribution/check_user_live_status/"  # [reserved] fetch_user_live_status 无 service 调用
    FOLLOW_USER_LIVE = f"{DOUYIN_DOMAIN}/webcast/web/feed/follow/"
    LIVE_IM_WSS = "wss://webcast5-ws-web-hl.douyin.com/webcast/im/push/v2/"  # [reserved] 未被引用

    # === 备用域名（备用端点，差异化风控） ===
    SLIDES_AWEME = f"{IESDOUYIN_DOMAIN}/web/api/v2/aweme/slidesinfo/"  # [reserved] 未被引用
    USER_FAVORITE_B = f"{IESDOUYIN_DOMAIN}/web/api/v2/aweme/like/"  # [reserved] 未被引用
