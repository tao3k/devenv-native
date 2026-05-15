"""document_extract_perf_script test slice 9."""

from __future__ import annotations

from .support import (
    _load_benchmark_module,
    pytest,
)


def test_attachment_class_summary_groups_precision_and_speed() -> None:
    benchmark = _load_benchmark_module()

    summary = benchmark.summarize_results(
        [
            {
                "fixture": "docx",
                "attachmentClass": "office",
                "totalRows": 10,
                "forceErrorRows": 0,
                "cacheErrorRows": 0,
                "shardCacheReuseErrorRows": 0,
                "requestCount": 2,
                "arrowIpcBytes": 100,
                "cacheSpeedup": 4.0,
                "duplicateMissConverterCalls": 1,
                "artifactErrorCount": 0,
                "structureRows": 4,
                "structureOcrPageBlocks": 0,
                "structureOcrRegionBlocks": 0,
                "structureBboxBlocks": 0,
                "structureReadingOrderSorted": True,
                "structureOrderStable": True,
                "structureOrderMismatchCount": 0,
                "structureParityPassed": None,
                "structureParityErrorCount": 0,
                "metricsRows": 0,
                "metricsResultChars": 0,
                "metricsBboxCount": 0,
                "metricsRustSchedulerElapsedMs": 0.0,
                "documentTimingRows": 3,
                "documentTimingTotalElapsedMs": 18.0,
                "documentTimingOverheadMs": 2.0,
                "documentTimingPhaseElapsedMs": {
                    "doclingConvert": 12.0,
                    "total": 18.0,
                },
                "forceRefreshMs": 20.0,
                "cacheHitP95Ms": 2.0,
                "wallTimeMs": 3.0,
                "resourcesRows": 4,
                "artifactReports": [
                    {
                        "resourceTypeCounts": {
                            "document": 1,
                            "docling_json": 1,
                            "image": 1,
                            "table": 1,
                        },
                        "resourceStatusCounts": {"ok": 4},
                        "structureBlockTypeCounts": {
                            "document": 1,
                            "image": 1,
                            "table": 1,
                        },
                        "metricsStatusCounts": {},
                        "documentTimingStatusCounts": {"ok": 3},
                        "documentTimingPhaseElapsedMs": {
                            "doclingConvert": 12.0,
                            "total": 18.0,
                        },
                    }
                ],
            },
            {
                "fixture": "image-png",
                "attachmentClass": "image",
                "totalRows": 5,
                "forceErrorRows": 0,
                "cacheErrorRows": 0,
                "shardCacheReuseErrorRows": 0,
                "requestCount": 1,
                "arrowIpcBytes": 80,
                "cacheSpeedup": 2.0,
                "duplicateMissConverterCalls": 1,
                "artifactErrorCount": 0,
                "structureRows": 1,
                "structureOcrPageBlocks": 0,
                "structureOcrRegionBlocks": 0,
                "structureBboxBlocks": 0,
                "structureReadingOrderSorted": True,
                "structureOrderStable": True,
                "structureOrderMismatchCount": 0,
                "structureParityPassed": None,
                "structureParityErrorCount": 0,
                "metricsRows": 0,
                "metricsResultChars": 0,
                "metricsBboxCount": 0,
                "metricsRustSchedulerElapsedMs": 0.0,
                "documentTimingRows": 3,
                "documentTimingTotalElapsedMs": 45.0,
                "documentTimingOverheadMs": 5.0,
                "documentTimingPhaseElapsedMs": {
                    "doclingConvert": 40.0,
                    "total": 45.0,
                },
                "forceRefreshMs": 50.0,
                "cacheHitP95Ms": 5.0,
                "wallTimeMs": 6.0,
                "resourcesRows": 3,
                "artifactReports": [
                    {
                        "resourceTypeCounts": {
                            "document": 1,
                            "docling_json": 1,
                            "table": 1,
                        },
                        "resourceStatusCounts": {"ok": 3},
                        "structureBlockTypeCounts": {"document": 1, "table": 1},
                        "metricsStatusCounts": {},
                        "documentTimingStatusCounts": {"ok": 3},
                        "documentTimingPhaseElapsedMs": {
                            "doclingConvert": 40.0,
                            "total": 45.0,
                        },
                        "imageAttachmentAudit": {
                            "format": "png",
                            "widthPx": 640,
                            "heightPx": 480,
                            "pixelCount": 307200,
                            "dimensionSource": "png_ihdr",
                            "rustAccelerationCandidate": ("image_ocr_cache_candidate"),
                        },
                    }
                ],
            },
        ],
    )

    class_summary = {
        item["attachmentClass"]: item for item in summary["attachmentClassSummary"]
    }
    assert set(class_summary) == {"image", "office"}
    assert summary["imageAttachmentAuditCount"] == 1
    assert summary["imageKnownDimensionCount"] == 1
    assert summary["imageFormatCounts"] == {"png": 1}
    assert summary["imageDimensionSourceCounts"] == {"png_ihdr": 1}
    assert summary["imageAccelerationCandidates"] == {"image_ocr_cache_candidate": 1}
    assert summary["maxImageWidthPx"] == 640
    assert summary["maxImageHeightPx"] == 480
    assert summary["maxImagePixelCount"] == 307200
    assert class_summary["office"]["fixtureCount"] == 1
    assert class_summary["office"]["fixtures"] == ["docx"]
    assert (
        class_summary["office"]["precisionSpeedSummary"]["precisionGatePassed"] is True
    )
    assert class_summary["office"]["precisionSpeedSummary"]["maxForceRefreshMs"] == 20.0
    assert class_summary["office"]["resourcesRows"] == 4
    assert class_summary["office"]["resourceTypeCounts"] == {
        "docling_json": 1,
        "document": 1,
        "image": 1,
        "table": 1,
    }
    assert class_summary["office"]["structureBlockTypeCounts"] == {
        "document": 1,
        "image": 1,
        "table": 1,
    }
    assert class_summary["office"]["slowestForceFixture"] == {
        "fixture": "docx",
        "latencyMs": 20.0,
    }
    assert class_summary["office"]["documentTimingTotalElapsedMs"] == 18.0
    assert class_summary["office"]["documentTimingOverheadMs"] == 2.0
    assert class_summary["office"]["documentTimingStatusCounts"] == {"ok": 3}
    assert class_summary["office"]["precisionSpeedSummary"][
        "maxDoclingConvertShare"
    ] == pytest.approx(12.0 / 18.0)
    assert class_summary["office"]["precisionSpeedSummary"][
        "maxDocumentTimingOverheadShare"
    ] == pytest.approx(0.1)
    assert class_summary["image"]["structureRows"] == 1
    assert class_summary["image"]["resourceTypeCounts"]["table"] == 1
    assert class_summary["image"]["imageAttachmentAuditCount"] == 1
    assert class_summary["image"]["imageKnownDimensionCount"] == 1
    assert class_summary["image"]["imageFormatCounts"] == {"png": 1}
    assert class_summary["image"]["imageDimensionSourceCounts"] == {"png_ihdr": 1}
    assert class_summary["image"]["imageAccelerationCandidates"] == {
        "image_ocr_cache_candidate": 1
    }
    assert class_summary["image"]["maxImageWidthPx"] == 640
    assert class_summary["image"]["maxImageHeightPx"] == 480
    assert class_summary["image"]["maxImagePixelCount"] == 307200
    assert class_summary["image"]["slowestCacheP95Fixture"] == {
        "fixture": "image-png",
        "latencyMs": 5.0,
    }
    assert class_summary["image"]["slowestTimingOverheadFixture"] == {
        "fixture": "image-png",
        "latencyMs": 5.0,
    }
    assert class_summary["image"]["precisionSpeedSummary"]["maxCacheHitP95Ms"] == 5.0
    assert (
        class_summary["image"]["precisionSpeedSummary"]["maxDocumentTimingOverheadMs"]
        == 5.0
    )
    assert class_summary["image"]["documentTimingPhaseElapsedMs"] == {
        "doclingConvert": 40.0,
        "total": 45.0,
    }
    assert class_summary["image"]["precisionSpeedSummary"]["maxDoclingConvertMs"] == (
        40.0
    )
    assert class_summary["image"]["precisionSpeedSummary"][
        "maxDoclingConvertShare"
    ] == pytest.approx(40.0 / 45.0)
