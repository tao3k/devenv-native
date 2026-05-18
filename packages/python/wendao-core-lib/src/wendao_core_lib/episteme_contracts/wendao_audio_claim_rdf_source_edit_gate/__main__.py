from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path
from typing import Any

from wendao_core_lib.episteme_contracts.wendao_audio_claim_acceptance.__main__ import (
    HIDDEN_PATH_MARKERS,
    RAW_TRANSCRIPT_MARKERS,
    load_manifest,
    ontology_root,
)


REPORT_SCHEMA_VERSION = "wendao.audio_claim_rdf_source_edit_gate_report.v1"
PREFLIGHT_SCHEMA_VERSION = "wendao.audio_claim_rdf_source_edit_preflight_report.v1"
DECISION_SCHEMA_VERSION = "wendao.audio_claim_source_edit_decision.v1"
APPROVED_REVIEW_STATE = "approved_for_manual_source_edit"


def load_contract() -> dict[str, Any]:
    manifest = load_manifest()
    return dict(manifest["audio_claim_rdf_source_edit_gate"])


def default_preflight_report_path() -> Path:
    manifest = load_manifest()
    preflight_contract = manifest["audio_claim_rdf_source_edit_preflight"]
    return ontology_root() / preflight_contract["example"]


def default_decision_path() -> Path:
    contract = load_contract()
    return ontology_root() / contract["source_edit_decision"]


def default_output_path() -> Path:
    contract = load_contract()
    return ontology_root() / contract["example"]


def build_source_edit_gate_report(
    preflight_report_path: Path,
    decision_path: Path,
) -> dict[str, Any]:
    preflight, preflight_errors = load_json_object(
        preflight_report_path, "preflight report"
    )
    decision, decision_errors = load_json_object(decision_path, "source edit decision")
    errors = validate_preflight(preflight)
    errors.extend(validate_decision(decision, preflight))
    errors.extend(preflight_errors)
    errors.extend(decision_errors)
    if errors:
        return rejected_report(
            preflight, decision, preflight_report_path, errors=errors
        )

    approved_diffs = []
    decision_diff_hashes = {
        (entry["target_rdf_file"], entry["diff_file"]): entry["diff_sha256"]
        for entry in decision["approved_diff_files"]
    }
    for diff_entry in preflight["diff_files"]:
        key = (diff_entry["target_rdf_file"], diff_entry["diff_file"])
        if decision_diff_hashes.get(key) != diff_entry["diff_sha256"]:
            errors.append(f"decision diff hash mismatch for {diff_entry['diff_file']}")
            continue
        current_source_hash = sha256_file(
            ontology_root() / diff_entry["target_rdf_file"]
        )
        if current_source_hash != diff_entry["source_sha256"]:
            errors.append(f"source hash mismatch for {diff_entry['target_rdf_file']}")
            continue
        current_diff_hash = sha256_file(ontology_root() / diff_entry["diff_file"])
        if current_diff_hash != diff_entry["diff_sha256"]:
            errors.append(f"diff artifact hash mismatch for {diff_entry['diff_file']}")
            continue
        approved_diffs.append(
            {
                "target_rdf_file": diff_entry["target_rdf_file"],
                "diff_file": diff_entry["diff_file"],
                "source_sha256": diff_entry["source_sha256"],
                "staged_sha256": diff_entry["staged_sha256"],
                "diff_sha256": diff_entry["diff_sha256"],
                "patch_count": diff_entry["patch_count"],
            }
        )

    if errors:
        return rejected_report(
            preflight, decision, preflight_report_path, errors=errors
        )

    return {
        "schema_version": REPORT_SCHEMA_VERSION,
        "proposal_id": preflight["proposal_id"],
        "source_edit_gate_state": "ready_for_manual_source_edit",
        "passed": True,
        "errors": [],
        "source_edit_decision_id": decision["source_edit_decision_id"],
        "review_state": decision["review_state"],
        "reviewer_id": decision["reviewer_id"],
        "reviewed_at": decision["reviewed_at"],
        "preflight_report": relative_to_ontology(preflight_report_path),
        "canonical_source_write_performed": False,
        "ontology_truth_promotion_performed": False,
        "raw_transcript_text_included": False,
        "manual_source_edit_required": True,
        "approved_diff_count": len(approved_diffs),
        "approved_diffs": sorted(
            approved_diffs, key=lambda item: item["target_rdf_file"]
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


def validate_preflight(preflight: dict[str, Any]) -> list[str]:
    errors = []
    if preflight.get("schema_version") != PREFLIGHT_SCHEMA_VERSION:
        errors.append("preflight report schema_version is not supported")
    if preflight.get("source_edit_preflight_state") != "diff_ready":
        errors.append("preflight report must be diff_ready")
    if preflight.get("passed") is not True:
        errors.append("preflight report must have passed=true")
    for key in (
        "canonical_source_write_performed",
        "ontology_truth_promotion_performed",
        "raw_transcript_text_included",
    ):
        if preflight.get(key) is not False:
            errors.append(f"preflight report {key} must be false")
    if not isinstance(preflight.get("diff_files"), list) or not preflight.get(
        "diff_files"
    ):
        errors.append("preflight report diff_files must be a non-empty list")
    return errors


def validate_decision(decision: dict[str, Any], preflight: dict[str, Any]) -> list[str]:
    errors = []
    if decision.get("schema_version") != DECISION_SCHEMA_VERSION:
        errors.append("source edit decision schema_version is not supported")
    for key in (
        "source_edit_decision_id",
        "proposal_id",
        "review_state",
        "reviewer_id",
        "reviewed_at",
    ):
        if not str(decision.get(key, "")).strip():
            errors.append(f"source edit decision {key} is required")
    if decision.get("proposal_id") != preflight.get("proposal_id"):
        errors.append("source edit decision proposal_id does not match preflight")
    if decision.get("review_state") != APPROVED_REVIEW_STATE:
        errors.append("source edit decision must approve manual source edit")
    for key in (
        "canonical_source_write_allowed",
        "ontology_truth_promotion_allowed",
        "raw_transcript_text_allowed",
    ):
        if decision.get(key) is not False:
            errors.append(f"source edit decision {key} must be false")
    if not isinstance(decision.get("approved_diff_files"), list) or not decision.get(
        "approved_diff_files"
    ):
        errors.append(
            "source edit decision approved_diff_files must be a non-empty list"
        )
    errors.extend(validate_no_raw_or_hidden_content(decision))
    return errors


def validate_no_raw_or_hidden_content(value: Any) -> list[str]:
    payload = "\n".join(iter_string_values(value)).lower()
    errors = []
    if any(marker in payload for marker in RAW_TRANSCRIPT_MARKERS):
        errors.append("source edit decision includes raw transcript content marker")
    if any(marker.lower() in payload for marker in HIDDEN_PATH_MARKERS):
        errors.append("source edit decision includes hidden workspace path marker")
    return errors


def iter_string_values(value: Any) -> list[str]:
    if isinstance(value, str):
        return [value]
    if isinstance(value, list):
        return [item for entry in value for item in iter_string_values(entry)]
    if isinstance(value, dict):
        return [item for entry in value.values() for item in iter_string_values(entry)]
    return []


def rejected_report(
    preflight: dict[str, Any],
    decision: dict[str, Any],
    preflight_report_path: Path,
    *,
    errors: list[str],
) -> dict[str, Any]:
    return {
        "schema_version": REPORT_SCHEMA_VERSION,
        "proposal_id": str(preflight.get("proposal_id", "")),
        "source_edit_gate_state": "rejected",
        "passed": False,
        "errors": errors,
        "source_edit_decision_id": str(decision.get("source_edit_decision_id", "")),
        "review_state": str(decision.get("review_state", "")),
        "reviewer_id": str(decision.get("reviewer_id", "")),
        "reviewed_at": str(decision.get("reviewed_at", "")),
        "preflight_report": relative_to_ontology(preflight_report_path),
        "canonical_source_write_performed": False,
        "ontology_truth_promotion_performed": False,
        "raw_transcript_text_included": False,
        "manual_source_edit_required": True,
        "approved_diff_count": 0,
        "approved_diffs": [],
    }


def sha256_file(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def relative_to_ontology(path: Path) -> str:
    try:
        return path.resolve().relative_to(ontology_root()).as_posix()
    except ValueError:
        return path.name


def report_text(report: dict[str, Any]) -> str:
    return json.dumps(report, indent=2, sort_keys=True) + "\n"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Validate source-edit diff approval without writing canonical RDF."
    )
    parser.add_argument(
        "--preflight-report",
        default=str(default_preflight_report_path()),
        help="Source-edit preflight report JSON path.",
    )
    parser.add_argument(
        "--decision",
        default=str(default_decision_path()),
        help="Source edit decision JSON path.",
    )
    parser.add_argument(
        "--output",
        default=str(default_output_path()),
        help="Optional source edit gate report output path.",
    )
    parser.add_argument(
        "--no-write",
        action="store_true",
        help="Print the report without writing output.",
    )
    args = parser.parse_args(argv)

    report = build_source_edit_gate_report(
        Path(args.preflight_report),
        Path(args.decision),
    )
    text = report_text(report)
    if args.no_write:
        sys.stdout.write(text)
    else:
        output = Path(args.output)
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(text)
        sys.stdout.write(text)
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
