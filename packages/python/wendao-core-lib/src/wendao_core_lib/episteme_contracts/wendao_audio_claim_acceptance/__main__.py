from __future__ import annotations

import argparse
import csv
import json
import sys
import tomllib
from pathlib import Path
from typing import Any

from wendao_core_lib.episteme_contracts.paths import episteme_root, ontology_root
from wendao_core_lib.episteme_contracts.wendao_ontology_registry.__main__ import (
    build_registry,
)


PROPOSAL_SCHEMA_VERSION = "xiuxian_wendao.episteme_audio_claim_promotion_proposal.v1"
REPORT_SCHEMA_VERSION = "wendao.audio_claim_acceptance_report.v1"
CLAIMS_TSV = "claims.tsv"
RECEIPT_JSON = "receipt.json"
EXPECTED_CLAIM_COLUMNS = [
    "claim_id",
    "evidence_segment_id",
    "ontology_subject",
    "ontology_predicate",
    "ontology_object",
    "object_kind",
    "reviewer_id",
    "reviewed_at",
    "evidence_quote_sha256",
    "review_note_sha256",
    "confidence",
    "status",
]
OBJECT_KINDS = {"entity", "literal", "quantity"}
HIDDEN_PATH_MARKERS = (
    "/Users/",
    ".cache/",
    "$PRJ_CACHE_HOME",
    "$PRJ_DATA_HOME",
    "$PRJ_RUNTIME_DIR",
)
RAW_TRANSCRIPT_MARKERS = ("raw_transcript", "transcript_text")


def load_manifest() -> dict[str, Any]:
    return tomllib.loads((ontology_root() / "manifest.toml").read_text())


def load_contract() -> dict[str, Any]:
    manifest = load_manifest()
    return dict(manifest["audio_claim_acceptance"])


def known_object_properties() -> set[str]:
    registry = build_registry()
    return {entry["iri"] for entry in registry["rdf_terms"]["object_properties"]}


def read_claim_rows(path: Path) -> tuple[list[dict[str, str]], list[str]]:
    errors = []
    with path.open(newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        if reader.fieldnames != EXPECTED_CLAIM_COLUMNS:
            errors.append(f"claims.tsv columns must be {EXPECTED_CLAIM_COLUMNS}")
            return [], errors
        rows = list(reader)
    return rows, errors


def build_acceptance_report(proposal_dir: Path) -> dict[str, Any]:
    contract = load_contract()
    proposal_dir = proposal_dir.resolve()
    receipt_path = proposal_dir / RECEIPT_JSON
    claims_path = proposal_dir / CLAIMS_TSV
    errors: list[str] = []
    receipt: dict[str, Any] = {}
    rows: list[dict[str, str]] = []

    if not claims_path.is_file():
        errors.append(f"missing {CLAIMS_TSV}")
    else:
        rows, row_errors = read_claim_rows(claims_path)
        errors.extend(row_errors)

    if not receipt_path.is_file():
        errors.append(f"missing {RECEIPT_JSON}")
    else:
        receipt = json.loads(receipt_path.read_text())
        errors.extend(validate_receipt(receipt))

    known_predicates = known_object_properties()
    errors.extend(validate_claim_rows(rows, known_predicates))
    if receipt:
        errors.extend(validate_receipt_counts(receipt, rows))

    passed = not errors
    return {
        "schema_version": REPORT_SCHEMA_VERSION,
        "proposal_id": receipt.get("proposal_id", ""),
        "acceptance_state": contract["acceptance_state"] if passed else "rejected",
        "passed": passed,
        "errors": errors,
        "claim_count": len(rows),
        "evidence_segment_count": len({row["evidence_segment_id"] for row in rows}),
        "known_predicate_count": sum(
            1 for row in rows if row["ontology_predicate"] in known_predicates
        ),
        "rdf_materialization_performed": False,
        "ontology_source_write_performed": False,
        "raw_transcript_text_included": False,
    }


def validate_receipt(receipt: dict[str, Any]) -> list[str]:
    errors = []
    if receipt.get("schema_version") != PROPOSAL_SCHEMA_VERSION:
        errors.append("receipt schema_version is not supported")
    for key in (
        "proposal_id",
        "claims_path",
        "receipt_path",
    ):
        if not str(receipt.get(key, "")).strip():
            errors.append(f"receipt {key} is required")
    for key in (
        "rdf_materialization_performed",
        "ontology_source_write_performed",
        "raw_transcript_promotion_allowed",
    ):
        if receipt.get(key) is not False:
            errors.append(f"receipt {key} must be false")
    return errors


def validate_receipt_counts(
    receipt: dict[str, Any], rows: list[dict[str, str]]
) -> list[str]:
    errors = []
    if receipt.get("claim_count") != len(rows):
        errors.append("receipt claim_count does not match claims.tsv")
    evidence_segment_count = len({row["evidence_segment_id"] for row in rows})
    if receipt.get("evidence_segment_count") != evidence_segment_count:
        errors.append("receipt evidence_segment_count does not match claims.tsv")
    return errors


def validate_claim_rows(
    rows: list[dict[str, str]], known_predicates: set[str]
) -> list[str]:
    errors = []
    seen_claim_ids = set()
    seen_segment_ids = set()
    for index, row in enumerate(rows, start=2):
        for key in EXPECTED_CLAIM_COLUMNS:
            if key == "review_note_sha256":
                continue
            if not row[key].strip():
                errors.append(f"row {index} {key} is required")
        if row["claim_id"] in seen_claim_ids:
            errors.append(f"row {index} duplicate claim_id: {row['claim_id']}")
        seen_claim_ids.add(row["claim_id"])
        seen_segment_ids.add(row["evidence_segment_id"])
        if row["object_kind"] not in OBJECT_KINDS:
            errors.append(f"row {index} object_kind is not supported")
        if row["status"] != "promotion-candidate":
            errors.append(f"row {index} status must be promotion-candidate")
        if row["ontology_predicate"] not in known_predicates:
            errors.append(
                f"row {index} ontology_predicate is not a known RDF object property"
            )
        errors.extend(validate_confidence(row["confidence"], index))
        errors.extend(validate_no_raw_or_hidden_content(row, index))
    if not rows:
        errors.append("claims.tsv must contain at least one claim row")
    if len(seen_segment_ids) != len(rows):
        errors.append(
            "each accepted proposal row must reference one distinct evidence_segment_id"
        )
    return errors


def validate_confidence(raw: str, row_number: int) -> list[str]:
    try:
        confidence = float(raw)
    except ValueError:
        return [f"row {row_number} confidence is not numeric"]
    if not 0.0 <= confidence <= 1.0:
        return [f"row {row_number} confidence must be within [0, 1]"]
    return []


def validate_no_raw_or_hidden_content(
    row: dict[str, str], row_number: int
) -> list[str]:
    errors = []
    for key, value in row.items():
        lowered_key = key.lower()
        lowered_value = value.lower()
        if any(
            marker in lowered_key or marker in lowered_value
            for marker in RAW_TRANSCRIPT_MARKERS
        ):
            errors.append(f"row {row_number} includes raw transcript content marker")
        if any(marker.lower() in lowered_value for marker in HIDDEN_PATH_MARKERS):
            errors.append(f"row {row_number} includes hidden workspace path marker")
    return errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Validate a reviewed audio claim proposal."
    )
    parser.add_argument(
        "--proposal-dir",
        default=str(ontology_root() / load_contract()["example"]),
        help="Proposal directory containing claims.tsv and receipt.json.",
    )
    parser.add_argument(
        "--output",
        help="Optional acceptance report JSON path.",
    )
    args = parser.parse_args(argv)

    report = build_acceptance_report(Path(args.proposal_dir))
    report_text = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        Path(args.output).write_text(report_text)
    else:
        sys.stdout.write(report_text)
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
