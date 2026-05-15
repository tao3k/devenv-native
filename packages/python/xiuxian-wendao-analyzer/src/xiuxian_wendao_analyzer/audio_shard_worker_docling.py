"""Docling audio shard worker."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from .audio_shard_results import failed_audio_shard_result, succeeded_audio_shard_result
from .audio_shard_worker_common import map_audio_rows, short_error_message

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping, Sequence


class DoclingAudioShardWorker:
    """Docling-backed audio transcript worker."""

    def __init__(
        self,
        *,
        max_workers: int | str | None = "auto",
        converter_factory: Callable[[], Any] | None = None,
    ) -> None:
        self._max_workers = max_workers
        self._converter_factory = converter_factory or new_docling_audio_converter

    def process(
        self,
        inputs: Sequence[Mapping[str, Any]],
        *,
        max_workers: int | str | None = None,
    ) -> Sequence[Mapping[str, Any]]:
        return map_audio_rows(
            inputs, max_workers or self._max_workers, self._process_one
        )

    def _process_one(self, input_row: Mapping[str, Any]) -> Mapping[str, Any]:
        try:
            converter = self._converter_factory()
            result = converter.convert(input_row["shardPath"])
            text = docling_result_text(result)
        except Exception as exc:
            return failed_audio_shard_result(
                input_row,
                f"Docling audio worker failed: {short_error_message(exc)}",
            )
        if not text.strip():
            return failed_audio_shard_result(
                input_row,
                "Docling audio worker returned empty text",
            )
        return succeeded_audio_shard_result(input_row, text.strip(), 1.0)


def new_docling_audio_converter() -> Any:
    """Create the default Docling audio converter."""

    try:
        from docling.document_converter import DocumentConverter
    except ImportError as exc:
        raise RuntimeError(
            "docling is not installed; install xiuxian-wendao-analyzer[documents-audio]"
        ) from exc
    return DocumentConverter()


def docling_result_text(result: Any) -> str:
    """Extract transcript text from a Docling conversion result."""

    document = getattr(result, "document", result)
    export_to_markdown = getattr(document, "export_to_markdown", None)
    if callable(export_to_markdown):
        return str(export_to_markdown())
    export_to_text = getattr(document, "export_to_text", None)
    if callable(export_to_text):
        return str(export_to_text())
    return str(document)
