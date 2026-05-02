"""Document extraction cache and warmup helpers."""

from __future__ import annotations

import hashlib
from typing import TYPE_CHECKING

import pyarrow as pa

from .document_metrics import (
    DOCUMENT_TIMING_SCHEMA,
    DOCUMENT_TIMING_SCHEMA_VERSION,
    DocumentTimingRecorder,
    write_document_timing_cache,
)
from .document_types import (
    DOCUMENT_RESOURCE_ARROW_CACHE_NAME,
    DOCUMENT_RESOURCE_SCHEMA,
    DOCUMENT_STRUCTURE_ARROW_CACHE_NAME,
    DOCUMENT_STRUCTURE_SCHEMA,
    DOCUMENT_STRUCTURE_SCHEMA_VERSION,
    DocumentConverterProtocol,
    DocumentResourceRow,
    DocumentStructureBlock,
    document_resources_to_table,
    document_structure_to_table,
)

if TYPE_CHECKING:
    from collections.abc import Mapping
    from pathlib import Path
    from typing import Any


_DOCUMENT_ARROW_RUNTIME_WARMED = False


def warm_document_arrow_runtime() -> None:
    """Pre-initialize Arrow table conversion and IPC writers."""

    global _DOCUMENT_ARROW_RUNTIME_WARMED
    if _DOCUMENT_ARROW_RUNTIME_WARMED:
        return
    for schema, row in (
        (DOCUMENT_RESOURCE_SCHEMA, _resource_warmup_row()),
        (DOCUMENT_STRUCTURE_SCHEMA, _structure_warmup_row()),
        (DOCUMENT_TIMING_SCHEMA, _timing_warmup_row()),
    ):
        table = pa.Table.from_pylist([row], schema=schema)
        sink = pa.BufferOutputStream()
        with pa.ipc.new_file(sink, schema) as writer:
            writer.write_table(table)
        sink.getvalue()
    _DOCUMENT_ARROW_RUNTIME_WARMED = True


def _write_document_timing_sidecar(
    output_dir: Path,
    timing: DocumentTimingRecorder,
) -> None:
    try:
        write_document_timing_cache(output_dir, timing.rows)
    except (OSError, pa.ArrowException, ValueError):
        return


def _resource_warmup_row() -> dict[str, object]:
    return {
        "sourcePath": "",
        "resourceType": "warmup",
        "resourcePath": "",
        "pageIndex": 0,
        "caption": "",
        "content": "",
        "mimeType": "application/x-wendao-warmup",
        "status": "ok",
        "elementId": "_warmup",
    }


def _structure_warmup_row() -> dict[str, object]:
    return {
        "contractVersion": DOCUMENT_STRUCTURE_SCHEMA_VERSION,
        "sourcePath": "",
        "sourceContentHash": "",
        "blockId": "_warmup",
        "parentBlockId": "",
        "pageIndex": 0,
        "blockIndex": 0,
        "readingOrderKey": "000000.000000",
        "blockType": "warmup",
        "resourceElementId": "_warmup",
        "content": "",
        "mimeType": "application/x-wendao-warmup",
        "status": "ok",
        "engine": "wendao",
        "confidence": None,
        "bboxLeft": None,
        "bboxTop": None,
        "bboxRight": None,
        "bboxBottom": None,
        "provenance": "{}",
    }


def _timing_warmup_row() -> dict[str, object]:
    return {
        "contractVersion": DOCUMENT_TIMING_SCHEMA_VERSION,
        "sourcePath": "",
        "sourceSuffix": "",
        "phase": "warmup",
        "elapsedMs": 0.0,
        "status": "ok",
        "detail": "",
        "resourceRows": 0,
        "structureRows": 0,
    }


def _new_docling_converter() -> DocumentConverterProtocol:
    try:
        from docling.document_converter import DocumentConverter
    except ModuleNotFoundError as exc:
        raise RuntimeError(
            "docling is not installed; install xiuxian-wendao-analyzer[documents] "
            "to enable document extraction"
        ) from exc
    return DocumentConverter()


def _read_cached_resources(
    source: Path, output_dir: Path
) -> list[DocumentResourceRow] | None:
    table = _read_cached_table(source, output_dir)
    if table is None:
        return None
    return [_resource_from_mapping(row) for row in table.to_pylist()]


def _read_cached_table(source: Path, output_dir: Path) -> pa.Table | None:
    marker = output_dir / "_complete.marker"
    resources_path = output_dir / DOCUMENT_RESOURCE_ARROW_CACHE_NAME
    if not marker.exists() or not resources_path.exists():
        return None
    if source.stat().st_mtime > marker.stat().st_mtime:
        return None

    try:
        with pa.ipc.open_file(resources_path) as reader:
            table = reader.read_all()
    except (pa.ArrowInvalid, OSError):
        return None
    if table.schema != DOCUMENT_RESOURCE_SCHEMA:
        return None
    return table


def _write_cached_resources(
    output_dir: Path, resources: list[DocumentResourceRow]
) -> None:
    resources_path = output_dir / DOCUMENT_RESOURCE_ARROW_CACHE_NAME
    table = document_resources_to_table(resources)
    with pa.ipc.new_file(resources_path, DOCUMENT_RESOURCE_SCHEMA) as writer:
        writer.write_table(table)
    (output_dir / "_complete.marker").touch()


def _write_cached_structure(
    output_dir: Path,
    blocks: list[DocumentStructureBlock],
) -> None:
    structure_path = output_dir / DOCUMENT_STRUCTURE_ARROW_CACHE_NAME
    table = document_structure_to_table(blocks)
    with pa.ipc.new_file(structure_path, DOCUMENT_STRUCTURE_SCHEMA) as writer:
        writer.write_table(table)


def _file_sha256(source: Path) -> str:
    hasher = hashlib.sha256()
    with source.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


def _resource_from_mapping(row: Mapping[str, Any]) -> DocumentResourceRow:
    return DocumentResourceRow(
        sourcePath=str(row.get("sourcePath", "")),
        resourceType=str(row.get("resourceType", "")),
        resourcePath=str(row.get("resourcePath", "")),
        pageIndex=int(row.get("pageIndex", 0)),
        caption=str(row.get("caption", "")),
        content=str(row.get("content", "")),
        mimeType=str(row.get("mimeType", "")),
        status=str(row.get("status", "")),
        elementId=str(row.get("elementId", "")),
    )
