"""Command-line entrypoint for the Wendao document extraction Flight service."""

from __future__ import annotations

import sys

from .audio_backend import (
    AudioBackendError,
    AudioBackendOptions,
    run_audio_backend_action,
)
from .document_service import DocumentExtractFlightServer
from .document_service_parser import build_document_extract_argument_parser
from .document_service_workers import (
    build_audio_worker,
    build_pdf_ocr_worker,
    resolve_audio_backend_action,
    resolve_ocr2_backend_action,
)
from .ocr2_backend import (
    Ocr2BackendError,
    Ocr2BackendOptions,
    run_ocr2_backend_action,
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
    audio_action = resolve_audio_backend_action(args)
    if audio_action is not None:
        options = AudioBackendOptions(
            model_path=args.audio_backend_model_path,
            backend_runner=args.audio_backend_runner,
            host=args.audio_backend_host,
            port=args.audio_backend_port,
        )
        try:
            return run_audio_backend_action(audio_action, options)
        except AudioBackendError as exc:
            sys.stderr.write(f"Error: {exc}\n")
            return 1

    location = f"grpc://{args.host}:{args.port}"
    server = DocumentExtractFlightServer(
        location,
        ocr_worker=build_pdf_ocr_worker(args.pdf_ocr_worker, args.pdf_ocr_workers),
        audio_worker=build_audio_worker(args.audio_worker, args.audio_workers),
    )
    sys.stdout.write(f"Wendao document extraction service listening on {location}\n")
    server.serve()
    return 0


__all__ = [
    "build_audio_worker",
    "build_document_extract_argument_parser",
    "build_pdf_ocr_worker",
    "document_extract_service_main",
    "resolve_audio_backend_action",
    "resolve_ocr2_backend_action",
]
