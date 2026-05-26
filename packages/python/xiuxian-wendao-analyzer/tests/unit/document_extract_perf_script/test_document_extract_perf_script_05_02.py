"""document_extract_perf_script test slice 5."""

from __future__ import annotations

from .support import (
    Path,
    _load_benchmark_module,
)


def test_structure_order_consistency_compares_force_cache_and_shard_reuse() -> None:
    benchmark = _load_benchmark_module()

    def report(signature: str) -> dict[str, object]:
        return {
            "artifactReports": [
                {
                    "structureArrowExists": True,
                    "structureRowCount": 21,
                    "structureOrderSignature": signature,
                    "structureOrderFirstKey": "000000|000000.000000|000000|page-0",
                    "structureOrderLastKey": "000020|000020.000000|000020|page-20",
                }
            ]
        }

    stable = benchmark.fixture_structure_order_consistency(
        report("same-order"),
        report("same-order"),
        report("same-order"),
    )
    mismatch = benchmark.fixture_structure_order_consistency(
        report("force-order"),
        report("cache-order"),
    )

    assert stable["structureOrderStable"] is True
    assert stable["structureOrderComparedRuns"] == 3
    assert stable["structureOrderMismatchCount"] == 0
    assert stable["structureOrderFirstKey"] == "000000|000000.000000|000000|page-0"
    assert stable["structureOrderLastKey"] == "000020|000020.000000|000020|page-20"
    assert mismatch["structureOrderStable"] is False
    assert mismatch["structureOrderMismatchCount"] == 1


def test_artifact_summary_carries_audio_timeline_metrics() -> None:
    benchmark = _load_benchmark_module()

    summary = benchmark.summarize_artifact_reports(
        [
            {
                "resourcesRowCount": 1,
                "audioTranscriptChars": 128,
                "audioTranscriptTimelineMarkerCount": 3,
                "audioTranscriptTimelineMarkedRows": 1,
                "audioTranscriptAdmissionReportBytes": 128,
                "audioTranscriptAdmissionEnabled": True,
                "audioTranscriptAdmissionHitCount": 2,
                "audioTranscriptAdmissionMissCount": 1,
                "audioTranscriptAdmissionStoredCount": 1,
                "audioTranscriptAdmissionStaleCount": 0,
            }
        ]
    )

    assert summary["audioTranscriptChars"] == 128
    assert summary["audioTranscriptTimelineMarkerCount"] == 3
    assert summary["audioTranscriptTimelineMarkedRows"] == 1
    assert summary["audioTranscriptAdmissionReportExists"] is True
    assert summary["audioTranscriptAdmissionEnabled"] is True
    assert summary["audioTranscriptAdmissionHitCount"] == 2
    assert summary["audioTranscriptAdmissionMissCount"] == 1
    assert summary["audioTranscriptAdmissionStoredCount"] == 1
    assert summary["audioTranscriptAdmissionStaleCount"] == 0


def test_audio_transcript_org_export_reads_resource_arrow(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    resources_path = tmp_path / "_resources.arrow"
    org_path = tmp_path / "report" / "audio-transcripts" / "meeting.org"
    _write_resource_arrow(
        resources_path,
        [
            ("sample.mp3", "document", "", "text/markdown", "ok", "_document"),
            (
                "sample.mp3",
                "audio-transcript",
                "[00:00.000-00:30.000] hello\n[00:30.000-01:00.000] world",
                "text/plain",
                "ok",
                "_audio_transcript",
            ),
        ],
    )

    report = benchmark.export_audio_transcript_org(resources_path, org_path)

    assert report == {
        "path": str(org_path),
        "rows": 1,
        "chars": 55,
        "timelineMarkerCount": 2,
    }
    text = org_path.read_text(encoding="utf-8")
    assert "#+TITLE: Audio Transcript Timeline" in text
    assert ":RESOURCE_TYPE: audio-transcript" in text
    assert "* 00:00:30.000 -- 00:01:00.000 sample.mp3 chunk 0001" in text
    assert ":START_SECONDS: 30.000" in text
    assert "world" in text


def test_audio_transcript_reference_draft_export_splits_timeline_segments(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    resources_path = tmp_path / "_resources.arrow"
    jsonl_path = tmp_path / "meeting.reference_draft.jsonl"
    tsv_path = tmp_path / "meeting.reference_draft.tsv"
    _write_resource_arrow(
        resources_path,
        [
            (
                "/private/sample.mp3",
                "audio-transcript",
                (
                    "[00:00.000-00:30.000] first line\n"
                    "continued first line\n"
                    "[00:30.000-01:00.000] second line"
                ),
                "text/plain",
                "ok",
                "_audio_transcript",
            ),
        ],
    )

    report = benchmark.export_audio_transcript_reference_drafts(
        resources_path,
        jsonl_path,
        tsv_path,
    )

    rows = [
        benchmark.json.loads(line) for line in jsonl_path.read_text(encoding="utf-8").splitlines()
    ]
    assert report["rows"] == 2
    assert rows[0]["source"] == "sample.mp3"
    assert rows[0]["sourceId"] == "/private/sample.mp3"
    assert rows[0]["chunkIndex"] == 0
    assert rows[0]["startSeconds"] == 0.0
    assert rows[0]["durationSeconds"] == 30.0
    assert rows[0]["referenceStatus"] == "candidate-draft"
    assert rows[0]["text"] == "first line\ncontinued first line"
    assert rows[1]["chunkIndex"] == 1
    assert tsv_path.read_text(encoding="utf-8").startswith("source\tsourceId\t")


def test_hosted_audio_trace_summary_reads_worker_logs(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    trace_path = tmp_path / "python-worker.hosted-audio.jsonl"
    trace_path.write_text(
        "\n".join(
            [
                benchmark.json.dumps(
                    {
                        "status": "succeeded",
                        "startedUnixMs": 1000,
                        "endedUnixMs": 2200,
                        "latencyMs": 1200.0,
                        "provider": "openrouter",
                        "model": "qwen/qwen3-asr-flash-2026-02-10",
                        "endpointKind": "audio-transcriptions",
                        "requestKind": "audio-shard",
                        "httpAttemptCount": 1,
                        "shardElementId": "audio-shard-0",
                        "readingOrderKey": "000000.000000000000",
                        "backendProfile": "hosted-audio-transcript-v1",
                        "shardProfile": "audio-shards-v1",
                        "audioFormat": "wav",
                        "sampleRateHz": 16000,
                        "channels": 1,
                        "mediaStartMs": 0,
                        "durationMs": 30000,
                        "mediaDurationMs": 30000,
                        "textChars": 62,
                    }
                ),
                benchmark.json.dumps(
                    {
                        "status": "succeeded",
                        "startedUnixMs": 1100,
                        "endedUnixMs": 2900,
                        "latencyMs": 1800.0,
                        "provider": "openrouter",
                        "model": "qwen/qwen3-asr-flash-2026-02-10",
                        "endpointKind": "audio-transcriptions",
                        "requestKind": "audio-shard",
                        "httpAttemptCount": 1,
                        "shardElementId": "audio-shard-1",
                        "shardProfile": "audio-shards-v1",
                        "audioFormat": "wav",
                        "sampleRateHz": 16000,
                        "channels": 1,
                        "mediaStartMs": 30000,
                        "durationMs": 30000,
                        "mediaDurationMs": 30000,
                        "textChars": 64,
                    }
                ),
                benchmark.json.dumps(
                    {
                        "status": "succeeded",
                        "startedUnixMs": 3000,
                        "endedUnixMs": 3600,
                        "latencyMs": 600.0,
                        "provider": "openrouter",
                        "model": "qwen/qwen3-asr-flash-2026-02-10",
                        "endpointKind": "audio-transcriptions",
                        "requestKind": "audio-shard",
                        "httpAttemptCount": 1,
                        "shardElementId": "audio-shard-0",
                        "shardProfile": "audio-shards-v1",
                        "audioFormat": "wav",
                        "sampleRateHz": 16000,
                        "channels": 1,
                        "mediaStartMs": 0,
                        "durationMs": 30000,
                        "mediaDurationMs": 30000,
                        "textChars": 62,
                    }
                ),
            ]
        ),
        encoding="utf-8",
    )

    summary = benchmark.summarize_hosted_audio_request_traces(tmp_path)

    assert summary["traceFileCount"] == 1
    assert summary["requestCount"] == 3
    assert summary["successCount"] == 3
    assert summary["failureCount"] == 0
    assert summary["latencyMsP50"] == 1200.0
    assert summary["latencyMsP95"] == 1800.0
    assert summary["requestWallSpanMs"] == 2600
    assert summary["requestLatencyOverlapRatio"] == 1.385
    assert summary["textCharCountTotal"] == 188
    assert summary["durationMsTotal"] == 90000
    assert summary["endpointKindCounts"] == {"audio-transcriptions": 3}
    assert summary["audioFormatCounts"] == {"wav": 3}
    assert summary["shardProfileCounts"] == {"audio-shards-v1": 3}
    assert summary["uniqueShardElementIdCount"] == 2
    assert summary["duplicateShardElementIdCount"] == 1
    assert summary["duplicateShardElementIdExtraCount"] == 1
    assert summary["duplicateShardElementIdCounts"] == {"audio-shard-0": 2}
    assert summary["uniqueMediaStartMsCount"] == 2
    assert summary["duplicateMediaStartMsCount"] == 1
    assert summary["duplicateMediaStartMsExtraCount"] == 1
    assert summary["duplicateMediaStartMsCounts"] == {"0": 2}
    assert summary["slowestRequests"][0]["shardElementId"] == "audio-shard-1"
    assert summary["slowestRequests"][0]["shardProfile"] == "audio-shards-v1"


def test_run_fixture_probe_exports_audio_transcript_org(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()

    def fake_run_cargo_perf_test(
        args,
        source,
        output_dir,
        *,
        force,
        iterations,
        concurrency,
        report_path,
        **_kwargs,
    ):
        _write_resource_arrow(
            output_dir / "_resources.arrow",
            [
                (
                    str(source),
                    "audio-transcript",
                    "[00:00.000-00:30.000] segment",
                    "text/plain",
                    "ok",
                    "_audio_transcript",
                )
            ],
        )
        return {
            "latenciesMs": [10.0 if force else 1.0],
            "requestCount": 1,
            "rowCount": 1,
            "batchCount": 1,
            "arrowIpcBytes": 1,
            "errorRowCount": 0,
            "statusCounts": {"ok": 1},
            "wallTimeMs": 1.0,
            "concurrency": concurrency,
            "artifactReports": [],
        }

    monkeypatch.setattr(benchmark, "run_cargo_perf_test", fake_run_cargo_perf_test)
    args = benchmark.argparse.Namespace(
        duplicate_miss_concurrency=0,
        fail_on_error_rows=True,
        fail_on_duplicate_conversions=False,
        fail_on_missing_ocr_metrics=False,
        fail_on_structure_order_mismatch=True,
        fail_on_docling_groundtruth_mismatch=False,
        compare_docling_groundtruth=False,
        docling_groundtruth_root=None,
        docling_groundtruth_min_char_coverage=0.98,
        docling_groundtruth_min_similarity=0.98,
        iterations=1,
        concurrency=1,
        shard_cache_reuse_probe=False,
        artifact_registry_reuse_probe=False,
        export_audio_transcript_org=True,
        report_dir_path=tmp_path / "report",
    )

    result = benchmark.run_fixture_probe(
        args,
        "meeting",
        tmp_path / "meeting.mp3",
        tmp_path / "out",
    )

    org_path = tmp_path / "report" / "audio-transcripts" / "meeting.org"
    assert result["audioTranscriptOrgPath"] == str(org_path)
    assert result["audioTranscriptOrgRows"] == 1
    assert result["audioTranscriptOrgTimelineMarkerCount"] == 1
    assert org_path.exists()
    draft_path = tmp_path / "report" / "audio-transcripts" / "meeting.reference_draft.jsonl"
    assert result["audioTranscriptReferenceDraftJsonlPath"] == str(draft_path)
    assert result["audioTranscriptReferenceDraftRows"] == 1
    assert result["audioTranscriptReferenceDraftChars"] == 7
    assert result["audioTranscriptReferenceDraftEmptyRows"] == 0
    assert result["audioTranscriptReferenceDraftMinChars"] == 7
    assert result["audioTranscriptReferenceDraftMaxChars"] == 7
    assert result["audioTranscriptReferenceDraftDuplicateTextHashCount"] == 0
    assert result["audioTranscriptReferenceDraftUniqueTextHashCount"] == 1
    assert draft_path.exists()


def test_cargo_perf_probe_uses_minimal_feature_set(monkeypatch, tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    report_path = tmp_path / "report.json"
    commands: list[list[str]] = []

    def fake_run(command: list[str], *, check: bool, env) -> None:
        commands.append(command)
        assert check
        assert env["WENDAO_DOCUMENT_EXTRACT_PERF_ENDPOINT"] == "http://127.0.0.1:50052"
        report_path.write_text(
            '{"latenciesMs":[1.0],"requestCount":1,"rowCount":1,'
            '"batchCount":1,"arrowIpcBytes":1,"errorRowCount":0,'
            '"statusCounts":{"ok":1},"wallTimeMs":1.0}',
            encoding="utf-8",
        )

    monkeypatch.setattr(benchmark.subprocess, "run", fake_run)
    monkeypatch.setenv("SDKROOT", "/tmp/macos-sdk")
    monkeypatch.setenv("LIBRARY_PATH", "/tmp/macos-sdk/usr/lib")
    args = benchmark.argparse.Namespace(
        benchmark_host="127.0.0.1",
        benchmark_port=50052,
        cargo="cargo",
        cargo_features="performance,studio,zhenfa-router,duckdb",
        flight_mode="async",
        wait_ms=100,
    )

    benchmark.run_cargo_perf_test(
        args,
        tmp_path / "sample.md",
        tmp_path / "out",
        force=False,
        iterations=1,
        concurrency=1,
        report_path=report_path,
    )

    assert "--no-default-features" in commands[0]
    assert commands[0][commands[0].index("--features") + 1] == (
        "performance,studio,zhenfa-router,duckdb"
    )
    assert commands[0][commands[0].index("--test") + 1] == "wendao-validation-gate"
    report = benchmark.json.loads(report_path.read_text(encoding="utf-8"))
    assert report["rustJobsStatusSummary"]["sampleCount"] == 0


def test_cargo_perf_probe_adds_pdf_source_range_for_hybrid_page_ocr(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    report_path = tmp_path / "report.json"
    commands: list[list[str]] = []

    def fake_run(command: list[str], *, check: bool, env) -> None:
        commands.append(command)
        assert check
        assert env["WENDAO_DOCUMENT_EXTRACT_PERF_MODE"] == "hybrid-page-ocr"
        assert env["WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_PROFILE"] == ("structure-text")
        report_path.write_text(
            '{"latenciesMs":[1.0],"requestCount":1,"rowCount":1,'
            '"batchCount":1,"arrowIpcBytes":1,"errorRowCount":0,'
            '"statusCounts":{"ok":1},"wallTimeMs":1.0}',
            encoding="utf-8",
        )

    monkeypatch.setattr(benchmark.subprocess, "run", fake_run)
    monkeypatch.setenv("SDKROOT", "/tmp/macos-sdk")
    monkeypatch.setenv("LIBRARY_PATH", "/tmp/macos-sdk/usr/lib")
    args = benchmark.argparse.Namespace(
        benchmark_host="127.0.0.1",
        benchmark_port=50052,
        cargo="cargo",
        cargo_features="performance,studio,zhenfa-router,duckdb",
        flight_mode="hybrid-page-ocr",
        rust_pdf_docling_page_range_profile="structure-text",
        wait_ms=0,
    )

    benchmark.run_cargo_perf_test(
        args,
        tmp_path / "sample.pdf",
        tmp_path / "out",
        force=False,
        iterations=1,
        concurrency=1,
        report_path=report_path,
    )

    assert commands[0][commands[0].index("--features") + 1] == (
        "performance,studio,zhenfa-router,duckdb,document-extract-pdf-source-range"
    )


def _write_resource_arrow(
    path: Path,
    rows: list[tuple[str, str, str, str, str, str]],
) -> None:
    import pyarrow as pa
    import pyarrow.ipc as ipc

    path.parent.mkdir(parents=True, exist_ok=True)
    batch = pa.record_batch(
        [
            pa.array([row[0] for row in rows], type=pa.string()),
            pa.array([row[1] for row in rows], type=pa.string()),
            pa.array(["" for _row in rows], type=pa.string()),
            pa.array([None for _row in rows], type=pa.int32()),
            pa.array(["" for _row in rows], type=pa.string()),
            pa.array([row[2] for row in rows], type=pa.string()),
            pa.array([row[3] for row in rows], type=pa.string()),
            pa.array([row[4] for row in rows], type=pa.string()),
            pa.array([row[5] for row in rows], type=pa.string()),
        ],
        names=[
            "sourcePath",
            "resourceType",
            "resourcePath",
            "pageIndex",
            "caption",
            "content",
            "mimeType",
            "status",
            "elementId",
        ],
    )
    with path.open("wb") as handle, ipc.new_file(handle, batch.schema) as writer:
        writer.write_batch(batch)
