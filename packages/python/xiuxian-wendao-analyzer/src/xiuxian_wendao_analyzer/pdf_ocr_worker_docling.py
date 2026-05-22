"""Docling-backed PDF OCR shard workers."""

from __future__ import annotations

import threading
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from typing import TYPE_CHECKING, Any

from .pdf_ocr_contracts import (
    PDF_OCR_BACKEND_TEXT_PROFILE,
    PDF_OCR_DEFAULT_PROFILE,
    PDF_OCR_FAST_TEXT_PROFILE,
    is_hosted_vlm_direct_profile,
)
from .pdf_ocr_grouping import (
    _flatten_group_results,
    _group_pdf_ocr_inputs,
    _is_source_pdf_page_range_group,
    _should_try_source_pdf_page_range,
)
from .pdf_ocr_ocr2 import (
    recognize_hosted_vlm_ocr_many as _recognize_hosted_vlm_ocr_many,
)
from .pdf_ocr_results import failed_pdf_ocr_shard_result, skipped_pdf_ocr_shard_result
from .pdf_ocr_tables import resolve_pdf_ocr_worker_count
from .pdf_ocr_worker_converter import (
    PDF_OCR_FAST_TEXT_SOURCE_BACKEND_TABLE_PROFILE,
    _factory_accepts_ocr_profile,
)
from .pdf_ocr_worker_fallback import (
    _verified_empty_backend_text_result,
    backend_text_empty_page_mode,
    backend_text_page_fallback_mode,
)
from .pdf_ocr_worker_options import (
    PDF_OCR_BACKEND_TEXT_EMPTY_PAGE_VERIFIED,
    PDF_OCR_BACKEND_TEXT_PAGE_FALLBACK_COMPATIBLE,
    PDF_OCR_FAST_TEXT_SOURCE_CONVERTER_BACKEND_TABLE,
    _ocr_profile,
    _try_export_source_page_batch_markdown,
    fast_text_source_converter_mode,
)
from .pdf_ocr_worker_prewarm import (
    PrewarmedSourcePageCache,
    _prewarm_converter_from_env,
    _prewarm_profiles_from_env,
    _prewarmed_source_page_key,
)

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping, Sequence

    from .documents import DocumentConverterProtocol


def _new_docling_converter_via_facade(
    ocr_profile: str = PDF_OCR_DEFAULT_PROFILE,
) -> DocumentConverterProtocol:
    from . import pdf_ocr_workers

    return pdf_ocr_workers._new_docling_converter(ocr_profile)


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
        converter_factory: (
            Callable[[], DocumentConverterProtocol]
            | Callable[[str], DocumentConverterProtocol]
            | None
        ) = None,
        max_workers: int | str | None = None,
    ) -> None:
        if converter is not None and converter_factory is not None:
            raise ValueError("converter and converter_factory are mutually exclusive")
        self._converter = converter
        self._converter_factory = converter_factory
        self._converter_factory_accepts_profile = _factory_accepts_ocr_profile(
            converter_factory
        )
        self._max_workers = max_workers
        self._thread_local = threading.local()
        self._shared_converters: dict[str, DocumentConverterProtocol] = {}
        self._prewarmed_source_pages: PrewarmedSourcePageCache = {}
        for profile in _prewarm_profiles_from_env():
            converter_profile = _source_pdf_converter_profile(profile)
            converter = self._build_converter(converter_profile)
            self._prewarmed_source_pages.update(
                _prewarm_converter_from_env(
                    converter,
                    converter_profile=converter_profile,
                )
            )
            self._shared_converters[converter_profile] = converter

    def recognize(
        self,
        inputs: Sequence[Mapping[str, Any]],
        *,
        max_workers: int | str | None = None,
    ) -> Sequence[Mapping[str, Any]]:
        input_rows = list(inputs)
        recognition_groups = _group_pdf_ocr_inputs(input_rows)
        effective_max_workers = (
            max_workers if max_workers is not None else self._max_workers
        )
        worker_count = resolve_pdf_ocr_worker_count(
            len(recognition_groups),
            effective_max_workers,
        )
        if self._converter is not None and self._converter_factory is None:
            worker_count = 1
        if worker_count <= 1:
            return _flatten_group_results(
                len(input_rows),
                [
                    self._recognize_group_with_thread_converter(
                        indexes,
                        rows,
                        max_workers,
                        True,
                    )
                    for indexes, rows in recognition_groups
                ],
            )
        with ThreadPoolExecutor(max_workers=worker_count) as executor:
            futures = [
                executor.submit(
                    self._recognize_group_with_thread_converter,
                    indexes,
                    rows,
                    max_workers,
                    False,
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
        max_workers: int | str | None,
        allow_shared_converter: bool,
    ) -> list[tuple[int, Mapping[str, Any]]]:
        ocr_profile = _ocr_profile(input_rows[0])
        if is_hosted_vlm_direct_profile(ocr_profile):
            return [
                (index, result)
                for index, result in zip(
                    indexes,
                    _recognize_hosted_vlm_ocr_many(
                        input_rows, request_concurrency=max_workers
                    ),
                    strict=True,
                )
            ]
        try:
            converter_profile = _source_pdf_group_converter_profile(
                ocr_profile,
                input_rows,
            )
            converter = self._converter_for_thread(
                converter_profile,
                allow_shared=allow_shared_converter,
            )
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
                self._recognize_many(converter, input_rows, converter_profile),
                strict=True,
            )
        ]

    def _recognize_many(
        self,
        converter: DocumentConverterProtocol,
        input_rows: Sequence[Mapping[str, Any]],
        converter_profile: str,
    ) -> list[Mapping[str, Any]]:
        if len(input_rows) > 1 and _is_source_pdf_page_range_group(input_rows):
            source_path = Path(str(input_rows[0]["sourcePath"]))
            cached_result = self._try_prewarmed_source_page_batch(
                converter_profile,
                input_rows,
                source_path,
            )
            if cached_result is not None:
                return cached_result
            result = self._try_convert_source_page_batch(
                converter,
                input_rows,
                source_path,
                _ocr_profile(input_rows[0]),
                converter_profile,
            )
            if result is not None:
                return result
        return [
            self._recognize_one(converter, input_row, converter_profile)
            for input_row in input_rows
        ]

    def _recognize_one(
        self,
        converter: DocumentConverterProtocol,
        input_row: Mapping[str, Any],
        converter_profile: str,
    ) -> Mapping[str, Any]:
        ocr_profile = _ocr_profile(input_row)
        if _should_try_source_pdf_page_range(input_row):
            source_path = Path(str(input_row["sourcePath"]))
            result = self._try_convert_source_page(
                converter,
                input_row,
                source_path,
                converter_profile,
            )
            if result is None and ocr_profile == PDF_OCR_BACKEND_TEXT_PROFILE:
                fast_text_profile = _source_pdf_converter_profile(
                    PDF_OCR_FAST_TEXT_PROFILE
                )
                result = self._try_convert_source_page(
                    self._converter_for_thread(
                        fast_text_profile,
                    ),
                    input_row,
                    source_path,
                    fast_text_profile,
                )
                if result is None and backend_text_page_fallback_mode() == (
                    PDF_OCR_BACKEND_TEXT_PAGE_FALLBACK_COMPATIBLE
                ):
                    result = self._try_convert_source_page(
                        self._converter_for_thread(PDF_OCR_DEFAULT_PROFILE),
                        input_row,
                        source_path,
                        PDF_OCR_DEFAULT_PROFILE,
                    )
                if result is None and backend_text_empty_page_mode() == (
                    PDF_OCR_BACKEND_TEXT_EMPTY_PAGE_VERIFIED
                ):
                    result = _verified_empty_backend_text_result()
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
        converter_profile: str,
    ) -> Mapping[str, Any] | None:
        cached_markdown = self._prewarmed_source_page_markdown(
            converter_profile,
            input_row,
            source_path,
        )
        if cached_markdown is not None:
            return _succeeded_markdown_result(cached_markdown)
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
        return _succeeded_markdown_result(markdown)

    def _try_convert_source_page_batch(
        self,
        converter: DocumentConverterProtocol,
        input_rows: Sequence[Mapping[str, Any]],
        source_path: Path,
        ocr_profile: str,
        converter_profile: str,
    ) -> list[Mapping[str, Any]] | None:
        cached_result = self._try_prewarmed_source_page_batch(
            converter_profile,
            input_rows,
            source_path,
        )
        if cached_result is not None:
            return cached_result
        try:
            start_page = int(input_rows[0]["pageIndex"]) + 1
            end_page = int(input_rows[-1]["pageIndex"]) + 1
            result = converter.convert(source_path, page_range=(start_page, end_page))
            page_markdowns = _try_export_source_page_batch_markdown(
                result.document,
                input_rows,
                allow_empty=ocr_profile == PDF_OCR_BACKEND_TEXT_PROFILE,
            )
            if page_markdowns is None:
                page_markdowns = [
                    result.document.export_to_markdown(
                        page_no=int(input_row["pageIndex"]) + 1
                    )
                    for input_row in input_rows
                ]
            rows = []
            fallback_converter = None
            fallback_converter_profile = _source_pdf_converter_profile(
                PDF_OCR_FAST_TEXT_PROFILE
            )
            for input_row, markdown in zip(input_rows, page_markdowns, strict=True):
                if not markdown.strip():
                    if ocr_profile != PDF_OCR_BACKEND_TEXT_PROFILE:
                        return None
                    if fallback_converter is None:
                        fallback_converter = self._converter_for_thread(
                            fallback_converter_profile
                        )
                    fallback = self._try_convert_source_page(
                        fallback_converter,
                        input_row,
                        source_path,
                        fallback_converter_profile,
                    )
                    if fallback is None and backend_text_page_fallback_mode() == (
                        PDF_OCR_BACKEND_TEXT_PAGE_FALLBACK_COMPATIBLE
                    ):
                        fallback = self._try_convert_source_page(
                            self._converter_for_thread(PDF_OCR_DEFAULT_PROFILE),
                            input_row,
                            source_path,
                            PDF_OCR_DEFAULT_PROFILE,
                        )
                    if fallback is None and backend_text_empty_page_mode() == (
                        PDF_OCR_BACKEND_TEXT_EMPTY_PAGE_VERIFIED
                    ):
                        fallback = _verified_empty_backend_text_result()
                    if fallback is None:
                        return None
                    rows.append(fallback)
                    continue
                rows.append(_succeeded_markdown_result(markdown))
        except Exception:
            return None
        return rows

    def _try_prewarmed_source_page_batch(
        self,
        converter_profile: str,
        input_rows: Sequence[Mapping[str, Any]],
        source_path: Path,
    ) -> list[Mapping[str, Any]] | None:
        markdown_rows = [
            self._prewarmed_source_page_markdown(
                converter_profile,
                input_row,
                source_path,
            )
            for input_row in input_rows
        ]
        if any(markdown is None for markdown in markdown_rows):
            return None
        return [
            _succeeded_markdown_result(markdown)
            for markdown in markdown_rows
            if markdown is not None
        ]

    def _prewarmed_source_page_markdown(
        self,
        converter_profile: str,
        input_row: Mapping[str, Any],
        source_path: Path,
    ) -> str | None:
        try:
            page_index = int(input_row["pageIndex"])
            key = _prewarmed_source_page_key(converter_profile, source_path, page_index)
        except (OSError, TypeError, ValueError):
            return None
        markdown = self._prewarmed_source_pages.get(key)
        if markdown is None or not markdown.strip():
            return None
        return markdown

    def _converter_for_thread(
        self,
        ocr_profile: str,
        *,
        allow_shared: bool = False,
    ) -> DocumentConverterProtocol:
        if self._converter is not None:
            return self._converter
        if allow_shared and ocr_profile in self._shared_converters:
            return self._shared_converters[ocr_profile]
        converters = getattr(self._thread_local, "converters", None)
        if converters is None:
            converters = {}
            self._thread_local.converters = converters
        converter = converters.get(ocr_profile)
        if converter is None:
            converter = self._build_converter(ocr_profile)
            converters[ocr_profile] = converter
        return converter

    def _build_converter(self, ocr_profile: str) -> DocumentConverterProtocol:
        if self._converter_factory is not None:
            if self._converter_factory_accepts_profile:
                return self._converter_factory(ocr_profile)
            return self._converter_factory()
        return _new_docling_converter_via_facade(ocr_profile)


def _source_pdf_group_converter_profile(
    ocr_profile: str,
    input_rows: Sequence[Mapping[str, Any]],
) -> str:
    if _is_source_pdf_page_range_group(input_rows):
        return _source_pdf_converter_profile(ocr_profile)
    return ocr_profile


def _source_pdf_converter_profile(ocr_profile: str) -> str:
    if (
        ocr_profile == PDF_OCR_FAST_TEXT_PROFILE
        and fast_text_source_converter_mode()
        == PDF_OCR_FAST_TEXT_SOURCE_CONVERTER_BACKEND_TABLE
    ):
        return PDF_OCR_FAST_TEXT_SOURCE_BACKEND_TABLE_PROFILE
    return ocr_profile


def _succeeded_markdown_result(markdown: str) -> Mapping[str, Any]:
    return {
        "status": "succeeded",
        "text": markdown,
        "textMimeType": "text/markdown",
        "confidence": None,
        "errorMessage": None,
    }
