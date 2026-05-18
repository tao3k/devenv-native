from __future__ import annotations

import argparse
import csv
import json
import re
import tomllib
from pathlib import Path
from typing import Any

from wendao_core_lib.episteme_contracts.paths import ontology_root
from wendao_core_lib.episteme_contracts.wendao_ontology_registry.__main__ import (
    build_registry,
)


FORBIDDEN_SQL_OPERATIONS = {
    "CREATE",
    "ALTER",
    "DROP",
    "INSERT",
    "UPDATE",
    "DELETE",
    "MERGE",
    "COPY",
    "ATTACH",
}

HEALTHCARE_MAPPING_PATH = (
    "30_Healthcare/mappings/healthcare_synthetic_care_delivery.toml"
)
RUNTIME_MATERIALIZATION_OWNER = "xiuxian-wendao"
HANDOFF_KIND = "arrow_flight_raw_tables"


def load_toml(path: Path) -> dict[str, Any]:
    return tomllib.loads(path.read_text())


def strip_sql_comments_and_literals(sql: str) -> str:
    without_line_comments = re.sub(r"--.*?$", " ", sql, flags=re.MULTILINE)
    without_block_comments = re.sub(
        r"/\*.*?\*/", " ", without_line_comments, flags=re.DOTALL
    )
    return re.sub(r"'(?:''|[^'])*'", "''", without_block_comments)


def assert_select_only_sql(path: Path) -> None:
    sql = strip_sql_comments_and_literals(path.read_text())
    forbidden = FORBIDDEN_SQL_OPERATIONS.intersection(
        match.group(0).upper() for match in re.finditer(r"\b[A-Za-z]+\b", sql)
    )
    if forbidden:
        raise ValueError(f"forbidden SQL operations in {path}: {sorted(forbidden)}")


def load_mapping_contract(
    relative_path: str = HEALTHCARE_MAPPING_PATH,
) -> dict[str, Any]:
    path = ontology_root() / relative_path
    contract = load_toml(path)
    contract["_path"] = relative_path
    return contract


def _csv_rows(path: Path) -> list[dict[str, str]]:
    with path.open(newline="") as handle:
        return list(csv.DictReader(handle))


def _csv_columns(path: Path) -> set[str]:
    with path.open(newline="") as handle:
        reader = csv.DictReader(handle)
        if reader.fieldnames is None:
            return set()
        return set(reader.fieldnames)


def _pyarrow_modules():
    try:
        import pyarrow as pa
        import pyarrow.ipc as ipc
    except ModuleNotFoundError as exc:
        raise RuntimeError(
            "pyarrow is required for Arrow IPC emission; run through `uv run python`."
        ) from exc
    return pa, ipc


def _materialization_sql_paths(contract: dict[str, Any]) -> dict[str, Path]:
    return {
        name: ontology_root() / relative_path
        for name, relative_path in contract["materialization"].items()
    }


def _known_rdf_terms() -> tuple[set[str], set[str]]:
    registry = build_registry()
    classes = {entry["iri"] for entry in registry["rdf_terms"]["classes"]}
    object_properties = {
        entry["iri"] for entry in registry["rdf_terms"]["object_properties"]
    }
    return classes, object_properties


def validate_contract_references(contract: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    known_classes, known_properties = _known_rdf_terms()

    ledger = ontology_root() / contract["ledger_org"]
    if not ledger.is_file():
        errors.append(f"missing ledger_org: {contract['ledger_org']}")

    for raw_table in contract["raw_tables"]:
        path = ontology_root() / raw_table["path"]
        if not path.is_file():
            errors.append(f"missing raw table fixture: {raw_table['path']}")
            continue
        columns = _csv_columns(path)
        missing = sorted(set(raw_table["required_columns"]) - columns)
        if missing:
            errors.append(f"{raw_table['name']} missing required columns: {missing}")
        unmapped = sorted(
            set(raw_table["required_columns"]) - set(raw_table["mapped_columns"])
        )
        if unmapped:
            errors.append(
                f"{raw_table['name']} required columns not mapped: {unmapped}"
            )

    for name, path in _materialization_sql_paths(contract).items():
        if not path.is_file():
            errors.append(
                f"missing materialization SQL {name}: {path.relative_to(ontology_root())}"
            )
            continue
        try:
            assert_select_only_sql(path)
        except ValueError as exc:
            errors.append(str(exc))

    for class_entry in contract["required_classes"]:
        if class_entry["rdf_class"] not in known_classes:
            errors.append(f"unknown required class: {class_entry['rdf_class']}")

    for property_entry in contract["required_properties"]:
        if property_entry["rdf_property"] not in known_properties:
            errors.append(
                f"unknown required property: {property_entry['rdf_property']}"
            )

    return errors


def raw_table_counts(contract: dict[str, Any] | None = None) -> dict[str, int]:
    if contract is None:
        contract = load_mapping_contract()

    counts: dict[str, int] = {}
    for raw_table in contract["raw_tables"]:
        path = ontology_root() / raw_table["path"]
        counts[raw_table["name"]] = len(_csv_rows(path)) if path.is_file() else 0
    return counts


def build_validation_report(contract: dict[str, Any] | None = None) -> dict[str, Any]:
    if contract is None:
        contract = load_mapping_contract()

    errors = validate_contract_references(contract)
    return {
        "schema_version": "wendao.dataset_ontology.source_contract_report.v1",
        "mapping_id": contract["mapping_id"],
        "domain": contract["domain"],
        "passed": not errors,
        "errors": errors,
        "runtime_materialization_owner": RUNTIME_MATERIALIZATION_OWNER,
        "handoff_kind": HANDOFF_KIND,
        "raw_table_counts": raw_table_counts(contract),
        "materialization_sql": {
            name: str(path.relative_to(ontology_root()))
            for name, path in _materialization_sql_paths(contract).items()
        },
        "validation_rules": list(contract["validation_rules"]),
    }


def emit_raw_table_arrow_ipc(
    output_dir: Path,
    contract: dict[str, Any] | None = None,
) -> dict[str, int]:
    if contract is None:
        contract = load_mapping_contract()

    pa, ipc = _pyarrow_modules()
    output_dir.mkdir(parents=True, exist_ok=True)
    counts: dict[str, int] = {}

    for raw_table in contract["raw_tables"]:
        source_path = ontology_root() / raw_table["path"]
        rows = _csv_rows(source_path)
        if rows:
            table = pa.Table.from_pylist(rows)
        else:
            table = pa.table({column: [] for column in raw_table["required_columns"]})

        output_path = output_dir / f"{raw_table['name']}.arrow"
        with output_path.open("wb") as handle:
            with ipc.new_stream(handle, table.schema) as writer:
                writer.write_table(table)
        counts[raw_table["name"]] = table.num_rows

    return counts


def read_raw_table_arrow_ipc_counts(output_dir: Path) -> dict[str, int]:
    _pa, ipc = _pyarrow_modules()
    counts: dict[str, int] = {}
    for path in sorted(output_dir.glob("*.arrow")):
        with path.open("rb") as handle:
            with ipc.open_stream(handle) as reader:
                counts[path.stem] = reader.read_all().num_rows
    return counts


def _write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Validate Wendao dataset-to-ontology source mappings."
    )
    parser.add_argument(
        "--mapping",
        default=HEALTHCARE_MAPPING_PATH,
        help="Mapping contract path relative to ontology root.",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Exit non-zero if mapping source-contract validation fails.",
    )
    parser.add_argument(
        "--emit-raw-arrow-dir",
        help="Optional directory for raw source tables encoded as Arrow IPC stream files.",
    )
    parser.add_argument(
        "--report-json",
        help="Optional path for a validation report JSON artifact.",
    )
    args = parser.parse_args(argv)

    contract = load_mapping_contract(args.mapping)
    report = build_validation_report(contract)

    if args.emit_raw_arrow_dir:
        arrow_counts = emit_raw_table_arrow_ipc(Path(args.emit_raw_arrow_dir), contract)
        report["raw_arrow_ipc"] = {
            "output_dir": args.emit_raw_arrow_dir,
            "counts": arrow_counts,
        }
    if args.report_json:
        _write_json(Path(args.report_json), report)

    if not args.report_json:
        print(json.dumps(report, indent=2, sort_keys=True))

    if args.check and not report["passed"]:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
