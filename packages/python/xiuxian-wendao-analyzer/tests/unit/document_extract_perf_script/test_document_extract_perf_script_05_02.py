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
        assert env["WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_PROFILE"] == (
            "structure-text"
        )
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
