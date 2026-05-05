"""Docling element to resource-row adapters."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .document_element_access import (
    _element_caption,
    _element_content,
    _element_page_index,
    _element_resource_path,
)
from .document_types import (
    DoclingDocumentDictExportProtocol,
    DoclingDocumentProtocol,
    DocumentResourceRow,
)


def _docling_json_resource(
    source: Path,
    output_dir: Path,
    document: DoclingDocumentProtocol,
) -> list[DocumentResourceRow]:
    if not isinstance(document, DoclingDocumentDictExportProtocol):
        return []

    content = json.dumps(document.export_to_dict(), ensure_ascii=False, sort_keys=True)
    resource_path = output_dir / f"{source.stem}.docling.json"
    resource_path.write_text(content, encoding="utf-8")
    return [
        DocumentResourceRow(
            sourcePath=str(source),
            resourceType="docling_json",
            resourcePath=str(resource_path),
            pageIndex=0,
            caption="",
            content="",
            mimeType="application/json",
            status="ok",
            elementId="_docling_json",
        )
    ]


def _iter_document_elements(
    document: DoclingDocumentProtocol,
    attribute_names: tuple[str, ...],
) -> list[Any]:
    elements: list[Any] = []
    seen_ids: set[int] = set()
    for attribute_name in attribute_names:
        value = getattr(document, attribute_name, None)
        if value is None:
            continue
        if isinstance(value, dict):
            candidates = value.values()
        elif isinstance(value, (str, bytes)):
            candidates = (value,)
        else:
            try:
                candidates = iter(value)
            except TypeError:
                candidates = (value,)
        for candidate in candidates:
            candidate_id = id(candidate)
            if candidate_id in seen_ids:
                continue
            seen_ids.add(candidate_id)
            elements.append(candidate)
    return elements


def _resource_from_element(
    source: Path,
    output_dir: Path,
    resource_type: str,
    element: Any,
    index: int,
) -> DocumentResourceRow | None:
    content = _element_content(element)
    resource_path = _element_resource_path(element)
    if content and not resource_path:
        suffix = _resource_file_suffix(resource_type)
        resource_file = output_dir / f"{resource_type}-{index}{suffix}"
        resource_file.write_text(content, encoding="utf-8")
        resource_path = str(resource_file)
    if not content and not resource_path:
        return None

    return DocumentResourceRow(
        sourcePath=str(source),
        resourceType=resource_type,
        resourcePath=resource_path,
        pageIndex=_element_page_index(element),
        caption=_element_caption(element),
        content=content,
        mimeType=_resource_mime_type(resource_type, resource_path),
        status="ok",
        elementId=_element_id(element, resource_type, index),
    )


def _element_id(element: Any, resource_type: str, index: int) -> str:
    for attribute_name in ("self_ref", "cref", "id", "name"):
        value = getattr(element, attribute_name, None)
        if value:
            return str(value).strip("#/").replace("/", "-")
    return f"{resource_type}-{index}"


def _resource_file_suffix(resource_type: str) -> str:
    return {
        "docling_json": ".json",
        "table": ".md",
        "image": ".txt",
        "formula": ".tex",
        "code": ".txt",
        "audio": ".txt",
        "subtitle": ".vtt",
    }.get(resource_type, ".txt")


def _resource_mime_type(resource_type: str, resource_path: str) -> str:
    if resource_path:
        suffix = Path(resource_path).suffix.lower()
        if suffix == ".json":
            return "application/json"
        if suffix == ".png":
            return "image/png"
        if suffix in {".jpg", ".jpeg"}:
            return "image/jpeg"
        if suffix in {".tif", ".tiff"}:
            return "image/tiff"
        if suffix == ".bmp":
            return "image/bmp"
        if suffix == ".webp":
            return "image/webp"
        if suffix == ".mp3":
            return "audio/mpeg"
        if suffix == ".wav":
            return "audio/wav"
        if suffix == ".m4a":
            return "audio/mp4"
        if suffix == ".html":
            return "text/html"
        if suffix in {".md", ".markdown"}:
            return "text/markdown"
        if suffix in {".tex", ".latex"}:
            return "application/x-tex"
        if suffix in {".vtt", ".webvtt"}:
            return "text/vtt"
    return {
        "docling_json": "application/json",
        "table": "text/markdown",
        "image": "text/plain",
        "formula": "application/x-tex",
        "code": "text/plain",
        "audio": "text/plain",
        "subtitle": "text/vtt",
    }.get(resource_type, "text/plain")
