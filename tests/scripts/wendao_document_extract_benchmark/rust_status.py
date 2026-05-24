"""Rust document extraction status summary helpers."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .common import Any


def summarize_rust_jobs_status_samples(samples: list[dict[str, Any]]) -> dict[str, Any]:
    if not samples:
        return {
            "sampleCount": 0,
            "maxQueuedJobs": None,
            "maxRunningJobs": None,
            "maxInProcessRunningConversions": None,
            "maxInProcessScheduledJobs": None,
            "minAvailableConversionPermits": None,
            "maxRunningConversions": None,
            "maxPdfOcrWorkers": None,
            "maxCurrentPdfOcrWorkerBudget": None,
            "minAvailablePdfOcrWorkerPermits": None,
            "maxInProcessPdfOcrWorkers": None,
            "maxInFlightPdfOcrShards": None,
            "maxPdfOcrCacheHits": None,
            "maxPdfOcrCacheMisses": None,
            "maxPdfOcrLiveRequests": None,
            "maxPdfOcrQueueWaitP95Ms": None,
            "maxPdfOcrLatencyP95Ms": None,
            "maxPdfOcrSourcePdfPageRangeShards": None,
            "maxPdfOcrRenderedPageShards": None,
            "maxPdfOcrRenderedRegionShards": None,
            "maxPdfOcrBudgetIncreaseEvents": None,
            "maxPdfOcrBudgetDecreaseEvents": None,
            "maxAudioShardWorkers": None,
            "maxCurrentAudioShardWorkerBudget": None,
            "maxAudioShardHealthyStreak": None,
            "maxAudioShardBudgetIncreaseEvents": None,
            "maxAudioShardBudgetDecreaseEvents": None,
            "lastConversionDurationMs": None,
            "maxConversionDurationMs": None,
        }
    return {
        "sampleCount": len(samples),
        "maxQueuedJobs": max_int_sample(samples, "queuedJobs"),
        "maxRunningJobs": max_int_sample(samples, "runningJobs"),
        "maxInProcessRunningConversions": max_int_sample(
            samples,
            "inProcessRunningConversions",
        ),
        "maxInProcessScheduledJobs": max_int_sample(samples, "inProcessScheduledJobs"),
        "minAvailableConversionPermits": min_int_sample(
            samples,
            "availableConversionPermits",
        ),
        "maxRunningConversions": max_int_sample(samples, "maxRunningConversions"),
        "maxPdfOcrWorkers": max_int_sample(samples, "maxPdfOcrWorkers"),
        "maxCurrentPdfOcrWorkerBudget": max_int_sample(
            samples,
            "currentPdfOcrWorkerBudget",
        ),
        "minAvailablePdfOcrWorkerPermits": min_int_sample(
            samples,
            "availablePdfOcrWorkerPermits",
        ),
        "maxInProcessPdfOcrWorkers": max_int_sample(samples, "inProcessPdfOcrWorkers"),
        "maxInFlightPdfOcrShards": max_int_sample(samples, "inFlightPdfOcrShards"),
        "maxPdfOcrCacheHits": max_int_sample(samples, "pdfOcrCacheHits"),
        "maxPdfOcrCacheMisses": max_int_sample(samples, "pdfOcrCacheMisses"),
        "maxPdfOcrLiveRequests": max_int_sample(samples, "pdfOcrLiveRequests"),
        "maxPdfOcrQueueWaitP95Ms": max_int_sample(samples, "pdfOcrQueueWaitP95Ms"),
        "maxPdfOcrLatencyP95Ms": max_int_sample(samples, "pdfOcrLatencyP95Ms"),
        "maxPdfOcrSourcePdfPageRangeShards": max_int_sample(
            samples,
            "pdfOcrSourcePdfPageRangeShards",
        ),
        "maxPdfOcrRenderedPageShards": max_int_sample(
            samples,
            "pdfOcrRenderedPageShards",
        ),
        "maxPdfOcrRenderedRegionShards": max_int_sample(
            samples,
            "pdfOcrRenderedRegionShards",
        ),
        "maxPdfOcrBudgetIncreaseEvents": max_int_sample(
            samples,
            "pdfOcrBudgetIncreaseEvents",
        ),
        "maxPdfOcrBudgetDecreaseEvents": max_int_sample(
            samples,
            "pdfOcrBudgetDecreaseEvents",
        ),
        "maxAudioShardWorkers": max_int_sample(samples, "maxAudioShardWorkers"),
        "maxCurrentAudioShardWorkerBudget": max_int_sample(
            samples,
            "currentAudioShardWorkerBudget",
        ),
        "maxAudioShardHealthyStreak": max_int_sample(
            samples,
            "audioShardHealthyStreak",
        ),
        "maxAudioShardBudgetIncreaseEvents": max_int_sample(
            samples,
            "audioShardBudgetIncreaseEvents",
        ),
        "maxAudioShardBudgetDecreaseEvents": max_int_sample(
            samples,
            "audioShardBudgetDecreaseEvents",
        ),
        "lastConversionDurationMs": last_present_sample(
            samples,
            "lastConversionDurationMs",
        ),
        "maxConversionDurationMs": max_int_sample(samples, "maxConversionDurationMs"),
    }


def max_int_sample(samples: list[dict[str, Any]], key: str) -> int | None:
    values = [value for sample in samples if isinstance((value := sample.get(key)), int)]
    return max(values, default=None)


def min_int_sample(samples: list[dict[str, Any]], key: str) -> int | None:
    values = [value for sample in samples if isinstance((value := sample.get(key)), int)]
    return min(values, default=None)


def last_present_sample(samples: list[dict[str, Any]], key: str) -> Any:
    for sample in reversed(samples):
        value = sample.get(key)
        if value is not None:
            return value
    return None


def combine_rust_jobs_status_summaries(
    summaries: list[dict[str, Any]],
) -> dict[str, Any]:
    samples = [summary for summary in summaries if summary and summary.get("sampleCount", 0) > 0]
    if not samples:
        return summarize_rust_jobs_status_samples([])
    return {
        "sampleCount": sum_int_values(samples, "sampleCount"),
        "maxQueuedJobs": max_optional_int(samples, "maxQueuedJobs"),
        "maxRunningJobs": max_optional_int(samples, "maxRunningJobs"),
        "maxInProcessRunningConversions": max_optional_int(
            samples,
            "maxInProcessRunningConversions",
        ),
        "maxInProcessScheduledJobs": max_optional_int(
            samples,
            "maxInProcessScheduledJobs",
        ),
        "minAvailableConversionPermits": min_optional_int(
            samples,
            "minAvailableConversionPermits",
        ),
        "maxRunningConversions": max_optional_int(samples, "maxRunningConversions"),
        "maxPdfOcrWorkers": max_optional_int(samples, "maxPdfOcrWorkers"),
        "maxCurrentPdfOcrWorkerBudget": max_optional_int(
            samples,
            "maxCurrentPdfOcrWorkerBudget",
        ),
        "minAvailablePdfOcrWorkerPermits": min_optional_int(
            samples,
            "minAvailablePdfOcrWorkerPermits",
        ),
        "maxInProcessPdfOcrWorkers": max_optional_int(
            samples,
            "maxInProcessPdfOcrWorkers",
        ),
        "maxInFlightPdfOcrShards": max_optional_int(samples, "maxInFlightPdfOcrShards"),
        "maxPdfOcrCacheHits": max_optional_int(samples, "maxPdfOcrCacheHits"),
        "maxPdfOcrCacheMisses": max_optional_int(samples, "maxPdfOcrCacheMisses"),
        "maxPdfOcrLiveRequests": max_optional_int(samples, "maxPdfOcrLiveRequests"),
        "maxPdfOcrQueueWaitP95Ms": max_optional_int(
            samples,
            "maxPdfOcrQueueWaitP95Ms",
        ),
        "maxPdfOcrLatencyP95Ms": max_optional_int(samples, "maxPdfOcrLatencyP95Ms"),
        "maxPdfOcrSourcePdfPageRangeShards": max_optional_int(
            samples,
            "maxPdfOcrSourcePdfPageRangeShards",
        ),
        "maxPdfOcrRenderedPageShards": max_optional_int(
            samples,
            "maxPdfOcrRenderedPageShards",
        ),
        "maxPdfOcrRenderedRegionShards": max_optional_int(
            samples,
            "maxPdfOcrRenderedRegionShards",
        ),
        "maxPdfOcrBudgetIncreaseEvents": max_optional_int(
            samples,
            "maxPdfOcrBudgetIncreaseEvents",
        ),
        "maxPdfOcrBudgetDecreaseEvents": max_optional_int(
            samples,
            "maxPdfOcrBudgetDecreaseEvents",
        ),
        "maxAudioShardWorkers": max_optional_int(samples, "maxAudioShardWorkers"),
        "maxCurrentAudioShardWorkerBudget": max_optional_int(
            samples,
            "maxCurrentAudioShardWorkerBudget",
        ),
        "maxAudioShardHealthyStreak": max_optional_int(
            samples,
            "maxAudioShardHealthyStreak",
        ),
        "maxAudioShardBudgetIncreaseEvents": max_optional_int(
            samples,
            "maxAudioShardBudgetIncreaseEvents",
        ),
        "maxAudioShardBudgetDecreaseEvents": max_optional_int(
            samples,
            "maxAudioShardBudgetDecreaseEvents",
        ),
        "lastConversionDurationMs": last_present_sample(
            samples,
            "lastConversionDurationMs",
        ),
        "maxConversionDurationMs": max_optional_int(samples, "maxConversionDurationMs"),
    }


def sum_int_values(items: list[dict[str, Any]], key: str) -> int:
    return sum(value for item in items if isinstance((value := item.get(key)), int))


def max_optional_int(items: list[dict[str, Any]], key: str) -> int | None:
    values = [value for item in items if isinstance((value := item.get(key)), int)]
    return max(values, default=None)


def min_optional_int(items: list[dict[str, Any]], key: str) -> int | None:
    values = [value for item in items if isinstance((value := item.get(key)), int)]
    return min(values, default=None)
