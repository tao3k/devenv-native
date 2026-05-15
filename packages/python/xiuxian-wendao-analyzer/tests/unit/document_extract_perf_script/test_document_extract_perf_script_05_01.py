"""document_extract_perf_script test slice 5."""

from __future__ import annotations

from .support import (
    _load_benchmark_module,
)


def test_artifact_report_summary_tracks_structure_precision() -> None:
    benchmark = _load_benchmark_module()

    summary = benchmark.summarize_artifact_reports(
        [
            {
                "resourcesArrowExists": True,
                "resourcesRowCount": 3,
                "structureArrowExists": True,
                "structureRowCount": 3,
                "structureOcrPageBlocks": 1,
                "structureOcrRegionBlocks": 2,
                "structureBboxBlocks": 2,
                "structureReadingOrderSorted": True,
                "structureOrderSignature": "order-a",
                "structureOrderFirstKey": "000000|000000.000000|000000|a",
                "structureOrderLastKey": "000000|000000.000002|000002|c",
                "structureParity": {
                    "baselineBlockCount": 2,
                    "candidateBlockCount": 3,
                    "baselinePageCount": 1,
                    "candidatePageCount": 1,
                    "baselineTextChars": 80,
                    "candidateTextChars": 120,
                    "protectedBlockCounts": {},
                },
                "structureParityError": None,
                "metricsArrowExists": True,
                "metricsRowCount": 3,
                "metricsResultChars": 120,
                "metricsBboxCount": 2,
                "metricsRustSchedulerElapsedMs": 10.5,
                "documentTimingArrowExists": True,
                "documentTimingRowCount": 3,
                "documentTimingTotalElapsedMs": 20.0,
                "documentTimingPhaseElapsedMs": {
                    "doclingConvert": 15.0,
                    "writeResourcesArrow": 2.0,
                    "total": 20.0,
                },
                "imageAttachmentAudit": {
                    "format": "png",
                    "widthPx": 640,
                    "heightPx": 480,
                    "pixelCount": 307200,
                    "dimensionSource": "png_ihdr",
                    "rustAccelerationCandidate": "image_ocr_cache_candidate",
                },
                "archiveAttachmentAudit": {
                    "archiveFormat": "tar.gz",
                    "memberCount": 10,
                    "regularFileCount": 10,
                    "xmlMemberCount": 1,
                    "imageMemberCount": 3,
                    "totalMemberSizeBytes": 267702,
                    "extensionCounts": {
                        "html": 3,
                        "tif": 3,
                        "txt": 3,
                        "xml": 1,
                    },
                    "largestMemberSizeBytes": 59518,
                    "rustAccelerationCandidate": "mets_gbs_member_manifest_candidate",
                },
                "artifactError": None,
            },
            {
                "resourcesArrowExists": True,
                "resourcesRowCount": 1,
                "structureArrowExists": True,
                "structureRowCount": 1,
                "structureOcrPageBlocks": 0,
                "structureOcrRegionBlocks": 1,
                "structureBboxBlocks": 1,
                "structureReadingOrderSorted": True,
                "structureOrderSignature": "order-b",
                "structureOrderFirstKey": "000001|000001.000000|000000|d",
                "structureOrderLastKey": "000001|000001.000000|000000|d",
                "metricsArrowExists": True,
                "metricsRowCount": 1,
                "metricsResultChars": 40,
                "metricsBboxCount": 1,
                "metricsRustSchedulerElapsedMs": 2.5,
                "documentTimingArrowExists": True,
                "documentTimingRowCount": 2,
                "documentTimingTotalElapsedMs": 5.0,
                "documentTimingPhaseElapsedMs": {
                    "doclingConvert": 4.0,
                    "total": 5.0,
                },
                "hybridPageOcrFallbackReason": (
                    "routing decision `full_docling_fallback` is not eligible for hybrid OCR"
                ),
                "artifactError": None,
            },
        ]
    )

    assert summary["resourcesArrowExists"] is True
    assert summary["resourcesRows"] == 4
    assert summary["structureArrowExists"] is True
    assert summary["structureRows"] == 4
    assert summary["structureOcrPageBlocks"] == 1
    assert summary["structureOcrRegionBlocks"] == 3
    assert summary["structureBboxBlocks"] == 3
    assert summary["structureReadingOrderSorted"] is True
    assert summary["structureParityChecked"] is True
    assert summary["structureParityPassed"] is True
    assert summary["structureParityErrorCount"] == 0
    assert summary["metricsArrowExists"] is True
    assert summary["metricsRows"] == 4
    assert summary["metricsResultChars"] == 160
    assert summary["metricsBboxCount"] == 3
    assert summary["metricsRustSchedulerElapsedMs"] == 13.0
    assert summary["documentTimingArrowExists"] is True
    assert summary["documentTimingRows"] == 5
    assert summary["documentTimingTotalElapsedMs"] == 25.0
    assert summary["documentTimingPhaseElapsedMs"] == {
        "doclingConvert": 19.0,
        "total": 25.0,
        "writeResourcesArrow": 2.0,
    }
    assert summary["hybridPageOcrFallbackReasons"] == [
        "routing decision `full_docling_fallback` is not eligible for hybrid OCR"
    ]
    assert summary["imageAttachmentAuditCount"] == 1
    assert summary["imageKnownDimensionCount"] == 1
    assert summary["imageFormatCounts"] == {"png": 1}
    assert summary["imageDimensionSourceCounts"] == {"png_ihdr": 1}
    assert summary["imageAccelerationCandidates"] == {"image_ocr_cache_candidate": 1}
    assert summary["maxImageWidthPx"] == 640
    assert summary["maxImageHeightPx"] == 480
    assert summary["maxImagePixelCount"] == 307200
    assert summary["archiveAttachmentAuditCount"] == 1
    assert summary["archiveMemberCount"] == 10
    assert summary["archiveRegularFileCount"] == 10
    assert summary["archiveXmlMemberCount"] == 1
    assert summary["archiveImageMemberCount"] == 3
    assert summary["archiveTotalMemberSizeBytes"] == 267702
    assert summary["archiveFormatCounts"] == {"tar.gz": 1}
    assert summary["archiveExtensionCounts"] == {
        "html": 3,
        "tif": 3,
        "txt": 3,
        "xml": 1,
    }
    assert summary["archiveAccelerationCandidates"] == {
        "mets_gbs_member_manifest_candidate": 1,
    }
    assert summary["maxArchiveLargestMemberSizeBytes"] == 59518
    assert summary["artifactErrorCount"] == 0
