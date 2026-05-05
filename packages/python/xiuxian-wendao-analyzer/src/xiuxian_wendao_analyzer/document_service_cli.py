"""Command-line entrypoint for the Wendao document extraction Flight service."""

from __future__ import annotations

import argparse
import sys

from .document_service import DocumentExtractFlightServer
from .ocr2_backend import (
    Ocr2BackendAction,
    Ocr2BackendError,
    Ocr2BackendOptions,
    run_ocr2_backend_action,
)
from .pdf_ocr import (
    DoclingPdfOcrShardWorker,
    PdfOcrShardWorkerProtocol,
    SkippingPdfOcrShardWorker,
)


def document_extract_service_main() -> int:
    """Run the Wendao document extraction Arrow Flight service."""

    parser = build_document_extract_argument_parser()
    args = parser.parse_args()
    ocr2_action = resolve_ocr2_backend_action(args)
    if ocr2_action is not None:
        options = Ocr2BackendOptions(
            repo_id=args.ocr2_repo_id,
            model_dir=args.ocr2_model_dir,
            model_path=args.ocr2_model_path,
            quantization=args.ocr2_quantization,
            backend_runner=args.ocr2_backend_runner,
        )
        try:
            return run_ocr2_backend_action(ocr2_action, options)
        except Ocr2BackendError as exc:
            sys.stderr.write(f"Error: {exc}\n")
            return 1

    location = f"grpc://{args.host}:{args.port}"
    server = DocumentExtractFlightServer(
        location,
        ocr_worker=build_pdf_ocr_worker(args.pdf_ocr_worker, args.pdf_ocr_workers),
    )
    sys.stdout.write(f"Wendao document extraction service listening on {location}\n")
    server.serve()
    return 0


def build_document_extract_argument_parser() -> argparse.ArgumentParser:
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
    ocr2_actions = parser.add_mutually_exclusive_group()
    ocr2_actions.add_argument(
        "--ocr2-fetch-models",
        action="store_const",
        const="fetch-models",
        dest="ocr2_backend_action",
        help="Fetch prebuilt DeepSeek-OCR-2 artifacts through the analyzer-owned backend manager.",
    )
    ocr2_actions.add_argument(
        "--ocr2-install-vllm-metal",
        action="store_const",
        const="install-vllm-metal",
        dest="ocr2_backend_action",
        help="Install the local vLLM Metal runtime used by macOS OCR2 probes.",
    )
    ocr2_actions.add_argument(
        "--ocr2-probe-vllm-metal",
        action="store_const",
        const="probe-vllm-metal",
        dest="ocr2_backend_action",
        help="Probe local vLLM Metal readiness without loading OCR2 weights.",
    )
    ocr2_actions.add_argument(
        "--ocr2-start-backend",
        action="store_const",
        const="start-backend",
        dest="ocr2_backend_action",
        help="Start the platform-selected OpenAI-compatible DeepSeek-OCR-2 backend.",
    )
    parser.set_defaults(ocr2_backend_action=None)
    parser.add_argument(
        "--ocr2-repo-id",
        default="",
        help="Hugging Face repo id for --ocr2-fetch-models.",
    )
    parser.add_argument(
        "--ocr2-model-dir",
        default="",
        help="Model directory name under PRJ_DATA_HOME/models for --ocr2-fetch-models.",
    )
    parser.add_argument(
        "--ocr2-model-path",
        default="",
        help="Local model path used by --ocr2-start-backend.",
    )
    parser.add_argument(
        "--ocr2-quantization",
        default="auto",
        help="vLLM quantization mode used by --ocr2-start-backend.",
    )
    parser.add_argument(
        "--ocr2-backend-runner",
        choices=("auto", "mlx-vlm", "metal-vllm", "generic-vllm", "official-vllm"),
        default="auto",
        help="Local OCR2 backend runner used by --ocr2-start-backend.",
    )
    return parser


def resolve_ocr2_backend_action(args: argparse.Namespace) -> Ocr2BackendAction | None:
    action = getattr(args, "ocr2_backend_action", None)
    return action


def build_pdf_ocr_worker(
    worker_name: str,
    max_workers: int | str | None = "auto",
) -> PdfOcrShardWorkerProtocol:
    if worker_name == "docling":
        return DoclingPdfOcrShardWorker(max_workers=max_workers)
    return SkippingPdfOcrShardWorker()


__all__ = [
    "build_document_extract_argument_parser",
    "build_pdf_ocr_worker",
    "document_extract_service_main",
    "resolve_ocr2_backend_action",
]
