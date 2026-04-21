"""OpenDataLoader-backed PDF extraction worker.

Aligned with Wendao markdown processing: produces one clean `.md` file
per PDF (like a hand-written note) plus standalone image/table/formula
resources.  Text is never fragmented into per-paragraph/heading chunks.
"""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any


def _needs_extraction(source_path: str, output_dir: str) -> bool:
    """Return True if extraction is required (cache miss or stale)."""
    source = Path(source_path)
    if not source.exists():
        return False

    marker = Path(output_dir) / "_complete.marker"
    if not marker.exists():
        return True

    source_mtime = source.stat().st_mtime
    marker_mtime = marker.stat().st_mtime
    return source_mtime > marker_mtime


def _heading_level(element: dict[str, Any]) -> int:
    """Resolve heading level from OpenDataLoader element."""
    level = element.get("heading level", element.get("level", 1))
    if isinstance(level, int):
        return max(1, min(6, level))
    if isinstance(level, str):
        # Handle strings like "Doctitle", "Subtitle", "1", "2"
        mapping = {"doctitle": 1, "subtitle": 2, "section": 3}
        lowered = level.lower()
        if lowered in mapping:
            return mapping[lowered]
        try:
            return max(1, min(6, int(level)))
        except ValueError:
            return 1
    return 1


def _strip_list_prefix(text: str) -> str:
    """Remove bullet/number prefix from list item text."""
    # Match patterns like "1. ", "- ", "• ", "(a) "
    return re.sub(r"^\s*(?:\d+\.\s*|[-•]\s*|\([a-zA-Z0-9]+\)\s*)", "", text)


def _elements_to_markdown(elements: list[dict[str, Any]], image_base_dir: Path) -> str:
    """Convert OpenDataLoader elements into clean Markdown text."""
    lines: list[str] = []

    for element in elements:
        etype = element.get("type", "paragraph")
        content = element.get("content", "")

        if etype == "heading":
            level = _heading_level(element)
            lines.append(f"{'#' * level} {content}")
            lines.append("")

        elif etype == "paragraph":
            lines.append(content)
            lines.append("")

        elif etype == "list":
            for item in element.get("list items", []):
                item_text = item.get("content", "")
                item_text = _strip_list_prefix(item_text)
                lines.append(f"- {item_text}")
            lines.append("")

        elif etype == "table":
            # Preserve HTML table; wrap in markdown for readability
            if content.strip():
                lines.append(content)
                lines.append("")

        elif etype == "formula":
            if content.strip():
                lines.append(f"$$ {content} $$")
                lines.append("")

        elif etype == "image":
            img_path = element.get("image_path", "")
            if img_path:
                # Use relative path from extracted dir for portability
                lines.append(f"![{content}]({img_path})")
                lines.append("")

    # Collapse multiple blank lines
    cleaned: list[str] = []
    prev_blank = False
    for line in lines:
        is_blank = line.strip() == ""
        if is_blank and prev_blank:
            continue
        cleaned.append(line)
        prev_blank = is_blank

    return "\n".join(cleaned).strip() + "\n"


def extract_pdf(
    source_path: str,
    output_dir: str,
    *,
    extract_images: bool = True,
    extract_tables: bool = True,
    extract_formulas: bool = True,
) -> list[dict[str, Any]]:
    """Extract one PDF into structured resources aligned with markdown processing.

    Returns:
        - ``_main`` document resource pointing to the generated ``.md`` file
          (content field contains full markdown text for indexing)
        - One resource per extracted image/table/formula
    """
    try:
        import opendataloader_pdf
    except ModuleNotFoundError as exc:
        raise RuntimeError(
            "opendataloader-pdf is not installed; install it to enable PDF extraction"
        ) from exc

    out = Path(output_dir)
    out.mkdir(parents=True, exist_ok=True)

    if not _needs_extraction(source_path, output_dir):
        metadata_path = out / "_metadata.json"
        if metadata_path.exists():
            with open(metadata_path, "r", encoding="utf-8") as fh:
                return json.load(fh)
        return []

    # Run OpenDataLoader conversion
    opendataloader_pdf.convert(
        input_path=[source_path],
        output_dir=str(out),
        format="json,markdown",
        image_output="external" if extract_images else "none",
    )

    # Parse JSON output
    stem = Path(source_path).stem
    json_path = out / f"{stem}.json"
    elements: list[dict[str, Any]] = []
    if json_path.exists():
        with open(json_path, "r", encoding="utf-8") as fh:
            doc = json.load(fh)
            if isinstance(doc, dict):
                elements = doc.get("kids", [])
            elif isinstance(doc, list):
                elements = doc

    # Build clean markdown from elements
    markdown_text = _elements_to_markdown(elements, out)
    md_path = out / f"{stem}.md"
    with open(md_path, "w", encoding="utf-8") as fh:
        fh.write(markdown_text)

    # Build resource descriptors
    resources: list[dict[str, Any]] = []

    # _main: the canonical markdown document
    resources.append({
        "sourcePath": source_path,
        "resourceType": "document",
        "resourcePath": str(md_path),
        "pageIndex": 0,
        "caption": "",
        "content": markdown_text,
        "mimeType": "text/markdown",
        "status": "ok",
        "boundingBox": [],
        "elementId": "_main",
    })

    # Standalone special elements only
    for element in elements:
        etype = element.get("type", "")
        page = element.get("page number", 0)
        bbox = element.get("bounding box", [])
        content = element.get("content", "")
        eid = element.get("id", "")

        if etype == "image" and extract_images:
            img_path = element.get("image_path", "")
            resources.append({
                "sourcePath": source_path,
                "resourceType": "image",
                "resourcePath": str(out / img_path) if img_path else "",
                "pageIndex": page,
                "caption": content,
                "content": "",
                "mimeType": _mime_from_path(img_path) if img_path else "image/png",
                "status": "ok",
                "boundingBox": bbox,
                "elementId": str(eid),
            })

        elif etype == "table" and extract_tables:
            resources.append({
                "sourcePath": source_path,
                "resourceType": "table",
                "resourcePath": "",
                "pageIndex": page,
                "caption": content,
                "content": content,
                "mimeType": "text/html",
                "status": "ok",
                "boundingBox": bbox,
                "elementId": str(eid),
            })

        elif etype == "formula" and extract_formulas:
            resources.append({
                "sourcePath": source_path,
                "resourceType": "formula",
                "resourcePath": "",
                "pageIndex": page,
                "caption": "",
                "content": content,
                "mimeType": "application/x-latex",
                "status": "ok",
                "boundingBox": bbox,
                "elementId": str(eid),
            })

    # Persist metadata for cache hits
    metadata_path = out / "_metadata.json"
    with open(metadata_path, "w", encoding="utf-8") as fh:
        json.dump(resources, fh, ensure_ascii=False, indent=2)

    marker = out / "_complete.marker"
    marker.touch()

    return resources


def _mime_from_path(path: str) -> str:
    ext = Path(path).suffix.lower()
    return {
        ".png": "image/png",
        ".jpg": "image/jpeg",
        ".jpeg": "image/jpeg",
        ".gif": "image/gif",
        ".webp": "image/webp",
        ".svg": "image/svg+xml",
        ".bmp": "image/bmp",
    }.get(ext, "application/octet-stream")
