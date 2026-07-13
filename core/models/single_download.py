"""Versioned Python -> Rust contract for a single Douyin post download."""

from enum import Enum
from typing import Literal, Optional

from pydantic import BaseModel, ConfigDict, Field, model_validator


SINGLE_DOWNLOAD_CONTRACT_VERSION = 1


class SingleMediaKind(str, Enum):
    VIDEO = "video"
    IMAGE = "image"
    LIVE_PHOTO = "live_photo"


class SingleAccessoryKind(str, Enum):
    MUSIC = "music"
    COVER = "cover"
    DESCRIPTION = "description"


class SingleOutputSpec(BaseModel):
    model_config = ConfigDict(extra="forbid")

    filename: str = Field(min_length=1)
    suffix: str = Field(min_length=1)
    folder_name: Optional[str]


class SingleAccessory(BaseModel):
    model_config = ConfigDict(extra="forbid")

    kind: SingleAccessoryKind
    output: SingleOutputSpec
    url: Optional[str] = None
    content: Optional[str] = None

    @model_validator(mode="after")
    def validate_payload(self) -> "SingleAccessory":
        if self.kind in (SingleAccessoryKind.MUSIC, SingleAccessoryKind.COVER):
            if not self.url or not self.url.strip():
                raise ValueError("music and cover accessories require url")
        elif self.kind is SingleAccessoryKind.DESCRIPTION and self.content is None:
            raise ValueError("description accessory requires content")
        return self


class SingleVideoMetadata(BaseModel):
    """Stable database metadata mirrored by Rust ``VideoInfo``."""

    model_config = ConfigDict(extra="ignore")

    aweme_id: str = Field(min_length=1)
    desc: Optional[str] = None
    aweme_type: int = 0
    author_nickname: Optional[str] = None
    author_sec_uid: Optional[str] = None
    author_uid: Optional[str] = None
    create_time: Optional[int] = None
    duration: int = 0
    video_url: Optional[str] = None
    cover_url: Optional[str] = None
    music_title: Optional[str] = None
    digg_count: int = 0
    comment_count: int = 0
    share_count: int = 0
    collect_count: int = 0
    mix_id: Optional[str] = None
    mix_name: Optional[str] = None
    author_nickname_raw: Optional[str] = None
    author_short_id: Optional[str] = None
    author_unique_id: Optional[str] = None
    desc_raw: Optional[str] = None
    is_ads: int = 0
    is_story: int = 0
    is_top: int = 0
    is_long_video: int = 0
    video_bit_rate: Optional[str] = None
    animated_cover: Optional[str] = None
    private_status: int = 0
    is_delete: int = 0
    music_author: Optional[str] = None
    music_author_raw: Optional[str] = None
    music_duration: int = 0
    music_id: Optional[str] = None
    music_mid: Optional[str] = None
    pgc_author: Optional[str] = None
    pgc_author_title: Optional[str] = None
    pgc_music_type: int = 0
    music_status: int = 0
    music_owner_handle: Optional[str] = None
    music_owner_id: Optional[str] = None
    music_owner_nickname: Optional[str] = None
    music_play_url: Optional[str] = None
    is_commerce_music: int = 0
    mix_desc: Optional[str] = None
    mix_create_time: int = 0
    mix_pic_type: int = 0
    mix_type: int = 0
    mix_share_url: Optional[str] = None
    can_comment: int = 0
    can_forward: int = 0
    can_share: int = 0
    download_setting: int = 0
    allow_douplus: int = 0
    allow_share: int = 0
    admire_count: int = 0
    hashtag_ids: Optional[str] = None
    hashtag_names: Optional[str] = None
    images: Optional[str] = None
    region: Optional[str] = None
    is_prohibited: int = 0
    updated_at: int = 0


class SingleDownloadItem(BaseModel):
    model_config = ConfigDict(extra="forbid")

    aweme_id: str = Field(min_length=1)
    urls: list[str] = Field(min_length=1)
    kind: SingleMediaKind
    output: SingleOutputSpec
    headers: dict[str, str] = Field(default_factory=dict)
    accessories: list[SingleAccessory] = Field(default_factory=list)
    metadata: SingleVideoMetadata

    @model_validator(mode="after")
    def validate_metadata_identity(self) -> "SingleDownloadItem":
        if self.metadata.aweme_id != self.aweme_id:
            raise ValueError("item metadata.aweme_id must match item aweme_id")
        return self


class SingleDownloadPlanV1(BaseModel):
    model_config = ConfigDict(extra="forbid")

    success: Literal[True] = True
    contract_version: Literal[SINGLE_DOWNLOAD_CONTRACT_VERSION] = (
        SINGLE_DOWNLOAD_CONTRACT_VERSION
    )
    mode: Literal["one"] = "one"
    save_dir: str = Field(min_length=1)
    items: list[SingleDownloadItem] = Field(min_length=1)
    total: int = Field(ge=0)

    @model_validator(mode="after")
    def validate_total(self) -> "SingleDownloadPlanV1":
        if self.total != len(self.items):
            raise ValueError("total must equal the number of items")
        return self
