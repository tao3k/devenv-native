"""Hosted VLM/OCR request trace writer."""

from __future__ import annotations

import json
import threading
import time
from typing import TYPE_CHECKING, Any

from .http import short_error_message

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence
    from pathlib import Path

_HOSTED_VLM_OCR_TRACE_LOCK = threading.Lock()


def write_trace_record(
    *,
    trace_path: Path | None,
    input_row: Mapping[str, Any],
    model: str,
    completion_url: str,
    scaffold_mode: str,
    image_optimization_mode: str,
    default_max_tokens: int,
    status: str,
    started: float,
    http_status: int | None,
    image_bytes: int,
    markdown_chars: int,
    error: BaseException | None,
    page_count: int = 1,
    input_rows: Sequence[Mapping[str, Any]] | None = None,
    max_tokens: int | None = None,
    request_kind: str | None = None,
    http_attempt_count: int = 1,
    scaffold_applied_count: int = 0,
    scaffold_validation_failure_count: int = 0,
    scaffold_json_chars: int = 0,
    canonical_markdown_chars: int = 0,
    hedge_winner: str | None = None,
    hedge_delay_seconds: float | None = None,
    hedge_primary_latency_ms: float | None = None,
    hedge_secondary_latency_ms: float | None = None,
) -> None:
    if trace_path is None:
        return
    rows = list(input_rows or [input_row])
    ended_unix_ms = int(time.time() * 1000)
    latency_ms = round((time.perf_counter() - started) * 1000.0, 3)
    started_unix_ms = max(0, ended_unix_ms - round(latency_ms))
    record = {
        "schema": "xiuxian_wendao.hosted_vlm_ocr_request_trace.v1",
        "timestampUnixMs": ended_unix_ms,
        "startedUnixMs": started_unix_ms,
        "endedUnixMs": ended_unix_ms,
        "status": status,
        "httpStatus": http_status,
        "latencyMs": latency_ms,
        "model": model,
        "endpoint": completion_url,
        "pageIndex": input_row.get("pageIndex"),
        "shardElementId": input_row.get("shardElementId"),
        "shardType": input_row.get("shardType"),
        "regionIndex": input_row.get("regionIndex"),
        "parentShardElementId": input_row.get("parentShardElementId"),
        "readingOrderKey": input_row.get("readingOrderKey"),
        "ocrProfile": input_row.get("ocrProfile"),
        "requestKind": request_kind or ocr2_trace_request_kind(input_row, page_count),
        "httpAttemptCount": max(1, http_attempt_count),
        "shardCount": len(rows),
        "shardTypeCounts": ocr2_trace_shard_type_counts(rows),
        "pageCount": page_count,
        "imageBytes": image_bytes,
        "sourcePixelArea": ocr2_trace_source_pixel_area(rows),
        "renderDpi": input_row.get("renderDpi"),
        "rasterWidthPx": input_row.get("rasterWidthPx"),
        "rasterHeightPx": input_row.get("rasterHeightPx"),
        "sourcePagePixelLeft": input_row.get("sourcePagePixelLeft"),
        "sourcePagePixelTop": input_row.get("sourcePagePixelTop"),
        "sourcePagePixelRight": input_row.get("sourcePagePixelRight"),
        "sourcePagePixelBottom": input_row.get("sourcePagePixelBottom"),
        "markdownChars": markdown_chars,
        "maxTokens": max_tokens if max_tokens is not None else default_max_tokens,
        "scaffoldMode": scaffold_mode,
        "imageOptimizationMode": image_optimization_mode,
        "scaffoldAppliedCount": scaffold_applied_count,
        "scaffoldValidationFailureCount": scaffold_validation_failure_count,
        "scaffoldJsonChars": scaffold_json_chars,
        "canonicalMarkdownChars": canonical_markdown_chars,
        "errorType": type(error).__name__ if error is not None else None,
        "errorMessage": short_error_message(error),
    }
    if hedge_winner:
        record["hedgeWinner"] = hedge_winner
    if hedge_delay_seconds is not None:
        record["hedgeDelaySeconds"] = hedge_delay_seconds
    if hedge_primary_latency_ms is not None:
        record["hedgePrimaryLatencyMs"] = hedge_primary_latency_ms
    if hedge_secondary_latency_ms is not None:
        record["hedgeSecondaryLatencyMs"] = hedge_secondary_latency_ms
    try:
        with _HOSTED_VLM_OCR_TRACE_LOCK:
            trace_path.parent.mkdir(parents=True, exist_ok=True)
            with trace_path.open("a", encoding="utf-8") as handle:
                handle.write(json.dumps(record, sort_keys=True))
                handle.write("\n")
    except OSError:
        return


def ocr2_trace_request_kind(input_row: Mapping[str, Any], page_count: int) -> str:
    if page_count > 1:
        return "page-window-canary"
    shard_type = str(input_row.get("shardType") or "")
    if shard_type == "region":
        return "region"
    return "page"


def ocr2_trace_shard_type_counts(
    input_rows: Sequence[Mapping[str, Any]],
) -> dict[str, int]:
    counts: dict[str, int] = {}
    for row in input_rows:
        shard_type = str(row.get("shardType") or "unknown")
        counts[shard_type] = counts.get(shard_type, 0) + 1
    return counts


def ocr2_trace_source_pixel_area(input_rows: Sequence[Mapping[str, Any]]) -> int:
    return sum(row_source_pixel_area(row) for row in input_rows)


def row_source_pixel_area(input_row: Mapping[str, Any]) -> int:
    try:
        left = int(input_row.get("sourcePagePixelLeft") or 0)
        top = int(input_row.get("sourcePagePixelTop") or 0)
        right = int(input_row.get("sourcePagePixelRight") or 0)
        bottom = int(input_row.get("sourcePagePixelBottom") or 0)
    except (TypeError, ValueError):
        return 0
    return max(0, right - left) * max(0, bottom - top)
