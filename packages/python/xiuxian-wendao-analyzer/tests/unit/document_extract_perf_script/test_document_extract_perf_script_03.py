"""document_extract_perf_script test slice 3."""

from __future__ import annotations

from .support import (
    _load_benchmark_module,
)


def test_rows_per_second_uses_wall_clock_time() -> None:
    benchmark = _load_benchmark_module()

    assert benchmark.rows_per_second(40, 200.0) == 200.0


def test_rust_jobs_status_summary_tracks_pressure() -> None:
    benchmark = _load_benchmark_module()

    summary = benchmark.summarize_rust_jobs_status_samples(
        [
            {
                "queuedJobs": 3,
                "runningJobs": 1,
                "inProcessRunningConversions": 1,
                "inProcessScheduledJobs": 4,
                "availableConversionPermits": 2,
                "maxRunningConversions": 4,
                "maxPdfOcrWorkers": 8,
                "currentPdfOcrWorkerBudget": 3,
                "availablePdfOcrWorkerPermits": 6,
                "inProcessPdfOcrWorkers": 2,
                "inFlightPdfOcrShards": 5,
                "pdfOcrCacheHits": 10,
                "pdfOcrCacheMisses": 4,
                "pdfOcrLiveRequests": 1,
                "pdfOcrQueueWaitP95Ms": 7,
                "pdfOcrLatencyP95Ms": 80,
                "pdfOcrSourcePdfPageRangeShards": 6,
                "pdfOcrRenderedPageShards": 2,
                "pdfOcrRenderedRegionShards": 1,
                "pdfOcrBudgetIncreaseEvents": 2,
                "pdfOcrBudgetDecreaseEvents": 0,
                "maxAudioShardWorkers": 8,
                "currentAudioShardWorkerBudget": 3,
                "audioShardHealthyStreak": 1,
                "audioShardBudgetIncreaseEvents": 2,
                "audioShardBudgetDecreaseEvents": 0,
            },
            {
                "queuedJobs": 1,
                "runningJobs": 2,
                "inProcessRunningConversions": 2,
                "inProcessScheduledJobs": 2,
                "availableConversionPermits": 1,
                "maxRunningConversions": 4,
                "maxPdfOcrWorkers": 8,
                "currentPdfOcrWorkerBudget": 4,
                "availablePdfOcrWorkerPermits": 5,
                "inProcessPdfOcrWorkers": 3,
                "inFlightPdfOcrShards": 2,
                "pdfOcrCacheHits": 12,
                "pdfOcrCacheMisses": 8,
                "pdfOcrLiveRequests": 2,
                "pdfOcrQueueWaitP95Ms": 9,
                "pdfOcrLatencyP95Ms": 120,
                "pdfOcrSourcePdfPageRangeShards": 8,
                "pdfOcrRenderedPageShards": 2,
                "pdfOcrRenderedRegionShards": 3,
                "pdfOcrBudgetIncreaseEvents": 3,
                "pdfOcrBudgetDecreaseEvents": 1,
                "maxAudioShardWorkers": 8,
                "currentAudioShardWorkerBudget": 4,
                "audioShardHealthyStreak": 2,
                "audioShardBudgetIncreaseEvents": 3,
                "audioShardBudgetDecreaseEvents": 1,
                "lastConversionDurationMs": 120,
                "maxConversionDurationMs": 300,
            },
        ]
    )

    assert summary["sampleCount"] == 2
    assert summary["maxQueuedJobs"] == 3
    assert summary["maxRunningJobs"] == 2
    assert summary["maxInProcessRunningConversions"] == 2
    assert summary["minAvailableConversionPermits"] == 1
    assert summary["maxPdfOcrWorkers"] == 8
    assert summary["maxCurrentPdfOcrWorkerBudget"] == 4
    assert summary["minAvailablePdfOcrWorkerPermits"] == 5
    assert summary["maxInFlightPdfOcrShards"] == 5
    assert summary["maxPdfOcrCacheHits"] == 12
    assert summary["maxPdfOcrCacheMisses"] == 8
    assert summary["maxPdfOcrLiveRequests"] == 2
    assert summary["maxPdfOcrLatencyP95Ms"] == 120
    assert summary["maxPdfOcrSourcePdfPageRangeShards"] == 8
    assert summary["maxPdfOcrRenderedRegionShards"] == 3
    assert summary["maxPdfOcrBudgetDecreaseEvents"] == 1
    assert summary["maxAudioShardWorkers"] == 8
    assert summary["maxCurrentAudioShardWorkerBudget"] == 4
    assert summary["maxAudioShardHealthyStreak"] == 2
    assert summary["maxAudioShardBudgetIncreaseEvents"] == 3
    assert summary["maxAudioShardBudgetDecreaseEvents"] == 1
    assert summary["lastConversionDurationMs"] == 120
    assert summary["maxConversionDurationMs"] == 300


def test_rust_jobs_status_summary_combines_fixture_phases() -> None:
    benchmark = _load_benchmark_module()

    combined = benchmark.combine_rust_jobs_status_summaries(
        [
            {
                "sampleCount": 2,
                "maxQueuedJobs": 4,
                "maxRunningJobs": 1,
                "maxInProcessRunningConversions": 1,
                "maxInProcessScheduledJobs": 4,
                "minAvailableConversionPermits": 3,
                "maxRunningConversions": 4,
                "maxPdfOcrWorkers": 8,
                "maxCurrentPdfOcrWorkerBudget": 3,
                "minAvailablePdfOcrWorkerPermits": 6,
                "maxInProcessPdfOcrWorkers": 2,
                "maxInFlightPdfOcrShards": 4,
                "maxPdfOcrCacheHits": 10,
                "maxPdfOcrCacheMisses": 5,
                "maxPdfOcrLiveRequests": 2,
                "maxPdfOcrQueueWaitP95Ms": 20,
                "maxPdfOcrLatencyP95Ms": 300,
                "maxPdfOcrSourcePdfPageRangeShards": 7,
                "maxPdfOcrRenderedPageShards": 2,
                "maxPdfOcrRenderedRegionShards": 1,
                "maxPdfOcrBudgetIncreaseEvents": 1,
                "maxPdfOcrBudgetDecreaseEvents": 0,
                "lastConversionDurationMs": None,
                "maxConversionDurationMs": None,
            },
            {
                "sampleCount": 1,
                "maxQueuedJobs": 0,
                "maxRunningJobs": 2,
                "maxInProcessRunningConversions": 2,
                "maxInProcessScheduledJobs": 2,
                "minAvailableConversionPermits": 2,
                "maxRunningConversions": 4,
                "maxPdfOcrWorkers": 8,
                "maxCurrentPdfOcrWorkerBudget": 5,
                "minAvailablePdfOcrWorkerPermits": 3,
                "maxInProcessPdfOcrWorkers": 5,
                "maxInFlightPdfOcrShards": 1,
                "maxPdfOcrCacheHits": 30,
                "maxPdfOcrCacheMisses": 9,
                "maxPdfOcrLiveRequests": 4,
                "maxPdfOcrQueueWaitP95Ms": 40,
                "maxPdfOcrLatencyP95Ms": 500,
                "maxPdfOcrSourcePdfPageRangeShards": 8,
                "maxPdfOcrRenderedPageShards": 3,
                "maxPdfOcrRenderedRegionShards": 2,
                "maxPdfOcrBudgetIncreaseEvents": 2,
                "maxPdfOcrBudgetDecreaseEvents": 1,
                "lastConversionDurationMs": 80,
                "maxConversionDurationMs": 120,
            },
        ]
    )

    assert combined["sampleCount"] == 3
    assert combined["maxQueuedJobs"] == 4
    assert combined["maxRunningJobs"] == 2
    assert combined["minAvailableConversionPermits"] == 2
    assert combined["maxCurrentPdfOcrWorkerBudget"] == 5
    assert combined["minAvailablePdfOcrWorkerPermits"] == 3
    assert combined["maxInProcessPdfOcrWorkers"] == 5
    assert combined["maxPdfOcrCacheHits"] == 30
    assert combined["maxPdfOcrLatencyP95Ms"] == 500
    assert combined["maxPdfOcrBudgetDecreaseEvents"] == 1
    assert combined["lastConversionDurationMs"] == 80


def test_fetch_rust_jobs_status_reads_gateway_payload(monkeypatch) -> None:
    benchmark = _load_benchmark_module()

    class Response:
        def __enter__(self):
            return self

        def __exit__(self, *args) -> None:
            return None

        def read(self) -> bytes:
            return b'{"queuedJobs":2,"runningJobs":1}'

    def fake_urlopen(url: str, timeout: float) -> Response:
        assert url == "http://127.0.0.1:7788/api/document-extract-jobs"
        assert timeout == 1.0
        return Response()

    monkeypatch.setattr(benchmark.urllib.request, "urlopen", fake_urlopen)
    monkeypatch.setattr(benchmark.time, "time", lambda: 42.5)

    status = benchmark.fetch_rust_jobs_status(
        "http://127.0.0.1:7788/",
        require_status=True,
    )

    assert status == {
        "queuedJobs": 2,
        "runningJobs": 1,
        "sampledAtMs": 42500,
    }


def test_rust_pdf_ocr_endpoint_pool_normalizes_repeated_endpoints() -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        rust_pdf_ocr_endpoint=[
            " http://127.0.0.1:52051/ ",
            "",
            "http://127.0.0.1:52052",
        ]
    )

    assert benchmark.rust_pdf_ocr_endpoint_pool(args) == (
        "http://127.0.0.1:52051,http://127.0.0.1:52052"
    )


def test_rust_document_extract_endpoint_pool_normalizes_repeated_endpoints() -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        rust_document_extract_endpoint=[
            " http://127.0.0.1:53051/ ",
            "",
            "http://127.0.0.1:53052",
            "http://127.0.0.1:53051",
        ]
    )

    assert benchmark.rust_document_extract_endpoint_pool(args) == (
        "http://127.0.0.1:53051,http://127.0.0.1:53052"
    )
