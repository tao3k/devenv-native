from __future__ import annotations

import importlib.util
import json
import tempfile
from pathlib import Path

_MODULE_PATH = (
    Path(__file__).resolve().with_name("render_wendao_gateway_perf_summary.py")
)
_MODULE_SPEC = importlib.util.spec_from_file_location(
    "render_wendao_gateway_perf_summary", _MODULE_PATH
)
assert _MODULE_SPEC is not None
assert _MODULE_SPEC.loader is not None
_MODULE = importlib.util.module_from_spec(_MODULE_SPEC)
_MODULE_SPEC.loader.exec_module(_MODULE)
FORMAL_GATEWAY_CASES = _MODULE.FORMAL_GATEWAY_CASES
REAL_WORKSPACE_GATEWAY_CASES = _MODULE.REAL_WORKSPACE_GATEWAY_CASES
SUMMARY_JSON_NAME = _MODULE.SUMMARY_JSON_NAME
SUMMARY_MARKDOWN_NAME = _MODULE.SUMMARY_MARKDOWN_NAME
render_gateway_perf_summary = _MODULE.render_gateway_perf_summary
build_markdown = _MODULE._build_markdown
write_summary_outputs = _MODULE._write_summary_outputs


def _write_json(path: Path, payload: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(payload, ensure_ascii=True, indent=2) + "\n", encoding="utf-8"
    )


def _report_payload(
    *,
    case: str,
    captured_at_unix_ms: int,
    p95_ms: float,
    p99_ms: float,
    throughput_qps: float,
    error_rate: float = 0.0,
    uri: str,
    extra: dict[str, str] | None = None,
) -> dict[str, object]:
    metadata: dict[str, object] = {
        "crate": "xiuxian-wendao",
        "gateway_uri": uri,
        "gateway_search_index": "total=6 idle=4 indexing=0 ready=2 degraded=0 failed=0 compactionPending=0 statusReason=none maintenance=none repoRead=budget=2, inFlight=1, requested=177, searchable=96, parallelism=2, fanoutCapped=true queryTelemetry=none",
        "gateway_repo_index": "total=1 ready=1 active=0 queued=0 checking=0 syncing=0 indexing=0 unsupported=0 failed=0 currentRepoId=none",
    }
    for key, value in (extra or {}).items():
        metadata[f"gateway_{key}"] = value
    return {
        "schema_version": "wendao.perf-report.v1",
        "suite": "xiuxian-wendao/perf-gateway",
        "case": case,
        "mode": "async",
        "captured_at_unix_ms": captured_at_unix_ms,
        "run_config": {
            "warmup_samples": 1,
            "samples": 6,
            "timeout_ms": 2_000,
            "concurrency": 1,
        },
        "summary": {
            "total_ops": 6,
            "success_ops": 6,
            "timeout_ops": 0,
            "error_ops": 0,
            "error_rate": error_rate,
            "throughput_qps": throughput_qps,
            "elapsed_ms": 12.0,
        },
        "quantiles": {
            "min_ms": p95_ms / 2.0,
            "mean_ms": p95_ms / 1.5,
            "max_ms": p99_ms,
            "p50_ms": p95_ms / 1.8,
            "p95_ms": p95_ms,
            "p99_ms": p99_ms,
        },
        "sample_latency_ms": [p95_ms],
        "metadata": metadata,
        "report_path": str(
            Path(tempfile.gettempdir()) / f"{case}-{captured_at_unix_ms}.json"
        ),
    }


def test_render_gateway_perf_summary_selects_latest_per_case_and_extracts_diagnostics(
    tmp_path: Path,
) -> None:
    formal_dir = tmp_path / "formal"
    real_workspace_dir = tmp_path / "real-workspace"
    _write_json(
        formal_dir / "repo_module_search_formal-100.json",
        _report_payload(
            case="repo_module_search_formal",
            captured_at_unix_ms=100,
            p95_ms=1.40,
            p99_ms=1.60,
            throughput_qps=500.0,
            uri="/api/repo/module-search?repo=gateway-sync&query=solve",
        ),
    )
    _write_json(
        formal_dir / "repo_module_search_formal-200.json",
        _report_payload(
            case="repo_module_search_formal",
            captured_at_unix_ms=200,
            p95_ms=1.05,
            p99_ms=1.25,
            throughput_qps=620.0,
            uri="/api/repo/module-search?repo=gateway-sync&query=solve",
        ),
    )
    _write_json(
        formal_dir / "studio_search_index_status_formal-300.json",
        _report_payload(
            case="studio_search_index_status_formal",
            captured_at_unix_ms=300,
            p95_ms=0.22,
            p99_ms=0.24,
            throughput_qps=2100.0,
            uri="/api/search/index/status",
            extra={
                "statusGatePressure": "maintenance=none; scopes=[gateway-sync(...)]"
            },
        ),
    )
    _write_json(
        formal_dir / "metadata_attach-400.json",
        _report_payload(
            case="metadata_attach",
            captured_at_unix_ms=400,
            p95_ms=0.001,
            p99_ms=0.001,
            throughput_qps=48000.0,
            uri="/api/repo/module-search?repo=gateway-sync&query=solve",
        ),
    )
    _write_json(
        real_workspace_dir / "repo_index_status_real_workspace_sample-500.json",
        _report_payload(
            case="repo_index_status_real_workspace_sample",
            captured_at_unix_ms=500,
            p95_ms=1.33,
            p99_ms=1.33,
            throughput_qps=670.0,
            uri="/api/repo/index/status",
            extra={
                "minRepos": "150",
                "repoReadPressure": "budget=4, inFlight=2, requested=177, searchable=64, parallelism=4, fanoutCapped=true",
            },
        )
        | {"suite": "xiuxian-wendao/perf-gateway-real-workspace"},
    )

    payload, errors = render_gateway_perf_summary(
        report_dir=formal_dir,
        real_workspace_report_dir=real_workspace_dir,
        runner_os="linux",
    )

    assert errors == []
    assert payload["overall"]["formal_case_count"] == 2
    assert payload["overall"]["real_workspace_available"] is True
    assert payload["overall"]["real_workspace_case_count"] == 1
    assert payload["formal"]["overall"]["auxiliary_case_count"] == 1
    assert payload["formal"]["overall"]["expected_case_count"] == len(
        FORMAL_GATEWAY_CASES
    )
    assert "repo_symbol_search_formal" in payload["formal"]["missing_cases"]
    assert payload["formal"]["auxiliary_cases"][0]["case"] == "metadata_attach"
    cases = {entry["case"]: entry for entry in payload["formal"]["cases"]}
    assert cases["repo_module_search_formal"]["captured_at_unix_ms"] == 200
    assert cases["repo_module_search_formal"]["p95_ms"] == 1.05
    assert cases["repo_module_search_formal"]["throughput_qps"] == 620.0
    assert (
        cases["repo_module_search_formal"]["repo_read_pressure"]
        == "budget=2, inFlight=1, requested=177, searchable=96, parallelism=2, fanoutCapped=true"
    )
    assert (
        cases["studio_search_index_status_formal"]["extra"]["statusGatePressure"]
        == "maintenance=none; scopes=[gateway-sync(...)]"
    )
    assert (
        "statusGatePressure=maintenance=none; scopes=[gateway-sync(...)]"
        in cases["studio_search_index_status_formal"]["diagnostics"]
    )
    real_cases = {entry["case"]: entry for entry in payload["real_workspace"]["cases"]}
    assert (
        real_cases["repo_index_status_real_workspace_sample"]["extra"]["minRepos"]
        == "150"
    )
    assert (
        real_cases["repo_index_status_real_workspace_sample"]["repo_read_pressure"]
        == "budget=4, inFlight=2, requested=177, searchable=64, parallelism=4, fanoutCapped=true"
    )
    assert payload["real_workspace"]["overall"]["expected_case_count"] == len(
        REAL_WORKSPACE_GATEWAY_CASES
    )


def test_render_gateway_perf_summary_records_errors_and_formats_markdown(
    tmp_path: Path,
) -> None:
    formal_dir = tmp_path / "formal"
    _write_json(
        formal_dir / "repo_symbol_search_formal-100.json",
        _report_payload(
            case="repo_symbol_search_formal",
            captured_at_unix_ms=100,
            p95_ms=1.10,
            p99_ms=1.30,
            throughput_qps=700.0,
            uri="/api/repo/symbol-search?repo=gateway-sync&query=solve",
        ),
    )
    (formal_dir / "broken.json").write_text("{not-json}\n", encoding="utf-8")

    payload, errors = render_gateway_perf_summary(
        report_dir=formal_dir,
        real_workspace_report_dir=tmp_path / "missing-real-workspace",
        runner_os="local",
    )
    markdown = build_markdown(payload)

    assert len(errors) == 1
    assert errors[0].startswith("broken.json:parse_error:")
    assert payload["overall"]["ok"] is False
    assert "## Wendao Gateway Perf Summary (local)" in markdown
    assert "| repo_symbol_search_formal | 1.100 | 1.300 | 700.0 | 0.0000 |" in markdown
    assert "### Formal Repo-Read Pressure" in markdown
    assert (
        "- `repo_symbol_search_formal`: budget=2, inFlight=1, requested=177, searchable=96, parallelism=2, fanoutCapped=true"
        in markdown
    )
    assert "- Ignored non-formal cases: `none`" in markdown
    assert "- Real-workspace samples not present in this run." in markdown
    assert "`broken.json:parse_error:" in markdown


def test_write_summary_outputs_mirrors_unified_summary_into_secondary_directory(
    tmp_path: Path,
) -> None:
    formal_dir = tmp_path / "formal"
    mirror_dir = tmp_path / "real-workspace"
    payload = {
        "schema": "xiuxian_wendao.gateway_perf_summary.v1",
        "runner_os": "local",
        "formal": {"cases": [], "auxiliary_cases": [], "missing_cases": []},
        "real_workspace": {"cases": [], "auxiliary_cases": [], "missing_cases": []},
        "overall": {"ok": True},
        "errors": [],
    }
    markdown = "## Wendao Gateway Perf Summary (local)\n"
    output_json = formal_dir / SUMMARY_JSON_NAME
    output_markdown = formal_dir / SUMMARY_MARKDOWN_NAME

    write_summary_outputs(
        payload=payload,
        markdown=markdown,
        output_json=output_json,
        output_markdown=output_markdown,
        mirror_output_dir=mirror_dir,
    )

    assert json.loads(output_json.read_text(encoding="utf-8")) == payload
    assert output_markdown.read_text(encoding="utf-8") == markdown
    assert (
        json.loads((mirror_dir / SUMMARY_JSON_NAME).read_text(encoding="utf-8"))
        == payload
    )
    assert (mirror_dir / SUMMARY_MARKDOWN_NAME).read_text(encoding="utf-8") == markdown
