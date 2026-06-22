"""document_extract_perf_script test slice 6."""

from __future__ import annotations

from .support import (
    Path,
    _load_benchmark_module,
    pytest,
)


def test_report_gate_requires_structure_parity_checks() -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        fail_on_precision_gate_failure=False,
        fail_on_structure_parity_mismatch=True,
        fail_on_pdf_milestone_regression=False,
    )
    payload = {
        "summary": {
            "structureParityCheckedFixtures": 0,
            "allStructureParityPassed": None,
            "totalStructureParityErrors": 0,
            "precisionSpeedSummary": {
                "pdfOcrMilestoneGuard": {"passed": True},
            },
        }
    }

    with pytest.raises(SystemExit, match="no fixtures checked"):
        benchmark.enforce_report_gates(args, payload)


def test_run_structure_baseline_probe_generates_sync_fixture_baselines(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    calls: list[dict[str, object]] = []

    def fake_run_cargo_perf_test(
        args: object,
        source: Path,
        output_dir: Path,
        *,
        force: bool,
        iterations: int,
        concurrency: int,
        report_path: Path,
        flight_mode: str,
        include_structure_baseline_root: Path | None,
        **_kwargs: object,
    ) -> dict[str, object]:
        calls.append(
            {
                "source": source,
                "output_dir": output_dir,
                "force": force,
                "iterations": iterations,
                "concurrency": concurrency,
                "report_path": report_path,
                "flight_mode": flight_mode,
                "include_structure_baseline_root": include_structure_baseline_root,
            }
        )
        return {
            "errorRowCount": 0,
            "artifactReports": [
                {
                    "resourcesArrowExists": True,
                    "resourcesRowCount": 2,
                    "structureArrowExists": True,
                    "structureRowCount": 2,
                    "structureReadingOrderSorted": True,
                }
            ],
        }

    monkeypatch.setattr(benchmark, "run_cargo_perf_test", fake_run_cargo_perf_test)
    args = benchmark.argparse.Namespace(
        generate_structure_baselines=True,
        fail_on_error_rows=True,
    )
    baseline_root = tmp_path / "baselines"

    report = benchmark.run_structure_baseline_probe(
        args,
        {
            "pdf": tmp_path / "sample.pdf",
            "image": tmp_path / "sample.png",
        },
        baseline_root,
    )

    assert report["enabled"] is True
    assert report["root"] == str(baseline_root)
    assert report["fixtureCount"] == 2
    assert report["totalStructureRows"] == 4
    assert report["allStructureReadingOrderSorted"] is True
    assert [call["output_dir"].name for call in calls] == ["pdf", "image"]
    assert all(call["flight_mode"] == "sync" for call in calls)
    assert all(call["force"] is True for call in calls)
    assert all(call["include_structure_baseline_root"] is False for call in calls)


def test_structure_baseline_root_defaults_to_report_dir_when_generating(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        generate_structure_baselines=True,
        structure_baseline_root=None,
    )

    assert (
        benchmark.resolve_structure_baseline_root(args, tmp_path)
        == (tmp_path / "structure-baselines").resolve()
    )


def test_pdf_render_shard_audit_command_adds_feature_and_fixture_manifest(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        cargo="cargo",
        cargo_features=(
            "performance,studio,zhenfa-router,duckdb,document-extract-attachment-audit"
        ),
        pdfium_library_path=None,
        prepare_pdfium_runtime=False,
        require_pdfium=False,
        pdf_render_selection="all-pages",
        pdf_render_region=[],
    )

    command, env = benchmark.build_pdf_render_shard_audit_command(
        args,
        {"pdf": tmp_path / "sample.pdf"},
        tmp_path / "reports",
    )

    assert command[:4] == ["cargo", "test", "-p", "xiuxian-wendao-studio"]
    assert command[command.index("--test") + 1] == "performance_test"
    assert command[command.index("--features") + 1] == (
        "performance,studio,zhenfa-router,duckdb,document-extract-pdf-render"
    )
    assert command[-4:] == [
        "pdf_render_page_render_shard_manifest",
        "--",
        "--ignored",
        "--nocapture",
    ]
    inputs = benchmark.json.loads(env["WENDAO_PDF_RENDER_SHARD_INPUTS_JSON"])
    assert inputs == [{"name": "pdf", "source": str(tmp_path / "sample.pdf")}]
    assert env["WENDAO_PDF_RENDER_SHARD_REPORT_DIR"] == str(tmp_path / "reports")
    assert env["WENDAO_PDF_RENDER_SELECTION"] == "all_pages"


def test_pdf_render_artifact_cache_summary_reads_manifest_totals(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    report_dir = tmp_path / "reports"
    report_dir.mkdir()
    manifest_path = report_dir / "pdf_page_render_shard_manifest.json"
    manifest_path.write_text(
        benchmark.json.dumps(
            {
                "totalInputs": 2,
                "totalRenderedShards": 5,
                "renderedInputs": 2,
                "fallbackInputs": 0,
                "artifactCacheBackendCounts": {"foyer": 2},
                "artifactCacheHitCount": 3,
                "artifactCacheMissCount": 2,
                "artifactCacheThrottledCount": 0,
                "artifactCacheByteCount": 4096,
                "records": [
                    {"artifactCacheBackend": "foyer"},
                    {"artifactCacheBackend": "foyer"},
                ],
            }
        ),
        encoding="utf-8",
    )

    summary_path = benchmark.write_pdf_render_artifact_cache_summary(report_dir)
    summary = benchmark.json.loads(summary_path.read_text(encoding="utf-8"))

    assert summary_path.name == "pdf_render_artifact_cache_summary.json"
    assert summary["schema"] == "xiuxian_wendao.pdf_render_artifact_cache_summary.v1"
    assert summary["artifactCacheBackendCounts"] == {"foyer": 2}
    assert summary["artifactCacheConfiguredRecordCount"] == 2
    assert summary["artifactCacheHitCount"] == 3
    assert summary["artifactCacheMissCount"] == 2
    assert summary["artifactCacheThrottledCount"] == 0
    assert summary["artifactCacheByteCount"] == 4096
    assert summary["artifactCacheReuseObserved"] is True
    assert summary["artifactCacheMaterializationObserved"] is True


def test_pdf_render_artifact_cache_summary_falls_back_to_record_counts(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    manifest_path = tmp_path / "pdf_page_render_shard_manifest.json"
    manifest_path.write_text(
        benchmark.json.dumps(
            {
                "records": [
                    {
                        "artifactCacheBackend": "filesystem",
                        "artifactCacheHitCount": 1,
                        "artifactCacheMissCount": 0,
                        "artifactCacheThrottledCount": 0,
                        "artifactCacheByteCount": 100,
                    },
                    {
                        "artifactCacheBackend": "filesystem",
                        "artifactCacheHitCount": 0,
                        "artifactCacheMissCount": 1,
                        "artifactCacheThrottledCount": 0,
                        "artifactCacheByteCount": 120,
                    },
                ]
            }
        ),
        encoding="utf-8",
    )

    summary = benchmark.summarize_pdf_render_artifact_cache(manifest_path)

    assert summary["recordCount"] == 2
    assert summary["artifactCacheBackendCounts"] == {"filesystem": 2}
    assert summary["artifactCacheHitCount"] == 1
    assert summary["artifactCacheMissCount"] == 1
    assert summary["artifactCacheByteCount"] == 220
