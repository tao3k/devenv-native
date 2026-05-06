"""OCR2 structural scaffold sidecar validation and canonicalization."""

from __future__ import annotations

import json
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

from .markers import ocr2_region_marker

_DEEPSEEK_OCR2_REGION_SCAFFOLD_FILE_NAME = "_ocr2_region_scaffolds.json"
_DEEPSEEK_OCR2_REGION_SCAFFOLD_SCHEMA = "xiuxian_wendao.ocr2_region_scaffold.v1"


def load_ocr2_region_scaffolds(
    input_rows: Sequence[Mapping[str, Any]],
) -> list[Mapping[str, Any]]:
    sidecars: dict[Path, Mapping[str, Any]] = {}
    scaffolds: list[Mapping[str, Any]] = []
    for row in input_rows:
        sidecar_path = resolve_ocr2_region_scaffold_sidecar_path(
            Path(str(row.get("imagePath") or ""))
        )
        if sidecar_path not in sidecars:
            sidecars[sidecar_path] = read_ocr2_region_scaffold_sidecar(sidecar_path)
        scaffolds.append(match_ocr2_region_scaffold(sidecars[sidecar_path], row))
    return scaffolds


def resolve_ocr2_region_scaffold_sidecar_path(image_path: Path) -> Path:
    direct_path = image_path.parent / _DEEPSEEK_OCR2_REGION_SCAFFOLD_FILE_NAME
    for depth, directory in enumerate(image_path.parents):
        if depth >= 6:
            break
        candidate = directory / _DEEPSEEK_OCR2_REGION_SCAFFOLD_FILE_NAME
        if candidate.is_file():
            return candidate
    return direct_path


def read_ocr2_region_scaffold_sidecar(path: Path) -> Mapping[str, Any]:
    if not path.is_file():
        raise ValueError(f"missing OCR2 region scaffold sidecar: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ValueError(f"malformed OCR2 region scaffold sidecar: {path}") from exc
    if not isinstance(payload, Mapping):
        raise ValueError("OCR2 region scaffold sidecar is not an object")
    if payload.get("schema") != _DEEPSEEK_OCR2_REGION_SCAFFOLD_SCHEMA:
        raise ValueError("OCR2 region scaffold sidecar has unsupported schema")
    items = payload.get("items")
    if not isinstance(items, list):
        raise ValueError("OCR2 region scaffold sidecar is missing items")
    return payload


def match_ocr2_region_scaffold(
    sidecar: Mapping[str, Any],
    input_row: Mapping[str, Any],
) -> Mapping[str, Any]:
    shard_element_id = str(input_row.get("shardElementId") or "")
    items = sidecar.get("items")
    if not isinstance(items, list):
        raise ValueError("OCR2 region scaffold sidecar is missing items")
    matches = [
        item
        for item in items
        if isinstance(item, Mapping)
        and str(item.get("shardElementId") or "") == shard_element_id
    ]
    if len(matches) != 1:
        raise ValueError("OCR2 region scaffold item count does not match input row")
    item = matches[0]
    checks = [
        ("parentShardElementId", "parentShardElementId"),
        ("sourceContentHash", "sourceContentHash"),
        ("rasterSha256", "rasterSha256"),
    ]
    for item_key, row_key in checks:
        if str(item.get(item_key) or "") != str(input_row.get(row_key) or ""):
            raise ValueError(
                f"OCR2 region scaffold fingerprint mismatch for {item_key}"
            )
    if int(item.get("pageIndex", -1)) != int(input_row.get("pageIndex", -2)):
        raise ValueError("OCR2 region scaffold page index mismatch")
    if int(item.get("regionIndex", -1)) != int(input_row.get("regionIndex", -2)):
        raise ValueError("OCR2 region scaffold region index mismatch")
    return item


def extract_ocr2_scaffold_markdown(
    response_text: str,
    input_rows: Sequence[Mapping[str, Any]],
) -> list[str]:
    if not response_text.strip():
        raise ValueError("OCR2 scaffold response returned empty text")
    try:
        payload = json.loads(response_text)
    except json.JSONDecodeError as exc:
        raise ValueError("OCR2 scaffold response is not valid JSON") from exc
    if not isinstance(payload, Mapping):
        raise ValueError("OCR2 scaffold response is not a JSON object")
    regions = payload.get("regions")
    if not isinstance(regions, list):
        raise ValueError("OCR2 scaffold response is missing regions")
    if len(regions) != len(input_rows):
        raise ValueError("OCR2 scaffold response row count mismatch")
    markdown_rows: list[str] = []
    for region, input_row in zip(regions, input_rows, strict=True):
        if not isinstance(region, Mapping):
            raise ValueError("OCR2 scaffold region is not an object")
        expected_marker = ocr2_region_marker(input_row)
        if str(region.get("marker") or "") != expected_marker:
            raise ValueError("OCR2 scaffold response marker mismatch")
        if str(region.get("shardElementId") or "") != str(
            input_row.get("shardElementId") or ""
        ):
            raise ValueError("OCR2 scaffold response shard id mismatch")
        markdown = canonicalize_ocr2_scaffold_region_markdown(region)
        if not markdown.strip():
            raise ValueError("OCR2 scaffold response returned empty canonical text")
        markdown_rows.append(markdown)
    return markdown_rows


def canonicalize_ocr2_scaffold_region_markdown(region: Mapping[str, Any]) -> str:
    parts: list[str] = []
    for text_key in ("text", "content", "markdown", "formula", "formulas", "lines"):
        text = region.get(text_key)
        if isinstance(text, str) and text.strip():
            parts.append(text.strip())
        elif isinstance(text, list):
            text_parts = [str(item).strip() for item in text if str(item).strip()]
            if text_parts:
                parts.append("\n".join(text_parts))
    tables = region.get("tables")
    if tables is not None:
        if not isinstance(tables, list):
            raise ValueError("OCR2 scaffold tables field is not a list")
        for table in tables:
            if not isinstance(table, Mapping):
                raise ValueError("OCR2 scaffold table is not an object")
            caption = table.get("caption")
            if isinstance(caption, str) and caption.strip():
                parts.append(caption.strip())
            rows = table.get("rows")
            parts.append(canonical_markdown_table(rows))
    return "\n\n".join(parts).strip()


def canonical_markdown_table(rows: Any) -> str:
    if not isinstance(rows, list) or not rows:
        raise ValueError("OCR2 scaffold table rows are empty")
    canonical_rows: list[list[str]] = []
    expected_width: int | None = None
    for row in rows:
        if not isinstance(row, list) or not row:
            raise ValueError("OCR2 scaffold table row has invalid cell shape")
        cells = [canonical_markdown_cell(cell) for cell in row]
        if expected_width is None:
            expected_width = len(cells)
        elif len(cells) != expected_width:
            raise ValueError("OCR2 scaffold table rows have inconsistent cell shape")
        canonical_rows.append(cells)
    header = canonical_rows[0]
    separator = ["---"] * len(header)
    body = canonical_rows[1:]
    table_rows = [header, separator, *body]
    return "\n".join("| " + " | ".join(cells) + " |" for cells in table_rows).strip()


def canonical_markdown_cell(value: Any) -> str:
    if value is None:
        return ""
    return str(value).replace("\n", "<br>").replace("|", "\\|").strip()
