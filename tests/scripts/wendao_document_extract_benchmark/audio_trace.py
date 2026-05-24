"""Hosted audio request trace aggregation for benchmark reports."""

from __future__ import annotations

import math

from .common import Any, Path, json

TRACE_FILE_GLOB = "*.hosted-audio.jsonl"


def summarize_hosted_audio_request_traces(log_dir: Path | None) -> dict[str, Any]:
    if log_dir is None or not log_dir.exists():
        return _empty_trace_summary()

    trace_files = sorted(log_dir.glob(TRACE_FILE_GLOB))
    summary = _empty_trace_summary()
    summary["traceFileCount"] = len(trace_files)
    latencies = []
    slowest_requests = []
    started_unix_ms_values = []
    ended_unix_ms_values = []
    shard_element_id_counts: dict[str, int] = {}
    media_start_ms_counts: dict[str, int] = {}
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
                shard_element_id_counts,
                media_start_ms_counts,
            )
    _record_identity_counts(summary, "shardElementId", shard_element_id_counts)
    _record_identity_counts(summary, "mediaStartMs", media_start_ms_counts)
    summary["latencyMsP50"] = _percentile(latencies, 50)
    summary["latencyMsP95"] = _percentile(latencies, 95)
    summary["latencyMsMax"] = round(max(latencies), 3) if latencies else None
    summary["requestLatencyMsTotal"] = round(sum(latencies), 3) if latencies else 0.0
    summary["slowestRequests"] = sorted(
        slowest_requests,
        key=lambda item: item["latencyMs"],
        reverse=True,
    )[:5]
    if started_unix_ms_values and ended_unix_ms_values:
        started_unix_ms = min(started_unix_ms_values)
        ended_unix_ms = max(ended_unix_ms_values)
        wall_span_ms = max(0, ended_unix_ms - started_unix_ms)
        summary["requestWallStartUnixMs"] = started_unix_ms
        summary["requestWallEndUnixMs"] = ended_unix_ms
        summary["requestWallSpanMs"] = wall_span_ms
        if wall_span_ms > 0 and latencies:
            summary["requestLatencyOverlapRatio"] = round(
                sum(latencies) / wall_span_ms,
                3,
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
        "providerCounts": {},
        "modelCounts": {},
        "endpointKindCounts": {},
        "requestKindCounts": {},
        "backendProfileCounts": {},
        "shardProfileCounts": {},
        "audioFormatCounts": {},
        "sampleRateHzCounts": {},
        "channelCounts": {},
        "uniqueShardElementIdCount": 0,
        "duplicateShardElementIdCount": 0,
        "duplicateShardElementIdExtraCount": 0,
        "duplicateShardElementIdCounts": {},
        "uniqueMediaStartMsCount": 0,
        "duplicateMediaStartMsCount": 0,
        "duplicateMediaStartMsExtraCount": 0,
        "duplicateMediaStartMsCounts": {},
        "textCharCountTotal": 0,
        "durationMsTotal": 0,
        "mediaDurationMsTotal": 0,
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
    shard_element_id_counts: dict[str, int],
    media_start_ms_counts: dict[str, int],
) -> None:
    summary["requestCount"] += 1
    summary["httpAttemptCountTotal"] += _int_or_default(
        record.get("httpAttemptCount"),
        1,
    )
    status = _string_value(record.get("status"), "unknown")
    _increment(summary["statusCounts"], status)
    if status == "succeeded":
        summary["successCount"] += 1
    else:
        summary["failureCount"] += 1
    _increment_optional_string(summary["providerCounts"], record.get("provider"))
    _increment_optional_string(summary["modelCounts"], record.get("model"))
    _increment_optional_string(summary["endpointKindCounts"], record.get("endpointKind"))
    _increment_optional_string(summary["requestKindCounts"], record.get("requestKind"))
    _increment_optional_string(
        summary["backendProfileCounts"],
        record.get("backendProfile"),
    )
    _increment_optional_string(summary["shardProfileCounts"], record.get("shardProfile"))
    _increment_optional_string(summary["audioFormatCounts"], record.get("audioFormat"))
    _increment_optional_int(summary["sampleRateHzCounts"], record.get("sampleRateHz"))
    _increment_optional_int(summary["channelCounts"], record.get("channels"))
    _increment_optional_string(shard_element_id_counts, record.get("shardElementId"))
    _increment_optional_int(media_start_ms_counts, record.get("mediaStartMs"))
    http_status = record.get("httpStatus")
    if isinstance(http_status, int):
        _increment(summary["httpStatusCounts"], str(http_status))
    _sum_int(summary, "textCharCountTotal", record.get("textChars"))
    _sum_int(summary, "durationMsTotal", record.get("durationMs"))
    _sum_int(summary, "mediaDurationMsTotal", record.get("mediaDurationMs"))

    latency = record.get("latencyMs")
    if isinstance(latency, int | float):
        latency_value = float(latency)
        latencies.append(latency_value)
        slowest_requests.append(_slow_request_diagnostic(record, latency_value))

    started_unix_ms = record.get("startedUnixMs")
    ended_unix_ms = record.get("endedUnixMs")
    if isinstance(started_unix_ms, int) and isinstance(ended_unix_ms, int):
        started_unix_ms_values.append(started_unix_ms)
        ended_unix_ms_values.append(ended_unix_ms)


def _slow_request_diagnostic(
    record: dict[str, Any],
    latency_ms: float,
) -> dict[str, Any]:
    return {
        "latencyMs": round(latency_ms, 3),
        "requestKind": _optional_string(record.get("requestKind")),
        "shardElementId": _optional_string(record.get("shardElementId")),
        "shardProfile": _optional_string(record.get("shardProfile")),
        "readingOrderKey": _optional_string(record.get("readingOrderKey")),
        "httpAttemptCount": _optional_int(record.get("httpAttemptCount")),
        "durationMs": _optional_int(record.get("durationMs")),
        "mediaStartMs": _optional_int(record.get("mediaStartMs")),
        "mediaDurationMs": _optional_int(record.get("mediaDurationMs")),
        "textChars": _optional_int(record.get("textChars")),
        "endpointKind": _optional_string(record.get("endpointKind")),
        "audioFormat": _optional_string(record.get("audioFormat")),
    }


def _increment(counts: dict[str, int], key: str) -> None:
    counts[key] = counts.get(key, 0) + 1


def _increment_optional_string(counts: dict[str, int], value: object) -> None:
    if isinstance(value, str) and value:
        _increment(counts, value)


def _increment_optional_int(counts: dict[str, int], value: object) -> None:
    if isinstance(value, int):
        _increment(counts, str(value))


def _record_identity_counts(
    summary: dict[str, Any],
    label: str,
    counts: dict[str, int],
) -> None:
    duplicate_counts = {key: count for key, count in sorted(counts.items()) if count > 1}
    summary[f"unique{label[0].upper()}{label[1:]}Count"] = len(counts)
    summary[f"duplicate{label[0].upper()}{label[1:]}Count"] = len(duplicate_counts)
    summary[f"duplicate{label[0].upper()}{label[1:]}ExtraCount"] = sum(
        count - 1 for count in duplicate_counts.values()
    )
    summary[f"duplicate{label[0].upper()}{label[1:]}Counts"] = duplicate_counts


def _sum_int(summary: dict[str, Any], key: str, value: object) -> None:
    if isinstance(value, int):
        summary[key] += value


def _int_or_default(value: object, default: int) -> int:
    return value if isinstance(value, int) else default


def _optional_string(value: object) -> str | None:
    if isinstance(value, str) and value:
        return value
    return None


def _optional_int(value: object) -> int | None:
    if isinstance(value, int):
        return value
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
