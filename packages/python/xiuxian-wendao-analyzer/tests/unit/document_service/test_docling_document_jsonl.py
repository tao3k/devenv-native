from __future__ import annotations

import csv
import hashlib
import json
from pathlib import Path

from xiuxian_wendao_analyzer.docling_document_jsonl import (
    run_docling_document_jsonl_tasks,
)


class FakeDoclingDocument:
    def __init__(self, markdown: str) -> None:
        self.markdown = markdown

    def export_to_markdown(self) -> str:
        return self.markdown


class FakeDoclingResult:
    def __init__(self, markdown: str) -> None:
        self.document = FakeDoclingDocument(markdown)


class FakeDoclingConverter:
    def __init__(self, markdown: str = "# Docling text\n") -> None:
        self.markdown = markdown
        self.calls: list[Path] = []

    def convert(self, source: str | Path) -> FakeDoclingResult:
        self.calls.append(Path(source))
        return FakeDoclingResult(self.markdown)


def _write_tasks(
    path: Path,
    *,
    source_sha256: str,
    relative_path: str = "docs/a.docx",
) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(
            handle,
            delimiter="\t",
            fieldnames=[
                "queue_id",
                "file_id",
                "relative_path",
                "extension",
                "category",
                "language",
                "extraction_route",
                "priority",
                "source_sha256",
                "planned_output_path",
                "output_contract",
                "status",
            ],
        )
        writer.writeheader()
        writer.writerow(
            {
                "queue_id": "ltc.extract.document.001",
                "file_id": "ltc.file.document.001",
                "relative_path": relative_path,
                "extension": Path(relative_path).suffix.lower().lstrip("."),
                "category": "synthetic",
                "language": "zh-CN",
                "extraction_route": "document_text_evidence",
                "priority": "normal",
                "source_sha256": source_sha256,
                "planned_output_path": "outputs/ltc.extract.document.001.json",
                "output_contract": "cache_only_no_rdf_promotion",
                "status": "planned",
            }
        )
        writer.writerow(
            {
                "queue_id": "ltc.extract.legacy.001",
                "file_id": "ltc.file.legacy.001",
                "relative_path": "docs/legacy.doc",
                "extension": "doc",
                "category": "synthetic",
                "language": "zh-CN",
                "extraction_route": "document_text_evidence",
                "priority": "normal",
                "source_sha256": "0" * 64,
                "planned_output_path": "outputs/ltc.extract.legacy.001.json",
                "output_contract": "cache_only_no_rdf_promotion",
                "status": "planned",
            }
        )


def test_docling_document_jsonl_writes_supported_document_sidecar(
    tmp_path: Path,
) -> None:
    corpus_root = tmp_path / "corpus"
    doc_path = corpus_root / "docs" / "a.docx"
    doc_path.parent.mkdir(parents=True)
    document_bytes = b"synthetic-docx"
    doc_path.write_bytes(document_bytes)
    tasks_path = tmp_path / "run" / "tasks.tsv"
    _write_tasks(tasks_path, source_sha256=hashlib.sha256(document_bytes).hexdigest())
    output_jsonl = tmp_path / "run" / "document_results.jsonl"
    converter = FakeDoclingConverter()

    report = run_docling_document_jsonl_tasks(
        tasks_path=tasks_path,
        corpus_root=corpus_root,
        output_jsonl_path=output_jsonl,
        profile="structure-text",
        converter_factory=lambda profile: converter,
    )

    assert report["passed"] is True
    assert report["eligible_count"] == 1
    assert report["skipped_count"] == 1
    assert report["succeeded_count"] == 1
    assert converter.calls == [doc_path]
    row = json.loads(output_jsonl.read_text(encoding="utf-8"))
    assert row["queue_id"] == "ltc.extract.document.001"
    assert row["text"] == "# Docling text"
    assert row["extractor"] == "docling"
    assert row["docling_profile"] == "structure-text"
    assert row["text_mime_type"] == "text/markdown"
    assert row["extension"] == "docx"


def test_docling_document_jsonl_blocks_source_hash_drift(tmp_path: Path) -> None:
    corpus_root = tmp_path / "corpus"
    doc_path = corpus_root / "docs" / "a.docx"
    doc_path.parent.mkdir(parents=True)
    doc_path.write_bytes(b"changed-docx")
    tasks_path = tmp_path / "run" / "tasks.tsv"
    _write_tasks(tasks_path, source_sha256="0" * 64)
    output_jsonl = tmp_path / "run" / "document_results.jsonl"
    converter = FakeDoclingConverter()

    report = run_docling_document_jsonl_tasks(
        tasks_path=tasks_path,
        corpus_root=corpus_root,
        output_jsonl_path=output_jsonl,
        converter_factory=lambda profile: converter,
    )

    assert report["passed"] is False
    assert report["eligible_count"] == 1
    assert report["succeeded_count"] == 0
    assert "sha256 drift" in report["errors"][0]
    assert converter.calls == []
    assert output_jsonl.read_text(encoding="utf-8") == ""


def test_docling_document_jsonl_blocks_corpus_path_escape(tmp_path: Path) -> None:
    corpus_root = tmp_path / "corpus"
    outside_path = tmp_path / "outside" / "a.docx"
    outside_path.parent.mkdir(parents=True)
    document_bytes = b"outside-docx"
    outside_path.write_bytes(document_bytes)
    tasks_path = tmp_path / "run" / "tasks.tsv"
    _write_tasks(
        tasks_path,
        source_sha256=hashlib.sha256(document_bytes).hexdigest(),
        relative_path="../outside/a.docx",
    )
    output_jsonl = tmp_path / "run" / "document_results.jsonl"
    converter = FakeDoclingConverter()

    report = run_docling_document_jsonl_tasks(
        tasks_path=tasks_path,
        corpus_root=corpus_root,
        output_jsonl_path=output_jsonl,
        converter_factory=lambda profile: converter,
    )

    assert report["passed"] is False
    assert report["succeeded_count"] == 0
    assert "escapes corpus root" in report["errors"][0]
    assert converter.calls == []
    assert output_jsonl.read_text(encoding="utf-8") == ""
