"""Audio shard result-row normalization helpers."""

from __future__ import annotations

import hashlib
from typing import TYPE_CHECKING, Any

from .audio_shard_contracts import AUDIO_SHARD_RESULT_SCHEMA_VERSION

if TYPE_CHECKING:
    from collections.abc import Mapping


def succeeded_audio_shard_result(
    input_row: Mapping[str, Any],
    text: str,
    confidence: float,
) -> dict[str, Any]:
    """Build one successful audio shard result row."""

    return _result_from_input(
        input_row,
        status="succeeded",
        text=text,
        confidence=confidence,
        error_message=None,
    )


def failed_audio_shard_result(
    input_row: Mapping[str, Any],
    error_message: str,
) -> dict[str, Any]:
    """Build one failed audio shard result row."""

    return _result_from_input(
        input_row,
        status="failed",
        text=None,
        confidence=None,
        error_message=error_message,
    )


def skipped_audio_shard_result(
    input_row: Mapping[str, Any],
    reason: str,
) -> dict[str, Any]:
    """Build one skipped audio shard result row."""

    return _result_from_input(
        input_row,
        status="skipped",
        text=None,
        confidence=None,
        error_message=reason,
    )


def normalize_audio_shard_result(
    input_row: Mapping[str, Any],
    result_row: Mapping[str, Any],
) -> dict[str, Any]:
    """Normalize one worker result to the stable audio result schema."""

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
            f"Unsupported audio shard result status: {result_row.get('status')}"
        )
    if row["status"] == "succeeded" and row["text"] is None:
        row["status"] = "failed"
        row["errorMessage"] = "audio shard result succeeded without text"
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
        "contractVersion": AUDIO_SHARD_RESULT_SCHEMA_VERSION,
        "sourcePath": input_row["sourcePath"],
        "sourceContentHash": input_row["sourceContentHash"],
        "shardPath": input_row["shardPath"],
        "shardSha256": input_row["shardSha256"],
        "shardProfile": input_row["shardProfile"],
        "taskProfile": input_row["taskProfile"],
        "backendProfile": input_row["backendProfile"],
        "status": status,
        "text": text,
        "textMimeType": "text/plain",
        "confidence": confidence,
        "errorMessage": error_message,
        "shardElementId": input_row["shardElementId"],
        "elementId": _audio_result_element_id(input_row),
    }


def _audio_result_element_id(input_row: Mapping[str, Any]) -> str:
    material = (
        f"{input_row['sourceContentHash']}:{input_row['shardSha256']}:"
        f"{input_row['shardProfile']}:{input_row['taskProfile']}:"
        f"{input_row['backendProfile']}:{input_row['shardElementId']}"
    )
    return hashlib.sha256(material.encode("utf-8")).hexdigest()
