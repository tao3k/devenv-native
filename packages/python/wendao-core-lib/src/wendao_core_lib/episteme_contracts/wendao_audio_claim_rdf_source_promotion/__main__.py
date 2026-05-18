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
from wendao_core_lib.episteme_contracts.wendao_audio_claim_rdf_preview.__main__ import (
    build_rdf_patch_preview,
)
from wendao_core_lib.episteme_contracts.wendao_ontology_registry.__main__ import (
    build_registry,
)


PROPOSAL_SCHEMA_VERSION = "wendao.audio_claim_rdf_source_promotion_proposal.v1"
REVIEW_DECISION_SCHEMA_VERSION = "wendao.audio_claim_source_review_decision.v1"
APPROVED_REVIEW_STATE = "approved_for_source_patch_proposal"
INSERT_BEFORE = "</rdf:RDF>"


def load_contract() -> dict[str, Any]:
    manifest = load_manifest()
    return dict(manifest["audio_claim_rdf_source_promotion"])


def default_proposal_dir() -> Path:
    manifest = load_manifest()
    acceptance_contract = manifest["audio_claim_acceptance"]
    return ontology_root() / acceptance_contract["example"]


def default_review_decision_path() -> Path:
    contract = load_contract()
    return ontology_root() / contract["review_decision"]


def build_source_promotion_proposal(
    proposal_dir: Path,
    review_decision_path: Path,
) -> dict[str, Any]:
    preview = build_rdf_patch_preview(proposal_dir)
    if not preview["passed"]:
        return rejected_proposal(preview, errors=list(preview["errors"]))

    decision, decision_errors = load_review_decision(review_decision_path)
    errors = validate_review_decision(decision, preview)
    errors.extend(decision_errors)
    if errors:
        return rejected_proposal(preview, decision=decision, errors=errors)

    predicate_sources = predicate_source_files()
    target_file_hashes: dict[str, str] = {}
    patches = []
    for patch in preview["patches"]:
        target_file = predicate_sources.get(patch["ontology_predicate"])
        if target_file is None:
            errors.append(
                f"claim {patch['claim_id']} ontology_predicate has no source file"
            )
            continue
        precondition = target_file_hashes.setdefault(
            target_file,
            sha256_file(ontology_root() / target_file),
        )
        patches.append(
            {
                "claim_id": patch["claim_id"],
                "evidence_segment_id": patch["evidence_segment_id"],
                "target_rdf_file": target_file,
                "target_file_precondition_sha256": precondition,
                "insert_before": INSERT_BEFORE,
                "rdf_xml_preview": patch["rdf_xml_preview"],
            }
        )

    if errors:
        return rejected_proposal(preview, decision=decision, errors=errors)

    target_files = [
        {"path": path, "precondition_sha256": digest}
        for path, digest in sorted(target_file_hashes.items())
    ]
    return {
        "schema_version": PROPOSAL_SCHEMA_VERSION,
        "proposal_id": preview["proposal_id"],
        "source_promotion_state": "ready_for_source_patch",
        "passed": True,
        "errors": [],
        "review_decision_id": decision["review_decision_id"],
        "review_state": decision["review_state"],
        "reviewer_id": decision["reviewer_id"],
        "reviewed_at": decision["reviewed_at"],
        "patch_count": len(patches),
        "rdf_source_write_performed": False,
        "ontology_truth_promotion_performed": False,
        "raw_transcript_text_included": False,
        "target_files": target_files,
        "patches": sorted(patches, key=lambda item: item["claim_id"]),
    }


def load_review_decision(path: Path) -> tuple[dict[str, Any], list[str]]:
    if not path.is_file():
        return {}, [f"missing review decision: {path.name}"]
    try:
        loaded = json.loads(path.read_text())
    except json.JSONDecodeError as error:
        return {}, [f"review decision is not valid JSON: {error.msg}"]
    if not isinstance(loaded, dict):
        return {}, ["review decision must be a JSON object"]
    return loaded, []


def validate_review_decision(
    decision: dict[str, Any],
    preview: dict[str, Any],
) -> list[str]:
    errors = []
    if decision.get("schema_version") != REVIEW_DECISION_SCHEMA_VERSION:
        errors.append("review decision schema_version is not supported")
    for key in (
        "review_decision_id",
        "proposal_id",
        "review_state",
        "reviewer_id",
        "reviewed_at",
    ):
        if not str(decision.get(key, "")).strip():
            errors.append(f"review decision {key} is required")
    if decision.get("proposal_id") != preview["proposal_id"]:
        errors.append("review decision proposal_id does not match preview")
    if decision.get("review_state") != APPROVED_REVIEW_STATE:
        errors.append("review decision must approve source patch proposal generation")
    for key in (
        "rdf_source_write_allowed",
        "ontology_truth_promotion_allowed",
        "raw_transcript_text_allowed",
    ):
        if decision.get(key) is not False:
            errors.append(f"review decision {key} must be false")
    approved_claim_ids = decision.get("approved_claim_ids")
    if not isinstance(approved_claim_ids, list):
        errors.append("review decision approved_claim_ids must be a list")
    else:
        expected_claim_ids = sorted(patch["claim_id"] for patch in preview["patches"])
        if sorted(approved_claim_ids) != expected_claim_ids:
            errors.append(
                "review decision approved_claim_ids do not match preview patches"
            )
    errors.extend(validate_no_raw_or_hidden_content(decision))
    return errors


def validate_no_raw_or_hidden_content(value: Any) -> list[str]:
    payload = "\n".join(iter_string_values(value)).lower()
    errors = []
    if any(marker in payload for marker in RAW_TRANSCRIPT_MARKERS):
        errors.append("review decision includes raw transcript content marker")
    if any(marker.lower() in payload for marker in HIDDEN_PATH_MARKERS):
        errors.append("review decision includes hidden workspace path marker")
    return errors


def iter_string_values(value: Any) -> list[str]:
    if isinstance(value, str):
        return [value]
    if isinstance(value, list):
        return [item for entry in value for item in iter_string_values(entry)]
    if isinstance(value, dict):
        return [item for entry in value.values() for item in iter_string_values(entry)]
    return []


def predicate_source_files() -> dict[str, str]:
    registry = build_registry()
    return {
        entry["iri"]: entry["source_file"]
        for entry in registry["rdf_terms"]["object_properties"]
    }


def sha256_file(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def rejected_proposal(
    preview: dict[str, Any],
    *,
    decision: dict[str, Any] | None = None,
    errors: list[str],
) -> dict[str, Any]:
    decision = decision or {}
    return {
        "schema_version": PROPOSAL_SCHEMA_VERSION,
        "proposal_id": preview.get("proposal_id", ""),
        "source_promotion_state": "rejected",
        "passed": False,
        "errors": errors,
        "review_decision_id": str(decision.get("review_decision_id", "")),
        "review_state": str(decision.get("review_state", "")),
        "reviewer_id": str(decision.get("reviewer_id", "")),
        "reviewed_at": str(decision.get("reviewed_at", "")),
        "patch_count": 0,
        "rdf_source_write_performed": False,
        "ontology_truth_promotion_performed": False,
        "raw_transcript_text_included": False,
        "target_files": [],
        "patches": [],
    }


def proposal_text(report: dict[str, Any]) -> str:
    return json.dumps(report, indent=2, sort_keys=True) + "\n"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Compile a preview-only RDF source-patch proposal from an approved "
            "audio claim RDF preview."
        )
    )
    parser.add_argument(
        "--proposal-dir",
        default=str(default_proposal_dir()),
        help="Proposal directory containing claims.tsv and receipt.json.",
    )
    parser.add_argument(
        "--review-decision",
        default=str(default_review_decision_path()),
        help="Review decision JSON path.",
    )
    parser.add_argument(
        "--output",
        help="Optional JSON output path for the source promotion proposal.",
    )
    args = parser.parse_args(argv)

    report = build_source_promotion_proposal(
        Path(args.proposal_dir),
        Path(args.review_decision),
    )
    report_text = proposal_text(report)
    if args.output:
        Path(args.output).write_text(report_text)
    else:
        sys.stdout.write(report_text)
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
