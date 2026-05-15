"""document_extract_perf_script test slice 10."""

from __future__ import annotations

from .payloads import (
    distinct_miss_report,
    distinct_miss_summary_result,
    markdown_report_payload,
)
from .support import (
    _load_benchmark_module,
    pytest,
)


def test_summary_and_markdown_report_distinct_miss_burst() -> None:
    benchmark = _load_benchmark_module()
    result = distinct_miss_summary_result(benchmark)
    distinct_report = distinct_miss_report()

    summary = benchmark.summarize_results([result], distinct_report)

    assert summary["distinctMissFixtureCount"] == 2
    assert summary["distinctMissConverterCalls"] == 2
    assert summary["totalErrorRows"] == 0
    assert summary["rustJobsStatusSummary"]["maxRunningJobs"] == 2
    assert summary["totalDocumentTimingRows"] == 3
    assert summary["totalDocumentTimingElapsedMs"] == 30.0
    assert summary["totalDocumentTimingOverheadMs"] == 8.0
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
    assert summary["archiveXmlMemberCount"] == 1
    assert summary["archiveImageMemberCount"] == 3
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
    assert summary["documentTimingPhaseElapsedMs"] == {
        "doclingConvert": 20.0,
        "total": 30.0,
    }
    assert summary["precisionSpeedSummary"]["maxForceRefreshMs"] == 10.0
    assert summary["precisionSpeedSummary"]["maxCacheHitP95Ms"] == 2.0
    assert summary["precisionSpeedSummary"]["totalDoclingConvertMs"] == 20.0
    assert summary["precisionSpeedSummary"]["maxDoclingConvertMs"] == 20.0
    assert summary["precisionSpeedSummary"]["maxDoclingConvertShare"] == pytest.approx(
        20.0 / 30.0
    )
    assert summary["precisionSpeedSummary"][
        "maxDocumentTimingOverheadShare"
    ] == pytest.approx(0.8)
    assert summary["precisionSpeedSummary"]["precisionGatePassed"] is True
    assert summary["precisionSpeedSummary"]["structureOrderStable"] is True
    assert summary["pageRangeDoclingFallbackChunkSummary"][
        "documentExtractProfileCounts"
    ] == {"structure-text": 1}
    assert summary["attachmentClassSummary"][0]["attachmentClass"] == "unknown"
    assert summary["attachmentClassSummary"][0]["archiveAttachmentAuditCount"] == 1
    assert summary["attachmentClassSummary"][0]["archiveMemberCount"] == 10
    assert summary["attachmentClassSummary"][0]["archiveFormatCounts"] == {"tar.gz": 1}
    assert summary["attachmentClassSummary"][0]["archiveAccelerationCandidates"] == {
        "mets_gbs_member_manifest_candidate": 1,
    }

    markdown = benchmark.render_markdown(
        markdown_report_payload(
            benchmark,
            summary=summary,
            result=result,
            distinct_report=distinct_report,
        )
    )
    assert "## Distinct Cold Miss Burst" in markdown
    assert "## Attachment Class Summary" in markdown
    assert "document=1, table=1" in markdown
    assert "image_ocr_cache_candidate=1" in markdown
    assert "small-md:10.000" in markdown
    assert "distinct-01" in markdown
    assert "Shard reuse force ms" in markdown
    assert "Artifact-registry reuse probe" in markdown
    assert "Rust OCR source-range trace" in markdown
    assert "Document extract prewarm page ranges resolved" in markdown
    assert "1:3,4:4,5:6,7:9" in markdown
    assert "Rust PDF Docling page-range hedge delay ms" in markdown
    assert "7000" in markdown
    assert "Rust PDF Docling page-range structure-cost budget" in markdown
    assert "2400" in markdown
    assert "Rust PDF Docling text-shortcut promotion" in markdown
    assert "disabled" in markdown
    assert "Artifact reuse ms" in markdown
    assert "9.000" in markdown
    assert "42.000" in markdown
    assert "OCR shard cache" in markdown
    assert "files=2" in markdown
    assert "Metrics sidecar" in markdown
    assert "chars=80" in markdown
    assert "Document timing sidecar" in markdown
    assert "Image audit summary" in markdown
    assert "knownDims=1" in markdown
    assert "dimensionSources=png_ihdr=1" in markdown
    assert "Archive audit summary" in markdown
    assert "members=10" in markdown
    assert "suffixes=html=3, tif=3, txt=3, xml=1" in markdown
    assert "mets_gbs_member_manifest_candidate=1" in markdown
    assert "doclingConvert=20.000" in markdown
    assert "overheadMs=8.000" in markdown
    assert "maxDoclingConvertMs=20.000" in markdown
    assert "maxDoclingShare=66.7%" in markdown
    assert "maxTimingOverheadMs=8.000" in markdown
    assert "maxBoundaryOverheadShare=80.0%" in markdown
    assert "doclingChunkProfiles=`structure-text=1`" in markdown
    assert "Rust PDF OCR source-range workers" in markdown
    assert "Structure parity" in markdown
    assert "Structure order stable across runs" in markdown
    assert "Structure baseline generation" in markdown
    assert "Precision-speed summary" in markdown
    assert "orderStable=True" in markdown
    assert "maxForceMs=10.000" in markdown
