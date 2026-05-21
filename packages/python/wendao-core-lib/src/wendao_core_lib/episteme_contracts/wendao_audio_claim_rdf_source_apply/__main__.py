from __future__ import annotations

import argparse
import hashlib
import json
import sys
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import Any

from wendao_core_lib.episteme_contracts.wendao_audio_claim_acceptance.__main__ import (
    load_manifest,
    ontology_root,
)


REPORT_SCHEMA_VERSION = "wendao.audio_claim_rdf_source_apply_report.v1"
GATE_SCHEMA_VERSION = "wendao.audio_claim_rdf_source_edit_gate_report.v1"
PREFLIGHT_SCHEMA_VERSION = "wendao.audio_claim_rdf_source_edit_preflight_report.v1"


def load_contract() -> dict[str, Any]:
    manifest = load_manifest()
    return dict(manifest["audio_claim_rdf_source_apply"])


def default_gate_report_path() -> Path:
    manifest = load_manifest()
    gate_contract = manifest["audio_claim_rdf_source_edit_gate"]
    return ontology_root() / gate_contract["example"]


def default_output_path() -> Path:
    contract = load_contract()
    return ontology_root() / contract["example"]


def build_source_apply_report(
    gate_report_path: Path,
    *,
    write_source: bool = False,
    target_root: Path | None = None,
) -> dict[str, Any]:
    root = target_root or ontology_root()
    gate, gate_errors = load_json_object(gate_report_path, "source edit gate report")
    errors = validate_gate_report(gate)
    errors.extend(gate_errors)
    if errors:
        return rejected_report(
            gate, gate_report_path, root, write_source=write_source, errors=errors
        )

    preflight_path = resolve_under_root(root, gate["preflight_report"])
    preflight, preflight_errors = load_json_object(
        preflight_path, "source edit preflight report"
    )
    errors.extend(validate_preflight_report(preflight))
    errors.extend(preflight_errors)
    if errors:
        return rejected_report(
            gate, gate_report_path, root, write_source=write_source, errors=errors
        )

    preflight_by_key = {
        (entry["target_rdf_file"], entry["diff_file"]): entry
        for entry in preflight["diff_files"]
    }
    applied_files = []
    pending_writes: dict[Path, str] = {}
    for approved in gate["approved_diffs"]:
        key = (approved["target_rdf_file"], approved["diff_file"])
        preflight_entry = preflight_by_key.get(key)
        if preflight_entry is None:
            errors.append(
                f"approved diff missing from preflight: {approved['diff_file']}"
            )
            continue
        errors.extend(
            validate_approved_diff_against_preflight(approved, preflight_entry)
        )
        if errors:
            continue

        target_path = resolve_under_root(root, approved["target_rdf_file"])
        staged_path = resolve_under_root(root, preflight_entry["staged_rdf_file"])
        diff_path = resolve_under_root(root, approved["diff_file"])
        if not target_path.is_file():
            errors.append(f"missing target RDF file: {approved['target_rdf_file']}")
            continue
        if not staged_path.is_file():
            errors.append(
                f"missing staged RDF file: {preflight_entry['staged_rdf_file']}"
            )
            continue
        if not diff_path.is_file():
            errors.append(f"missing diff file: {approved['diff_file']}")
            continue

        current_source_hash = sha256_file(target_path)
        staged_hash = sha256_file(staged_path)
        diff_hash = sha256_file(diff_path)
        if current_source_hash != approved["source_sha256"]:
            errors.append(f"source hash mismatch for {approved['target_rdf_file']}")
            continue
        if staged_hash != approved["staged_sha256"]:
            errors.append(
                f"staged hash mismatch for {preflight_entry['staged_rdf_file']}"
            )
            continue
        if diff_hash != approved["diff_sha256"]:
            errors.append(f"diff artifact hash mismatch for {approved['diff_file']}")
            continue

        staged_text = staged_path.read_text()
        try:
            parse_rdf_text(staged_text, preflight_entry["staged_rdf_file"])
        except ValueError as error:
            errors.append(str(error))
            continue
        if write_source:
            pending_writes[target_path] = staged_text
        applied_files.append(
            {
                "target_rdf_file": approved["target_rdf_file"],
                "staged_rdf_file": preflight_entry["staged_rdf_file"],
                "diff_file": approved["diff_file"],
                "previous_source_sha256": current_source_hash,
                "applied_source_sha256": staged_hash,
                "diff_sha256": diff_hash,
                "patch_count": approved["patch_count"],
                "source_write_performed": write_source,
            }
        )

    if errors:
        return rejected_report(
            gate, gate_report_path, root, write_source=write_source, errors=errors
        )

    for path, text in pending_writes.items():
        path.write_text(text)

    return {
        "schema_version": REPORT_SCHEMA_VERSION,
        "proposal_id": gate["proposal_id"],
        "source_apply_state": "applied" if write_source else "ready_for_source_apply",
        "passed": True,
        "errors": [],
        "source_edit_gate_report": relative_to_root(gate_report_path, root),
        "source_write_mode": "write_source" if write_source else "dry_run",
        "canonical_source_write_performed": write_source,
        "ontology_truth_promotion_performed": False,
        "raw_transcript_text_included": False,
        "applied_file_count": len(applied_files),
        "applied_files": sorted(
            applied_files, key=lambda item: item["target_rdf_file"]
        ),
    }


def load_json_object(path: Path, label: str) -> tuple[dict[str, Any], list[str]]:
    if not path.is_file():
        return {}, [f"missing {label}: {path.name}"]
    try:
        loaded = json.loads(path.read_text())
    except json.JSONDecodeError as error:
        return {}, [f"{label} is not valid JSON: {error.msg}"]
    if not isinstance(loaded, dict):
        return {}, [f"{label} must be a JSON object"]
    return loaded, []


def validate_gate_report(gate: dict[str, Any]) -> list[str]:
    errors = []
    if gate.get("schema_version") != GATE_SCHEMA_VERSION:
        errors.append("source edit gate report schema_version is not supported")
    if gate.get("source_edit_gate_state") != "ready_for_manual_source_edit":
        errors.append("source edit gate report must be ready_for_manual_source_edit")
    if gate.get("passed") is not True:
        errors.append("source edit gate report must have passed=true")
    if gate.get("manual_source_edit_required") is not True:
        errors.append("source edit gate report must require manual source edit")
    for key in (
        "canonical_source_write_performed",
        "ontology_truth_promotion_performed",
        "raw_transcript_text_included",
    ):
        if gate.get(key) is not False:
            errors.append(f"source edit gate report {key} must be false")
    if not isinstance(gate.get("approved_diffs"), list) or not gate.get(
        "approved_diffs"
    ):
        errors.append("source edit gate report approved_diffs must be a non-empty list")
    return errors


def validate_preflight_report(preflight: dict[str, Any]) -> list[str]:
    errors = []
    if preflight.get("schema_version") != PREFLIGHT_SCHEMA_VERSION:
        errors.append("source edit preflight report schema_version is not supported")
    if preflight.get("source_edit_preflight_state") != "diff_ready":
        errors.append("source edit preflight report must be diff_ready")
    if preflight.get("passed") is not True:
        errors.append("source edit preflight report must have passed=true")
    for key in (
        "canonical_source_write_performed",
        "ontology_truth_promotion_performed",
        "raw_transcript_text_included",
    ):
        if preflight.get(key) is not False:
            errors.append(f"source edit preflight report {key} must be false")
    if not isinstance(preflight.get("diff_files"), list) or not preflight.get(
        "diff_files"
    ):
        errors.append(
            "source edit preflight report diff_files must be a non-empty list"
        )
    return errors


def validate_approved_diff_against_preflight(
    approved: dict[str, Any],
    preflight_entry: dict[str, Any],
) -> list[str]:
    errors = []
    for gate_key, preflight_key in (
        ("source_sha256", "source_sha256"),
        ("staged_sha256", "staged_sha256"),
        ("diff_sha256", "diff_sha256"),
        ("patch_count", "patch_count"),
    ):
        if approved.get(gate_key) != preflight_entry.get(preflight_key):
            errors.append(
                f"approved diff {gate_key} mismatch for {approved.get('diff_file', '')}"
            )
    return errors


def rejected_report(
    gate: dict[str, Any],
    gate_report_path: Path,
    root: Path,
    *,
    write_source: bool,
    errors: list[str],
) -> dict[str, Any]:
    return {
        "schema_version": REPORT_SCHEMA_VERSION,
        "proposal_id": str(gate.get("proposal_id", "")),
        "source_apply_state": "rejected",
        "passed": False,
        "errors": errors,
        "source_edit_gate_report": relative_to_root(gate_report_path, root),
        "source_write_mode": "write_source" if write_source else "dry_run",
        "canonical_source_write_performed": False,
        "ontology_truth_promotion_performed": False,
        "raw_transcript_text_included": False,
        "applied_file_count": 0,
        "applied_files": [],
    }


def parse_rdf_text(text: str, source_label: str) -> None:
    try:
        ET.fromstring(text)
    except ET.ParseError as error:
        raise ValueError(
            f"staged RDF is not valid XML for {source_label}: {error}"
        ) from error


def resolve_under_root(root: Path, value: str | Path) -> Path:
    path = Path(value)
    if path.is_absolute():
        return path
    return root / path


def sha256_file(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def relative_to_root(path: Path, root: Path) -> str:
    try:
        return path.resolve().relative_to(root.resolve()).as_posix()
    except ValueError:
        return path.name


def report_text(report: dict[str, Any]) -> str:
    return json.dumps(report, indent=2, sort_keys=True) + "\n"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Apply a source-edit gate report into canonical RDF source files."
    )
    parser.add_argument(
        "--gate-report",
        default=str(default_gate_report_path()),
        help="Source edit gate report JSON path.",
    )
    parser.add_argument(
        "--output",
        default=str(default_output_path()),
        help="Optional source apply report output path.",
    )
    parser.add_argument(
        "--write-source",
        action="store_true",
        help="Write approved staged RDF files into canonical RDF source files.",
    )
    parser.add_argument(
        "--no-write",
        action="store_true",
        help="Keep canonical RDF source files unchanged. This is the default.",
    )
    args = parser.parse_args(argv)

    report = build_source_apply_report(
        Path(args.gate_report),
        write_source=args.write_source and not args.no_write,
    )
    text = report_text(report)
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(text)
    sys.stdout.write(text)
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
