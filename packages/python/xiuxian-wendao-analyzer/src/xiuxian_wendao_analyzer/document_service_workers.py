"""Worker factories and action resolvers for the document extraction CLI."""

from __future__ import annotations

from typing import TYPE_CHECKING

from .audio_shard_workers import build_audio_shard_worker
from .pdf_ocr import (
    DoclingPdfOcrShardWorker,
    PdfOcrShardWorkerProtocol,
    SkippingPdfOcrShardWorker,
)

if TYPE_CHECKING:
    import argparse

    from .audio_backend import AudioBackendAction
    from .audio_shard_contracts import AudioShardWorkerProtocol
    from .ocr2_backend import Ocr2BackendAction


def resolve_ocr2_backend_action(args: argparse.Namespace) -> Ocr2BackendAction | None:
    action = getattr(args, "ocr2_backend_action", None)
    return action


def resolve_audio_backend_action(args: argparse.Namespace) -> AudioBackendAction | None:
    action = getattr(args, "audio_backend_action", None)
    return action


def build_pdf_ocr_worker(
    worker_name: str,
    max_workers: int | str | None = "auto",
) -> PdfOcrShardWorkerProtocol:
    if worker_name == "docling":
        return DoclingPdfOcrShardWorker(max_workers=max_workers)
    return SkippingPdfOcrShardWorker()


def build_audio_worker(
    worker_name: str | None = None,
    max_workers: int | str | None = "auto",
) -> AudioShardWorkerProtocol:
    return build_audio_shard_worker(worker_name, max_workers)
