"""PDF OCR Arrow table entrypoints."""

from __future__ import annotations

import os

import pyarrow as pa

from .pdf_ocr_contracts import (
    PDF_OCR_MAX_WORKERS_ENV,
    PDF_OCR_SHARD_INPUT_SCHEMA,
    PDF_OCR_SHARD_INPUT_SCHEMA_VERSION,
    PDF_OCR_SHARD_RESULT_SCHEMA,
    PDF_OCR_WORKERS_ENV,
    PdfOcrShardWorkerProtocol,
)
from .pdf_ocr_results import normalize_pdf_ocr_shard_result


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
    if worker is None:
        from .pdf_ocr_workers import SkippingPdfOcrShardWorker

        effective_worker = SkippingPdfOcrShardWorker()
    else:
        effective_worker = worker
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
