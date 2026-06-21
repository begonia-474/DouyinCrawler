"""抖音 API 端点定义"""

DOUYIN_DOMAIN = "https://www.douyin.com"
IESDOUYIN_DOMAIN = "https://www.iesdouyin.com"
LIVE_DOMAIN = "https://live.douyin.com"
LIVE_DOMAIN2 = "https://webcast.amemv.com"


class DouyinAPIEndpoints:
    """抖音 API 端点常量"""

    # === 用户相关 ===
    USER_SHORT_INFO = f"{DOUYIN_DOMAIN}/aweme/v1/web/im/user/info/"
    USER_DETAIL = f"{DOUYIN_DOMAIN}/aweme/v1/web/user/profile/other/"
    USER_POST = f"{DOUYIN_DOMAIN}/aweme/v1/web/aweme/post/"
    USER_FAVORITE = f"{DOUYIN_DOMAIN}/aweme/v1/web/aweme/favorite/"
    USER_FOLLOWING = f"{DOUYIN_DOMAIN}/aweme/v1/web/user/following/list/"
    USER_FOLLOWER = f"{DOUYIN_DOMAIN}/aweme/v1/web/user/follower/list/"
    USER_HISTORY = f"{DOUYIN_DOMAIN}/aweme/v1/web/history/read/"
    QUERY_USER = f"{DOUYIN_DOMAIN}/aweme/v1/web/query/user/"

    # === 视频相关 ===
    POST_DETAIL = f"{DOUYIN_DOMAIN}/aweme/v1/web/aweme/detail/"
    POST_RELATED = f"{DOUYIN_DOMAIN}/aweme/v1/web/aweme/related/"
    POST_STATS = f"{DOUYIN_DOMAIN}/aweme/v2/web/aweme/stats/"
    LOCATE_POST = f"{DOUYIN_DOMAIN}/aweme/v1/web/locate/post/"
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
    HOME_POST_SEARCH = f"{DOUYIN_DOMAIN}/aweme/v1/web/home/search/item/"
    SUGGEST_WORDS = f"{DOUYIN_DOMAIN}/aweme/v1/web/api/suggest_words/"

    # === 信息流 ===
    TAB_FEED = f"{DOUYIN_DOMAIN}/aweme/v1/web/tab/feed/"
    FRIEND_FEED = f"{DOUYIN_DOMAIN}/aweme/v1/web/familiar/feed/"
    FOLLOW_FEED = f"{DOUYIN_DOMAIN}/aweme/v1/web/follow/feed/"

    # === 直播 ===
    LIVE_INFO = f"{LIVE_DOMAIN}/webcast/room/web/enter/"
    LIVE_INFO_ROOM_ID = f"{LIVE_DOMAIN2}/webcast/room/reflow/info/"
    LIVE_USER_INFO = f"{LIVE_DOMAIN}/webcast/user/me/"
    LIVE_IM_FETCH = f"{LIVE_DOMAIN}/webcast/im/fetch/"
    USER_LIVE_STATUS = f"{LIVE_DOMAIN}/webcast/distribution/check_user_live_status/"
    FOLLOW_USER_LIVE = f"{DOUYIN_DOMAIN}/webcast/web/feed/follow/"
    LIVE_IM_WSS = "wss://webcast5-ws-web-hl.douyin.com/webcast/im/push/v2/"

    # === 备用域名 ===
    SLIDES_AWEME = f"{IESDOUYIN_DOMAIN}/web/api/v2/aweme/slidesinfo/"
    USER_FAVORITE_B = f"{IESDOUYIN_DOMAIN}/web/api/v2/aweme/like/"
