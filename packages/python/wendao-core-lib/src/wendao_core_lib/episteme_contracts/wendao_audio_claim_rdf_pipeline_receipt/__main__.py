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


REPORT_SCHEMA_VERSION = "wendao.audio_claim_rdf_pipeline_receipt.v1"


PIPELINE_STEPS = [
    {
        "contract": "audio_claim_acceptance",
        "path_kind": "acceptance_report",
        "schema_version": "wendao.audio_claim_acceptance_report.v1",
        "state_key": "acceptance_state",
        "expected_state": "accepted_for_source_contract_review",
    },
    {
        "contract": "audio_claim_rdf_preview",
        "path_kind": "example",
        "schema_version": "wendao.audio_claim_rdf_patch_preview.v1",
        "state_key": "preview_state",
        "expected_state": "preview_ready",
    },
    {
        "contract": "audio_claim_rdf_source_promotion",
        "path_kind": "example",
        "schema_version": "wendao.audio_claim_rdf_source_promotion_proposal.v1",
        "state_key": "source_promotion_state",
        "expected_state": "ready_for_source_patch",
    },
    {
        "contract": "audio_claim_rdf_staged_apply",
        "path_kind": "example",
        "schema_version": "wendao.audio_claim_rdf_staged_apply_report.v1",
        "state_key": "staged_apply_state",
        "expected_state": "staged",
    },
    {
        "contract": "audio_claim_rdf_source_edit_preflight",
        "path_kind": "example",
        "schema_version": "wendao.audio_claim_rdf_source_edit_preflight_report.v1",
        "state_key": "source_edit_preflight_state",
        "expected_state": "diff_ready",
    },
    {
        "contract": "audio_claim_rdf_source_edit_gate",
        "path_kind": "example",
        "schema_version": "wendao.audio_claim_rdf_source_edit_gate_report.v1",
        "state_key": "source_edit_gate_state",
        "expected_state": "ready_for_manual_source_edit",
    },
    {
        "contract": "audio_claim_rdf_source_apply",
        "path_kind": "example",
        "schema_version": "wendao.audio_claim_rdf_source_apply_report.v1",
        "state_key": "source_apply_state",
        "expected_state": "ready_for_source_apply",
    },
    {
        "contract": "audio_claim_rdf_source_apply_verification",
        "path_kind": "example",
        "schema_version": "wendao.audio_claim_rdf_source_apply_verification_report.v1",
        "state_key": "verification_state",
        "expected_state": "dry_run_verified",
    },
]


MUTATION_FIELDS = (
    "rdf_materialization_performed",
    "ontology_source_write_performed",
    "rdf_source_write_performed",
    "canonical_source_write_performed",
    "canonical_source_write_observed",
    "ontology_truth_promotion_performed",
    "raw_transcript_text_included",
)


def load_contract() -> dict[str, Any]:
    manifest = load_manifest()
    return dict(manifest["audio_claim_rdf_pipeline_receipt"])


def default_output_path() -> Path:
    contract = load_contract()
    return ontology_root() / contract["example"]


def build_pipeline_receipt(*, root: Path | None = None) -> dict[str, Any]:
    ontology = root or ontology_root()
    manifest = load_manifest() if root is None else load_manifest_from_root(ontology)
    loaded_reports: list[dict[str, Any]] = []
    report_entries: list[dict[str, Any]] = []
    errors: list[str] = []

    for step in PIPELINE_STEPS:
        report_path = report_path_for_step(manifest, ontology, step)
        report, load_errors = load_json_object(report_path, step["contract"])
        errors.extend(load_errors)
        if load_errors:
            continue
        loaded_reports.append(report)
        report_entries.append(
            {
                "contract": step["contract"],
                "path": relative_to_root(report_path, ontology),
                "schema_version": str(report.get("schema_version", "")),
                "state": str(report.get(step["state_key"], "")),
                "passed": bool(report.get("passed") is True),
                "sha256": sha256_file(report_path),
            }
        )
        errors.extend(validate_step(report, step))

    errors.extend(validate_proposal_ids(loaded_reports))
    errors.extend(validate_no_raw_or_hidden_content(loaded_reports))
    if errors:
        return rejected_report(report_entries, loaded_reports, errors=errors)

    acceptance = loaded_reports[0]
    preview = loaded_reports[1]
    source_promotion = loaded_reports[2]
    verification = loaded_reports[7]
    return {
        "schema_version": REPORT_SCHEMA_VERSION,
        "proposal_id": acceptance["proposal_id"],
        "pipeline_state": "source_contract_pipeline_ready",
        "passed": True,
        "errors": [],
        "report_count": len(report_entries),
        "source_write_mode": verification["source_write_mode"],
        "canonical_source_write_performed": False,
        "canonical_source_write_observed": verification[
            "canonical_source_write_observed"
        ],
        "ontology_truth_promotion_performed": False,
        "raw_transcript_text_included": False,
        "claim_count": acceptance["claim_count"],
        "patch_count": preview["patch_count"],
        "target_file_count": len(source_promotion["target_files"]),
        "verified_file_count": verification["verified_file_count"],
        "registry_class_count": verification["registry_class_count"],
        "registry_object_property_count": verification[
            "registry_object_property_count"
        ],
        "reports": report_entries,
    }


def load_manifest_from_root(root: Path) -> dict[str, Any]:
    import tomllib

    return tomllib.loads((root / "manifest.toml").read_text())


def report_path_for_step(
    manifest: dict[str, Any],
    root: Path,
    step: dict[str, Any],
) -> Path:
    contract = manifest[step["contract"]]
    if step["path_kind"] == "acceptance_report":
        return root / contract["example"] / "acceptance_report.json"
    return root / contract["example"]


def load_json_object(path: Path, label: str) -> tuple[dict[str, Any], list[str]]:
    if not path.is_file():
        return {}, [f"missing {label} report: {path.name}"]
    try:
        loaded = json.loads(path.read_text())
    except json.JSONDecodeError as error:
        return {}, [f"{label} report is not valid JSON: {error.msg}"]
    if not isinstance(loaded, dict):
        return {}, [f"{label} report must be a JSON object"]
    return loaded, []


def validate_step(report: dict[str, Any], step: dict[str, Any]) -> list[str]:
    errors = []
    contract = step["contract"]
    if report.get("schema_version") != step["schema_version"]:
        errors.append(f"{contract} schema_version is not supported")
    if report.get(step["state_key"]) != step["expected_state"]:
        errors.append(f"{contract} state is not {step['expected_state']}")
    if report.get("passed") is not True:
        errors.append(f"{contract} must have passed=true")
    for field in MUTATION_FIELDS:
        if report.get(field) is not None and report.get(field) is not False:
            errors.append(f"{contract} {field} must be false")
    return errors


def validate_proposal_ids(reports: list[dict[str, Any]]) -> list[str]:
    proposal_ids = {str(report.get("proposal_id", "")) for report in reports}
    if len(proposal_ids) != 1 or "" in proposal_ids:
        return ["pipeline reports must share one non-empty proposal_id"]
    return []


def validate_no_raw_or_hidden_content(reports: list[dict[str, Any]]) -> list[str]:
    payload = "\n".join(iter_string_values(reports)).lower()
    errors = []
    if any(marker in payload for marker in RAW_TRANSCRIPT_MARKERS):
        errors.append("pipeline receipt input includes raw transcript content marker")
    if any(marker.lower() in payload for marker in HIDDEN_PATH_MARKERS):
        errors.append("pipeline receipt input includes hidden workspace path marker")
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
    report_entries: list[dict[str, Any]],
    reports: list[dict[str, Any]],
    *,
    errors: list[str],
) -> dict[str, Any]:
    proposal_id = ""
    for report in reports:
        if str(report.get("proposal_id", "")).strip():
            proposal_id = str(report["proposal_id"])
            break
    return {
        "schema_version": REPORT_SCHEMA_VERSION,
        "proposal_id": proposal_id,
        "pipeline_state": "rejected",
        "passed": False,
        "errors": errors,
        "report_count": len(report_entries),
        "source_write_mode": "dry_run",
        "canonical_source_write_performed": False,
        "canonical_source_write_observed": False,
        "ontology_truth_promotion_performed": False,
        "raw_transcript_text_included": False,
        "claim_count": 0,
        "patch_count": 0,
        "target_file_count": 0,
        "verified_file_count": 0,
        "registry_class_count": 0,
        "registry_object_property_count": 0,
        "reports": report_entries,
    }


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
        description="Compile a read-only receipt for the audio claim RDF source-contract pipeline."
    )
    parser.add_argument(
        "--output",
        default=str(default_output_path()),
        help="Optional pipeline receipt output path.",
    )
    parser.add_argument(
        "--no-write",
        action="store_true",
        help="Print the receipt without writing output.",
    )
    args = parser.parse_args(argv)

    report = build_pipeline_receipt()
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
