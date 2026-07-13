"""Strict versioned Python -> Rust contract for live recording."""

from pathlib import PurePath
from typing import Literal, Optional
from urllib.parse import urlparse

from pydantic import BaseModel, ConfigDict, Field, field_validator


LIVE_PLAN_CONTRACT_VERSION = 1


class LiveOutputV1(BaseModel):
    model_config = ConfigDict(extra="forbid", strict=True)

    save_dir: str = Field(min_length=1)
    filename: str = Field(min_length=1)
    suffix: Literal[".flv"] = ".flv"

    @field_validator("save_dir")
    @classmethod
    def validate_save_dir(cls, value: str) -> str:
        if not value.strip():
            raise ValueError("save_dir must not be blank")
        return value

    @field_validator("filename")
    @classmethod
    def validate_filename(cls, value: str) -> str:
        if value in {".", ".."} or PurePath(value).name != value:
            raise ValueError("filename must be one path component")
        if "/" in value or "\\" in value or "\x00" in value:
            raise ValueError("filename contains an invalid path character")
        return value


class LivePlanV1(BaseModel):
    model_config = ConfigDict(extra="forbid", strict=True)

    success: Literal[True] = True
    contract_version: Literal[LIVE_PLAN_CONTRACT_VERSION] = LIVE_PLAN_CONTRACT_VERSION
    mode: Literal["live"] = "live"
    web_rid: str = Field(min_length=1)
    room_id: str = Field(min_length=1)
    title: str
    nickname: str = Field(min_length=1)
    sec_user_id: str = Field(min_length=1)
    user_id: Optional[str]
    cover_url: str
    user_count: int = Field(ge=0)
    m3u8_url: str = Field(min_length=1)
    output: LiveOutputV1
    headers: dict[str, str]

    @field_validator("web_rid", "room_id", "nickname", "sec_user_id")
    @classmethod
    def validate_required_identity(cls, value: str) -> str:
        if not value.strip():
            raise ValueError("live identity fields must not be blank")
        return value

    @field_validator("m3u8_url")
    @classmethod
    def validate_m3u8_url(cls, value: str) -> str:
        parsed = urlparse(value)
        if parsed.scheme not in {"http", "https"} or not parsed.netloc:
            raise ValueError("m3u8_url must be an absolute HTTP URL")
        return value

    @field_validator("headers")
    @classmethod
    def validate_headers(cls, value: dict[str, str]) -> dict[str, str]:
        for name, header_value in value.items():
            if not name.strip() or "\r" in name or "\n" in name:
                raise ValueError("header names must be non-empty and single-line")
            if "\r" in header_value or "\n" in header_value:
                raise ValueError("header values must be single-line")
        return value
