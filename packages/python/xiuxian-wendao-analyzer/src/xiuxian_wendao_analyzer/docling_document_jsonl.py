"""Queue-keyed Docling document JSONL adapter for private evidence bridges."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import sys
from collections.abc import Callable
from pathlib import Path
from typing import Any

from .document_profiles import new_docling_converter_for_profile
from .document_types import DocumentConverterProtocol
from .source_contract_paths import resolve_corpus_relative_path

DOCLING_DOCUMENT_JSONL_RUN_SCHEMA_VERSION = (
    "xiuxian_wendao.docling_document_jsonl_run.v1"
)
DOCLING_DOCUMENT_JSONL_TASK_ROUTE = "document_text_evidence"
DOCLING_DOCUMENT_JSONL_SUPPORTED_EXTENSIONS = frozenset({"pdf", "docx", "pptx", "xlsx"})
DOCLING_DOCUMENT_JSONL_DEFAULT_PROFILE = "full"
DOCLING_DOCUMENT_JSONL_TEXT_MIME_TYPE = "text/markdown"

ConverterFactory = Callable[[str | None], DocumentConverterProtocol]


def run_docling_document_jsonl_tasks(
    *,
    tasks_path: Path,
    corpus_root: Path,
    output_jsonl_path: Path,
    profile: str | None = None,
    converter_factory: ConverterFactory | None = None,
) -> dict[str, Any]:
    """Run Docling for supported document task rows and write queue-keyed JSONL.

    Raises:
        ValueError: If task rows are malformed or source hashes drift.
        OSError: If input or output paths cannot be read or written.
    """

    tasks = read_docling_document_tasks(tasks_path)
    converter = (converter_factory or new_docling_converter_for_profile)(profile)
    output_rows: list[dict[str, Any]] = []
    errors: list[str] = []
    skipped_count = 0

    for row in tasks:
        queue_id = row["queue_id"]
        extension = task_extension(row)
        if extension not in DOCLING_DOCUMENT_JSONL_SUPPORTED_EXTENSIONS:
            skipped_count += 1
            continue
        try:
            source_path = resolve_corpus_relative_path(
                corpus_root=corpus_root,
                relative_path=row["relative_path"],
                task_label=f"Docling document task {queue_id}",
            )
            if not source_path.is_file():
                raise ValueError(f"source document does not exist: {source_path}")
            source_sha256 = sha256_file(source_path)
            if source_sha256 != row["source_sha256"]:
                raise ValueError(f"source sha256 drift for {queue_id}")
            result = converter.convert(source_path)
            text = normalize_text(result.document.export_to_markdown())
            if not text:
                raise ValueError(f"Docling returned empty markdown for {queue_id}")
            output_rows.append(
                {
                    "queue_id": queue_id,
                    "text": text,
                    "extractor": "docling",
                    "docling_profile": profile
                    or DOCLING_DOCUMENT_JSONL_DEFAULT_PROFILE,
                    "text_mime_type": DOCLING_DOCUMENT_JSONL_TEXT_MIME_TYPE,
                    "source_sha256": source_sha256,
                    "extension": extension,
                }
            )
        except Exception as exc:
            errors.append(f"{queue_id}: {exc}")

    write_jsonl(output_jsonl_path, output_rows)
    return {
        "schema_version": DOCLING_DOCUMENT_JSONL_RUN_SCHEMA_VERSION,
        "passed": not errors,
        "tasks_path": str(tasks_path),
        "output_jsonl_path": str(output_jsonl_path),
        "attempted_count": len(tasks),
        "eligible_count": len(tasks) - skipped_count,
        "skipped_count": skipped_count,
        "succeeded_count": len(output_rows),
        "failed_count": len(errors),
        "errors": errors,
        "docling_profile": profile or DOCLING_DOCUMENT_JSONL_DEFAULT_PROFILE,
        "supported_extensions": sorted(DOCLING_DOCUMENT_JSONL_SUPPORTED_EXTENSIONS),
        "output_contract": "queue_keyed_docling_document_jsonl",
    }


def read_docling_document_tasks(path: Path) -> list[dict[str, str]]:
    with path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        fieldnames = set(reader.fieldnames or [])
        required = {
            "queue_id",
            "relative_path",
            "extraction_route",
            "source_sha256",
        }
        missing = sorted(required - fieldnames)
        if missing:
            raise ValueError(
                f"Docling document task TSV missing fields: {', '.join(missing)}"
            )
        rows = [
            dict(row)
            for row in reader
            if row.get("extraction_route") == DOCLING_DOCUMENT_JSONL_TASK_ROUTE
        ]
    seen: set[str] = set()
    for row in rows:
        queue_id = row.get("queue_id", "").strip()
        if not queue_id:
            raise ValueError("Docling document task row missing queue_id")
        if queue_id in seen:
            raise ValueError(f"duplicate Docling document queue_id: {queue_id}")
        seen.add(queue_id)
        if not row.get("relative_path", "").strip():
            raise ValueError(f"Docling document task {queue_id} missing relative_path")
        if not row.get("source_sha256", "").strip():
            raise ValueError(f"Docling document task {queue_id} missing source_sha256")
    return rows


def task_extension(row: dict[str, str]) -> str:
    extension = (row.get("extension") or "").strip().lower()
    if extension:
        return extension
    return Path(row["relative_path"]).suffix.lower().lstrip(".")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def normalize_text(value: str) -> str:
    return "\n".join(
        line.rstrip() for line in value.replace("\r", "\n").split("\n")
    ).strip()


def write_jsonl(path: Path, rows: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        for row in rows:
            handle.write(json.dumps(row, ensure_ascii=False, sort_keys=True))
            handle.write("\n")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Run Docling for document task TSV rows."
    )
    parser.add_argument(
        "--tasks", required=True, help="Path to a source-contract tasks.tsv file."
    )
    parser.add_argument(
        "--corpus-root", required=True, help="Root directory for task relative paths."
    )
    parser.add_argument(
        "--output-jsonl", required=True, help="Queue-keyed document JSONL output path."
    )
    parser.add_argument(
        "--profile",
        default=DOCLING_DOCUMENT_JSONL_DEFAULT_PROFILE,
        help="Wendao Docling document extraction profile.",
    )
    args = parser.parse_args(argv)
    try:
        report = run_docling_document_jsonl_tasks(
            tasks_path=Path(args.tasks).expanduser().resolve(),
            corpus_root=Path(args.corpus_root).expanduser().resolve(),
            output_jsonl_path=Path(args.output_jsonl).expanduser().resolve(),
            profile=args.profile,
        )
    except Exception as exc:
        report = {
            "schema_version": DOCLING_DOCUMENT_JSONL_RUN_SCHEMA_VERSION,
            "passed": False,
            "errors": [str(exc)],
        }
    print(json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
