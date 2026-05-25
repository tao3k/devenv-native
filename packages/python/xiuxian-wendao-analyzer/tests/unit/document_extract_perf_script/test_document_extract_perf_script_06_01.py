"""document_extract_perf_script test slice 6."""

from __future__ import annotations

from .support import (
    Path,
    _load_benchmark_module,
    pytest,
)


@pytest.mark.parametrize(
    ("flight_mode", "artifact_registry_reuse_probe", "expected"),
    [
        ("sync", False, False),
        ("sync", True, True),
        ("async", False, True),
        ("hybrid-page-ocr", False, True),
        ("audio-shards", False, True),
    ],
)
def test_artifact_registry_reuse_probe_routes_through_rust_provider(
    flight_mode: str,
    artifact_registry_reuse_probe: bool,
    expected: bool,
) -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        flight_mode=flight_mode,
        artifact_registry_reuse_probe=artifact_registry_reuse_probe,
    )

    assert benchmark.should_start_local_rust_provider(args) is expected


def test_report_payload_exposes_top_level_precision_speed_summary(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    audio_trace_dir = tmp_path / "audio-traces"
    audio_trace_dir.mkdir()
    (audio_trace_dir / "worker.hosted-audio.jsonl").write_text(
        "\n".join(
            [
                benchmark.json.dumps(
                    {
                        "status": "succeeded",
                        "provider": "openrouter",
                        "model": "qwen/qwen3-asr-flash-2026-02-10",
                        "endpointKind": "audio-transcriptions",
                        "requestKind": "audio-shard",
                        "shardElementId": "audio-shard-1",
                        "shardProfile": "audio-shards-v1",
                        "mediaStartMs": 0,
                        "durationMs": 30_000,
                        "mediaDurationMs": 30_000,
                        "latencyMs": 80.0,
                        "textChars": 42,
                        "startedUnixMs": 1_000,
                        "endedUnixMs": 1_080,
                    }
                )
            ]
        ),
        encoding="utf-8",
    )
    args = benchmark.argparse.Namespace(
        real_docling=False,
        benchmark_host="127.0.0.1",
        benchmark_port=50052,
        rust_rest_endpoint=None,
        iterations=1,
        concurrency=1,
        flight_mode="sync",
        wait_ms=0,
        pdf_ocr_worker="skip",
        pdf_ocr_workers="auto",
        pdf_ocr_prewarm_profile=["docling-fast-text-ocr"],
        pdf_ocr_fast_text_source_converter="backend-table",
        pdf_ocr_backend_text_empty_page="verified-empty",
        local_python_ocr_endpoint_count=1,
        rust_pdf_ocr_workers=None,
        rust_pdf_ocr_source_range_workers=None,
        rust_pdf_local_backend_text="rust-lopdf",
        rust_pdf_local_backend_text_empty="fail-fast",
        rust_pdf_local_fast_text="rust-lopdf",
        rust_pdf_fast_text_source_range_split="single-page",
        rust_pdf_backend_text_topup="disabled",
        rust_audio_artifact_cache_dir=tmp_path / "audio-artifacts",
        rust_audio_transcript_admission_dir=tmp_path / "audio-transcript-admissions",
        rust_audio_speech_segments_jsonl=tmp_path / "speech.jsonl",
        rust_audio_speech_merge_gap_ms=500,
        rust_audio_speech_min_window_ms=5_000,
        rust_audio_speech_limit_chunks=24,
        rust_document_extract_endpoint=[],
        rust_pdf_ocr_endpoint=[],
        structure_baseline_root=None,
        shard_cache_reuse_probe=False,
        artifact_registry_reuse_probe=True,
        ocr_shard_cache_root=tmp_path / "ocr-shards",
        hosted_vlm_ocr_image_optimization="region-whitespace-trim",
        hosted_audio_request_trace_log_dir=audio_trace_dir,
    )
    result = {
        "fixture": "markdown",
        "attachmentClass": "structured_text",
        "forceRefreshMs": 10.0,
        "artifactRegistryReuseForceMs": 4.0,
        "cacheHitP95Ms": 2.0,
        "wallTimeMs": 3.0,
        "cacheSpeedup": 5.0,
        "forceErrorRows": 0,
        "artifactRegistryReuseErrorRows": 0,
        "cacheErrorRows": 0,
        "totalRows": 2,
        "requestCount": 1,
        "arrowIpcBytes": 64,
        "duplicateMissConverterCalls": None,
        "forceAudioMaterializationWorkflowTotalElapsedMs": 112.0,
        "forceAudioMaterializationWorkflowStageElapsedMs": {
            "audio.base.call_analyzer_flight": 82.0,
            "audio.base.materialize_shards": 30.0,
        },
        "structureRows": 1,
        "structureReadingOrderSorted": True,
        "structureOrderStable": True,
        "structureOrderMismatchCount": 0,
    }

    payload = benchmark.build_report_payload(
        args,
        real_fixture_root=None,
        results=[result],
        distinct_miss_report=None,
        structure_baseline_report=None,
        ocr_shard_cache_summary={
            "root": str(tmp_path),
            "fileCount": 0,
            "totalBytes": 0,
        },
    )

    assert payload["precisionSpeedSummary"] == payload["summary"]["precisionSpeedSummary"]
    assert payload["precisionSpeedSummary"]["maxArtifactRegistryReuseForceMs"] == 4.0
    assert payload["hostedVlmOcr"]["imageOptimizationMode"] == "region-whitespace-trim"
    assert payload["pdfOcrPrewarmProfiles"] == ["docling-fast-text-ocr"]
    assert payload["pdfOcrPrewarmSourcePath"] is None
    assert payload["pdfOcrPrewarmPageIndex"] is None
    assert payload["pdfOcrPrewarmPageIndices"] is None
    assert payload["pdfOcrPrewarmEndpointCount"] is None
    assert payload["pdfOcrFastTextSourceConverter"] == "backend-table"
    assert payload["pdfOcrBackendTextEmptyPage"] == "verified-empty"
    assert payload["rustPdfLocalBackendText"] == "rust-lopdf"
    assert payload["rustPdfLocalBackendTextEmpty"] == "fail-fast"
    assert payload["rustPdfLocalFastText"] == "rust-lopdf"
    assert payload["rustPdfFastTextSourceRangeSplit"] == "single-page"
    assert payload["rustPdfFastTextEndpointAffinity"] == "disabled"
    assert payload["rustPdfOcrSchedulerLaneFairness"] == "disabled"
    assert payload["rustPdfBackendTextTopup"] == "disabled"
    assert payload["rustAudioArtifactCacheDir"] == str(tmp_path / "audio-artifacts")
    assert payload["rustAudioTranscriptAdmissionDir"] == str(
        tmp_path / "audio-transcript-admissions"
    )
    assert payload["rustAudioSpeechSegmentsJsonl"] == str(tmp_path / "speech.jsonl")
    assert payload["rustAudioSpeechMergeGapMs"] == 500
    assert payload["rustAudioSpeechMinWindowMs"] == 5_000
    assert payload["rustAudioSpeechLimitChunks"] == 24
    assert payload["summary"]["forceAudioHostedRequestWallSpanMs"] == 80.0
    assert payload["summary"]["forceAudioHostedAnalyzerCallMs"] == 82.0
    assert payload["summary"]["forceAudioHostedAnalyzerRequestWallGapMs"] == 2.0
    assert payload["summary"]["forceAudioHostedWorkflowRequestWallGapMs"] == 32.0


def test_report_gate_rejects_structure_parity_failures() -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        fail_on_precision_gate_failure=False,
        fail_on_structure_parity_mismatch=True,
        fail_on_pdf_milestone_regression=False,
    )
    payload = {
        "summary": {
            "structureParityCheckedFixtures": 1,
            "allStructureParityPassed": False,
            "totalStructureParityErrors": 1,
            "precisionSpeedSummary": {
                "pdfOcrMilestoneGuard": {"passed": True},
            },
        }
    }

    with pytest.raises(SystemExit, match="structure parity gate failed"):
        benchmark.enforce_report_gates(args, payload)


def test_report_gate_rejects_precision_gate_failures() -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        fail_on_precision_gate_failure=True,
        fail_on_structure_parity_mismatch=False,
        fail_on_pdf_milestone_regression=False,
    )
    payload = {
        "summary": {
            "structureParityCheckedFixtures": 1,
            "allStructureParityPassed": False,
            "totalStructureParityErrors": 1,
            "precisionSpeedSummary": {
                "precisionGatePassed": False,
                "pdfOcrMilestoneGuard": {"passed": True},
            },
        }
    }

    with pytest.raises(SystemExit, match="precision gate failed"):
        benchmark.enforce_report_gates(args, payload)
