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
    result["forceHybridPageOcrTimingOcr2RegionRenderReportedElapsedMs"] = 42.5
    result["forceHybridPageOcrTimingOcr2RegionPipelinePlannedRenderChunkCount"] = 3
    result["forceHybridPageOcrTimingOcr2RegionPipelineEndpointCount"] = 4
    result["forceHybridPageOcrTimingOcr2RegionPipelineRenderAheadLimit"] = 3
    result["forceHybridPageOcrTimingOcr2RegionPipelineRenderSpawnCount"] = 3
    distinct_report = distinct_miss_report()

    summary = benchmark.summarize_results([result], distinct_report)

    assert summary["distinctMissFixtureCount"] == 2
    assert summary["distinctMissConverterCalls"] == 2
    assert summary["totalErrorRows"] == 0
    assert summary["rustJobsStatusSummary"]["maxRunningJobs"] == 2
    assert summary["totalDocumentTimingRows"] == 3
    assert summary["totalDocumentTimingElapsedMs"] == 30.0
    assert summary["totalDocumentTimingOverheadMs"] == 8.0
    assert summary["totalAudioTranscriptChars"] == 128
    assert summary["totalAudioTranscriptTimelineMarkerCount"] == 3
    assert summary["totalAudioTranscriptTimelineMarkedRows"] == 2
    assert summary["totalAudioTranscriptOrgRows"] == 1
    assert summary["totalAudioTranscriptOrgChars"] == 128
    assert summary["totalAudioTranscriptOrgTimelineMarkerCount"] == 3
    assert summary["totalAudioTranscriptReferenceDraftRows"] == 2
    assert summary["totalAudioTranscriptReferenceDraftChars"] == 126
    assert summary["totalAudioTranscriptReferenceDraftEmptyRows"] == 0
    assert summary["totalAudioTranscriptReferenceDraftDuplicateTextHashCount"] == 0
    assert summary["minAudioTranscriptReferenceDraftChars"] == 61
    assert summary["maxAudioTranscriptReferenceDraftChars"] == 65
    assert summary["totalForceAudioTranscriptAdmissionMissCount"] == 2
    assert summary["totalForceAudioTranscriptAdmissionStoredCount"] == 2
    assert summary["totalArtifactReuseAudioTranscriptAdmissionHitCount"] == 2
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
    assert summary["precisionSpeedSummary"]["maxDoclingConvertShare"] == pytest.approx(20.0 / 30.0)
    assert summary["precisionSpeedSummary"]["maxDocumentTimingOverheadShare"] == pytest.approx(0.8)
    assert summary["precisionSpeedSummary"]["precisionGatePassed"] is True
    assert summary["precisionSpeedSummary"]["structureOrderStable"] is True
    assert summary["pageRangeDoclingFallbackChunkSummary"]["documentExtractProfileCounts"] == {
        "structure-text": 1
    }
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
    assert "Rust adaptive audio" in markdown
    assert "budget=3" in markdown
    assert "budgetDown=1" in markdown
    assert "Rust hosted region render trace" in markdown
    assert "reportedMs=42.500" in markdown
    assert "plannedChunks=3" in markdown
    assert "endpoints=4" in markdown
    assert "renderAhead=3" in markdown
    assert "renderSpawns=3" in markdown
    assert "profile=docling-fast-text-ocr" in markdown
    assert "shardType=page" in markdown
    assert "longestChars=80.000" in markdown
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
    assert "Hosted VLM/OCR requests" in markdown
    assert "sourcePixelsMax=1795980" in markdown
    assert "Hosted VLM/OCR slowest requests" in markdown
    assert "page=12 region=1 latencyMs=6924.510" in markdown
    assert "kind=region-hedged" in markdown
    assert "Hosted VLM/OCR speculative retry minimums" in markdown
    assert "sourcePixels=1000000" in markdown
    assert "OpenRouter provider routing" in markdown
    assert '{"sort":{"by":"latency"}}' in markdown
    assert "Audio transcript evidence" in markdown
    assert "timelineMarkers=3" in markdown
    assert "referenceDraftRows=2" in markdown
    assert "referenceDraftMinChars=61" in markdown
    assert "referenceDraftDuplicateTextHashes=0" in markdown
    assert "Audio transcript admission" in markdown
    assert "forceMisses=2" in markdown
    assert "reuseHits=2" in markdown
    assert "Audio hosted non-model timing" in markdown
    assert "Hosted audio requests" in markdown
    assert "uniqueMediaStarts=2" in markdown
    assert "duplicateMediaStarts=0" in markdown
    assert "shardProfiles=audio-shards-v1=2" in markdown
    assert "p95Ms=1800.000" in markdown
    assert "Hosted audio slowest requests" in markdown
    assert "shard=audio-shard-1" in markdown
    assert "profile=audio-shards-v1" in markdown
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
    assert "Rust PDF Hosted VLM/OCR region target pixels" in markdown
    assert "750000.0" in markdown
    assert "Rust PDF Hosted VLM/OCR region max slices" in markdown
    assert "artifactHits=" in markdown
    assert "artifactBytes=" in markdown
    assert "Structure parity" in markdown
    assert "Structure order stable across runs" in markdown
    assert "Structure baseline generation" in markdown
    assert "Precision-speed summary" in markdown
    assert "orderStable=True" in markdown
    assert "maxForceMs=10.000" in markdown
    assert "defaultPromotionCandidate=" in markdown
    assert "optInPromotionControls=" in markdown


def test_summary_treats_disabled_shard_region_artifact_cache_fields_as_zero() -> None:
    benchmark = _load_benchmark_module()
    result = distinct_miss_summary_result(benchmark)
    result["forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheHitCount"] = 1
    result["forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheMissCount"] = 2
    result["forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheThrottledCount"] = 3
    result["forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheByteCount"] = 4
    result["shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheHitCount"] = None
    result["shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheMissCount"] = None
    result["shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheThrottledCount"] = None
    result["shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheByteCount"] = None

    summary = benchmark.summarize_results([result])

    assert summary["forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheHitCount"] == 1
    assert summary["forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheMissCount"] == 2
    assert summary["forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheThrottledCount"] == 3
    assert summary["forceHybridPageOcrTimingOcr2RegionRenderArtifactCacheByteCount"] == 4
    assert summary["shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheHitCount"] == 0
    assert summary["shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheMissCount"] == 0
    assert (
        summary["shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheThrottledCount"]
        == 0
    )
    assert summary["shardCacheReuseHybridPageOcrTimingOcr2RegionRenderArtifactCacheByteCount"] == 0
