from __future__ import annotations

import argparse
from pathlib import Path
from tempfile import TemporaryDirectory

from xiuxian_wendao_analyzer import (
    DOCLING_COMMON_SOURCE_SUFFIXES,
    DOCLING_SUPPORTED_DOCUMENT_FORMATS,
    extract_document_table,
    is_known_docling_source,
)


class FixtureDocument:
    def __init__(self, markdown: str) -> None:
        self.markdown = markdown

    def export_to_markdown(self) -> str:
        return self.markdown


class FixtureConversionResult:
    def __init__(self, markdown: str) -> None:
        self.document = FixtureDocument(markdown)


class FixtureDocumentConverter:
    def convert(self, source: str | Path) -> FixtureConversionResult:
        return FixtureConversionResult(f"# Parsed fixture\n\nSource: {Path(source).name}\n")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run a Docling-backed multi-format document extraction workflow into Arrow rows.",
    )
    parser.add_argument("--mode", choices=("fixture", "docling"), default="fixture")
    parser.add_argument("--source", default="examples/fixtures/sample.docx")
    parser.add_argument("--output-dir", default="")
    parser.add_argument("--force", action="store_true")
    parser.add_argument("--error-row", action="store_true")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    with TemporaryDirectory(prefix="xiuxian-wendao-docs-") as temp_dir:
        source = Path(args.source)
        if args.mode == "fixture":
            if args.source == "examples/fixtures/sample.docx":
                source = Path(temp_dir) / "sample.docx"
            source.parent.mkdir(parents=True, exist_ok=True)
            source.write_bytes(b"Docling fixture document\n")
            converter = FixtureDocumentConverter()
        else:
            converter = None

        output_dir = args.output_dir or None
        table = extract_document_table(
            source,
            output_dir,
            converter=converter,
            force=args.force,
            error_row=args.error_row,
        )
        rows = table.to_pylist()

        print("mode=", args.mode)
        print("known_docling_source=", is_known_docling_source(source))
        print("supported_formats=", ",".join(DOCLING_SUPPORTED_DOCUMENT_FORMATS))
        print("common_suffixes=", ",".join(DOCLING_COMMON_SOURCE_SUFFIXES))
        print("rows=", table.num_rows)
        print("columns=", ",".join(table.column_names))
        if rows:
            print("top_status=", rows[0]["status"])
            print("top_resource_type=", rows[0]["resourceType"])
            print("top_mime_type=", rows[0]["mimeType"])
            print("top_content=", rows[0]["content"].splitlines()[0])


if __name__ == "__main__":
    main()
