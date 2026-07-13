"""Shared Python planner for typed single and paged media items."""

from collections.abc import Iterable
from typing import Any

from core.download.downloader import format_filename
from core.models.single_download import (
    MediaAccessoryKindV1,
    MediaAccessoryV1,
    MediaDownloadItemV1,
    MediaKindV1,
    MediaMetadataV1,
    MediaOutputSpecV1,
)


def _cover_suffix(cover_url: str) -> str:
    return ".webp" if cover_url and "animated_cover" in cover_url else ".jpeg"


def _accessories(
    detail: Any,
    base_name: str,
    *,
    include_description: bool,
) -> list[MediaAccessoryV1]:
    accessories: list[MediaAccessoryV1] = []
    if detail.music_url:
        accessories.append(
            MediaAccessoryV1(
                kind=MediaAccessoryKindV1.MUSIC,
                url=detail.music_url,
                output=MediaOutputSpecV1(
                    filename=f"{base_name}_music", suffix=".mp3", folder_name=None
                ),
            )
        )
    if detail.cover_url:
        accessories.append(
            MediaAccessoryV1(
                kind=MediaAccessoryKindV1.COVER,
                url=detail.cover_url,
                output=MediaOutputSpecV1(
                    filename=f"{base_name}_cover",
                    suffix=_cover_suffix(detail.cover_url),
                    folder_name=None,
                ),
            )
        )
    if include_description and detail.desc:
        accessories.append(
            MediaAccessoryV1(
                kind=MediaAccessoryKindV1.DESCRIPTION,
                content=detail.desc,
                output=MediaOutputSpecV1(
                    filename=f"{base_name}_desc", suffix=".txt", folder_name=None
                ),
            )
        )
    return accessories


def build_media_items_v1(
    details: Iterable[Any],
    *,
    naming: str,
    folderize: bool,
    headers: dict[str, str],
) -> list[MediaDownloadItemV1]:
    """Build stable ordered media plans without performing any side effects."""
    items: list[MediaDownloadItemV1] = []
    for detail in details:
        aweme_id = str(getattr(detail, "aweme_id", "") or "").strip()
        if not aweme_id or getattr(detail, "is_prohibited", False):
            continue

        base_name = format_filename(naming, detail.to_dict())
        folder_name = base_name if folderize else None
        metadata = MediaMetadataV1.model_validate(detail.to_db_dict())
        work_items: list[MediaDownloadItemV1] = []

        if detail.is_image_post and (detail.images or detail.images_video):
            live_count = 0
            for live_url in (detail.images_video or []):
                if live_url:
                    live_count += 1
                    work_items.append(
                        MediaDownloadItemV1(
                            media_key=f"{aweme_id}:live_photo:{live_count}",
                            aweme_id=aweme_id,
                            urls=[live_url],
                            kind=MediaKindV1.LIVE_PHOTO,
                            output=MediaOutputSpecV1(
                                filename=f"{base_name}_live_{live_count}",
                                suffix=".mp4",
                                folder_name=folder_name,
                            ),
                            headers=headers.copy(),
                            metadata=metadata,
                        )
                    )
            image_count = 0
            for image_url in (detail.images or []):
                if image_url:
                    image_count += 1
                    work_items.append(
                        MediaDownloadItemV1(
                            media_key=f"{aweme_id}:image:{image_count}",
                            aweme_id=aweme_id,
                            urls=[image_url],
                            kind=MediaKindV1.IMAGE,
                            output=MediaOutputSpecV1(
                                filename=f"{base_name}_image_{image_count}",
                                suffix=".webp",
                                folder_name=folder_name,
                            ),
                            headers=headers.copy(),
                            metadata=metadata,
                        )
                    )
            if work_items:
                work_items[0].accessories.extend(
                    _accessories(detail, base_name, include_description=False)
                )
        else:
            video_urls = [url for url in (detail.video_urls or [detail.video_url]) if url]
            if video_urls:
                work_items.append(
                    MediaDownloadItemV1(
                        media_key=f"{aweme_id}:video:0",
                        aweme_id=aweme_id,
                        urls=video_urls,
                        kind=MediaKindV1.VIDEO,
                        output=MediaOutputSpecV1(
                            filename=f"{base_name}_video",
                            suffix=".mp4",
                            folder_name=folder_name,
                        ),
                        headers=headers.copy(),
                        metadata=metadata,
                        accessories=_accessories(
                            detail, base_name, include_description=True
                        ),
                    )
                )
        items.extend(work_items)
    return items


def ordered_aweme_ids(items: Iterable[MediaDownloadItemV1]) -> list[str]:
    """Return ordered work identities that actually have downloadable media."""
    seen: set[str] = set()
    ordered: list[str] = []
    for item in items:
        aweme_id = item.aweme_id
        if aweme_id in seen:
            continue
        seen.add(aweme_id)
        ordered.append(aweme_id)
    return ordered
