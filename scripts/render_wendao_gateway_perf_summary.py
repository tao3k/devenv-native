#!/usr/bin/env python3
"""Render xiuxian-wendao gateway perf summary from persisted perf reports."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

SUMMARY_SCHEMA = "xiuxian_wendao.gateway_perf_summary.v1"
PERF_REPORT_SCHEMA = "wendao.perf-report.v1"
PERF_GATEWAY_SUITE = "xiuxian-wendao/perf-gateway"
REAL_WORKSPACE_PERF_GATEWAY_SUITE = "xiuxian-wendao/perf-gateway-real-workspace"
FORMAL_GATEWAY_CASES = (
    "repo_module_search_formal",
    "repo_symbol_search_formal",
    "repo_example_search_formal",
    "repo_projected_page_search_formal",
    "studio_code_search_formal",
    "studio_search_index_status_formal",
)
REAL_WORKSPACE_GATEWAY_CASES = (
    "repo_index_status_real_workspace_sample",
    "studio_code_search_real_workspace_sample",
)
GATEWAY_URI_METADATA_KEY = "gateway_uri"
GATEWAY_SEARCH_INDEX_METADATA_KEY = "gateway_search_index"
GATEWAY_REPO_INDEX_METADATA_KEY = "gateway_repo_index"
GATEWAY_METADATA_PREFIX = "gateway_"
SUMMARY_JSON_NAME = "gateway_perf_summary.json"
SUMMARY_MARKDOWN_NAME = "gateway_perf_summary.md"


def _load_json(path: Path) -> tuple[dict[str, Any] | None, str | None]:
    if not path.exists():
        return None, "missing"
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:  # pragma: no cover - defensive error shaping
        return None, f"parse_error:{exc}"
    if not isinstance(payload, dict):
        return None, "invalid_payload"
    return payload, None


def _as_float(value: Any, default: float = 0.0) -> float:
    if isinstance(value, bool):
        return default
    if isinstance(value, (int, float)):
        return float(value)
    if isinstance(value, str) and value.strip():
        try:
            return float(value.strip())
        except ValueError:
            return default
    return default


def _as_int(value: Any, default: int = 0) -> int:
    if isinstance(value, bool):
        return default
    if isinstance(value, int):
        return value
    if isinstance(value, float):
        return int(value)
    if isinstance(value, str) and value.strip():
        try:
            return int(value.strip())
        except ValueError:
            return default
    return default


def _case_diagnostics(
    metadata: dict[str, Any],
) -> tuple[str, str, str, dict[str, str], str]:
    uri = str(metadata.get(GATEWAY_URI_METADATA_KEY, "")).strip()
    search_index = str(metadata.get(GATEWAY_SEARCH_INDEX_METADATA_KEY, "")).strip()
    repo_index = str(metadata.get(GATEWAY_REPO_INDEX_METADATA_KEY, "")).strip()
    extra: dict[str, str] = {}
    for key, value in metadata.items():
        if not key.startswith(GATEWAY_METADATA_PREFIX):
            continue
        if key in {
            GATEWAY_URI_METADATA_KEY,
            GATEWAY_SEARCH_INDEX_METADATA_KEY,
            GATEWAY_REPO_INDEX_METADATA_KEY,
        }:
            continue
        extra[key.removeprefix(GATEWAY_METADATA_PREFIX)] = str(value)

    parts: list[str] = []
    parts.append(f"uri={uri or 'none'}")
    if search_index:
        parts.append(f"searchIndex={search_index}")
    if repo_index:
        parts.append(f"repoIndex={repo_index}")
    for key, value in sorted(extra.items()):
        parts.append(f"{key}={value}")
    return uri, search_index, repo_index, extra, "; ".join(parts)


def _extract_repo_read_pressure(search_index: str, extra: dict[str, str]) -> str:
    explicit = extra.get("repoReadPressure", "").strip()
    if explicit:
        return explicit
    marker = " repoRead="
    suffix = " queryTelemetry="
    start = search_index.find(marker)
    if start < 0:
        return "none"
    start += len(marker)
    end = search_index.find(suffix, start)
    if end < 0:
        extracted = search_index[start:].strip()
    else:
        extracted = search_index[start:end].strip()
    return extracted or "none"


def _build_case_summary(report_path: Path, payload: dict[str, Any]) -> dict[str, Any]:
    summary = payload.get("summary", {})
    quantiles = payload.get("quantiles", {})
    metadata = payload.get("metadata", {})
    if not isinstance(summary, dict):
        summary = {}
    if not isinstance(quantiles, dict):
        quantiles = {}
    if not isinstance(metadata, dict):
        metadata = {}

    uri, search_index, repo_index, extra, diagnostics = _case_diagnostics(metadata)
    return {
        "case": str(payload.get("case", "")).strip(),
        "suite": str(payload.get("suite", "")).strip(),
        "captured_at_unix_ms": _as_int(payload.get("captured_at_unix_ms")),
        "report_path": str(report_path),
        "p95_ms": _as_float(quantiles.get("p95_ms")),
        "p99_ms": _as_float(quantiles.get("p99_ms")),
        "throughput_qps": _as_float(summary.get("throughput_qps")),
        "error_rate": _as_float(summary.get("error_rate")),
        "uri": uri,
        "search_index": search_index,
        "repo_index": repo_index,
        "repo_read_pressure": _extract_repo_read_pressure(search_index, extra),
        "extra": extra,
        "diagnostics": diagnostics,
    }


def _collect_suite_summary(
    *,
    report_dir: Path,
    suite: str,
    expected_cases: tuple[str, ...] = FORMAL_GATEWAY_CASES,
    optional: bool = False,
) -> tuple[dict[str, Any], list[str]]:
    errors: list[str] = []
    expected_case_set = set(expected_cases)
    latest_cases: dict[str, dict[str, Any]] = {}
    auxiliary_cases: dict[str, dict[str, Any]] = {}

    if not report_dir.exists():
        if optional:
            return (
                {
                    "report_dir": str(report_dir),
                    "expected_cases": list(expected_cases),
                    "cases": [],
                    "auxiliary_cases": [],
                    "missing_cases": list(expected_cases),
                    "available": False,
                    "overall": {
                        "ok": True,
                        "case_count": 0,
                        "auxiliary_case_count": 0,
                        "expected_case_count": len(expected_cases),
                        "error_count": 0,
                    },
                },
                [],
            )
        errors.append(f"report_dir_missing:{report_dir}")
    for path in sorted(report_dir.glob("*.json")):
        if path.name == "gateway_perf_summary.json":
            continue
        payload, error = _load_json(path)
        if error is not None:
            errors.append(f"{path.name}:{error}")
            continue
        assert payload is not None
        schema_version = str(payload.get("schema_version", "")).strip()
        if schema_version != PERF_REPORT_SCHEMA:
            errors.append(f"{path.name}:schema_mismatch:{schema_version!r}")
            continue
        report_suite = str(payload.get("suite", "")).strip()
        if report_suite != suite:
            continue
        case = str(payload.get("case", "")).strip()
        if not case:
            errors.append(f"{path.name}:missing_case")
            continue
        case_summary = _build_case_summary(path, payload)
        if case not in expected_case_set:
            current = auxiliary_cases.get(case)
            if (
                current is None
                or case_summary["captured_at_unix_ms"] >= current["captured_at_unix_ms"]
            ):
                auxiliary_cases[case] = case_summary
            continue
        current = latest_cases.get(case)
        if (
            current is None
            or case_summary["captured_at_unix_ms"] >= current["captured_at_unix_ms"]
        ):
            latest_cases[case] = case_summary

    cases = sorted(latest_cases.values(), key=lambda item: str(item["case"]))
    auxiliary = sorted(auxiliary_cases.values(), key=lambda item: str(item["case"]))
    missing_cases = [case for case in expected_cases if case not in latest_cases]
    payload: dict[str, Any] = {
        "report_dir": str(report_dir),
        "expected_cases": list(expected_cases),
        "cases": cases,
        "auxiliary_cases": auxiliary,
        "missing_cases": missing_cases,
        "available": True,
        "overall": {
            "ok": not errors and not missing_cases,
            "case_count": len(cases),
            "auxiliary_case_count": len(auxiliary),
            "expected_case_count": len(expected_cases),
            "error_count": len(errors),
        },
    }
    return payload, errors


def render_gateway_perf_summary(
    *,
    report_dir: Path,
    real_workspace_report_dir: Path | None,
    runner_os: str,
    expected_cases: tuple[str, ...] = FORMAL_GATEWAY_CASES,
    real_workspace_expected_cases: tuple[str, ...] = REAL_WORKSPACE_GATEWAY_CASES,
) -> tuple[dict[str, Any], list[str]]:
    formal, formal_errors = _collect_suite_summary(
        report_dir=report_dir,
        suite=PERF_GATEWAY_SUITE,
        expected_cases=expected_cases,
    )
    real_workspace_dir = (
        real_workspace_report_dir
        if real_workspace_report_dir is not None
        else report_dir.parent / "perf-gateway-real-workspace"
    )
    real_workspace, real_workspace_errors = _collect_suite_summary(
        report_dir=real_workspace_dir,
        suite=REAL_WORKSPACE_PERF_GATEWAY_SUITE,
        expected_cases=real_workspace_expected_cases,
        optional=True,
    )
    errors = formal_errors + real_workspace_errors
    payload: dict[str, Any] = {
        "schema": SUMMARY_SCHEMA,
        "runner_os": runner_os,
        "formal": formal,
        "real_workspace": real_workspace,
        "overall": {
            "ok": bool(formal.get("overall", {}).get("ok", False)) and not errors,
            "formal_ok": bool(formal.get("overall", {}).get("ok", False)),
            "formal_case_count": int(formal.get("overall", {}).get("case_count", 0)),
            "real_workspace_available": bool(real_workspace.get("available", False)),
            "real_workspace_case_count": int(
                real_workspace.get("overall", {}).get("case_count", 0)
            ),
            "error_count": len(errors),
        },
        "errors": errors,
    }
    return payload, errors


def _build_markdown(payload: dict[str, Any]) -> str:
    formal = payload.get("formal", {})
    real_workspace = payload.get("real_workspace", {})
    cases = formal.get("cases", [])
    auxiliary_cases = formal.get("auxiliary_cases", [])
    missing_cases = formal.get("missing_cases", [])
    errors = payload.get("errors", [])
    overall = payload.get("overall", {})

    lines = [
        f"## Wendao Gateway Perf Summary ({payload.get('runner_os', 'unknown')})",
        "",
        (
            f"- Formal reports present: `{overall.get('formal_case_count', 0)}/"
            f"{formal.get('overall', {}).get('expected_case_count', 0)}`"
        ),
        f"- Summary healthy: `{str(overall.get('ok', False)).lower()}`",
        (
            f"- Missing expected cases: `{', '.join(missing_cases) if missing_cases else 'none'}`"
        ),
        (
            f"- Ignored non-formal cases: "
            f"`{', '.join(entry.get('case', 'unknown') for entry in auxiliary_cases) if auxiliary_cases else 'none'}`"
        ),
        "",
        "### Formal Warm-Cache Cases",
        "",
    ]
    if cases:
        lines.extend(
            [
                "| Case | P95 (ms) | P99 (ms) | QPS | Error Rate | URI |",
                "| --- | ---: | ---: | ---: | ---: | --- |",
            ]
        )
        for case in cases:
            lines.append(
                "| {case} | {p95:.3f} | {p99:.3f} | {qps:.1f} | {error_rate:.4f} | `{uri}` |".format(
                    case=case.get("case", "unknown"),
                    p95=_as_float(case.get("p95_ms")),
                    p99=_as_float(case.get("p99_ms")),
                    qps=_as_float(case.get("throughput_qps")),
                    error_rate=_as_float(case.get("error_rate")),
                    uri=case.get("uri", "") or "none",
                )
            )
        lines.extend(["", "### Diagnostics", ""])
        for case in cases:
            lines.append(
                f"- `{case.get('case', 'unknown')}`: {case.get('diagnostics', 'none')}"
            )
        lines.extend(["", "### Formal Repo-Read Pressure", ""])
        for case in cases:
            lines.append(
                f"- `{case.get('case', 'unknown')}`: {case.get('repo_read_pressure', 'none')}"
            )
        lines.append("")
    else:
        lines.extend(["- No gateway perf reports found.", ""])

    lines.extend(["### Real Workspace Samples", ""])
    if real_workspace.get("available", False):
        real_cases = real_workspace.get("cases", [])
        if real_cases:
            lines.extend(
                [
                    "| Case | P95 (ms) | P99 (ms) | QPS | Error Rate | URI |",
                    "| --- | ---: | ---: | ---: | ---: | --- |",
                ]
            )
            for case in real_cases:
                lines.append(
                    "| {case} | {p95:.3f} | {p99:.3f} | {qps:.1f} | {error_rate:.4f} | `{uri}` |".format(
                        case=case.get("case", "unknown"),
                        p95=_as_float(case.get("p95_ms")),
                        p99=_as_float(case.get("p99_ms")),
                        qps=_as_float(case.get("throughput_qps")),
                        error_rate=_as_float(case.get("error_rate")),
                        uri=case.get("uri", "") or "none",
                    )
                )
            lines.extend(["", "- Real-workspace diagnostics:", ""])
            for case in real_cases:
                lines.append(
                    f"  - `{case.get('case', 'unknown')}`: {case.get('diagnostics', 'none')}"
                )
            lines.extend(["", "- Real-workspace repo-read pressure:", ""])
            for case in real_cases:
                lines.append(
                    f"  - `{case.get('case', 'unknown')}`: {case.get('repo_read_pressure', 'none')}"
                )
            lines.append("")
        else:
            lines.extend(
                [
                    "- Real-workspace report directory exists, but no sample reports were found.",
                    "",
                ]
            )
    else:
        lines.extend(["- Real-workspace samples not present in this run.", ""])

    if errors:
        lines.extend(["### Errors", ""])
        for error in errors:
            lines.append(f"- `{error}`")
        lines.append("")
    return "\n".join(lines)


def _write_summary_outputs(
    *,
    payload: dict[str, Any],
    markdown: str,
    output_json: Path | None,
    output_markdown: Path | None,
    mirror_output_dir: Path | None,
) -> None:
    if output_json is not None:
        output_json.parent.mkdir(parents=True, exist_ok=True)
        output_json.write_text(
            json.dumps(payload, ensure_ascii=True, indent=2) + "\n", encoding="utf-8"
        )
    if output_markdown is not None:
        output_markdown.parent.mkdir(parents=True, exist_ok=True)
        output_markdown.write_text(markdown, encoding="utf-8")
    if mirror_output_dir is not None:
        mirror_output_dir.mkdir(parents=True, exist_ok=True)
        (mirror_output_dir / SUMMARY_JSON_NAME).write_text(
            json.dumps(payload, ensure_ascii=True, indent=2) + "\n", encoding="utf-8"
        )
        (mirror_output_dir / SUMMARY_MARKDOWN_NAME).write_text(
            markdown, encoding="utf-8"
        )


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Render xiuxian-wendao gateway perf summary"
    )
    parser.add_argument(
        "--report-dir",
        default=".run/reports/xiuxian-wendao/perf-gateway",
        help="gateway perf report directory",
    )
    parser.add_argument(
        "--real-workspace-report-dir",
        default=".run/reports/xiuxian-wendao/perf-gateway-real-workspace",
        help="real-workspace gateway perf report directory",
    )
    parser.add_argument("--runner-os", default="", help="runner os label for summary")
    parser.add_argument("--output-json", default="", help="optional output JSON path")
    parser.add_argument(
        "--output-markdown", default="", help="optional output markdown path"
    )
    parser.add_argument(
        "--mirror-output-dir",
        default="",
        help="optional directory that also receives gateway_perf_summary.json/.md",
    )
    args = parser.parse_args()

    report_dir = Path(str(args.report_dir)).expanduser().resolve()
    real_workspace_report_dir = (
        Path(str(args.real_workspace_report_dir)).expanduser().resolve()
        if str(args.real_workspace_report_dir).strip()
        else None
    )
    output_json = (
        Path(str(args.output_json)).expanduser().resolve()
        if str(args.output_json).strip()
        else None
    )
    output_markdown = (
        Path(str(args.output_markdown)).expanduser().resolve()
        if str(args.output_markdown).strip()
        else None
    )
    mirror_output_dir = (
        Path(str(args.mirror_output_dir)).expanduser().resolve()
        if str(args.mirror_output_dir).strip()
        else None
    )

    payload, _errors = render_gateway_perf_summary(
        report_dir=report_dir,
        real_workspace_report_dir=real_workspace_report_dir,
        runner_os=str(args.runner_os).strip() or "local",
    )
    markdown = _build_markdown(payload)
    _write_summary_outputs(
        payload=payload,
        markdown=markdown,
        output_json=output_json,
        output_markdown=output_markdown,
        mirror_output_dir=mirror_output_dir,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
