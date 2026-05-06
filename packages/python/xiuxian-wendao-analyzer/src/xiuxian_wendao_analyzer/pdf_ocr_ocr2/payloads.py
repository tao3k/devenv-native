"""OpenAI-compatible OCR2 request payload builders."""

from __future__ import annotations

import base64
import json
from typing import TYPE_CHECKING, Any

from .markers import ocr2_page_marker, ocr2_region_marker

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
                        "image_url": {"url": image_data_url(input_row, image_path)},
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


def image_data_url(input_row: Mapping[str, Any], image_path: Path) -> str:
    image_bytes = image_path.read_bytes()
    image_mime_type = str(input_row.get("imageMimeType") or "image/png")
    encoded = base64.b64encode(image_bytes).decode("ascii")
    return f"data:{image_mime_type};base64,{encoded}"


def window_prompt(prompt: str, input_rows: Sequence[Mapping[str, Any]]) -> str:
    markers = "\n".join(ocr2_page_marker(row) for row in input_rows)
    return (
        f"{prompt}\n\n"
        "You will receive multiple page images from the same PDF. Convert "
        "each image to Markdown independently and preserve all visible text, "
        "tables, formulas, headings, and reading order. Return exactly one "
        "section for each image, in the same order as the images. Start each "
        "section with the exact marker assigned to that image. Do not merge "
        "pages and do not omit empty-looking pages; if a page has no text, "
        "write the marker followed by a blank line.\n\n"
        "Required section markers:\n"
        f"{markers}"
    )


def region_composite_prompt(
    prompt: str, input_rows: Sequence[Mapping[str, Any]]
) -> str:
    markers = "\n".join(ocr2_region_marker(row) for row in input_rows)
    return (
        f"{prompt}\n\n"
        "You will receive multiple cropped recovery-region images from the "
        "same PDF page and parent page OCR shard. Convert each region to "
        "Markdown independently and preserve all visible text, tables, "
        "formulas, headings, and reading order. Return exactly one section "
        "for each region, in the same order as the images. Start each "
        "section with the exact marker assigned to that region. Do not "
        "merge regions and do not invent missing context.\n\n"
        "Required section markers:\n"
        f"{markers}"
    )


def region_scaffold_prompt(
    prompt: str,
    input_rows: Sequence[Mapping[str, Any]],
    scaffolds: Sequence[Mapping[str, Any]],
    *,
    composite: bool,
) -> str:
    regions = []
    for row, scaffold in zip(input_rows, scaffolds, strict=True):
        regions.append(
            {
                "marker": ocr2_region_marker(row),
                "shardElementId": str(row.get("shardElementId") or ""),
                "pageIndex": row.get("pageIndex"),
                "regionIndex": row.get("regionIndex"),
                "scaffoldKind": scaffold.get("scaffoldKind"),
                "parentShardElementId": row.get("parentShardElementId"),
                "cropBox": scaffold.get("cropBox"),
                "sourcePagePixelBox": scaffold.get("sourcePagePixelBox"),
                "sourcePageProfile": scaffold.get("sourcePageProfile"),
            }
        )
    request_kind = "composite region" if composite else "single region"
    scaffold_json = json.dumps({"regions": regions}, sort_keys=True)
    return (
        f"{prompt}\n\n"
        f"You will receive {request_kind} recovery image input for a PDF. "
        "Use the structural scaffold below as the authoritative routing "
        "contract. Do not infer row or column counts unless they are visible "
        "in the image. Preserve all visible text, table cells, formulas, and "
        "reading order. Return JSON only, with no Markdown fences and no "
        "explanatory text.\n\n"
        "Output schema:\n"
        '{"regions":[{"marker":"exact marker","shardElementId":"exact shard id",'
        '"text":"optional surrounding text","tables":[{"caption":"optional caption",'
        '"rows":[["cell","cell"],["cell","cell"]]}]}]}\n\n'
        "Rules:\n"
        "- Return exactly one regions[] item per scaffold region, in the same order.\n"
        "- Copy each marker and shardElementId exactly.\n"
        "- Use tables[].rows only when a table is visible; all rows in a table "
        "must have the same cell count.\n"
        "- Every region must contain non-empty recognized content. If a table "
        "shape is unclear, put all visible words, numbers, formulas, and symbols "
        "into text instead of returning an empty table or empty text.\n"
        "- If surrounding visible text exists, place it in text.\n"
        "- Do not add cells, rows, columns, or headings that are not visible.\n\n"
        "Structural scaffold:\n"
        f"{scaffold_json}"
    )
