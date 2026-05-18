from __future__ import annotations

import argparse
import hashlib
import json
import sys
import tomllib
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import Any

from wendao_core_lib.episteme_contracts.wendao_audio_claim_acceptance.__main__ import (
    load_manifest,
    ontology_root,
)


REPORT_SCHEMA_VERSION = "wendao.audio_claim_rdf_source_apply_verification_report.v1"
APPLY_SCHEMA_VERSION = "wendao.audio_claim_rdf_source_apply_report.v1"
RDF_NS = "{http://www.w3.org/1999/02/22-rdf-syntax-ns#}"
OWL_NS = "{http://www.w3.org/2002/07/owl#}"


def load_contract() -> dict[str, Any]:
    manifest = load_manifest()
    return dict(manifest["audio_claim_rdf_source_apply_verification"])


def default_apply_report_path() -> Path:
    manifest = load_manifest()
    apply_contract = manifest["audio_claim_rdf_source_apply"]
    return ontology_root() / apply_contract["example"]


def default_output_path() -> Path:
    contract = load_contract()
    return ontology_root() / contract["example"]


def build_source_apply_verification_report(
    apply_report_path: Path,
    *,
    target_root: Path | None = None,
) -> dict[str, Any]:
    root = target_root or ontology_root()
    apply_report, load_errors = load_json_object(
        apply_report_path, "source apply report"
    )
    errors = validate_apply_report(apply_report)
    errors.extend(load_errors)
    if errors:
        return rejected_report(apply_report, apply_report_path, root, errors=errors)

    verified_files = []
    for applied_file in apply_report["applied_files"]:
        target_path = resolve_under_root(root, applied_file["target_rdf_file"])
        staged_path = resolve_under_root(root, applied_file["staged_rdf_file"])
        diff_path = resolve_under_root(root, applied_file["diff_file"])
        if not target_path.is_file():
            errors.append(f"missing target RDF file: {applied_file['target_rdf_file']}")
            continue
        if not staged_path.is_file():
            errors.append(f"missing staged RDF file: {applied_file['staged_rdf_file']}")
            continue
        if not diff_path.is_file():
            errors.append(f"missing diff file: {applied_file['diff_file']}")
            continue

        observed_source_hash = sha256_file(target_path)
        staged_hash = sha256_file(staged_path)
        diff_hash = sha256_file(diff_path)
        expected_source_hash = expected_source_hash_for_mode(apply_report, applied_file)
        if observed_source_hash != expected_source_hash:
            errors.append(
                f"source apply verification hash mismatch for {applied_file['target_rdf_file']}"
            )
            continue
        if staged_hash != applied_file["applied_source_sha256"]:
            errors.append(f"staged hash mismatch for {applied_file['staged_rdf_file']}")
            continue
        if diff_hash != applied_file["diff_sha256"]:
            errors.append(
                f"diff artifact hash mismatch for {applied_file['diff_file']}"
            )
            continue

        try:
            parse_rdf_file(target_path)
            parse_rdf_file(staged_path)
        except ValueError as error:
            errors.append(str(error))
            continue

        source_write_observed = (
            observed_source_hash == applied_file["applied_source_sha256"]
        )
        if source_write_observed != applied_file["source_write_performed"]:
            errors.append(
                f"source write observation mismatch for {applied_file['target_rdf_file']}"
            )
            continue

        verified_files.append(
            {
                "target_rdf_file": applied_file["target_rdf_file"],
                "staged_rdf_file": applied_file["staged_rdf_file"],
                "diff_file": applied_file["diff_file"],
                "expected_source_sha256": expected_source_hash,
                "observed_source_sha256": observed_source_hash,
                "staged_sha256": staged_hash,
                "diff_sha256": diff_hash,
                "source_write_observed": source_write_observed,
                "xml_parse_verified": True,
                "registry_read_verified": True,
            }
        )

    if errors:
        return rejected_report(apply_report, apply_report_path, root, errors=errors)

    registry_counts = collect_registry_term_counts(root)
    write_mode = apply_report["source_write_mode"]
    return {
        "schema_version": REPORT_SCHEMA_VERSION,
        "proposal_id": apply_report["proposal_id"],
        "verification_state": (
            "applied_source_verified"
            if write_mode == "write_source"
            else "dry_run_verified"
        ),
        "passed": True,
        "errors": [],
        "source_apply_report": relative_to_root(apply_report_path, root),
        "source_write_mode": write_mode,
        "canonical_source_write_observed": write_mode == "write_source",
        "ontology_truth_promotion_performed": False,
        "raw_transcript_text_included": False,
        "verified_file_count": len(verified_files),
        "registry_class_count": registry_counts["classes"],
        "registry_object_property_count": registry_counts["object_properties"],
        "verified_files": sorted(
            verified_files, key=lambda item: item["target_rdf_file"]
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


def validate_apply_report(report: dict[str, Any]) -> list[str]:
    errors = []
    if report.get("schema_version") != APPLY_SCHEMA_VERSION:
        errors.append("source apply report schema_version is not supported")
    if report.get("source_apply_state") not in {"ready_for_source_apply", "applied"}:
        errors.append("source apply report must be ready_for_source_apply or applied")
    if report.get("passed") is not True:
        errors.append("source apply report must have passed=true")
    write_mode = report.get("source_write_mode")
    if write_mode not in {"dry_run", "write_source"}:
        errors.append("source apply report source_write_mode is not supported")
    if (write_mode == "write_source") != bool(
        report.get("canonical_source_write_performed")
    ):
        errors.append(
            "source apply report write mode does not match canonical write flag"
        )
    for key in (
        "ontology_truth_promotion_performed",
        "raw_transcript_text_included",
    ):
        if report.get(key) is not False:
            errors.append(f"source apply report {key} must be false")
    if not isinstance(report.get("applied_files"), list) or not report.get(
        "applied_files"
    ):
        errors.append("source apply report applied_files must be a non-empty list")
    return errors


def expected_source_hash_for_mode(
    apply_report: dict[str, Any],
    applied_file: dict[str, Any],
) -> str:
    if apply_report["source_write_mode"] == "write_source":
        return applied_file["applied_source_sha256"]
    return applied_file["previous_source_sha256"]


def rejected_report(
    apply_report: dict[str, Any],
    apply_report_path: Path,
    root: Path,
    *,
    errors: list[str],
) -> dict[str, Any]:
    write_mode = str(apply_report.get("source_write_mode", "dry_run"))
    if write_mode not in {"dry_run", "write_source"}:
        write_mode = "dry_run"
    return {
        "schema_version": REPORT_SCHEMA_VERSION,
        "proposal_id": str(apply_report.get("proposal_id", "")),
        "verification_state": "rejected",
        "passed": False,
        "errors": errors,
        "source_apply_report": relative_to_root(apply_report_path, root),
        "source_write_mode": write_mode,
        "canonical_source_write_observed": False,
        "ontology_truth_promotion_performed": False,
        "raw_transcript_text_included": False,
        "verified_file_count": 0,
        "registry_class_count": 0,
        "registry_object_property_count": 0,
        "verified_files": [],
    }


def collect_registry_term_counts(root: Path) -> dict[str, int]:
    manifest = tomllib.loads((root / "manifest.toml").read_text())
    class_count = 0
    object_property_count = 0
    for domain in manifest["domains"]:
        for relative_path in domain.get("rdf_files", []):
            rdf_root = parse_rdf_file(root / relative_path)
            class_count += len(rdf_root.findall(f"{OWL_NS}Class"))
            object_property_count += len(rdf_root.findall(f"{OWL_NS}ObjectProperty"))
    return {
        "classes": class_count,
        "object_properties": object_property_count,
    }


def parse_rdf_file(path: Path) -> ET.Element:
    try:
        return ET.parse(path).getroot()
    except ET.ParseError as error:
        raise ValueError(
            f"RDF file is not valid XML for {path.name}: {error}"
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
        description="Verify a source-apply report against the current ontology source tree."
    )
    parser.add_argument(
        "--source-apply-report",
        default=str(default_apply_report_path()),
        help="Source apply report JSON path.",
    )
    parser.add_argument(
        "--output",
        default=str(default_output_path()),
        help="Optional source apply verification report output path.",
    )
    parser.add_argument(
        "--no-write",
        action="store_true",
        help="Print the verification report without writing output.",
    )
    args = parser.parse_args(argv)

    report = build_source_apply_verification_report(Path(args.source_apply_report))
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
