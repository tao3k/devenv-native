from __future__ import annotations

import argparse
import difflib
import hashlib
import json
import sys
from pathlib import Path
from typing import Any

from wendao_core_lib.episteme_contracts.wendao_audio_claim_acceptance.__main__ import (
    load_manifest,
    ontology_root,
)


REPORT_SCHEMA_VERSION = "wendao.audio_claim_rdf_source_edit_preflight_report.v1"
STAGED_REPORT_SCHEMA_VERSION = "wendao.audio_claim_rdf_staged_apply_report.v1"


def load_contract() -> dict[str, Any]:
    manifest = load_manifest()
    return dict(manifest["audio_claim_rdf_source_edit_preflight"])


def default_staged_report_path() -> Path:
    manifest = load_manifest()
    staged_contract = manifest["audio_claim_rdf_staged_apply"]
    return ontology_root() / staged_contract["example"]


def default_output_dir() -> Path:
    contract = load_contract()
    return ontology_root() / contract["diff_output"]


def build_source_edit_preflight_report(
    staged_report_path: Path,
    output_dir: Path,
    *,
    write_artifacts: bool = False,
) -> dict[str, Any]:
    staged_report, load_errors = load_staged_report(staged_report_path)
    errors = validate_staged_report(staged_report)
    errors.extend(load_errors)
    if errors:
        return rejected_report(
            staged_report, staged_report_path, output_dir, errors=errors
        )

    diff_files = []
    rendered_diffs: dict[Path, str] = {}
    for staged_file in staged_report["staged_files"]:
        target_path = ontology_root() / staged_file["target_rdf_file"]
        staged_path = ontology_root() / staged_file["staged_rdf_file"]
        source_hash = sha256_file(target_path)
        if source_hash != staged_file["source_precondition_sha256"]:
            errors.append(f"source hash mismatch for {staged_file['target_rdf_file']}")
            continue
        staged_hash = sha256_file(staged_path)
        if staged_hash != staged_file["staged_sha256"]:
            errors.append(f"staged hash mismatch for {staged_file['staged_rdf_file']}")
            continue
        diff_text = unified_diff(
            staged_file["target_rdf_file"],
            staged_file["staged_rdf_file"],
            target_path.read_text(),
            staged_path.read_text(),
        )
        diff_path = output_dir / f"{staged_file['target_rdf_file']}.diff"
        rendered_diffs[diff_path] = diff_text
        diff_files.append(
            {
                "target_rdf_file": staged_file["target_rdf_file"],
                "staged_rdf_file": staged_file["staged_rdf_file"],
                "diff_file": relative_to_ontology(diff_path),
                "source_sha256": source_hash,
                "staged_sha256": staged_hash,
                "diff_sha256": sha256_text(diff_text),
                "patch_count": staged_file["patch_count"],
            }
        )

    if errors:
        return rejected_report(
            staged_report, staged_report_path, output_dir, errors=errors
        )

    report = {
        "schema_version": REPORT_SCHEMA_VERSION,
        "proposal_id": staged_report["proposal_id"],
        "source_edit_preflight_state": "diff_ready",
        "passed": True,
        "errors": [],
        "staged_apply_report": relative_to_ontology(staged_report_path),
        "diff_output_dir": relative_to_ontology(output_dir),
        "diff_file_count": len(diff_files),
        "canonical_source_write_performed": False,
        "ontology_truth_promotion_performed": False,
        "raw_transcript_text_included": False,
        "diff_files": sorted(diff_files, key=lambda item: item["target_rdf_file"]),
    }
    if write_artifacts:
        write_preflight_artifacts(output_dir, rendered_diffs, report)
    return report


def load_staged_report(path: Path) -> tuple[dict[str, Any], list[str]]:
    if not path.is_file():
        return {}, [f"missing staged apply report: {path.name}"]
    try:
        loaded = json.loads(path.read_text())
    except json.JSONDecodeError as error:
        return {}, [f"staged apply report is not valid JSON: {error.msg}"]
    if not isinstance(loaded, dict):
        return {}, ["staged apply report must be a JSON object"]
    return loaded, []


def validate_staged_report(report: dict[str, Any]) -> list[str]:
    errors = []
    if report.get("schema_version") != STAGED_REPORT_SCHEMA_VERSION:
        errors.append("staged apply report schema_version is not supported")
    if report.get("staged_apply_state") != "staged":
        errors.append("staged apply report must be staged")
    if report.get("passed") is not True:
        errors.append("staged apply report must have passed=true")
    for key in (
        "canonical_source_write_performed",
        "ontology_truth_promotion_performed",
        "raw_transcript_text_included",
    ):
        if report.get(key) is not False:
            errors.append(f"staged apply report {key} must be false")
    if not isinstance(report.get("staged_files"), list) or not report.get(
        "staged_files"
    ):
        errors.append("staged apply report staged_files must be a non-empty list")
    return errors


def unified_diff(
    source_path: str,
    staged_path: str,
    source_text: str,
    staged_text: str,
) -> str:
    diff = difflib.unified_diff(
        source_text.splitlines(keepends=True),
        staged_text.splitlines(keepends=True),
        fromfile=source_path,
        tofile=staged_path,
    )
    return "".join(diff)


def write_preflight_artifacts(
    output_dir: Path,
    rendered_diffs: dict[Path, str],
    report: dict[str, Any],
) -> None:
    for path, text in rendered_diffs.items():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
    output_dir.mkdir(parents=True, exist_ok=True)
    (output_dir / "report.json").write_text(report_text(report))


def rejected_report(
    staged_report: dict[str, Any],
    staged_report_path: Path,
    output_dir: Path,
    *,
    errors: list[str],
) -> dict[str, Any]:
    return {
        "schema_version": REPORT_SCHEMA_VERSION,
        "proposal_id": str(staged_report.get("proposal_id", "")),
        "source_edit_preflight_state": "rejected",
        "passed": False,
        "errors": errors,
        "staged_apply_report": relative_to_ontology(staged_report_path),
        "diff_output_dir": relative_to_ontology(output_dir),
        "diff_file_count": 0,
        "canonical_source_write_performed": False,
        "ontology_truth_promotion_performed": False,
        "raw_transcript_text_included": False,
        "diff_files": [],
    }


def sha256_file(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def sha256_text(text: str) -> str:
    return "sha256:" + hashlib.sha256(text.encode()).hexdigest()


def relative_to_ontology(path: Path) -> str:
    try:
        return path.resolve().relative_to(ontology_root()).as_posix()
    except ValueError:
        return path.name


def report_text(report: dict[str, Any]) -> str:
    return json.dumps(report, indent=2, sort_keys=True) + "\n"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Compile a source-edit diff preflight from a staged RDF apply report."
    )
    parser.add_argument(
        "--staged-report",
        default=str(default_staged_report_path()),
        help="Staged apply report JSON path.",
    )
    parser.add_argument(
        "--output-dir",
        default=str(default_output_dir()),
        help="Diff preflight output directory.",
    )
    parser.add_argument(
        "--no-write",
        action="store_true",
        help="Print the report without writing diff artifacts.",
    )
    args = parser.parse_args(argv)

    report = build_source_edit_preflight_report(
        Path(args.staged_report),
        Path(args.output_dir),
        write_artifacts=not args.no_write,
    )
    sys.stdout.write(report_text(report))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
