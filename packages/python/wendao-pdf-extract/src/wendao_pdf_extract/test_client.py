"""End-to-end test client for the PDF Extract Arrow Flight server."""

from __future__ import annotations

import argparse
import os
import sys
import tempfile
from pathlib import Path

import pyarrow as pa
import pyarrow.flight as flight

ANALYSIS_PDF_EXTRACT_ROUTE = "/analysis/pdf-extract"

WENDAO_SCHEMA_VERSION_HEADER = "x-wendao-schema-version"
WENDAO_PDF_EXTRACT_SOURCE_PATH_HEADER = "x-wendao-pdf-extract-source-path"
WENDAO_PDF_EXTRACT_OUTPUT_DIR_HEADER = "x-wendao-pdf-extract-output-dir"
WENDAO_PDF_EXTRACT_IMAGES_HEADER = "x-wendao-pdf-extract-images"
WENDAO_PDF_EXTRACT_TABLES_HEADER = "x-wendao-pdf-extract-tables"
WENDAO_PDF_EXTRACT_FORMULAS_HEADER = "x-wendao-pdf-extract-formulas"

EXPECTED_SCHEMA_VERSION = "v2"

_RESULT_SCHEMA = pa.schema(
    [
        pa.field("sourcePath", pa.utf8()),
        pa.field("resourceType", pa.utf8()),
        pa.field("resourcePath", pa.utf8()),
        pa.field("pageIndex", pa.int32()),
        pa.field("caption", pa.utf8()),
        pa.field("content", pa.utf8()),
        pa.field("mimeType", pa.utf8()),
        pa.field("status", pa.utf8()),
        pa.field("elementId", pa.utf8()),
    ]
)


def _find_test_pdf() -> str | None:
    """Search for a test PDF in common locations."""
    search_paths = [
        os.environ.get("WENDAO_TEST_PDF"),
        "test.pdf",
        "tests/test.pdf",
        "../tests/test.pdf",
        "../../tests/test.pdf",
        "/tmp/test.pdf",
    ]
    for p in search_paths:
        if p and Path(p).is_file():
            return str(Path(p).resolve())
    return None


def _create_minimal_test_pdf(path: str) -> None:
    """Create a minimal single-page PDF for testing."""
    try:
        from reportlab.pdfgen import canvas
        from reportlab.lib.pagesizes import letter
    except ModuleNotFoundError:
        raise RuntimeError(
            "reportlab is not installed; "
            "install it with 'pip install reportlab' to run tests, "
            "or set WENDAO_TEST_PDF to an existing PDF"
        )

    c = canvas.Canvas(path, pagesize=letter)
    c.drawString(100, 700, "Hello, Wendao PDF Extract!")
    c.drawString(100, 680, "This is a test PDF for Arrow Flight roundtrip.")
    c.showPage()
    c.save()


def run_test(
    location: str = "grpc://localhost:50051",
    pdf_path: str | None = None,
    verbose: bool = False,
) -> int:
    """Run end-to-end test against PDF Extract Flight server.

    Returns 0 on success, 1 on failure.
    """
    # Determine test PDF
    if pdf_path is None:
        pdf_path = _find_test_pdf()

    if pdf_path is None:
        pdf_path = "/tmp/wendao_test_minimal.pdf"
        print(f"Creating minimal test PDF: {pdf_path}")
        _create_minimal_test_pdf(pdf_path)

    if not Path(pdf_path).is_file():
        print(f"ERROR: PDF not found: {pdf_path}", file=sys.stderr)
        return 1

    print(f"Test PDF: {pdf_path}")
    print(f"Server:   {location}")
    print("")

    # Connect
    try:
        client = flight.connect(location)
    except Exception as exc:
        print(f"ERROR: Failed to connect to {location}: {exc}", file=sys.stderr)
        return 1

    # Build headers
    output_dir = f"{pdf_path}.extracted"
    options = flight.FlightCallOptions(
        headers=[
            (WENDAO_SCHEMA_VERSION_HEADER, EXPECTED_SCHEMA_VERSION),
            (WENDAO_PDF_EXTRACT_SOURCE_PATH_HEADER, pdf_path),
            (WENDAO_PDF_EXTRACT_OUTPUT_DIR_HEADER, output_dir),
            (WENDAO_PDF_EXTRACT_IMAGES_HEADER, "true"),
            (WENDAO_PDF_EXTRACT_TABLES_HEADER, "true"),
            (WENDAO_PDF_EXTRACT_FORMULAS_HEADER, "true"),
        ]
    )

    # Step 1: get_flight_info
    descriptor = flight.FlightDescriptor.for_path("analysis", "pdf-extract")
    print("→ get_flight_info ...")
    try:
        info = client.get_flight_info(descriptor, options=options)
    except Exception as exc:
        print(f"ERROR: get_flight_info failed: {exc}", file=sys.stderr)
        return 1

    print(f"  schema:    {info.schema}")
    print(f"  records:   {info.total_records}")
    print(f"  bytes:     {info.total_bytes}")
    print(f"  endpoints: {len(info.endpoints)}")

    if not info.endpoints:
        print("ERROR: No endpoints returned", file=sys.stderr)
        return 1

    ticket = info.endpoints[0].ticket
    if ticket is None:
        print("ERROR: Endpoint missing ticket", file=sys.stderr)
        return 1

    # Step 2: do_get
    print("")
    print("→ do_get ...")
    try:
        reader = client.do_get(ticket, options=options)
    except Exception as exc:
        print(f"ERROR: do_get failed: {exc}", file=sys.stderr)
        return 1

    table = reader.read_all()
    print(f"  rows:      {table.num_rows}")
    print(f"  columns:   {table.num_columns}")

    # Validate schema
    actual_names = [f.name for f in table.schema]
    expected_names = [f.name for f in _RESULT_SCHEMA]
    if actual_names != expected_names:
        print(f"ERROR: Schema mismatch", file=sys.stderr)
        print(f"  expected: {expected_names}", file=sys.stderr)
        print(f"  actual:   {actual_names}", file=sys.stderr)
        return 1

    # Validate content
    if table.num_rows == 0:
        print("ERROR: No rows returned", file=sys.stderr)
        return 1

    if verbose and table.num_rows > 0:
        print("")
        print("First 5 rows:")
        for i in range(min(5, table.num_rows)):
            row = table.slice(i, 1).to_pydict()
            print(f"  [{i}] type={row['resourceType'][0]!r} "
                  f"mime={row['mimeType'][0]!r} "
                  f"page={row['pageIndex'][0]} "
                  f"path={row['resourcePath'][0]!r}")
            content = row["content"][0]
            if content:
                preview = content[:80].replace("\n", " ")
                print(f"      content={preview!r}")

    print("")
    print(f"✓ All checks passed ({table.num_rows} resources extracted)")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="PDF Extract Arrow Flight test client")
    parser.add_argument(
        "--location",
        default="grpc://localhost:50051",
        help="Arrow Flight server location (default: grpc://localhost:50051)",
    )
    parser.add_argument(
        "--pdf",
        default=None,
        help="Path to test PDF (default: auto-detect or create minimal)",
    )
    parser.add_argument(
        "-v", "--verbose",
        action="store_true",
        help="Print row details",
    )
    args = parser.parse_args()

    return run_test(location=args.location, pdf_path=args.pdf, verbose=args.verbose)


if __name__ == "__main__":
    sys.exit(main())
