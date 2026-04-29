"""Arrow contracts for PDF OCR shard workers."""

from __future__ import annotations

import hashlib
import os
import threading
from concurrent.futures import ThreadPoolExecutor
from itertools import pairwise
from pathlib import Path
from typing import TYPE_CHECKING, Any, Protocol

import pyarrow as pa

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping, Sequence

    from .documents import DocumentConverterProtocol

PDF_OCR_SHARD_INPUT_SCHEMA_VERSION = "xiuxian_wendao.pdf_ocr_shard_input.v1"
PDF_OCR_SHARD_RESULT_SCHEMA_VERSION = "xiuxian_wendao.pdf_ocr_shard_result.v1"
PDF_OCR_DEFAULT_PROFILE = "docling-compatible-page-ocr-v1"
PDF_OCR_FAST_TEXT_PROFILE = "docling-fast-text-ocr"
PDF_OCR_WORKERS_ENV = "WENDAO_PDF_OCR_WORKERS"
PDF_OCR_MAX_WORKERS_ENV = "WENDAO_PDF_OCR_MAX_WORKERS"

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
        *,
        max_workers: int | str | None = None,
    ) -> Sequence[Mapping[str, Any]]:
        """Return OCR result rows for input shard rows."""


class SkippingPdfOcrShardWorker:
    """Default no-model OCR worker used when no real engine is configured."""

    def recognize(
        self,
        inputs: Sequence[Mapping[str, Any]],
        *,
        max_workers: int | str | None = None,
    ) -> Sequence[Mapping[str, Any]]:
        _ = max_workers
        return [
            skipped_pdf_ocr_shard_result(
                input_row, "OCR shard worker is not configured"
            )
            for input_row in inputs
        ]


class DoclingPdfOcrShardWorker:
    """Docling-backed OCR worker for Rust-rendered PDF page images."""

    def __init__(
        self,
        converter: DocumentConverterProtocol | None = None,
        *,
        converter_factory: Callable[[], DocumentConverterProtocol] | None = None,
        max_workers: int | str | None = None,
    ) -> None:
        if converter is not None and converter_factory is not None:
            raise ValueError("converter and converter_factory are mutually exclusive")
        self._converter = converter
        self._converter_factory = converter_factory
        self._max_workers = max_workers
        self._thread_local = threading.local()

    def recognize(
        self,
        inputs: Sequence[Mapping[str, Any]],
        *,
        max_workers: int | str | None = None,
    ) -> Sequence[Mapping[str, Any]]:
        input_rows = list(inputs)
        recognition_groups = _group_pdf_ocr_inputs(input_rows)
        worker_count = resolve_pdf_ocr_worker_count(
            len(recognition_groups),
            max_workers if max_workers is not None else self._max_workers,
        )
        if self._converter is not None and self._converter_factory is None:
            worker_count = 1
        if worker_count <= 1:
            return _flatten_group_results(
                len(input_rows),
                [
                    self._recognize_group_with_thread_converter(indexes, rows)
                    for indexes, rows in recognition_groups
                ],
            )
        with ThreadPoolExecutor(max_workers=worker_count) as executor:
            futures = [
                executor.submit(
                    self._recognize_group_with_thread_converter,
                    indexes,
                    rows,
                )
                for indexes, rows in recognition_groups
            ]
            return _flatten_group_results(
                len(input_rows),
                [future.result() for future in futures],
            )

    def _recognize_group_with_thread_converter(
        self,
        indexes: Sequence[int],
        input_rows: Sequence[Mapping[str, Any]],
    ) -> list[tuple[int, Mapping[str, Any]]]:
        try:
            converter = self._converter_for_thread()
        except Exception as exc:
            return [
                (
                    index,
                    failed_pdf_ocr_shard_result(
                        input_row,
                        f"Docling OCR converter initialization failed: {exc}",
                    ),
                )
                for index, input_row in zip(indexes, input_rows, strict=True)
            ]
        return [
            (index, result)
            for index, result in zip(
                indexes,
                self._recognize_many(converter, input_rows),
                strict=True,
            )
        ]

    def _recognize_many(
        self,
        converter: DocumentConverterProtocol,
        input_rows: Sequence[Mapping[str, Any]],
    ) -> list[Mapping[str, Any]]:
        if len(input_rows) > 1 and _is_source_pdf_page_range_group(input_rows):
            source_path = Path(str(input_rows[0]["sourcePath"]))
            result = self._try_convert_source_page_batch(
                converter,
                input_rows,
                source_path,
            )
            if result is not None:
                return result
        return [self._recognize_one(converter, input_row) for input_row in input_rows]

    def _recognize_one(
        self,
        converter: DocumentConverterProtocol,
        input_row: Mapping[str, Any],
    ) -> Mapping[str, Any]:
        if _should_try_source_pdf_page_range(input_row):
            source_path = Path(str(input_row["sourcePath"]))
            result = self._try_convert_source_page(converter, input_row, source_path)
            if result is not None:
                return result

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

    def _try_convert_source_page(
        self,
        converter: DocumentConverterProtocol,
        input_row: Mapping[str, Any],
        source_path: Path,
    ) -> Mapping[str, Any] | None:
        try:
            page_number = int(input_row["pageIndex"]) + 1
            result = converter.convert(
                source_path, page_range=(page_number, page_number)
            )
            markdown = result.document.export_to_markdown()
        except Exception:
            return None
        if not markdown.strip():
            return None
        return {
            "status": "succeeded",
            "text": markdown,
            "textMimeType": "text/markdown",
            "confidence": None,
            "errorMessage": None,
        }

    def _try_convert_source_page_batch(
        self,
        converter: DocumentConverterProtocol,
        input_rows: Sequence[Mapping[str, Any]],
        source_path: Path,
    ) -> list[Mapping[str, Any]] | None:
        try:
            start_page = int(input_rows[0]["pageIndex"]) + 1
            end_page = int(input_rows[-1]["pageIndex"]) + 1
            result = converter.convert(source_path, page_range=(start_page, end_page))
            rows = []
            for input_row in input_rows:
                page_number = int(input_row["pageIndex"]) + 1
                markdown = result.document.export_to_markdown(page_no=page_number)
                if not markdown.strip():
                    return None
                rows.append(
                    {
                        "status": "succeeded",
                        "text": markdown,
                        "textMimeType": "text/markdown",
                        "confidence": None,
                        "errorMessage": None,
                    }
                )
        except Exception:
            return None
        return rows

    def _converter_for_thread(self) -> DocumentConverterProtocol:
        if self._converter is not None:
            return self._converter
        converter = getattr(self._thread_local, "converter", None)
        if converter is None:
            factory = self._converter_factory or _new_docling_converter
            converter = factory()
            self._thread_local.converter = converter
        return converter


def build_pdf_ocr_shard_result_table(
    input_table: pa.Table,
    *,
    worker: PdfOcrShardWorkerProtocol | None = None,
    max_workers: int | str | None = None,
) -> pa.Table:
    """Build an OCR result table from OCR shard input rows.

    # Errors

    Raises `ValueError` when the input schema, contract version, or worker
    result count is invalid.
    """

    validate_pdf_ocr_shard_input_table(input_table)
    input_rows = input_table.to_pylist()
    effective_worker = worker or SkippingPdfOcrShardWorker()
    result_rows = list(effective_worker.recognize(input_rows, max_workers=max_workers))
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


def _group_pdf_ocr_inputs(
    input_rows: Sequence[Mapping[str, Any]],
) -> list[tuple[list[int], list[Mapping[str, Any]]]]:
    groups: list[tuple[list[int], list[Mapping[str, Any]]]] = []
    current_indexes: list[int] = []
    current_rows: list[Mapping[str, Any]] = []
    for index, input_row in enumerate(input_rows):
        if current_rows and _can_extend_source_page_group(current_rows[-1], input_row):
            current_indexes.append(index)
            current_rows.append(input_row)
            continue
        if current_rows:
            groups.append((current_indexes, current_rows))
        current_indexes = [index]
        current_rows = [input_row]
    if current_rows:
        groups.append((current_indexes, current_rows))
    return groups


def _can_extend_source_page_group(
    previous_row: Mapping[str, Any],
    input_row: Mapping[str, Any],
) -> bool:
    if not _should_try_source_pdf_page_range(previous_row):
        return False
    if not _should_try_source_pdf_page_range(input_row):
        return False
    if str(previous_row["sourcePath"]) != str(input_row["sourcePath"]):
        return False
    return int(input_row["pageIndex"]) == int(previous_row["pageIndex"]) + 1


def _is_source_pdf_page_range_group(
    input_rows: Sequence[Mapping[str, Any]],
) -> bool:
    return all(_should_try_source_pdf_page_range(row) for row in input_rows) and all(
        _can_extend_source_page_group(previous_row, input_row)
        for previous_row, input_row in pairwise(input_rows)
    )


def _flatten_group_results(
    input_count: int,
    group_results: Sequence[Sequence[tuple[int, Mapping[str, Any]]]],
) -> list[Mapping[str, Any]]:
    ordered: list[Mapping[str, Any] | None] = [None] * input_count
    for group in group_results:
        for index, result in group:
            ordered[index] = result
    return [
        (
            result
            if result is not None
            else {"status": "failed", "errorMessage": "missing result"}
        )
        for result in ordered
    ]


def resolve_pdf_ocr_worker_count(
    input_count: int,
    requested: int | str | None = None,
) -> int:
    """Resolve the bounded OCR worker count for a shard request."""

    if input_count <= 0:
        return 1
    requested_value = requested
    if requested_value is None:
        requested_value = os.environ.get(PDF_OCR_WORKERS_ENV, "auto")
    if isinstance(requested_value, str):
        normalized = requested_value.strip().lower()
        if normalized and normalized != "auto":
            parsed = _parse_positive_int(normalized)
            if parsed is not None:
                return _cap_pdf_ocr_worker_count(input_count, parsed)
        cpu_count = os.cpu_count() or 1
        return _cap_pdf_ocr_worker_count(input_count, cpu_count)
    return _cap_pdf_ocr_worker_count(input_count, int(requested_value))


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


def _should_try_source_pdf_page_range(input_row: Mapping[str, Any]) -> bool:
    if str(input_row.get("shardType", "")) != "page":
        return False
    source_path = Path(str(input_row.get("sourcePath", "")))
    return source_path.suffix.lower() == ".pdf" and source_path.is_file()


def _cap_pdf_ocr_worker_count(input_count: int, worker_count: int) -> int:
    capped = max(1, min(input_count, worker_count))
    max_worker_count = _parse_positive_int(os.environ.get(PDF_OCR_MAX_WORKERS_ENV, ""))
    if max_worker_count is not None:
        capped = min(capped, max_worker_count)
    return max(1, capped)


def _parse_positive_int(value: str | None) -> int | None:
    if not value:
        return None
    try:
        parsed = int(value)
    except ValueError:
        return None
    return parsed if parsed > 0 else None


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
    "PDF_OCR_FAST_TEXT_PROFILE",
    "PDF_OCR_MAX_WORKERS_ENV",
    "PDF_OCR_SHARD_INPUT_SCHEMA",
    "PDF_OCR_SHARD_INPUT_SCHEMA_VERSION",
    "PDF_OCR_SHARD_RESULT_SCHEMA",
    "PDF_OCR_SHARD_RESULT_SCHEMA_VERSION",
    "PDF_OCR_WORKERS_ENV",
    "DoclingPdfOcrShardWorker",
    "PdfOcrShardWorkerProtocol",
    "SkippingPdfOcrShardWorker",
    "build_pdf_ocr_shard_result_table",
    "failed_pdf_ocr_shard_result",
    "normalize_pdf_ocr_shard_result",
    "resolve_pdf_ocr_worker_count",
    "skipped_pdf_ocr_shard_result",
    "succeeded_pdf_ocr_shard_result",
    "validate_pdf_ocr_shard_input_table",
]
