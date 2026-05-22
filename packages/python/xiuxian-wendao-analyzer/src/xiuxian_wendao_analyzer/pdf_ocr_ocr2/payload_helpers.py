"""OCR2 image encoding and prompt helpers."""

from __future__ import annotations

import base64
import json
from typing import TYPE_CHECKING, Any

from ..pdf_ocr_contracts import (
    HOSTED_VLM_OCR_COMPACT_REGION_MARKDOWN_PROMPT_MODE,
    HOSTED_VLM_OCR_DEFAULT_IMAGE_OPTIMIZATION,
)
from .image_payload import hosted_vlm_image_payload
from .markers import ocr2_page_marker, ocr2_region_marker

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence
    from pathlib import Path


def image_data_url(
    input_row: Mapping[str, Any],
    image_path: Path,
    *,
    image_optimization_mode: str = HOSTED_VLM_OCR_DEFAULT_IMAGE_OPTIMIZATION,
) -> str:
    image_payload = hosted_vlm_image_payload(
        input_row,
        image_path,
        image_optimization_mode=image_optimization_mode,
    )
    return image_bytes_data_url(
        image_payload.image_bytes,
        image_payload.image_mime_type,
    )


def image_bytes_data_url(image_bytes: bytes, image_mime_type: str) -> str:
    encoded = base64.b64encode(image_bytes).decode("ascii")
    return f"data:{image_mime_type};base64,{encoded}"


def single_region_prompt(
    prompt: str,
    input_row: Mapping[str, Any],
    *,
    region_prompt_mode: str | None,
) -> str:
    if (
        str(input_row.get("shardType") or "") != "region"
        or region_prompt_mode != HOSTED_VLM_OCR_COMPACT_REGION_MARKDOWN_PROMPT_MODE
    ):
        return prompt
    source_box = {
        "left": input_row.get("sourcePagePixelLeft"),
        "top": input_row.get("sourcePagePixelTop"),
        "right": input_row.get("sourcePagePixelRight"),
        "bottom": input_row.get("sourcePagePixelBottom"),
    }
    region_json = json.dumps(
        {
            "pageIndex": input_row.get("pageIndex"),
            "regionIndex": input_row.get("regionIndex"),
            "shardElementId": str(input_row.get("shardElementId") or ""),
            "parentShardElementId": str(input_row.get("parentShardElementId") or ""),
            "sourcePagePixelBox": source_box,
        },
        sort_keys=True,
    )
    return (
        f"{prompt}\n\n"
        "You will receive exactly one cropped OCR recovery region from a PDF. "
        "Treat the image as a region patch, not as a full document page. "
        "Return compact Markdown for only the visible crop. Preserve all "
        "visible text, table cells, formulas, symbols, and reading order. "
        "Do not add introductions, code fences, section markers, page titles, "
        "or context that is not visible in the crop. If the table shape is "
        "unclear, output the visible words, numbers, formulas, and symbols in "
        "reading order instead of inventing rows or columns.\n\n"
        "Region metadata:\n"
        f"{region_json}"
    )


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


def region_atlas_prompt(prompt: str, input_rows: Sequence[Mapping[str, Any]]) -> str:
    regions = [
        {
            "panel": f"REGION {index}",
            "marker": ocr2_region_marker(row),
            "shardElementId": str(row.get("shardElementId") or ""),
            "pageIndex": row.get("pageIndex"),
            "regionIndex": row.get("regionIndex"),
            "parentShardElementId": row.get("parentShardElementId"),
            "sourcePagePixelBox": {
                "left": row.get("sourcePagePixelLeft"),
                "top": row.get("sourcePagePixelTop"),
                "right": row.get("sourcePagePixelRight"),
                "bottom": row.get("sourcePagePixelBottom"),
            },
        }
        for index, row in enumerate(input_rows, start=1)
    ]
    mapping_json = json.dumps({"regions": regions}, sort_keys=True)
    return (
        f"{prompt}\n\n"
        "You will receive one atlas image containing cropped OCR recovery "
        "regions from the same PDF page. Each panel starts with a visible "
        "black label such as REGION 1. Recognize each panel independently; "
        "do not merge panels and do not use content from one panel to fill "
        "another. Preserve all visible text, table cells, formulas, symbols, "
        "and reading order inside each panel. Return JSON only, with no "
        "Markdown fences and no explanatory text.\n\n"
        "Output schema:\n"
        '{"regions":[{"panel":"REGION 1","marker":"exact marker",'
        '"shardElementId":"exact shard id","text":"optional text",'
        '"tables":[{"caption":"optional caption","rows":[["cell","cell"]]}]}]}\n\n'
        "Rules:\n"
        "- Return exactly one regions[] item per atlas panel, in mapping order.\n"
        "- Copy panel, marker, and shardElementId exactly from the mapping.\n"
        "- Use tables[].rows only when a table is visible; all rows in a table "
        "must have the same cell count.\n"
        "- Every region must contain non-empty recognized content. If a table "
        "shape is unclear, put all visible words, numbers, formulas, and "
        "symbols into text.\n"
        "- Do not add cells, rows, columns, or headings that are not visible.\n\n"
        "Atlas panel mapping:\n"
        f"{mapping_json}"
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
