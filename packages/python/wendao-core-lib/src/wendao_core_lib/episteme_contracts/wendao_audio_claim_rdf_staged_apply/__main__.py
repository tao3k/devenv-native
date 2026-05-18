from __future__ import annotations

import argparse
import hashlib
import json
import sys
import xml.etree.ElementTree as ET
from collections import defaultdict
from pathlib import Path
from typing import Any

from wendao_core_lib.episteme_contracts.wendao_audio_claim_acceptance.__main__ import (
    load_manifest,
    ontology_root,
)


REPORT_SCHEMA_VERSION = "wendao.audio_claim_rdf_staged_apply_report.v1"
SOURCE_PROPOSAL_SCHEMA_VERSION = "wendao.audio_claim_rdf_source_promotion_proposal.v1"
INSERT_BEFORE = "</rdf:RDF>"


def load_contract() -> dict[str, Any]:
    manifest = load_manifest()
    return dict(manifest["audio_claim_rdf_staged_apply"])


def default_source_promotion_path() -> Path:
    manifest = load_manifest()
    source_contract = manifest["audio_claim_rdf_source_promotion"]
    return ontology_root() / source_contract["example"]


def default_output_dir() -> Path:
    contract = load_contract()
    return ontology_root() / contract["staged_output"]


def build_staged_apply_report(
    source_promotion_path: Path,
    output_dir: Path,
    *,
    write_artifacts: bool = False,
) -> dict[str, Any]:
    proposal, load_errors = load_source_promotion_proposal(source_promotion_path)
    errors = validate_source_promotion_proposal(proposal)
    errors.extend(load_errors)
    if errors:
        return rejected_report(
            proposal, source_promotion_path, output_dir, errors=errors
        )

    grouped_patches: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for patch in proposal["patches"]:
        grouped_patches[patch["target_rdf_file"]].append(patch)

    staged_files = []
    rendered_files: dict[Path, str] = {}
    for target_file, patches in sorted(grouped_patches.items()):
        source_path = ontology_root() / target_file
        current_hash = sha256_file(source_path)
        expected_hash = proposal_precondition_for_file(proposal, target_file)
        if current_hash != expected_hash:
            errors.append(f"precondition mismatch for {target_file}")
            continue
        rendered = render_staged_rdf(source_path.read_text(), patches)
        parse_rdf_text(rendered, target_file)
        staged_path = output_dir / target_file
        rendered_files[staged_path] = rendered
        staged_files.append(
            {
                "target_rdf_file": target_file,
                "staged_rdf_file": relative_to_ontology(staged_path),
                "source_precondition_sha256": current_hash,
                "staged_sha256": sha256_text(rendered),
                "patch_count": len(patches),
            }
        )

    if errors:
        return rejected_report(
            proposal, source_promotion_path, output_dir, errors=errors
        )

    report = {
        "schema_version": REPORT_SCHEMA_VERSION,
        "proposal_id": proposal["proposal_id"],
        "staged_apply_state": "staged",
        "passed": True,
        "errors": [],
        "source_promotion_proposal": relative_to_ontology(source_promotion_path),
        "staged_output_dir": relative_to_ontology(output_dir),
        "staged_file_count": len(staged_files),
        "canonical_source_write_performed": False,
        "ontology_truth_promotion_performed": False,
        "raw_transcript_text_included": False,
        "staged_files": staged_files,
    }
    if write_artifacts:
        write_staged_artifacts(output_dir, rendered_files, report)
    return report


def load_source_promotion_proposal(path: Path) -> tuple[dict[str, Any], list[str]]:
    if not path.is_file():
        return {}, [f"missing source promotion proposal: {path.name}"]
    try:
        loaded = json.loads(path.read_text())
    except json.JSONDecodeError as error:
        return {}, [f"source promotion proposal is not valid JSON: {error.msg}"]
    if not isinstance(loaded, dict):
        return {}, ["source promotion proposal must be a JSON object"]
    return loaded, []


def validate_source_promotion_proposal(proposal: dict[str, Any]) -> list[str]:
    errors = []
    if proposal.get("schema_version") != SOURCE_PROPOSAL_SCHEMA_VERSION:
        errors.append("source promotion proposal schema_version is not supported")
    if proposal.get("source_promotion_state") != "ready_for_source_patch":
        errors.append("source promotion proposal must be ready_for_source_patch")
    if proposal.get("passed") is not True:
        errors.append("source promotion proposal must have passed=true")
    for key in (
        "rdf_source_write_performed",
        "ontology_truth_promotion_performed",
        "raw_transcript_text_included",
    ):
        if proposal.get(key) is not False:
            errors.append(f"source promotion proposal {key} must be false")
    if not isinstance(proposal.get("patches"), list) or not proposal.get("patches"):
        errors.append("source promotion proposal patches must be a non-empty list")
    if not isinstance(proposal.get("target_files"), list) or not proposal.get(
        "target_files"
    ):
        errors.append("source promotion proposal target_files must be a non-empty list")
    return errors


def proposal_precondition_for_file(proposal: dict[str, Any], target_file: str) -> str:
    for target in proposal["target_files"]:
        if target["path"] == target_file:
            return target["precondition_sha256"]
    return ""


def render_staged_rdf(source: str, patches: list[dict[str, Any]]) -> str:
    if INSERT_BEFORE not in source:
        raise ValueError("target RDF source missing closing rdf:RDF tag")
    snippets = []
    for patch in sorted(patches, key=lambda item: item["claim_id"]):
        snippets.append(f"  <!-- staged audio claim patch: {patch['claim_id']} -->")
        snippets.append(indent_xml_snippet(patch["rdf_xml_preview"]))
    insertion = "\n" + "\n".join(snippets) + "\n"
    head, tail = source.rsplit(INSERT_BEFORE, 1)
    return head.rstrip() + insertion + INSERT_BEFORE + tail


def indent_xml_snippet(snippet: str) -> str:
    return "\n".join("  " + line for line in snippet.splitlines())


def parse_rdf_text(text: str, target_file: str) -> None:
    try:
        ET.fromstring(text)
    except ET.ParseError as error:
        raise ValueError(
            f"staged RDF is not valid XML for {target_file}: {error}"
        ) from error


def write_staged_artifacts(
    output_dir: Path,
    rendered_files: dict[Path, str],
    report: dict[str, Any],
) -> None:
    for path, text in rendered_files.items():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
    output_dir.mkdir(parents=True, exist_ok=True)
    (output_dir / "report.json").write_text(report_text(report))


def rejected_report(
    proposal: dict[str, Any],
    source_promotion_path: Path,
    output_dir: Path,
    *,
    errors: list[str],
) -> dict[str, Any]:
    return {
        "schema_version": REPORT_SCHEMA_VERSION,
        "proposal_id": str(proposal.get("proposal_id", "")),
        "staged_apply_state": "rejected",
        "passed": False,
        "errors": errors,
        "source_promotion_proposal": relative_to_ontology(source_promotion_path),
        "staged_output_dir": relative_to_ontology(output_dir),
        "staged_file_count": 0,
        "canonical_source_write_performed": False,
        "ontology_truth_promotion_performed": False,
        "raw_transcript_text_included": False,
        "staged_files": [],
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
        description="Apply an RDF source-promotion proposal into a staged output tree."
    )
    parser.add_argument(
        "--source-promotion",
        default=str(default_source_promotion_path()),
        help="Source-promotion proposal JSON path.",
    )
    parser.add_argument(
        "--output-dir",
        default=str(default_output_dir()),
        help="Staged output directory.",
    )
    parser.add_argument(
        "--no-write",
        action="store_true",
        help="Print the report without writing staged artifacts.",
    )
    args = parser.parse_args(argv)

    report = build_staged_apply_report(
        Path(args.source_promotion),
        Path(args.output_dir),
        write_artifacts=not args.no_write,
    )
    sys.stdout.write(report_text(report))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
