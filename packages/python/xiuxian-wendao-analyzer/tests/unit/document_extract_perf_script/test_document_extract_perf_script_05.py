"""document_extract_perf_script test slice 5."""

from __future__ import annotations

from .support import (
    Path,
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


def test_cargo_perf_probe_forwards_structure_baseline_root(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    report_path = tmp_path / "report.json"
    baseline_root = tmp_path / "baselines"
    captured_env = {}

    def fake_run(command: list[str], *, check: bool, env) -> None:
        assert command[0] == "cargo"
        assert check
        captured_env.update(env)
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
        wait_ms=0,
        structure_baseline_root=baseline_root,
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

    assert captured_env["WENDAO_DOCUMENT_EXTRACT_PERF_STRUCTURE_BASELINE_ROOT"] == str(
        baseline_root
    )


def test_cargo_perf_probe_can_override_flight_mode_without_self_parity(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    report_path = tmp_path / "report.json"
    captured_env = {}
    commands: list[list[str]] = []

    def fake_run(command: list[str], *, check: bool, env) -> None:
        commands.append(command)
        assert check
        captured_env.update(env)
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
        wait_ms=0,
        structure_baseline_root=tmp_path / "baselines",
    )

    benchmark.run_cargo_perf_test(
        args,
        tmp_path / "sample.pdf",
        tmp_path / "baseline",
        force=True,
        iterations=1,
        concurrency=1,
        report_path=report_path,
        flight_mode="sync",
        include_structure_baseline_root=False,
    )

    assert captured_env["WENDAO_DOCUMENT_EXTRACT_PERF_MODE"] == "sync"
    assert "WENDAO_DOCUMENT_EXTRACT_PERF_STRUCTURE_BASELINE_ROOT" not in captured_env
    assert commands[0][commands[0].index("--features") + 1] == (
        "performance,studio,zhenfa-router,duckdb"
    )
