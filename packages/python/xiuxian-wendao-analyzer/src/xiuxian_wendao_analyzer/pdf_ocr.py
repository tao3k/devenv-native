"""Arrow contracts for PDF OCR shard workers."""

from __future__ import annotations

import hashlib
from pathlib import Path
from typing import TYPE_CHECKING, Any, Protocol

import pyarrow as pa

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence

    from .documents import DocumentConverterProtocol

PDF_OCR_SHARD_INPUT_SCHEMA_VERSION = "xiuxian_wendao.pdf_ocr_shard_input.v2"
PDF_OCR_SHARD_RESULT_SCHEMA_VERSION = "xiuxian_wendao.pdf_ocr_shard_result.v1"
PDF_OCR_DEFAULT_PROFILE = "docling-compatible-page-ocr-v1"

PDF_OCR_SHARD_INPUT_SCHEMA = pa.schema(
    [
        pa.field("contractVersion", pa.string(), nullable=False),
        pa.field("sourcePath", pa.string(), nullable=False),
        pa.field("sourceContentHash", pa.string(), nullable=False),
        pa.field("pageIndex", pa.int32(), nullable=False),
        pa.field("imagePath", pa.string(), nullable=False),
        pa.field("imageMimeType", pa.string(), nullable=False),
        pa.field("rasterSha256", pa.string(), nullable=False),
        pa.field("renderProfile", pa.string(), nullable=False),
        pa.field("ocrProfile", pa.string(), nullable=False),
        pa.field("ocrEngine", pa.string(), nullable=False),
        pa.field("preferredLanguages", pa.string(), nullable=False),
        pa.field("minConfidence", pa.float64(), nullable=False),
        pa.field("preserveLayout", pa.bool_(), nullable=False),
        pa.field("rasterWidthPx", pa.int32(), nullable=False),
        pa.field("rasterHeightPx", pa.int32(), nullable=False),
        pa.field("renderDpi", pa.int32(), nullable=False),
        pa.field("rotationDegrees", pa.int32(), nullable=False),
        pa.field("cropLeft", pa.float64(), nullable=False),
        pa.field("cropBottom", pa.float64(), nullable=False),
        pa.field("cropRight", pa.float64(), nullable=False),
        pa.field("cropTop", pa.float64(), nullable=False),
        pa.field("pointToPixelScaleX", pa.float64(), nullable=False),
        pa.field("pointToPixelScaleY", pa.float64(), nullable=False),
        pa.field("shardElementId", pa.string(), nullable=False),
        pa.field("shardType", pa.string(), nullable=False),
        pa.field("regionIndex", pa.int32(), nullable=False),
        pa.field("parentShardElementId", pa.string(), nullable=False),
        pa.field("readingOrderKey", pa.string(), nullable=False),
        pa.field("sourcePagePixelLeft", pa.int32(), nullable=False),
        pa.field("sourcePagePixelTop", pa.int32(), nullable=False),
        pa.field("sourcePagePixelRight", pa.int32(), nullable=False),
        pa.field("sourcePagePixelBottom", pa.int32(), nullable=False),
    ],
)

PDF_OCR_SHARD_RESULT_SCHEMA = pa.schema(
    [
        pa.field("contractVersion", pa.string(), nullable=False),
        pa.field("sourcePath", pa.string(), nullable=False),
        pa.field("sourceContentHash", pa.string(), nullable=False),
        pa.field("pageIndex", pa.int32(), nullable=False),
        pa.field("imagePath", pa.string(), nullable=False),
        pa.field("imageMimeType", pa.string(), nullable=False),
        pa.field("rasterSha256", pa.string(), nullable=False),
        pa.field("renderProfile", pa.string(), nullable=False),
        pa.field("ocrProfile", pa.string(), nullable=False),
        pa.field("status", pa.string(), nullable=False),
        pa.field("text", pa.string(), nullable=True),
        pa.field("textMimeType", pa.string(), nullable=False),
        pa.field("confidence", pa.float64(), nullable=True),
        pa.field("errorMessage", pa.string(), nullable=True),
        pa.field("shardElementId", pa.string(), nullable=False),
        pa.field("elementId", pa.string(), nullable=False),
    ],
)


class PdfOcrShardWorkerProtocol(Protocol):
    """Protocol implemented by injected OCR shard workers."""

    def recognize(
        self,
        inputs: Sequence[Mapping[str, Any]],
    ) -> Sequence[Mapping[str, Any]]:
        """Return OCR result rows for input shard rows."""


class SkippingPdfOcrShardWorker:
    """Default no-model OCR worker used when no real engine is configured."""

    def recognize(
        self,
        inputs: Sequence[Mapping[str, Any]],
    ) -> Sequence[Mapping[str, Any]]:
        return [
            skipped_pdf_ocr_shard_result(
                input_row, "OCR shard worker is not configured"
            )
            for input_row in inputs
        ]


class DoclingPdfOcrShardWorker:
    """Docling-backed OCR worker for Rust-rendered PDF page images."""

    def __init__(self, converter: DocumentConverterProtocol | None = None) -> None:
        self._converter = converter

    def recognize(
        self,
        inputs: Sequence[Mapping[str, Any]],
    ) -> Sequence[Mapping[str, Any]]:
        converter = self._converter
        if converter is None:
            converter = _new_docling_converter()
            self._converter = converter
        return [self._recognize_one(converter, input_row) for input_row in inputs]

    def _recognize_one(
        self,
        converter: DocumentConverterProtocol,
        input_row: Mapping[str, Any],
    ) -> Mapping[str, Any]:
        image_path = Path(str(input_row["imagePath"]))
        if not image_path.is_file():
            return failed_pdf_ocr_shard_result(
                input_row,
                f"OCR shard image does not exist: {image_path}",
            )
        try:
            result = converter.convert(image_path)
            markdown = result.document.export_to_markdown()
        except Exception as exc:
            return failed_pdf_ocr_shard_result(input_row, f"Docling OCR failed: {exc}")
        if not markdown.strip():
            return failed_pdf_ocr_shard_result(
                input_row,
                "Docling OCR returned empty text",
            )
        return {
            "status": "succeeded",
            "text": markdown,
            "textMimeType": "text/markdown",
            "confidence": None,
            "errorMessage": None,
        }


def build_pdf_ocr_shard_result_table(
    input_table: pa.Table,
    *,
    worker: PdfOcrShardWorkerProtocol | None = None,
) -> pa.Table:
    """Build an OCR result table from OCR shard input rows.

    # Errors

    Raises `ValueError` when the input schema, contract version, or worker
    result count is invalid.
    """

    validate_pdf_ocr_shard_input_table(input_table)
    input_rows = input_table.to_pylist()
    effective_worker = worker or SkippingPdfOcrShardWorker()
    result_rows = list(effective_worker.recognize(input_rows))
    if len(result_rows) != len(input_rows):
        raise ValueError(
            f"OCR worker returned {len(result_rows)} rows for {len(input_rows)} input rows"
        )
    normalized_rows = [
        normalize_pdf_ocr_shard_result(input_row, result_row)
        for input_row, result_row in zip(input_rows, result_rows, strict=True)
    ]
    return pa.Table.from_pylist(normalized_rows, schema=PDF_OCR_SHARD_RESULT_SCHEMA)


def validate_pdf_ocr_shard_input_table(input_table: pa.Table) -> None:
    """Validate the OCR shard input Arrow table.

    # Errors

    Raises `ValueError` when required columns or contract versions are invalid.
    """

    _validate_schema_compatible(input_table.schema, PDF_OCR_SHARD_INPUT_SCHEMA)
    versions = set(input_table.column("contractVersion").to_pylist())
    if versions - {PDF_OCR_SHARD_INPUT_SCHEMA_VERSION}:
        raise ValueError(
            f"Unexpected OCR shard input contract versions: {sorted(versions)}"
        )
    shard_types = set(input_table.column("shardType").to_pylist())
    if shard_types - {"page", "region"}:
        raise ValueError(f"Unexpected OCR shard types: {sorted(shard_types)}")


def succeeded_pdf_ocr_shard_result(
    input_row: Mapping[str, Any],
    text: str,
    confidence: float,
) -> dict[str, Any]:
    """Build one successful OCR result row."""

    return _result_from_input(
        input_row,
        status="succeeded",
        text=text,
        confidence=confidence,
        error_message=None,
    )


def failed_pdf_ocr_shard_result(
    input_row: Mapping[str, Any],
    error_message: str,
) -> dict[str, Any]:
    """Build one failed OCR result row."""

    return _result_from_input(
        input_row,
        status="failed",
        text=None,
        confidence=None,
        error_message=error_message,
    )


def skipped_pdf_ocr_shard_result(
    input_row: Mapping[str, Any],
    reason: str,
) -> dict[str, Any]:
    """Build one skipped OCR result row."""

    return _result_from_input(
        input_row,
        status="skipped",
        text=None,
        confidence=None,
        error_message=reason,
    )


def normalize_pdf_ocr_shard_result(
    input_row: Mapping[str, Any],
    result_row: Mapping[str, Any],
) -> dict[str, Any]:
    """Normalize one worker result to the stable OCR result schema."""

    row = _result_from_input(
        input_row,
        status=str(result_row.get("status", "failed")),
        text=result_row.get("text"),
        confidence=result_row.get("confidence"),
        error_message=result_row.get("errorMessage"),
    )
    for key in ("textMimeType", "elementId"):
        if result_row.get(key):
            row[key] = result_row[key]
    if row["status"] not in {"succeeded", "failed", "skipped"}:
        row["status"] = "failed"
        row["text"] = None
        row["confidence"] = None
        row["errorMessage"] = (
            f"Unsupported OCR result status: {result_row.get('status')}"
        )
    if row["status"] == "succeeded" and row["text"] is None:
        row["status"] = "failed"
        row["errorMessage"] = "OCR result succeeded without text"
        row["confidence"] = None
    return row


def _result_from_input(
    input_row: Mapping[str, Any],
    *,
    status: str,
    text: Any,
    confidence: Any,
    error_message: Any,
) -> dict[str, Any]:
    return {
        "contractVersion": PDF_OCR_SHARD_RESULT_SCHEMA_VERSION,
        "sourcePath": input_row["sourcePath"],
        "sourceContentHash": input_row["sourceContentHash"],
        "pageIndex": input_row["pageIndex"],
        "imagePath": input_row["imagePath"],
        "imageMimeType": input_row["imageMimeType"],
        "rasterSha256": input_row["rasterSha256"],
        "renderProfile": input_row["renderProfile"],
        "ocrProfile": input_row["ocrProfile"],
        "status": status,
        "text": text,
        "textMimeType": "text/plain",
        "confidence": confidence,
        "errorMessage": error_message,
        "shardElementId": input_row["shardElementId"],
        "elementId": _ocr_result_element_id(input_row),
    }


def _ocr_result_element_id(input_row: Mapping[str, Any]) -> str:
    material = (
        f"{input_row['sourceContentHash']}:{input_row['pageIndex']}:"
        f"{input_row['renderProfile']}:{input_row['ocrProfile']}:"
        f"{input_row['shardElementId']}:{input_row['rasterSha256']}"
    )
    return hashlib.sha256(material.encode("utf-8")).hexdigest()


def _validate_schema_compatible(actual: pa.Schema, expected: pa.Schema) -> None:
    if actual.names != expected.names:
        raise ValueError(f"Unexpected OCR shard input columns: {actual.names}")
    for index, field in enumerate(expected):
        actual_field = actual.field(index)
        if actual_field.type != field.type:
            raise ValueError(
                f"Unexpected OCR shard input type for `{field.name}`: {actual_field.type}"
            )


def _new_docling_converter() -> DocumentConverterProtocol:
    try:
        from docling.document_converter import DocumentConverter
    except ModuleNotFoundError as exc:
        raise RuntimeError(
            "docling is not installed; install xiuxian-wendao-analyzer[documents] "
            "to enable Docling-backed PDF OCR shards"
        ) from exc
    return DocumentConverter()


__all__ = [
    "PDF_OCR_DEFAULT_PROFILE",
    "PDF_OCR_SHARD_INPUT_SCHEMA",
    "PDF_OCR_SHARD_INPUT_SCHEMA_VERSION",
    "PDF_OCR_SHARD_RESULT_SCHEMA",
    "PDF_OCR_SHARD_RESULT_SCHEMA_VERSION",
    "DoclingPdfOcrShardWorker",
    "PdfOcrShardWorkerProtocol",
    "SkippingPdfOcrShardWorker",
    "build_pdf_ocr_shard_result_table",
    "failed_pdf_ocr_shard_result",
    "normalize_pdf_ocr_shard_result",
    "skipped_pdf_ocr_shard_result",
    "succeeded_pdf_ocr_shard_result",
    "validate_pdf_ocr_shard_input_table",
]
