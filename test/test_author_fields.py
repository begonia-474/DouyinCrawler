import asyncio, json, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from core.crawler import DouyinCrawler
from core.filter import PostDetailFilter

async def test():
    cookie = json.load(open("backend/config.json"))["cookie"]
    async with DouyinCrawler(cookie=cookie, encryption="ab") as c:
        data = await c.fetch_post_detail("7650450403901017571")
    d = PostDetailFilter(data)
    db = d.to_db_dict()
    print("author_avatar_url:", db.get("author_avatar_url", "MISSING")[:80])
    print("author_signature:", db.get("author_signature", "MISSING")[:40])
    print("author_follower_count:", db.get("author_follower_count", "MISSING"))
    print("author_aweme_count:", db.get("author_aweme_count", "MISSING"))
    print("author_following_count:", db.get("author_following_count", "MISSING"))
    print("author_total_favorited:", db.get("author_total_favorited", "MISSING"))
    print("author_ip_location:", db.get("author_ip_location", "MISSING"))
    print("total fields:", len(db))

asyncio.run(test())
