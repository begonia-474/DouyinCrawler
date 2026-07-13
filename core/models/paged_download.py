"""Versioned Python -> Rust contract for one parsed post page."""

from typing import Literal, Optional

from pydantic import BaseModel, ConfigDict, Field, model_validator

from core.models.single_download import (
    MediaDownloadItemV1,
)


PAGED_DOWNLOAD_CONTRACT_VERSION = 1

class PagedUserProfileV1(BaseModel):
    """Strict profile shape consumed by Rust ``UserInfo`` persistence."""

    model_config = ConfigDict(extra="forbid")

    sec_user_id: str = Field(min_length=1)
    nickname: Optional[str] = None
    uid: Optional[str] = None
    avatar_url: Optional[str] = None
    unique_id: Optional[str] = None
    signature: Optional[str] = None
    aweme_count: int = 0
    follower_count: int = 0
    following_count: int = 0
    total_favorited: int = 0
    ip_location: Optional[str] = None
    live_status: int = 0
    room_id: Optional[str] = None
    city: Optional[str] = None
    country: Optional[str] = None
    favoriting_count: int = 0
    gender: int = 0
    is_ban: int = 0
    is_block: int = 0
    is_blocked: int = 0
    is_star: int = 0
    mix_count: int = 0
    mplatform_followers_count: int = 0
    nickname_raw: Optional[str] = None
    school_name: Optional[str] = None
    short_id: Optional[str] = None
    signature_raw: Optional[str] = None
    user_age: int = 0
    custom_verify: Optional[str] = None
    updated_at: int = 0


class PagedDownloadPlanV1(BaseModel):
    model_config = ConfigDict(extra="forbid")

    success: Literal[True] = True
    contract_version: Literal[PAGED_DOWNLOAD_CONTRACT_VERSION] = PAGED_DOWNLOAD_CONTRACT_VERSION
    mode: Literal["post"] = "post"
    save_dir: str = Field(min_length=1)
    items: list[MediaDownloadItemV1] = Field(default_factory=list)
    next_cursor: Optional[int] = None
    has_more: bool = False
    page_aweme_ids: list[str] = Field(default_factory=list)
    user_profile: Optional[PagedUserProfileV1] = None

    @model_validator(mode="after")
    def validate_page(self) -> "PagedDownloadPlanV1":
        if self.has_more and self.next_cursor is None:
            raise ValueError("has_more=True requires next_cursor")
        if not self.has_more and self.next_cursor is not None:
            raise ValueError("has_more=False requires next_cursor=None")
        if self.has_more and not self.items:
            raise ValueError("has_more=True requires non-empty items")
        if any(not aweme_id.strip() for aweme_id in self.page_aweme_ids):
            raise ValueError("page_aweme_ids must contain non-empty IDs")
        if len(set(self.page_aweme_ids)) != len(self.page_aweme_ids):
            raise ValueError("page_aweme_ids must be ordered and unique")
        page_ids = set(self.page_aweme_ids)
        if any(item.aweme_id not in page_ids for item in self.items):
            raise ValueError("every item must belong to page_aweme_ids")
        if self.items and not self.page_aweme_ids:
            raise ValueError("non-empty items require page_aweme_ids")
        item_ids = {item.aweme_id for item in self.items}
        if any(aweme_id not in item_ids for aweme_id in self.page_aweme_ids):
            raise ValueError("every page_aweme_id must have at least one media item")
        if len({item.media_key for item in self.items}) != len(self.items):
            raise ValueError("media_key must be unique within a page")
        return self
