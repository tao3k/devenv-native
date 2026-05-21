"""Hosted VLM/OCR image payload preparation."""

from __future__ import annotations

from dataclasses import dataclass
from io import BytesIO
from typing import TYPE_CHECKING, Any

from ..pdf_ocr_contracts import (
    HOSTED_VLM_OCR_DEFAULT_IMAGE_OPTIMIZATION,
    HOSTED_VLM_OCR_REGION_WHITESPACE_TRIM_OPTIMIZATION,
)

if TYPE_CHECKING:
    from collections.abc import Mapping
    from pathlib import Path

_REGION_TRIM_BACKGROUND_THRESHOLD = 12
_REGION_TRIM_PADDING_PX = 24
_REGION_TRIM_MIN_AREA_REDUCTION = 0.10
_REGION_TRIM_MIN_DIMENSION_PX = 32


@dataclass(frozen=True)
class HostedVlmImagePayload:
    image_bytes: bytes
    image_mime_type: str


def hosted_vlm_image_payload(
    input_row: Mapping[str, Any],
    image_path: Path,
    *,
    image_optimization_mode: str = HOSTED_VLM_OCR_DEFAULT_IMAGE_OPTIMIZATION,
) -> HostedVlmImagePayload:
    image_bytes = image_path.read_bytes()
    image_mime_type = str(input_row.get("imageMimeType") or "image/png")
    if (
        image_optimization_mode != HOSTED_VLM_OCR_REGION_WHITESPACE_TRIM_OPTIMIZATION
        or str(input_row.get("shardType") or "") != "region"
        or image_mime_type != "image/png"
    ):
        return HostedVlmImagePayload(image_bytes, image_mime_type)
    trimmed = _trim_png_region_whitespace(image_bytes)
    if trimmed is None:
        return HostedVlmImagePayload(image_bytes, image_mime_type)
    return HostedVlmImagePayload(trimmed, "image/png")


def _trim_png_region_whitespace(image_bytes: bytes) -> bytes | None:
    try:
        from PIL import Image, ImageChops
    except ImportError:
        return None
    try:
        with Image.open(BytesIO(image_bytes)) as image:
            rgb_image = image.convert("RGB")
            background = Image.new("RGB", rgb_image.size, (255, 255, 255))
            diff = ImageChops.difference(rgb_image, background).convert("L")
            mask = diff.point(
                lambda value: 255 if value > _REGION_TRIM_BACKGROUND_THRESHOLD else 0
            )
            bbox = mask.getbbox()
            if bbox is None:
                return None
            left, top, right, bottom = _padded_bbox(bbox, rgb_image.size)
            cropped_width = right - left
            cropped_height = bottom - top
            if (
                cropped_width < _REGION_TRIM_MIN_DIMENSION_PX
                or cropped_height < _REGION_TRIM_MIN_DIMENSION_PX
            ):
                return None
            original_area = rgb_image.width * rgb_image.height
            cropped_area = cropped_width * cropped_height
            if cropped_area >= original_area * (1.0 - _REGION_TRIM_MIN_AREA_REDUCTION):
                return None
            output = BytesIO()
            rgb_image.crop((left, top, right, bottom)).save(
                output,
                format="PNG",
                optimize=True,
            )
            return output.getvalue()
    except OSError:
        return None


def _padded_bbox(
    bbox: tuple[int, int, int, int],
    size: tuple[int, int],
) -> tuple[int, int, int, int]:
    width, height = size
    left, top, right, bottom = bbox
    return (
        max(0, left - _REGION_TRIM_PADDING_PX),
        max(0, top - _REGION_TRIM_PADDING_PX),
        min(width, right + _REGION_TRIM_PADDING_PX),
        min(height, bottom + _REGION_TRIM_PADDING_PX),
    )
