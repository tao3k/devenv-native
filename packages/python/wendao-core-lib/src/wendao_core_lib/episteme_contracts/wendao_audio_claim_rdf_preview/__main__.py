from __future__ import annotations

import argparse
import json
import sys
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import Any

from wendao_core_lib.episteme_contracts.wendao_audio_claim_acceptance.__main__ import (
    CLAIMS_TSV,
    build_acceptance_report,
    load_manifest,
    ontology_root,
    read_claim_rows,
)


PREVIEW_SCHEMA_VERSION = "wendao.audio_claim_rdf_patch_preview.v1"
RDF_NS = "http://www.w3.org/1999/02/22-rdf-syntax-ns#"
CORE_NS = "https://wendao.ai/ontology/core#"

ET.register_namespace("rdf", RDF_NS)
ET.register_namespace("core", CORE_NS)


def default_proposal_dir() -> Path:
    manifest = load_manifest()
    acceptance_contract = manifest["audio_claim_acceptance"]
    return ontology_root() / acceptance_contract["example"]


def build_rdf_patch_preview(proposal_dir: Path) -> dict[str, Any]:
    acceptance_report = build_acceptance_report(proposal_dir)
    if not acceptance_report["passed"]:
        return rejected_preview(acceptance_report)

    rows, errors = read_claim_rows(proposal_dir / CLAIMS_TSV)
    if errors:
        return rejected_preview({**acceptance_report, "errors": errors})

    patches = [build_patch(row) for row in rows]
    return {
        "schema_version": PREVIEW_SCHEMA_VERSION,
        "proposal_id": acceptance_report["proposal_id"],
        "preview_state": "preview_ready",
        "passed": True,
        "errors": [],
        "patch_count": len(patches),
        "rdf_source_write_performed": False,
        "ontology_truth_promotion_performed": False,
        "raw_transcript_text_included": False,
        "patches": patches,
    }


def rejected_preview(acceptance_report: dict[str, Any]) -> dict[str, Any]:
    return {
        "schema_version": PREVIEW_SCHEMA_VERSION,
        "proposal_id": acceptance_report.get("proposal_id", ""),
        "preview_state": "rejected",
        "passed": False,
        "errors": list(acceptance_report.get("errors", [])),
        "patch_count": 0,
        "rdf_source_write_performed": False,
        "ontology_truth_promotion_performed": False,
        "raw_transcript_text_included": False,
        "patches": [],
    }


def build_patch(row: dict[str, str]) -> dict[str, Any]:
    return {
        "claim_id": row["claim_id"],
        "evidence_segment_id": row["evidence_segment_id"],
        "ontology_subject": row["ontology_subject"],
        "ontology_predicate": row["ontology_predicate"],
        "ontology_object": row["ontology_object"],
        "object_kind": row["object_kind"],
        "reviewer_id": row["reviewer_id"],
        "reviewed_at": row["reviewed_at"],
        "confidence": float(row["confidence"]),
        "rdf_xml_preview": build_rdf_statement_preview(row),
    }


def build_rdf_statement_preview(row: dict[str, str]) -> str:
    statement = ET.Element(
        f"{{{RDF_NS}}}Statement",
        {f"{{{RDF_NS}}}about": f"episteme://audio-claim-preview/{row['claim_id']}"},
    )
    ET.SubElement(
        statement,
        f"{{{RDF_NS}}}subject",
        {f"{{{RDF_NS}}}resource": row["ontology_subject"]},
    )
    ET.SubElement(
        statement,
        f"{{{RDF_NS}}}predicate",
        {f"{{{RDF_NS}}}resource": row["ontology_predicate"]},
    )
    object_attributes = {f"{{{RDF_NS}}}resource": row["ontology_object"]}
    if row["object_kind"] != "entity":
        object_attributes = {}
    object_node = ET.SubElement(statement, f"{{{RDF_NS}}}object", object_attributes)
    if row["object_kind"] != "entity":
        object_node.text = row["ontology_object"]
    ET.SubElement(
        statement,
        f"{{{CORE_NS}}}basedOn",
        {f"{{{RDF_NS}}}resource": row["evidence_segment_id"]},
    )
    reviewer = ET.SubElement(statement, f"{{{CORE_NS}}}reviewedBy")
    reviewer.text = row["reviewer_id"]
    confidence = ET.SubElement(statement, f"{{{CORE_NS}}}confidence")
    confidence.text = row["confidence"]
    return ET.tostring(statement, encoding="unicode", short_empty_elements=True)


def preview_text(report: dict[str, Any]) -> str:
    return json.dumps(report, indent=2, sort_keys=True) + "\n"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Compile a preview-only RDF patch from an accepted audio claim proposal."
    )
    parser.add_argument(
        "--proposal-dir",
        default=str(default_proposal_dir()),
        help="Proposal directory containing claims.tsv and receipt.json.",
    )
    parser.add_argument(
        "--output",
        help="Optional JSON output path for the preview artifact.",
    )
    args = parser.parse_args(argv)

    report = build_rdf_patch_preview(Path(args.proposal_dir))
    report_text = preview_text(report)
    if args.output:
        Path(args.output).write_text(report_text)
    else:
        sys.stdout.write(report_text)
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
