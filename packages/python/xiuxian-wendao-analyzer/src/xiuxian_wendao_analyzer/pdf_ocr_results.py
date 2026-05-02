"""PDF OCR result-row normalization helpers."""

from __future__ import annotations

import hashlib
from typing import TYPE_CHECKING, Any

from .pdf_ocr_contracts import PDF_OCR_SHARD_RESULT_SCHEMA_VERSION

if TYPE_CHECKING:
    from collections.abc import Mapping


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
