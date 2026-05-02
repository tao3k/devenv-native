"""Docling structure block extraction helpers."""

from __future__ import annotations

import json
from typing import TYPE_CHECKING

from .document_element_access import (
    _element_bbox,
    _element_confidence,
    _element_provenance,
)
from .document_resource_elements import (
    _docling_json_resource,
    _element_id,
    _iter_document_elements,
    _resource_from_element,
)
from .document_types import (
    DOCUMENT_STRUCTURE_SCHEMA_VERSION,
    DoclingDocumentProtocol,
    DocumentResourceRow,
    DocumentStructureBlock,
)

if TYPE_CHECKING:
    from pathlib import Path


def _structured_document_resources(
    source: Path,
    output_dir: Path,
    document: DoclingDocumentProtocol,
) -> list[DocumentResourceRow]:
    resources: list[DocumentResourceRow] = []
    resources.extend(_docling_json_resource(source, output_dir, document))

    for resource_type, attribute_names in _STRUCTURED_RESOURCE_COLLECTIONS:
        for index, element in enumerate(
            _iter_document_elements(document, attribute_names),
            start=1,
        ):
            row = _resource_from_element(
                source, output_dir, resource_type, element, index
            )
            if row is not None:
                resources.append(row)

    return resources


def _document_structure_blocks(
    source: Path,
    document: DoclingDocumentProtocol,
    resources: list[DocumentResourceRow],
    *,
    source_content_hash: str,
) -> list[DocumentStructureBlock]:
    blocks: list[DocumentStructureBlock] = []
    resource_by_element_id = {resource.elementId: resource for resource in resources}
    main_resource = resource_by_element_id.get("_main")
    markdown_text = main_resource.content if main_resource is not None else ""
    blocks.append(
        _structure_block(
            source,
            source_content_hash,
            block_id="docling-main",
            parent_block_id="",
            page_index=0,
            block_index=0,
            block_type="document",
            resource_element_id="_main",
            content=markdown_text,
            mime_type="text/markdown",
            status="ok",
            engine="docling",
            confidence=None,
            bbox=None,
            provenance={"source": "docling_export_to_markdown"},
        )
    )

    block_index = 1
    for resource_type, attribute_names in _STRUCTURED_RESOURCE_COLLECTIONS:
        for element_index, element in enumerate(
            _iter_document_elements(document, attribute_names),
            start=1,
        ):
            element_id = _element_id(element, resource_type, element_index)
            resource = resource_by_element_id.get(element_id)
            if resource is None:
                continue
            page_index = resource.pageIndex
            blocks.append(
                _structure_block(
                    source,
                    source_content_hash,
                    block_id=element_id,
                    parent_block_id="docling-main",
                    page_index=page_index,
                    block_index=block_index,
                    block_type=resource.resourceType,
                    resource_element_id=resource.elementId,
                    content=resource.content,
                    mime_type=resource.mimeType,
                    status=resource.status,
                    engine="docling",
                    confidence=_element_confidence(element),
                    bbox=_element_bbox(element),
                    provenance=_element_provenance(element),
                )
            )
            block_index += 1
    return blocks


def _structure_block(
    source: Path,
    source_content_hash: str,
    *,
    block_id: str,
    parent_block_id: str,
    page_index: int,
    block_index: int,
    block_type: str,
    resource_element_id: str,
    content: str,
    mime_type: str,
    status: str,
    engine: str,
    confidence: float | None,
    bbox: tuple[float, float, float, float] | None,
    provenance: object,
) -> DocumentStructureBlock:
    bbox_left, bbox_top, bbox_right, bbox_bottom = bbox or (None, None, None, None)
    return DocumentStructureBlock(
        contractVersion=DOCUMENT_STRUCTURE_SCHEMA_VERSION,
        sourcePath=str(source),
        sourceContentHash=source_content_hash,
        blockId=block_id,
        parentBlockId=parent_block_id,
        pageIndex=page_index,
        blockIndex=block_index,
        readingOrderKey=f"{page_index:06}.{block_index:06}",
        blockType=block_type,
        resourceElementId=resource_element_id,
        content=content,
        mimeType=mime_type,
        status=status,
        engine=engine,
        confidence=confidence,
        bboxLeft=bbox_left,
        bboxTop=bbox_top,
        bboxRight=bbox_right,
        bboxBottom=bbox_bottom,
        provenance=json.dumps(provenance, ensure_ascii=False, sort_keys=True),
    )


_STRUCTURED_RESOURCE_COLLECTIONS: tuple[tuple[str, tuple[str, ...]], ...] = (
    ("table", ("tables",)),
    ("image", ("pictures", "images", "figures")),
    ("formula", ("formulas", "equations")),
    ("code", ("code_blocks", "codeblocks")),
    ("audio", ("audio_segments", "audio", "tracks")),
    ("subtitle", ("subtitles", "cues", "vtt_cues")),
)
