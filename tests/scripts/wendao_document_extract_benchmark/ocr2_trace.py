"""Hosted VLM/OCR request trace aggregation for benchmark reports."""

from __future__ import annotations

import math

from .common import Any, Path, json

TRACE_FILE_GLOB = "*.hosted-vlm-ocr.jsonl"


def summarize_hosted_vlm_ocr_request_traces(log_dir: Path | None) -> dict[str, Any]:
    if log_dir is None or not log_dir.exists():
        return _empty_trace_summary()

    trace_files = sorted(log_dir.glob(TRACE_FILE_GLOB))
    summary = _empty_trace_summary()
    summary["traceFileCount"] = len(trace_files)
    latencies = []
    slowest_requests = []
    started_unix_ms_values = []
    ended_unix_ms_values = []
    for trace_file in trace_files:
        for line in trace_file.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            try:
                record = json.loads(line)
            except json.JSONDecodeError:
                summary["parseErrorCount"] += 1
                continue
            if not isinstance(record, dict):
                summary["parseErrorCount"] += 1
                continue
            _accumulate_trace_record(
                summary,
                record,
                latencies,
                slowest_requests,
                started_unix_ms_values,
                ended_unix_ms_values,
            )

    summary["latencyMsP50"] = _percentile(latencies, 50)
    summary["latencyMsP95"] = _percentile(latencies, 95)
    summary["latencyMsMax"] = round(max(latencies), 3) if latencies else None
    summary["requestLatencyMsTotal"] = round(sum(latencies), 3) if latencies else 0.0
    summary["slowestRequests"] = sorted(
        slowest_requests,
        key=lambda item: item["latencyMs"],
        reverse=True,
    )[:5]
    if summary["requestCount"] > 0:
        summary["imageBytesPerRequestAvg"] = round(
            summary["imageBytesTotal"] / summary["requestCount"],
            3,
        )
        summary["sourcePixelAreaPerRequestAvg"] = round(
            summary["sourcePixelAreaTotal"] / summary["requestCount"],
            3,
        )
    if started_unix_ms_values and ended_unix_ms_values:
        started_unix_ms = min(started_unix_ms_values)
        ended_unix_ms = max(ended_unix_ms_values)
        wall_span_ms = max(0, ended_unix_ms - started_unix_ms)
        summary["requestWallStartUnixMs"] = started_unix_ms
        summary["requestWallEndUnixMs"] = ended_unix_ms
        summary["requestWallSpanMs"] = wall_span_ms
        if wall_span_ms > 0 and latencies:
            summary["requestLatencyOverlapRatio"] = round(
                sum(latencies) / wall_span_ms, 3
            )
    return summary


def _empty_trace_summary() -> dict[str, Any]:
    return {
        "traceFileCount": 0,
        "requestCount": 0,
        "httpAttemptCountTotal": 0,
        "successCount": 0,
        "failureCount": 0,
        "parseErrorCount": 0,
        "statusCounts": {},
        "httpStatusCounts": {},
        "modelCounts": {},
        "requestKindCounts": {},
        "hedgeWinnerCounts": {},
        "scaffoldModeCounts": {},
        "imageOptimizationModeCounts": {},
        "shardTypeCounts": {},
        "renderDpiCounts": {},
        "pageCountTotal": 0,
        "shardCountTotal": 0,
        "regionShardCount": 0,
        "pageShardCount": 0,
        "charCountTotal": 0,
        "scaffoldAppliedCount": 0,
        "scaffoldValidationFailureCount": 0,
        "scaffoldJsonCharCountTotal": 0,
        "canonicalMarkdownCharCountTotal": 0,
        "imageBytesTotal": 0,
        "imageBytesMax": 0,
        "imageBytesPerRequestAvg": 0.0,
        "sourcePixelAreaTotal": 0,
        "sourcePixelAreaMax": 0,
        "sourcePixelAreaPerRequestAvg": 0.0,
        "latencyMsP50": None,
        "latencyMsP95": None,
        "latencyMsMax": None,
        "requestLatencyMsTotal": 0.0,
        "slowestRequests": [],
        "requestWallStartUnixMs": None,
        "requestWallEndUnixMs": None,
        "requestWallSpanMs": None,
        "requestLatencyOverlapRatio": None,
    }


def _accumulate_trace_record(
    summary: dict[str, Any],
    record: dict[str, Any],
    latencies: list[float],
    slowest_requests: list[dict[str, Any]],
    started_unix_ms_values: list[int],
    ended_unix_ms_values: list[int],
) -> None:
    summary["requestCount"] += 1
    http_attempt_count = record.get("httpAttemptCount")
    summary["httpAttemptCountTotal"] += (
        http_attempt_count if isinstance(http_attempt_count, int) else 1
    )
    status = _string_value(record.get("status"), "unknown")
    _increment(summary["statusCounts"], status)
    if status == "succeeded":
        summary["successCount"] += 1
    else:
        summary["failureCount"] += 1

    http_status = record.get("httpStatus")
    if isinstance(http_status, int):
        _increment(summary["httpStatusCounts"], str(http_status))

    model = record.get("model")
    if isinstance(model, str) and model:
        _increment(summary["modelCounts"], model)

    request_kind = record.get("requestKind")
    if isinstance(request_kind, str) and request_kind:
        _increment(summary["requestKindCounts"], request_kind)

    hedge_winner = record.get("hedgeWinner")
    if isinstance(hedge_winner, str) and hedge_winner:
        _increment(summary["hedgeWinnerCounts"], hedge_winner)

    scaffold_mode = record.get("scaffoldMode")
    if isinstance(scaffold_mode, str) and scaffold_mode:
        _increment(summary["scaffoldModeCounts"], scaffold_mode)

    image_optimization_mode = record.get("imageOptimizationMode")
    if isinstance(image_optimization_mode, str) and image_optimization_mode:
        _increment(
            summary["imageOptimizationModeCounts"],
            image_optimization_mode,
        )

    render_dpi = record.get("renderDpi")
    if isinstance(render_dpi, int):
        _increment(summary["renderDpiCounts"], str(render_dpi))

    shard_type_counts = record.get("shardTypeCounts")
    if isinstance(shard_type_counts, dict):
        for shard_type, count in shard_type_counts.items():
            if isinstance(shard_type, str) and isinstance(count, int):
                summary["shardTypeCounts"][shard_type] = (
                    summary["shardTypeCounts"].get(shard_type, 0) + count
                )
                if shard_type == "region":
                    summary["regionShardCount"] += count
                elif shard_type == "page":
                    summary["pageShardCount"] += count
    else:
        shard_type = record.get("shardType")
        if isinstance(shard_type, str) and shard_type:
            _increment(summary["shardTypeCounts"], shard_type)
            if shard_type == "region":
                summary["regionShardCount"] += 1
            elif shard_type == "page":
                summary["pageShardCount"] += 1

    latency = record.get("latencyMs")
    if isinstance(latency, int | float):
        latencies.append(float(latency))
        slowest_requests.append(_slow_request_diagnostic(record, float(latency)))

    started_unix_ms = record.get("startedUnixMs")
    ended_unix_ms = record.get("endedUnixMs")
    if isinstance(started_unix_ms, int) and isinstance(ended_unix_ms, int):
        started_unix_ms_values.append(started_unix_ms)
        ended_unix_ms_values.append(ended_unix_ms)

    markdown_chars = record.get("markdownChars")
    if isinstance(markdown_chars, int):
        summary["charCountTotal"] += markdown_chars

    scaffold_applied_count = record.get("scaffoldAppliedCount")
    if isinstance(scaffold_applied_count, int):
        summary["scaffoldAppliedCount"] += scaffold_applied_count

    scaffold_validation_failure_count = record.get("scaffoldValidationFailureCount")
    if isinstance(scaffold_validation_failure_count, int):
        summary["scaffoldValidationFailureCount"] += scaffold_validation_failure_count

    scaffold_json_chars = record.get("scaffoldJsonChars")
    if isinstance(scaffold_json_chars, int):
        summary["scaffoldJsonCharCountTotal"] += scaffold_json_chars

    canonical_markdown_chars = record.get("canonicalMarkdownChars")
    if isinstance(canonical_markdown_chars, int):
        summary["canonicalMarkdownCharCountTotal"] += canonical_markdown_chars

    image_bytes = record.get("imageBytes")
    if isinstance(image_bytes, int):
        summary["imageBytesTotal"] += image_bytes
        summary["imageBytesMax"] = max(summary["imageBytesMax"], image_bytes)

    source_pixel_area = record.get("sourcePixelArea")
    if isinstance(source_pixel_area, int):
        summary["sourcePixelAreaTotal"] += source_pixel_area
        summary["sourcePixelAreaMax"] = max(
            summary["sourcePixelAreaMax"],
            source_pixel_area,
        )

    shard_count = record.get("shardCount")
    if isinstance(shard_count, int):
        summary["shardCountTotal"] += shard_count
    else:
        summary["shardCountTotal"] += 1

    page_count = record.get("pageCount")
    if isinstance(page_count, int):
        summary["pageCountTotal"] += page_count
    else:
        summary["pageCountTotal"] += 1


def _increment(counts: dict[str, int], key: str) -> None:
    counts[key] = counts.get(key, 0) + 1


def _slow_request_diagnostic(
    record: dict[str, Any], latency_ms: float
) -> dict[str, Any]:
    diagnostic = {
        "latencyMs": round(latency_ms, 3),
        "requestKind": _optional_string(record.get("requestKind")),
        "pageIndex": _optional_int(record.get("pageIndex")),
        "regionIndex": _optional_int(record.get("regionIndex")),
        "readingOrderKey": _optional_string(record.get("readingOrderKey")),
        "httpAttemptCount": _optional_int(record.get("httpAttemptCount")),
        "imageBytes": _optional_int(record.get("imageBytes")),
        "sourcePixelArea": _optional_int(record.get("sourcePixelArea")),
        "markdownChars": _optional_int(record.get("markdownChars")),
        "rasterWidthPx": _optional_int(record.get("rasterWidthPx")),
        "rasterHeightPx": _optional_int(record.get("rasterHeightPx")),
    }
    source_pixel_box = {
        "left": _optional_int(record.get("sourcePagePixelLeft")),
        "top": _optional_int(record.get("sourcePagePixelTop")),
        "right": _optional_int(record.get("sourcePagePixelRight")),
        "bottom": _optional_int(record.get("sourcePagePixelBottom")),
    }
    if any(value is not None for value in source_pixel_box.values()):
        diagnostic["sourcePixelBox"] = source_pixel_box
    hedge = {
        "hedgeWinner": _optional_string(record.get("hedgeWinner")),
        "hedgeDelaySeconds": _optional_float(record.get("hedgeDelaySeconds")),
        "hedgePrimaryLatencyMs": _optional_float(record.get("hedgePrimaryLatencyMs")),
        "hedgeSecondaryLatencyMs": _optional_float(
            record.get("hedgeSecondaryLatencyMs")
        ),
    }
    if any(value is not None for value in hedge.values()):
        diagnostic.update(hedge)
    return diagnostic


def _optional_string(value: object) -> str | None:
    if isinstance(value, str) and value:
        return value
    return None


def _optional_int(value: object) -> int | None:
    if isinstance(value, int):
        return value
    return None


def _optional_float(value: object) -> float | None:
    if isinstance(value, int | float):
        return round(float(value), 3)
    return None


def _string_value(value: object, default: str) -> str:
    if isinstance(value, str) and value:
        return value
    return default


def _percentile(values: list[float], percentile: int) -> float | None:
    if not values:
        return None
    ordered = sorted(values)
    index = math.ceil((percentile / 100.0) * len(ordered)) - 1
    index = min(max(index, 0), len(ordered) - 1)
    return round(ordered[index], 3)
