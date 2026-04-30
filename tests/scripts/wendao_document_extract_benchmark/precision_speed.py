"""Precision and speed observation helpers for document extraction reports."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .common import Any


def precision_speed_summary(
    results: list[dict[str, Any]],
    distinct_miss_report: dict[str, Any] | None,
    *,
    total_error_rows: int,
    artifact_error_count: int,
    structure_parity_error_count: int,
    structure_reading_order_sorted: bool | None,
    structure_order_stable: bool | None,
    structure_order_mismatch_count: int,
    structure_parity_passed: bool | None,
) -> dict[str, Any]:
    precision_gate_passed = (
        total_error_rows == 0
        and artifact_error_count == 0
        and structure_parity_error_count == 0
        and structure_reading_order_sorted is not False
        and structure_order_stable is not False
        and structure_order_mismatch_count == 0
        and structure_parity_passed is not False
    )
    return {
        "precisionGatePassed": precision_gate_passed,
        "errorRows": total_error_rows,
        "artifactErrors": artifact_error_count,
        "structureReadingOrderSorted": structure_reading_order_sorted,
        "structureOrderStable": structure_order_stable,
        "structureOrderMismatches": structure_order_mismatch_count,
        "structureParityPassed": structure_parity_passed,
        "structureParityErrors": structure_parity_error_count,
        "structureRows": sum(result.get("structureRows", 0) for result in results),
        "ocrPageBlocks": sum(
            result.get("structureOcrPageBlocks", 0) for result in results
        ),
        "ocrRegionBlocks": sum(
            result.get("structureOcrRegionBlocks", 0) for result in results
        ),
        "bboxBlocks": sum(result.get("structureBboxBlocks", 0) for result in results),
        "metricsRows": sum(result.get("metricsRows", 0) for result in results),
        "metricsResultChars": sum(
            result.get("metricsResultChars", 0) for result in results
        ),
        **speed_observation_summary(results, distinct_miss_report),
    }


def all_structure_reading_order_sorted(results: list[dict[str, Any]]) -> bool | None:
    values = [
        result.get("structureReadingOrderSorted")
        for result in results
        if result.get("structureReadingOrderSorted") is not None
    ]
    return all(bool(value) for value in values) if values else None


def all_structure_parity_passed(results: list[dict[str, Any]]) -> bool | None:
    values = [
        result.get("structureParityPassed")
        for result in results
        if result.get("structureParityPassed") is not None
    ]
    return all(bool(value) for value in values) if values else None


def all_structure_order_stable(results: list[dict[str, Any]]) -> bool | None:
    values = [
        result.get("structureOrderStable")
        for result in results
        if result.get("structureOrderStable") is not None
    ]
    return all(bool(value) for value in values) if values else None


def structure_order_mismatch_count(results: list[dict[str, Any]]) -> int:
    return sum(
        int(value)
        for result in results
        if isinstance((value := result.get("structureOrderMismatchCount")), int)
    )


def speed_observation_summary(
    results: list[dict[str, Any]],
    distinct_miss_report: dict[str, Any] | None = None,
) -> dict[str, Any]:
    return {
        "maxForceRefreshMs": max_numeric(results, "forceRefreshMs"),
        "maxCacheHitP95Ms": max_numeric(results, "cacheHitP95Ms"),
        "maxShardCacheReuseForceMs": max_numeric(results, "shardCacheReuseForceMs"),
        "maxWallTimeMs": max_numeric(results, "wallTimeMs"),
        "minCacheSpeedup": min_numeric(results, "cacheSpeedup"),
        "totalRustSchedulerElapsedMs": sum_numeric(
            results,
            "metricsRustSchedulerElapsedMs",
        ),
        "distinctMissWallTimeMs": (
            distinct_miss_report.get("wallTimeMs") if distinct_miss_report else None
        ),
    }


def max_numeric(results: list[dict[str, Any]], key: str) -> float | None:
    values = numeric_values(results, key)
    return max(values) if values else None


def min_numeric(results: list[dict[str, Any]], key: str) -> float | None:
    values = numeric_values(results, key)
    return min(values) if values else None


def sum_numeric(results: list[dict[str, Any]], key: str) -> float:
    return sum(numeric_values(results, key))


def numeric_values(results: list[dict[str, Any]], key: str) -> list[float]:
    return [
        float(value)
        for result in results
        if isinstance((value := result.get(key)), int | float)
    ]
