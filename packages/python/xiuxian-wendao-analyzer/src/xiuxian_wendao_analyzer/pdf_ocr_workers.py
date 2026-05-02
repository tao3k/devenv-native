"""PDF OCR shard worker implementations."""

from __future__ import annotations

import threading
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from typing import TYPE_CHECKING, Any

from .pdf_ocr_contracts import PDF_OCR_PAGE_BREAK_SENTINEL
from .pdf_ocr_grouping import (
    _flatten_group_results,
    _group_pdf_ocr_inputs,
    _is_source_pdf_page_range_group,
    _should_try_source_pdf_page_range,
)
from .pdf_ocr_results import failed_pdf_ocr_shard_result, skipped_pdf_ocr_shard_result
from .pdf_ocr_tables import resolve_pdf_ocr_worker_count

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping, Sequence

    from .documents import DocumentConverterProtocol


class SkippingPdfOcrShardWorker:
    """Default no-model OCR worker used when no real engine is configured."""

    def recognize(
        self,
        inputs: Sequence[Mapping[str, Any]],
        *,
        max_workers: int | str | None = None,
    ) -> Sequence[Mapping[str, Any]]:
        _ = max_workers
        return [
            skipped_pdf_ocr_shard_result(
                input_row, "OCR shard worker is not configured"
            )
            for input_row in inputs
        ]


class DoclingPdfOcrShardWorker:
    """Docling-backed OCR worker for Rust-rendered PDF page images."""

    def __init__(
        self,
        converter: DocumentConverterProtocol | None = None,
        *,
        converter_factory: Callable[[], DocumentConverterProtocol] | None = None,
        max_workers: int | str | None = None,
    ) -> None:
        if converter is not None and converter_factory is not None:
            raise ValueError("converter and converter_factory are mutually exclusive")
        self._converter = converter
        self._converter_factory = converter_factory
        self._max_workers = max_workers
        self._thread_local = threading.local()

    def recognize(
        self,
        inputs: Sequence[Mapping[str, Any]],
        *,
        max_workers: int | str | None = None,
    ) -> Sequence[Mapping[str, Any]]:
        input_rows = list(inputs)
        recognition_groups = _group_pdf_ocr_inputs(input_rows)
        worker_count = resolve_pdf_ocr_worker_count(
            len(recognition_groups),
            max_workers if max_workers is not None else self._max_workers,
        )
        if self._converter is not None and self._converter_factory is None:
            worker_count = 1
        if worker_count <= 1:
            return _flatten_group_results(
                len(input_rows),
                [
                    self._recognize_group_with_thread_converter(indexes, rows)
                    for indexes, rows in recognition_groups
                ],
            )
        with ThreadPoolExecutor(max_workers=worker_count) as executor:
            futures = [
                executor.submit(
                    self._recognize_group_with_thread_converter,
                    indexes,
                    rows,
                )
                for indexes, rows in recognition_groups
            ]
            return _flatten_group_results(
                len(input_rows),
                [future.result() for future in futures],
            )

    def _recognize_group_with_thread_converter(
        self,
        indexes: Sequence[int],
        input_rows: Sequence[Mapping[str, Any]],
    ) -> list[tuple[int, Mapping[str, Any]]]:
        try:
            converter = self._converter_for_thread()
        except Exception as exc:
            return [
                (
                    index,
                    failed_pdf_ocr_shard_result(
                        input_row,
                        f"Docling OCR converter initialization failed: {exc}",
                    ),
                )
                for index, input_row in zip(indexes, input_rows, strict=True)
            ]
        return [
            (index, result)
            for index, result in zip(
                indexes,
                self._recognize_many(converter, input_rows),
                strict=True,
            )
        ]

    def _recognize_many(
        self,
        converter: DocumentConverterProtocol,
        input_rows: Sequence[Mapping[str, Any]],
    ) -> list[Mapping[str, Any]]:
        if len(input_rows) > 1 and _is_source_pdf_page_range_group(input_rows):
            source_path = Path(str(input_rows[0]["sourcePath"]))
            result = self._try_convert_source_page_batch(
                converter,
                input_rows,
                source_path,
            )
            if result is not None:
                return result
        return [self._recognize_one(converter, input_row) for input_row in input_rows]

    def _recognize_one(
        self,
        converter: DocumentConverterProtocol,
        input_row: Mapping[str, Any],
    ) -> Mapping[str, Any]:
        if _should_try_source_pdf_page_range(input_row):
            source_path = Path(str(input_row["sourcePath"]))
            result = self._try_convert_source_page(converter, input_row, source_path)
            if result is not None:
                return result

        image_path = Path(str(input_row["imagePath"]))
        if not image_path.is_file():
            return failed_pdf_ocr_shard_result(
                input_row,
                f"OCR shard image does not exist: {image_path}",
            )
        try:
            result = converter.convert(image_path)
            markdown = result.document.export_to_markdown()
        except Exception as exc:
            return failed_pdf_ocr_shard_result(input_row, f"Docling OCR failed: {exc}")
        if not markdown.strip():
            return failed_pdf_ocr_shard_result(
                input_row,
                "Docling OCR returned empty text",
            )
        return {
            "status": "succeeded",
            "text": markdown,
            "textMimeType": "text/markdown",
            "confidence": None,
            "errorMessage": None,
        }

    def _try_convert_source_page(
        self,
        converter: DocumentConverterProtocol,
        input_row: Mapping[str, Any],
        source_path: Path,
    ) -> Mapping[str, Any] | None:
        try:
            page_number = int(input_row["pageIndex"]) + 1
            result = converter.convert(
                source_path, page_range=(page_number, page_number)
            )
            markdown = result.document.export_to_markdown()
        except Exception:
            return None
        if not markdown.strip():
            return None
        return {
            "status": "succeeded",
            "text": markdown,
            "textMimeType": "text/markdown",
            "confidence": None,
            "errorMessage": None,
        }

    def _try_convert_source_page_batch(
        self,
        converter: DocumentConverterProtocol,
        input_rows: Sequence[Mapping[str, Any]],
        source_path: Path,
    ) -> list[Mapping[str, Any]] | None:
        try:
            start_page = int(input_rows[0]["pageIndex"]) + 1
            end_page = int(input_rows[-1]["pageIndex"]) + 1
            result = converter.convert(source_path, page_range=(start_page, end_page))
            page_markdowns = _try_export_source_page_batch_markdown(
                result.document,
                input_rows,
            )
            if page_markdowns is None:
                page_markdowns = [
                    result.document.export_to_markdown(
                        page_no=int(input_row["pageIndex"]) + 1
                    )
                    for input_row in input_rows
                ]
            rows = []
            for markdown in page_markdowns:
                if not markdown.strip():
                    return None
                rows.append(
                    {
                        "status": "succeeded",
                        "text": markdown,
                        "textMimeType": "text/markdown",
                        "confidence": None,
                        "errorMessage": None,
                    }
                )
        except Exception:
            return None
        return rows

    def _converter_for_thread(self) -> DocumentConverterProtocol:
        if self._converter is not None:
            return self._converter
        converter = getattr(self._thread_local, "converter", None)
        if converter is None:
            factory = self._converter_factory or _new_docling_converter
            converter = factory()
            self._thread_local.converter = converter
        return converter


def _try_export_source_page_batch_markdown(
    document: Any,
    input_rows: Sequence[Mapping[str, Any]],
) -> list[str] | None:
    try:
        markdown = document.export_to_markdown(
            page_break_placeholder=PDF_OCR_PAGE_BREAK_SENTINEL
        )
    except (AttributeError, TypeError, ValueError):
        return None
    if not isinstance(markdown, str):
        return None
    parts = [part.strip() for part in markdown.split(PDF_OCR_PAGE_BREAK_SENTINEL)]
    if len(parts) != len(input_rows):
        return None
    if any(not part for part in parts):
        return None
    return parts


def _new_docling_converter() -> DocumentConverterProtocol:
    try:
        from docling.document_converter import DocumentConverter
    except ModuleNotFoundError as exc:
        raise RuntimeError(
            "docling is not installed; install xiuxian-wendao-analyzer[documents] "
            "to enable Docling-backed PDF OCR shards"
        ) from exc
    return DocumentConverter()
