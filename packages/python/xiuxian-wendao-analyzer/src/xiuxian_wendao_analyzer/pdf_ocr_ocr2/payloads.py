"""OpenAI-compatible Hosted VLM/OCR request payload builders."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from ..pdf_ocr_contracts import HOSTED_VLM_OCR_DEFAULT_IMAGE_OPTIMIZATION
from .markers import ocr2_page_marker, ocr2_region_marker
from .payload_helpers import (
    image_bytes_data_url,
    image_data_url,
    region_atlas_prompt,
    region_composite_prompt,
    region_scaffold_prompt,
    window_prompt,
)

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence
    from pathlib import Path


def request_payload(
    *,
    model: str,
    prompt: str,
    input_row: Mapping[str, Any],
    image_path: Path,
    max_tokens: int,
    image_data_url_value: str | None = None,
    image_optimization_mode: str = HOSTED_VLM_OCR_DEFAULT_IMAGE_OPTIMIZATION,
) -> dict[str, Any]:
    return {
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": prompt},
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": image_data_url_value
                            or image_data_url(
                                input_row,
                                image_path,
                                image_optimization_mode=image_optimization_mode,
                            )
                        },
                    },
                ],
            }
        ],
        "max_tokens": max_tokens,
        "temperature": 0,
    }


def window_request_payload(
    *,
    model: str,
    prompt: str,
    input_rows: Sequence[Mapping[str, Any]],
    image_paths: Sequence[Path],
    max_tokens: int,
) -> dict[str, Any]:
    content: list[dict[str, Any]] = [
        {"type": "text", "text": window_prompt(prompt, input_rows)}
    ]
    for ordinal, (input_row, image_path) in enumerate(
        zip(input_rows, image_paths, strict=True),
        start=1,
    ):
        marker = ocr2_page_marker(input_row)
        content.append(
            {
                "type": "text",
                "text": f"Image {ordinal} must produce section marker {marker}.",
            }
        )
        content.append(
            {
                "type": "image_url",
                "image_url": {"url": image_data_url(input_row, image_path)},
            }
        )
    return {
        "model": model,
        "messages": [{"role": "user", "content": content}],
        "max_tokens": max_tokens,
        "temperature": 0,
    }


def region_composite_request_payload(
    *,
    model: str,
    prompt: str,
    input_rows: Sequence[Mapping[str, Any]],
    image_paths: Sequence[Path],
    max_tokens: int,
) -> dict[str, Any]:
    content: list[dict[str, Any]] = [
        {"type": "text", "text": region_composite_prompt(prompt, input_rows)}
    ]
    for ordinal, (input_row, image_path) in enumerate(
        zip(input_rows, image_paths, strict=True),
        start=1,
    ):
        marker = ocr2_region_marker(input_row)
        content.append(
            {
                "type": "text",
                "text": f"Region image {ordinal} must produce section marker {marker}.",
            }
        )
        content.append(
            {
                "type": "image_url",
                "image_url": {"url": image_data_url(input_row, image_path)},
            }
        )
    return {
        "model": model,
        "messages": [{"role": "user", "content": content}],
        "max_tokens": max_tokens,
        "temperature": 0,
    }


def region_atlas_request_payload(
    *,
    model: str,
    prompt: str,
    input_rows: Sequence[Mapping[str, Any]],
    atlas_image_bytes: bytes,
    max_tokens: int,
) -> dict[str, Any]:
    return {
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": region_atlas_prompt(prompt, input_rows),
                    },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": image_bytes_data_url(atlas_image_bytes, "image/png")
                        },
                    },
                ],
            }
        ],
        "max_tokens": max_tokens,
        "temperature": 0,
    }


def region_scaffold_request_payload(
    *,
    model: str,
    prompt: str,
    input_rows: Sequence[Mapping[str, Any]],
    image_paths: Sequence[Path],
    scaffolds: Sequence[Mapping[str, Any]],
    max_tokens: int,
    composite: bool,
) -> dict[str, Any]:
    content: list[dict[str, Any]] = [
        {
            "type": "text",
            "text": region_scaffold_prompt(
                prompt, input_rows, scaffolds, composite=composite
            ),
        }
    ]
    for ordinal, (input_row, image_path) in enumerate(
        zip(input_rows, image_paths, strict=True),
        start=1,
    ):
        marker = ocr2_region_marker(input_row)
        content.append(
            {
                "type": "text",
                "text": f"Region image {ordinal} must produce JSON entry marker {marker}.",
            }
        )
        content.append(
            {
                "type": "image_url",
                "image_url": {"url": image_data_url(input_row, image_path)},
            }
        )
    return {
        "model": model,
        "messages": [{"role": "user", "content": content}],
        "max_tokens": max_tokens,
        "temperature": 0,
    }
