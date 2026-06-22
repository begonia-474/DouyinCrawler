"""请求/响应 Pydantic 模型"""

from pydantic import BaseModel
from typing import Any, Optional


class ApiResponse(BaseModel):
    success: bool
    data: Optional[Any] = None
    error: Optional[str] = None


class UrlRequest(BaseModel):
    url: str


class KeywordRequest(BaseModel):
    keyword: str
    offset: int = 0
    count: int = 10


class CommentRequest(BaseModel):
    url: str
    cursor: int = 0
    count: int = 20


class CommentReplyRequest(BaseModel):
    url: str
    comment_id: str
    cursor: int = 0
    count: int = 3


class FeedRequest(BaseModel):
    cursor: int = 0
    count: int = 10


class MusicRequest(BaseModel):
    cursor: int = 0
    count: int = 18


class UserListRequest(BaseModel):
    url: str
    offset: int = 0
    count: int = 20


class CollectsVideoRequest(BaseModel):
    collects_id: str
    cursor: int = 0
    count: int = 20


class ConfigRequest(BaseModel):
    cookie: Optional[str] = None
    download_path: Optional[str] = None
    naming: Optional[str] = None
    encryption: Optional[str] = None
    proxy: Optional[str] = None


class LiveRecordRequest(BaseModel):
    url: str


class MusicDownloadRequest(BaseModel):
    play_url: str
    title: str
    author: str = ""
