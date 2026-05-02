"""Command-line entrypoint for the Wendao document extraction Flight service."""

from __future__ import annotations

import argparse
import sys

from .document_service import DocumentExtractFlightServer
from .pdf_ocr import (
    DoclingPdfOcrShardWorker,
    PdfOcrShardWorkerProtocol,
    SkippingPdfOcrShardWorker,
)


def document_extract_service_main() -> int:
    """Run the Wendao document extraction Arrow Flight service."""

    parser = argparse.ArgumentParser(
        description="Wendao document extraction Arrow Flight service"
    )
    parser.add_argument("--host", default="0.0.0.0", help="Bind host")
    parser.add_argument("--port", type=int, default=50051, help="Bind port")
    parser.add_argument(
        "--pdf-ocr-worker",
        choices=("skip", "docling"),
        default="skip",
        help="OCR worker used by the internal /analysis/pdf-ocr-shards exchange",
    )
    parser.add_argument(
        "--pdf-ocr-workers",
        default="auto",
        help=(
            "Maximum Docling OCR shard workers for direct local requests. "
            "Rust providers may override this per request with "
            "x-wendao-pdf-ocr-workers."
        ),
    )
    args = parser.parse_args()

    location = f"grpc://{args.host}:{args.port}"
    server = DocumentExtractFlightServer(
        location,
        ocr_worker=build_pdf_ocr_worker(args.pdf_ocr_worker, args.pdf_ocr_workers),
    )
    sys.stdout.write(f"Wendao document extraction service listening on {location}\n")
    server.serve()
    return 0


def build_pdf_ocr_worker(
    worker_name: str,
    max_workers: int | str | None = "auto",
) -> PdfOcrShardWorkerProtocol:
    if worker_name == "docling":
        return DoclingPdfOcrShardWorker(max_workers=max_workers)
    return SkippingPdfOcrShardWorker()


__all__ = ["build_pdf_ocr_worker", "document_extract_service_main"]
