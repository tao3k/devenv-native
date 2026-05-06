"""PDF OCR shard worker implementations."""

from __future__ import annotations

import base64
import json
import os
import threading
import time
import urllib.error
import urllib.request
from collections.abc import Callable, Mapping, Sequence
from concurrent.futures import ThreadPoolExecutor
from inspect import signature
from pathlib import Path
from typing import TYPE_CHECKING, Any

from .pdf_ocr_contracts import (
    DEEPSEEK_OCR2_API_KEY_ENV,
    DEEPSEEK_OCR2_BASE_URL_ENV,
    DEEPSEEK_OCR2_DEFAULT_API_KEY,
    DEEPSEEK_OCR2_DEFAULT_BASE_URL,
    DEEPSEEK_OCR2_DEFAULT_MAX_TOKENS,
    DEEPSEEK_OCR2_DEFAULT_MODEL,
    DEEPSEEK_OCR2_DEFAULT_PAGE_WINDOW_SIZE,
    DEEPSEEK_OCR2_DEFAULT_PROMPT,
    DEEPSEEK_OCR2_DEFAULT_REGION_COMPOSITE_SIZE,
    DEEPSEEK_OCR2_DEFAULT_REGION_MAX_TOKENS,
    DEEPSEEK_OCR2_DEFAULT_REQUEST_CONCURRENCY,
    DEEPSEEK_OCR2_DEFAULT_TIMEOUT_SECONDS,
    DEEPSEEK_OCR2_MAX_TOKENS_ENV,
    DEEPSEEK_OCR2_MODEL_ENV,
    DEEPSEEK_OCR2_OPENROUTE_COMPAT_API_KEY_ENV,
    DEEPSEEK_OCR2_OPENROUTER_API_KEY_ENV,
    DEEPSEEK_OCR2_OPENROUTER_BASE_URL,
    DEEPSEEK_OCR2_OPENROUTER_HTTP_REFERER_ENV,
    DEEPSEEK_OCR2_OPENROUTER_MODEL_ENV,
    DEEPSEEK_OCR2_OPENROUTER_PROVIDER,
    DEEPSEEK_OCR2_OPENROUTER_PUBLIC_API_KEY_ENV,
    DEEPSEEK_OCR2_OPENROUTER_TEST_MODEL,
    DEEPSEEK_OCR2_OPENROUTER_TITLE_ENV,
    DEEPSEEK_OCR2_PAGE_WINDOW_SIZE_ENV,
    DEEPSEEK_OCR2_PROMPT_ENV,
    DEEPSEEK_OCR2_PROVIDER_ENV,
    DEEPSEEK_OCR2_REGION_COMPOSITE_SIZE_ENV,
    DEEPSEEK_OCR2_REGION_MAX_TOKENS_ENV,
    DEEPSEEK_OCR2_REQUEST_CONCURRENCY_ENV,
    DEEPSEEK_OCR2_TIMEOUT_SECONDS_ENV,
    DEEPSEEK_OCR2_TRACE_PATH_ENV,
    PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE,
    PDF_OCR_DEFAULT_PROFILE,
    PDF_OCR_DOCLING_VLM_DEEPSEEK_OCR_PROFILE,
    PDF_OCR_FAST_TEXT_PROFILE,
    PDF_OCR_PAGE_BREAK_SENTINEL,
)
from .pdf_ocr_grouping import (
    _flatten_group_results,
    _group_pdf_ocr_inputs,
    _is_source_pdf_page_range_group,
    _should_try_source_pdf_page_range,
)
from .pdf_ocr_results import failed_pdf_ocr_shard_result, skipped_pdf_ocr_shard_result
from .pdf_ocr_tables import resolve_pdf_ocr_worker_count

if TYPE_CHECKING:
    from .documents import DocumentConverterProtocol


_DEEPSEEK_OCR2_TRACE_LOCK = threading.Lock()
_DEEPSEEK_OCR2_PAGE_WINDOW_PROBE_LOCK = threading.Lock()
_DEEPSEEK_OCR2_PAGE_WINDOW_COMPATIBILITY: dict[tuple[str, int], bool] = {}
_DEEPSEEK_OCR2_PAGE_MARKER_PREFIX = "<!-- xiuxian-wendao-ocr2-page:"
_DEEPSEEK_OCR2_PAGE_MARKER_SUFFIX = " -->"
_DEEPSEEK_OCR2_REGION_MARKER_PREFIX = "<!-- xiuxian-wendao-ocr2-region:"
_DEEPSEEK_OCR2_REGION_MARKER_SUFFIX = " -->"
_DEEPSEEK_OCR2_TRANSIENT_HTTP_STATUS = {408, 409, 425, 429, 500, 502, 503, 504}
_DEEPSEEK_OCR2_MAX_TRANSIENT_RETRIES = 3
_DEEPSEEK_OCR2_RETRY_BASE_SECONDS = 0.25
_DEEPSEEK_OCR2_RATE_LIMIT_RETRY_BASE_SECONDS = 2.0
_DEEPSEEK_OCR2_MAX_RETRY_DELAY_SECONDS = 8.0
_DEEPSEEK_OCR2_GROUP_RETRY_DELAYS_SECONDS = (8.0, 16.0)


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
    ) -> list[tuple[int, Mapping[str, Any]]]:
        ocr_profile = _ocr_profile(input_rows[0])
        if ocr_profile == PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE:
            return [
                (index, result)
                for index, result in zip(
                    indexes,
                    _recognize_deepseek_ocr2_many(
                        input_rows, request_concurrency=max_workers
                    ),
                    strict=True,
                )
            ]
        try:
            converter = self._converter_for_thread(ocr_profile)
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

    def _converter_for_thread(self, ocr_profile: str) -> DocumentConverterProtocol:
        if self._converter is not None:
            return self._converter
        converters = getattr(self._thread_local, "converters", None)
        if converters is None:
            converters = {}
            self._thread_local.converters = converters
        converter = converters.get(ocr_profile)
        if converter is None:
            if self._converter_factory is not None:
                if self._converter_factory_accepts_profile:
                    converter = self._converter_factory(ocr_profile)
                else:
                    converter = self._converter_factory()
            else:
                converter = _new_docling_converter(ocr_profile)
            converters[ocr_profile] = converter
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


def _ocr_profile(input_row: Mapping[str, Any]) -> str:
    profile = str(input_row.get("ocrProfile", "")).strip()
    return profile or PDF_OCR_DEFAULT_PROFILE


def _factory_accepts_ocr_profile(factory: Callable[..., Any] | None) -> bool:
    if factory is None:
        return False
    try:
        signature(factory).bind(PDF_OCR_DEFAULT_PROFILE)
    except (TypeError, ValueError):
        return False
    return True


def _new_docling_converter(
    ocr_profile: str = PDF_OCR_DEFAULT_PROFILE,
) -> DocumentConverterProtocol:
    try:
        from docling.datamodel.base_models import InputFormat
        from docling.datamodel.pipeline_options import (
            PdfPipelineOptions,
            TableFormerMode,
        )
        from docling.document_converter import DocumentConverter, PdfFormatOption
    except ModuleNotFoundError as exc:
        raise RuntimeError(
            "docling is not installed; install xiuxian-wendao-analyzer[documents] "
            "to enable Docling-backed PDF OCR shards"
        ) from exc
    if ocr_profile == PDF_OCR_FAST_TEXT_PROFILE:
        options = PdfPipelineOptions()
        options.table_structure_options.mode = TableFormerMode.FAST
        return DocumentConverter(
            format_options={
                InputFormat.PDF: PdfFormatOption(pipeline_options=options),
            }
        )
    if ocr_profile == PDF_OCR_DOCLING_VLM_DEEPSEEK_OCR_PROFILE:
        try:
            from docling.datamodel.base_models import InputFormat
            from docling.datamodel.pipeline_options import (
                VlmConvertOptions,
                VlmPipelineOptions,
            )
            from docling.document_converter import DocumentConverter, PdfFormatOption
            from docling.pipeline.vlm_pipeline import VlmPipeline
        except ModuleNotFoundError as exc:
            raise RuntimeError(
                "Docling VLM dependencies are not installed; install "
                "xiuxian-wendao-analyzer[documents] to enable Docling VLM OCR"
            ) from exc
        vlm_options = VlmConvertOptions.from_preset("deepseek_ocr")
        pipeline_options = VlmPipelineOptions(
            enable_remote_services=True,
            vlm_options=vlm_options,
        )
        return DocumentConverter(
            format_options={
                InputFormat.PDF: PdfFormatOption(
                    pipeline_cls=VlmPipeline,
                    pipeline_options=pipeline_options,
                ),
            }
        )
    return DocumentConverter()


def _recognize_deepseek_ocr2_many(
    input_rows: Sequence[Mapping[str, Any]],
    *,
    request_concurrency: int | str | None = None,
) -> list[Mapping[str, Any]]:
    try:
        client = _DeepSeekOcr2OpenAiClient.from_env(
            request_concurrency=request_concurrency
        )
    except ValueError as exc:
        return [
            failed_pdf_ocr_shard_result(input_row, f"DeepSeek-OCR-2 OCR failed: {exc}")
            for input_row in input_rows
        ]
    return client.recognize_many(input_rows)


class _DeepSeekOcr2OpenAiClient:
    def __init__(
        self,
        *,
        base_url: str,
        model: str,
        api_key: str,
        prompt: str,
        max_tokens: int,
        region_max_tokens: int,
        region_composite_size: int,
        timeout_seconds: float,
        request_concurrency: int,
        page_window_size: int,
        trace_path: Path | None = None,
        extra_headers: Mapping[str, str] | None = None,
    ) -> None:
        self._completion_url = _chat_completion_url(base_url)
        self._model = model
        self._api_key = api_key
        self._prompt = prompt
        self._max_tokens = max_tokens
        self._region_max_tokens = region_max_tokens
        self._region_composite_size = region_composite_size
        self._timeout_seconds = timeout_seconds
        self._request_concurrency = request_concurrency
        self._page_window_size = page_window_size
        self._trace_path = trace_path
        self._extra_headers = dict(extra_headers or {})

    @classmethod
    def from_env(
        cls,
        *,
        request_concurrency: int | str | None = None,
    ) -> _DeepSeekOcr2OpenAiClient:
        resolved_request_concurrency = _positive_int_value(request_concurrency)
        provider = _env_value(DEEPSEEK_OCR2_PROVIDER_ENV, "")
        if provider == DEEPSEEK_OCR2_OPENROUTER_PROVIDER:
            return cls(
                base_url=_env_value(
                    DEEPSEEK_OCR2_BASE_URL_ENV,
                    DEEPSEEK_OCR2_OPENROUTER_BASE_URL,
                ),
                model=_env_value(
                    DEEPSEEK_OCR2_MODEL_ENV,
                    _env_value(
                        DEEPSEEK_OCR2_OPENROUTER_MODEL_ENV,
                        DEEPSEEK_OCR2_OPENROUTER_TEST_MODEL,
                    ),
                ),
                api_key=_resolve_openrouter_api_key(),
                prompt=_env_value(
                    DEEPSEEK_OCR2_PROMPT_ENV, DEEPSEEK_OCR2_DEFAULT_PROMPT
                ),
                max_tokens=_positive_int_env(
                    DEEPSEEK_OCR2_MAX_TOKENS_ENV,
                    DEEPSEEK_OCR2_DEFAULT_MAX_TOKENS,
                ),
                region_max_tokens=_positive_int_env(
                    DEEPSEEK_OCR2_REGION_MAX_TOKENS_ENV,
                    DEEPSEEK_OCR2_DEFAULT_REGION_MAX_TOKENS,
                ),
                region_composite_size=_positive_int_env(
                    DEEPSEEK_OCR2_REGION_COMPOSITE_SIZE_ENV,
                    DEEPSEEK_OCR2_DEFAULT_REGION_COMPOSITE_SIZE,
                ),
                timeout_seconds=_positive_float_env(
                    DEEPSEEK_OCR2_TIMEOUT_SECONDS_ENV,
                    DEEPSEEK_OCR2_DEFAULT_TIMEOUT_SECONDS,
                ),
                request_concurrency=(
                    resolved_request_concurrency
                    or _positive_int_env(
                        DEEPSEEK_OCR2_REQUEST_CONCURRENCY_ENV,
                        DEEPSEEK_OCR2_DEFAULT_REQUEST_CONCURRENCY,
                    )
                ),
                page_window_size=_positive_int_env(
                    DEEPSEEK_OCR2_PAGE_WINDOW_SIZE_ENV,
                    DEEPSEEK_OCR2_DEFAULT_PAGE_WINDOW_SIZE,
                ),
                trace_path=_optional_path_env(DEEPSEEK_OCR2_TRACE_PATH_ENV),
                extra_headers=_openrouter_headers(),
            )
        if provider and provider != "openai-compatible":
            raise ValueError(
                f"unsupported {DEEPSEEK_OCR2_PROVIDER_ENV}={provider}; "
                "supported values: openai-compatible, openrouter"
            )
        return cls(
            base_url=_env_value(
                DEEPSEEK_OCR2_BASE_URL_ENV,
                DEEPSEEK_OCR2_DEFAULT_BASE_URL,
            ),
            model=_env_value(
                DEEPSEEK_OCR2_MODEL_ENV,
                DEEPSEEK_OCR2_DEFAULT_MODEL,
            ),
            api_key=_env_value(
                DEEPSEEK_OCR2_API_KEY_ENV,
                DEEPSEEK_OCR2_DEFAULT_API_KEY,
            ),
            prompt=_env_value(
                DEEPSEEK_OCR2_PROMPT_ENV,
                DEEPSEEK_OCR2_DEFAULT_PROMPT,
            ),
            max_tokens=_positive_int_env(
                DEEPSEEK_OCR2_MAX_TOKENS_ENV,
                DEEPSEEK_OCR2_DEFAULT_MAX_TOKENS,
            ),
            region_max_tokens=_positive_int_env(
                DEEPSEEK_OCR2_REGION_MAX_TOKENS_ENV,
                DEEPSEEK_OCR2_DEFAULT_REGION_MAX_TOKENS,
            ),
            region_composite_size=_positive_int_env(
                DEEPSEEK_OCR2_REGION_COMPOSITE_SIZE_ENV,
                DEEPSEEK_OCR2_DEFAULT_REGION_COMPOSITE_SIZE,
            ),
            timeout_seconds=_positive_float_env(
                DEEPSEEK_OCR2_TIMEOUT_SECONDS_ENV,
                DEEPSEEK_OCR2_DEFAULT_TIMEOUT_SECONDS,
            ),
            request_concurrency=(
                resolved_request_concurrency
                or _positive_int_env(
                    DEEPSEEK_OCR2_REQUEST_CONCURRENCY_ENV,
                    DEEPSEEK_OCR2_DEFAULT_REQUEST_CONCURRENCY,
                )
            ),
            page_window_size=_positive_int_env(
                DEEPSEEK_OCR2_PAGE_WINDOW_SIZE_ENV,
                DEEPSEEK_OCR2_DEFAULT_PAGE_WINDOW_SIZE,
            ),
            trace_path=_optional_path_env(DEEPSEEK_OCR2_TRACE_PATH_ENV),
        )

    def recognize_many(
        self,
        input_rows: Sequence[Mapping[str, Any]],
    ) -> list[Mapping[str, Any]]:
        rows = list(input_rows)
        if self._region_composite_size > 1:
            region_tasks = _ocr2_region_composite_tasks(
                rows,
                self._region_composite_size,
            )
            if any(len(task) > 1 for task in region_tasks):
                return self._retry_transient_failed_results(
                    rows,
                    self._recognize_region_composite_tasks(region_tasks),
                )
        if self._page_window_size <= 1:
            return self._retry_transient_failed_results(
                rows,
                self._recognize_page_tasks_once(rows),
            )
        windows = _ocr2_page_windows(rows, self._page_window_size)
        if len(windows) == len(rows) and all(len(window) == 1 for window in windows):
            return self._retry_transient_failed_results(
                rows,
                self._recognize_page_tasks_once(rows),
            )
        probe_index = next(
            (index for index, window in enumerate(windows) if len(window) > 1),
            None,
        )
        if probe_index is None:
            return self._retry_transient_failed_results(
                rows,
                self._recognize_page_tasks_once(rows),
            )
        probe_result = self._probe_page_window_compatibility(windows[probe_index])
        if probe_result is None:
            return self._retry_transient_failed_results(
                rows,
                self._recognize_page_tasks_once(rows),
            )
        if len(windows) == 1:
            return self._retry_transient_failed_results(rows, probe_result)
        window_results: list[list[Mapping[str, Any]] | None] = [None] * len(windows)
        window_results[probe_index] = probe_result
        pending_windows = [
            (index, window)
            for index, window in enumerate(windows)
            if index != probe_index
        ]
        for index, result in self._recognize_indexed_window_tasks(pending_windows):
            window_results[index] = result
        return self._retry_transient_failed_results(
            rows,
            _flatten_page_window_results(
                [result for result in window_results if result is not None]
            ),
        )

    def _probe_page_window_compatibility(
        self,
        input_rows: Sequence[Mapping[str, Any]],
    ) -> list[Mapping[str, Any]] | None:
        compatibility_key = (self._model, self._page_window_size)
        with _DEEPSEEK_OCR2_PAGE_WINDOW_PROBE_LOCK:
            is_compatible = _DEEPSEEK_OCR2_PAGE_WINDOW_COMPATIBILITY.get(
                compatibility_key
            )
            if is_compatible is False:
                return None
            if is_compatible is True:
                should_probe = False
            else:
                should_probe = True
                probe_result = self._try_recognize_page_window(input_rows)
                _DEEPSEEK_OCR2_PAGE_WINDOW_COMPATIBILITY[compatibility_key] = (
                    probe_result is not None
                )
                return probe_result
        if not should_probe:
            return self._try_recognize_page_window(input_rows)
        return None

    def _recognize_page_tasks_once(
        self,
        rows: Sequence[Mapping[str, Any]],
    ) -> list[Mapping[str, Any]]:
        if self._request_concurrency <= 1 or len(rows) <= 1:
            return [self.recognize(input_row) for input_row in rows]
        worker_count = min(self._request_concurrency, len(rows))
        with ThreadPoolExecutor(max_workers=worker_count) as executor:
            futures = [executor.submit(self.recognize, input_row) for input_row in rows]
            return [future.result() for future in futures]

    def _retry_transient_failed_results(
        self,
        rows: Sequence[Mapping[str, Any]],
        results: Sequence[Mapping[str, Any]],
    ) -> list[Mapping[str, Any]]:
        output = list(results)
        if len(output) != len(rows):
            return output
        for delay_seconds in _DEEPSEEK_OCR2_GROUP_RETRY_DELAYS_SECONDS:
            retry_indexes = [
                index
                for index, result in enumerate(output)
                if _is_transient_ocr2_failure(result)
            ]
            if not retry_indexes:
                break
            time.sleep(delay_seconds)
            retry_rows = [rows[index] for index in retry_indexes]
            retry_results = self._recognize_page_tasks_once(retry_rows)
            for index, result in zip(retry_indexes, retry_results, strict=True):
                output[index] = result
        return output

    def _recognize_window_tasks(
        self,
        windows: Sequence[Sequence[Mapping[str, Any]]],
    ) -> list[list[Mapping[str, Any]]]:
        if self._request_concurrency <= 1 or len(windows) <= 1:
            return [self.recognize_window(window) for window in windows]
        worker_count = min(self._request_concurrency, len(windows))
        with ThreadPoolExecutor(max_workers=worker_count) as executor:
            futures = [
                executor.submit(self.recognize_window, window) for window in windows
            ]
            return [future.result() for future in futures]

    def _recognize_indexed_window_tasks(
        self,
        windows: Sequence[tuple[int, Sequence[Mapping[str, Any]]]],
    ) -> list[tuple[int, list[Mapping[str, Any]]]]:
        if self._request_concurrency <= 1 or len(windows) <= 1:
            return [(index, self.recognize_window(window)) for index, window in windows]
        worker_count = min(self._request_concurrency, len(windows))
        with ThreadPoolExecutor(max_workers=worker_count) as executor:
            futures = [
                executor.submit(self.recognize_window, window)
                for _index, window in windows
            ]
            return [
                (index, future.result())
                for (index, _window), future in zip(windows, futures, strict=True)
            ]

    def _recognize_region_composite_tasks(
        self,
        tasks: Sequence[Sequence[Mapping[str, Any]]],
    ) -> list[Mapping[str, Any]]:
        if self._request_concurrency <= 1 or len(tasks) <= 1:
            return _flatten_page_window_results(
                [self.recognize_region_composite(task) for task in tasks]
            )
        worker_count = min(self._request_concurrency, len(tasks))
        with ThreadPoolExecutor(max_workers=worker_count) as executor:
            futures = [
                executor.submit(self.recognize_region_composite, task) for task in tasks
            ]
            return _flatten_page_window_results([future.result() for future in futures])

    def recognize_window(
        self,
        input_rows: Sequence[Mapping[str, Any]],
    ) -> list[Mapping[str, Any]]:
        rows = list(input_rows)
        if len(rows) <= 1:
            return [self.recognize(row) for row in rows]
        batch_result = self._try_recognize_page_window(rows)
        if batch_result is not None:
            return batch_result
        return [self.recognize(row) for row in rows]

    def recognize_region_composite(
        self,
        input_rows: Sequence[Mapping[str, Any]],
    ) -> list[Mapping[str, Any]]:
        rows = list(input_rows)
        if len(rows) <= 1:
            return [self.recognize(row) for row in rows]
        composite_result = self._try_recognize_region_composite(rows)
        if composite_result is not None:
            return composite_result
        return [self.recognize(row) for row in rows]

    def _try_recognize_page_window(
        self,
        input_rows: Sequence[Mapping[str, Any]],
    ) -> list[Mapping[str, Any]] | None:
        image_paths = [Path(str(input_row["imagePath"])) for input_row in input_rows]
        missing_path = next(
            (image_path for image_path in image_paths if not image_path.is_file()),
            None,
        )
        if missing_path is not None:
            return None
        image_bytes = sum(image_path.stat().st_size for image_path in image_paths)
        started = time.perf_counter()
        http_status = None
        try:
            payload = self._window_request_payload(input_rows, image_paths)
            http_status, response_payload = self._send_completion_request(payload)
            markdown = _extract_openai_message_content(response_payload)
            page_texts = _extract_ocr2_page_window_markdown(markdown, input_rows)
        except urllib.error.HTTPError as exc:
            self._write_trace(
                input_rows[0],
                status="failed",
                started=started,
                http_status=exc.code,
                image_bytes=image_bytes,
                markdown_chars=0,
                error=exc,
                page_count=len(input_rows),
                input_rows=input_rows,
                max_tokens=self._max_tokens,
            )
            return None
        except (OSError, ValueError, urllib.error.URLError) as exc:
            self._write_trace(
                input_rows[0],
                status="failed",
                started=started,
                http_status=http_status,
                image_bytes=image_bytes,
                markdown_chars=0,
                error=exc,
                page_count=len(input_rows),
                input_rows=input_rows,
                max_tokens=self._max_tokens,
            )
            return None
        self._write_trace(
            input_rows[0],
            status="succeeded",
            started=started,
            http_status=http_status,
            image_bytes=image_bytes,
            markdown_chars=sum(len(text) for text in page_texts),
            error=None,
            page_count=len(input_rows),
            input_rows=input_rows,
            max_tokens=self._max_tokens,
        )
        return [
            {
                "status": "succeeded",
                "text": text,
                "textMimeType": "text/markdown",
                "confidence": None,
                "errorMessage": None,
            }
            for text in page_texts
        ]

    def _try_recognize_region_composite(
        self,
        input_rows: Sequence[Mapping[str, Any]],
    ) -> list[Mapping[str, Any]] | None:
        image_paths = [Path(str(input_row["imagePath"])) for input_row in input_rows]
        missing_path = next(
            (image_path for image_path in image_paths if not image_path.is_file()),
            None,
        )
        if missing_path is not None:
            return None
        image_bytes = sum(image_path.stat().st_size for image_path in image_paths)
        max_tokens = self._max_tokens_for_region_composite(input_rows)
        started = time.perf_counter()
        http_status = None
        try:
            payload = self._region_composite_request_payload(
                input_rows,
                image_paths,
                max_tokens,
            )
            http_status, response_payload = self._send_completion_request(payload)
            markdown = _extract_openai_message_content(response_payload)
            region_texts = _extract_ocr2_region_composite_markdown(
                markdown,
                input_rows,
            )
        except urllib.error.HTTPError as exc:
            self._write_trace(
                input_rows[0],
                status="failed",
                started=started,
                http_status=exc.code,
                image_bytes=image_bytes,
                markdown_chars=0,
                error=exc,
                input_rows=input_rows,
                max_tokens=max_tokens,
                request_kind="region-composite",
            )
            return None
        except (OSError, ValueError, urllib.error.URLError) as exc:
            self._write_trace(
                input_rows[0],
                status="failed",
                started=started,
                http_status=http_status,
                image_bytes=image_bytes,
                markdown_chars=0,
                error=exc,
                input_rows=input_rows,
                max_tokens=max_tokens,
                request_kind="region-composite",
            )
            return None
        self._write_trace(
            input_rows[0],
            status="succeeded",
            started=started,
            http_status=http_status,
            image_bytes=image_bytes,
            markdown_chars=sum(len(text) for text in region_texts),
            error=None,
            input_rows=input_rows,
            max_tokens=max_tokens,
            request_kind="region-composite",
        )
        return [
            {
                "status": "succeeded",
                "text": text,
                "textMimeType": "text/markdown",
                "confidence": None,
                "errorMessage": None,
            }
            for text in region_texts
        ]

    def recognize(self, input_row: Mapping[str, Any]) -> Mapping[str, Any]:
        image_path = Path(str(input_row["imagePath"]))
        if not image_path.is_file():
            return failed_pdf_ocr_shard_result(
                input_row,
                f"DeepSeek-OCR-2 shard image does not exist: {image_path}",
            )
        image_bytes = image_path.stat().st_size
        started = time.perf_counter()
        http_status = None
        max_tokens = self._max_tokens_for_row(input_row)
        try:
            payload = self._request_payload(input_row, image_path, max_tokens)
            http_status, response_payload = self._send_completion_request(payload)
            markdown = _extract_openai_message_content(response_payload)
        except urllib.error.HTTPError as exc:
            self._write_trace(
                input_row,
                status="failed",
                started=started,
                http_status=exc.code,
                image_bytes=image_bytes,
                markdown_chars=0,
                error=exc,
                max_tokens=max_tokens,
            )
            return failed_pdf_ocr_shard_result(
                input_row, f"DeepSeek-OCR-2 OCR failed: {exc}"
            )
        except (OSError, ValueError, urllib.error.URLError) as exc:
            self._write_trace(
                input_row,
                status="failed",
                started=started,
                http_status=http_status,
                image_bytes=image_bytes,
                markdown_chars=0,
                error=exc,
                max_tokens=max_tokens,
            )
            return failed_pdf_ocr_shard_result(
                input_row, f"DeepSeek-OCR-2 OCR failed: {exc}"
            )
        if not markdown.strip():
            self._write_trace(
                input_row,
                status="failed",
                started=started,
                http_status=http_status,
                image_bytes=image_bytes,
                markdown_chars=0,
                error=ValueError("empty OCR text"),
                max_tokens=max_tokens,
            )
            return failed_pdf_ocr_shard_result(
                input_row,
                "DeepSeek-OCR-2 OCR returned empty text",
            )
        self._write_trace(
            input_row,
            status="succeeded",
            started=started,
            http_status=http_status,
            image_bytes=image_bytes,
            markdown_chars=len(markdown),
            error=None,
            max_tokens=max_tokens,
        )
        return {
            "status": "succeeded",
            "text": markdown,
            "textMimeType": "text/markdown",
            "confidence": None,
            "errorMessage": None,
        }

    def _send_completion_request(
        self, payload: Mapping[str, Any]
    ) -> tuple[int | None, Any]:
        request_data = json.dumps(payload).encode("utf-8")
        for attempt in range(_DEEPSEEK_OCR2_MAX_TRANSIENT_RETRIES + 1):
            request = urllib.request.Request(
                self._completion_url,
                data=request_data,
                headers=self._headers(),
                method="POST",
            )
            try:
                with urllib.request.urlopen(
                    request,
                    timeout=self._timeout_seconds,
                ) as response:
                    http_status = _response_http_status(response)
                    response_payload = json.loads(response.read().decode("utf-8"))
                return http_status, response_payload
            except urllib.error.HTTPError as exc:
                if not _should_retry_ocr2_http_error(exc, attempt):
                    raise
                time.sleep(_ocr2_retry_delay_seconds(attempt, exc))
            except (OSError, urllib.error.URLError):
                if attempt >= _DEEPSEEK_OCR2_MAX_TRANSIENT_RETRIES:
                    raise
                time.sleep(_ocr2_retry_delay_seconds(attempt, None))
        raise RuntimeError("unreachable OCR2 retry loop")

    def _request_payload(
        self,
        input_row: Mapping[str, Any],
        image_path: Path,
        max_tokens: int,
    ) -> dict[str, Any]:
        image_bytes = image_path.read_bytes()
        image_mime_type = str(input_row.get("imageMimeType") or "image/png")
        image_data_url = f"data:{image_mime_type};base64,{base64.b64encode(image_bytes).decode('ascii')}"
        return {
            "model": self._model,
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {"type": "text", "text": self._prompt},
                        {"type": "image_url", "image_url": {"url": image_data_url}},
                    ],
                }
            ],
            "max_tokens": max_tokens,
            "temperature": 0,
        }

    def _window_request_payload(
        self,
        input_rows: Sequence[Mapping[str, Any]],
        image_paths: Sequence[Path],
    ) -> dict[str, Any]:
        content: list[dict[str, Any]] = [
            {"type": "text", "text": self._window_prompt(input_rows)}
        ]
        for ordinal, (input_row, image_path) in enumerate(
            zip(input_rows, image_paths, strict=True),
            start=1,
        ):
            image_bytes = image_path.read_bytes()
            image_mime_type = str(input_row.get("imageMimeType") or "image/png")
            image_data_url = f"data:{image_mime_type};base64,{base64.b64encode(image_bytes).decode('ascii')}"
            marker = _ocr2_page_marker(input_row)
            content.append(
                {
                    "type": "text",
                    "text": f"Image {ordinal} must produce section marker {marker}.",
                }
            )
            content.append({"type": "image_url", "image_url": {"url": image_data_url}})
        return {
            "model": self._model,
            "messages": [{"role": "user", "content": content}],
            "max_tokens": self._max_tokens,
            "temperature": 0,
        }

    def _region_composite_request_payload(
        self,
        input_rows: Sequence[Mapping[str, Any]],
        image_paths: Sequence[Path],
        max_tokens: int,
    ) -> dict[str, Any]:
        content: list[dict[str, Any]] = [
            {"type": "text", "text": self._region_composite_prompt(input_rows)}
        ]
        for ordinal, (input_row, image_path) in enumerate(
            zip(input_rows, image_paths, strict=True),
            start=1,
        ):
            image_bytes = image_path.read_bytes()
            image_mime_type = str(input_row.get("imageMimeType") or "image/png")
            image_data_url = f"data:{image_mime_type};base64,{base64.b64encode(image_bytes).decode('ascii')}"
            marker = _ocr2_region_marker(input_row)
            content.append(
                {
                    "type": "text",
                    "text": f"Region image {ordinal} must produce section marker {marker}.",
                }
            )
            content.append({"type": "image_url", "image_url": {"url": image_data_url}})
        return {
            "model": self._model,
            "messages": [{"role": "user", "content": content}],
            "max_tokens": max_tokens,
            "temperature": 0,
        }

    def _max_tokens_for_row(self, input_row: Mapping[str, Any]) -> int:
        if str(input_row.get("shardType") or "") == "region":
            return min(self._max_tokens, self._region_max_tokens)
        return self._max_tokens

    def _max_tokens_for_region_composite(
        self,
        input_rows: Sequence[Mapping[str, Any]],
    ) -> int:
        return min(self._max_tokens, self._region_max_tokens * len(input_rows))

    def _window_prompt(self, input_rows: Sequence[Mapping[str, Any]]) -> str:
        markers = "\n".join(_ocr2_page_marker(row) for row in input_rows)
        return (
            f"{self._prompt}\n\n"
            "You will receive multiple page images from the same PDF. Convert "
            "each image to Markdown independently and preserve all visible text, "
            "tables, formulas, headings, and reading order. Return exactly one "
            "section for each image, in the same order as the images. Start each "
            "section with the exact marker assigned to that image. Do not merge "
            "pages and do not omit empty-looking pages; if a page has no text, "
            "write the marker followed by a blank line.\n\n"
            "Required section markers:\n"
            f"{markers}"
        )

    def _region_composite_prompt(
        self,
        input_rows: Sequence[Mapping[str, Any]],
    ) -> str:
        markers = "\n".join(_ocr2_region_marker(row) for row in input_rows)
        return (
            f"{self._prompt}\n\n"
            "You will receive multiple cropped recovery-region images from the "
            "same PDF page and parent page OCR shard. Convert each region to "
            "Markdown independently and preserve all visible text, tables, "
            "formulas, headings, and reading order. Return exactly one section "
            "for each region, in the same order as the images. Start each "
            "section with the exact marker assigned to that region. Do not "
            "merge regions and do not invent missing context.\n\n"
            "Required section markers:\n"
            f"{markers}"
        )

    def _headers(self) -> dict[str, str]:
        headers = {"Content-Type": "application/json", **self._extra_headers}
        if self._api_key and self._api_key != DEEPSEEK_OCR2_DEFAULT_API_KEY:
            headers["Authorization"] = f"Bearer {self._api_key}"
        return headers

    def _write_trace(
        self,
        input_row: Mapping[str, Any],
        *,
        status: str,
        started: float,
        http_status: int | None,
        image_bytes: int,
        markdown_chars: int,
        error: BaseException | None,
        page_count: int = 1,
        input_rows: Sequence[Mapping[str, Any]] | None = None,
        max_tokens: int | None = None,
        request_kind: str | None = None,
    ) -> None:
        if self._trace_path is None:
            return
        rows = list(input_rows or [input_row])
        ended_unix_ms = int(time.time() * 1000)
        latency_ms = round((time.perf_counter() - started) * 1000.0, 3)
        started_unix_ms = max(0, ended_unix_ms - round(latency_ms))
        record = {
            "schema": "xiuxian_wendao.deepseek_ocr2_request_trace.v1",
            "timestampUnixMs": ended_unix_ms,
            "startedUnixMs": started_unix_ms,
            "endedUnixMs": ended_unix_ms,
            "status": status,
            "httpStatus": http_status,
            "latencyMs": latency_ms,
            "model": self._model,
            "endpoint": self._completion_url,
            "pageIndex": input_row.get("pageIndex"),
            "shardElementId": input_row.get("shardElementId"),
            "shardType": input_row.get("shardType"),
            "regionIndex": input_row.get("regionIndex"),
            "parentShardElementId": input_row.get("parentShardElementId"),
            "readingOrderKey": input_row.get("readingOrderKey"),
            "ocrProfile": input_row.get("ocrProfile"),
            "requestKind": (
                request_kind
                if request_kind is not None
                else _ocr2_trace_request_kind(input_row, page_count)
            ),
            "shardCount": len(rows),
            "shardTypeCounts": _ocr2_trace_shard_type_counts(rows),
            "pageCount": page_count,
            "imageBytes": image_bytes,
            "sourcePixelArea": _ocr2_trace_source_pixel_area(rows),
            "renderDpi": input_row.get("renderDpi"),
            "rasterWidthPx": input_row.get("rasterWidthPx"),
            "rasterHeightPx": input_row.get("rasterHeightPx"),
            "sourcePagePixelLeft": input_row.get("sourcePagePixelLeft"),
            "sourcePagePixelTop": input_row.get("sourcePagePixelTop"),
            "sourcePagePixelRight": input_row.get("sourcePagePixelRight"),
            "sourcePagePixelBottom": input_row.get("sourcePagePixelBottom"),
            "markdownChars": markdown_chars,
            "maxTokens": max_tokens if max_tokens is not None else self._max_tokens,
            "errorType": type(error).__name__ if error is not None else None,
            "errorMessage": _short_error_message(error),
        }
        try:
            with _DEEPSEEK_OCR2_TRACE_LOCK:
                self._trace_path.parent.mkdir(parents=True, exist_ok=True)
                with self._trace_path.open("a", encoding="utf-8") as handle:
                    handle.write(json.dumps(record, sort_keys=True))
                    handle.write("\n")
        except OSError:
            return


def _ocr2_trace_request_kind(input_row: Mapping[str, Any], page_count: int) -> str:
    if page_count > 1:
        return "page-window-canary"
    shard_type = str(input_row.get("shardType") or "")
    if shard_type == "region":
        return "region"
    return "page"


def _ocr2_trace_shard_type_counts(
    input_rows: Sequence[Mapping[str, Any]],
) -> dict[str, int]:
    counts: dict[str, int] = {}
    for row in input_rows:
        shard_type = str(row.get("shardType") or "unknown")
        counts[shard_type] = counts.get(shard_type, 0) + 1
    return counts


def _ocr2_trace_source_pixel_area(input_rows: Sequence[Mapping[str, Any]]) -> int:
    return sum(_row_source_pixel_area(row) for row in input_rows)


def _row_source_pixel_area(input_row: Mapping[str, Any]) -> int:
    try:
        left = int(input_row.get("sourcePagePixelLeft") or 0)
        top = int(input_row.get("sourcePagePixelTop") or 0)
        right = int(input_row.get("sourcePagePixelRight") or 0)
        bottom = int(input_row.get("sourcePagePixelBottom") or 0)
    except (TypeError, ValueError):
        return 0
    return max(0, right - left) * max(0, bottom - top)


def _resolve_openrouter_api_key() -> str:
    api_key = _env_value(
        DEEPSEEK_OCR2_API_KEY_ENV,
        _env_value(
            DEEPSEEK_OCR2_OPENROUTER_API_KEY_ENV,
            _env_value(
                DEEPSEEK_OCR2_OPENROUTER_PUBLIC_API_KEY_ENV,
                _env_value(
                    DEEPSEEK_OCR2_OPENROUTE_COMPAT_API_KEY_ENV,
                    DEEPSEEK_OCR2_DEFAULT_API_KEY,
                ),
            ),
        ),
    )
    if not api_key or api_key == DEEPSEEK_OCR2_DEFAULT_API_KEY:
        raise ValueError(
            "OpenRouter OCR provider requires WENDAO_OPENROUTER_API_KEY, "
            "OPENROUTER_API_KEY, OPENROUTE_API_KEY, or "
            "WENDAO_DEEPSEEK_OCR2_API_KEY"
        )
    return api_key


def _ocr2_page_windows(
    input_rows: Sequence[Mapping[str, Any]],
    window_size: int,
) -> list[list[Mapping[str, Any]]]:
    windows: list[list[Mapping[str, Any]]] = []
    current: list[Mapping[str, Any]] = []
    for row in input_rows:
        if not _is_page_window_candidate(row):
            if current:
                windows.append(current)
                current = []
            windows.append([row])
            continue
        if not current:
            current.append(row)
            continue
        if len(current) >= window_size or not _can_extend_ocr2_page_window(
            current[-1],
            row,
        ):
            windows.append(current)
            current = [row]
            continue
        current.append(row)
    if current:
        windows.append(current)
    return windows


def _ocr2_region_composite_tasks(
    input_rows: Sequence[Mapping[str, Any]],
    composite_size: int,
) -> list[list[Mapping[str, Any]]]:
    tasks: list[list[Mapping[str, Any]]] = []
    current: list[Mapping[str, Any]] = []
    for row in input_rows:
        if not _is_region_composite_candidate(row):
            if current:
                tasks.append(current)
                current = []
            tasks.append([row])
            continue
        if not current:
            current.append(row)
            continue
        if len(current) >= composite_size or not _can_extend_ocr2_region_composite(
            current[-1],
            row,
        ):
            tasks.append(current)
            current = [row]
            continue
        current.append(row)
    if current:
        tasks.append(current)
    return tasks


def _is_page_window_candidate(row: Mapping[str, Any]) -> bool:
    return str(row.get("shardType") or "") == "page"


def _is_region_composite_candidate(row: Mapping[str, Any]) -> bool:
    return str(row.get("shardType") or "") == "region"


def _can_extend_ocr2_page_window(
    previous: Mapping[str, Any],
    current: Mapping[str, Any],
) -> bool:
    if str(previous.get("sourcePath")) != str(current.get("sourcePath")):
        return False
    if str(previous.get("sourceContentHash")) != str(current.get("sourceContentHash")):
        return False
    previous_page = previous.get("pageIndex")
    current_page = current.get("pageIndex")
    return (
        isinstance(previous_page, int)
        and isinstance(current_page, int)
        and current_page == previous_page + 1
    )


def _can_extend_ocr2_region_composite(
    previous: Mapping[str, Any],
    current: Mapping[str, Any],
) -> bool:
    return (
        str(previous.get("sourcePath")) == str(current.get("sourcePath"))
        and str(previous.get("sourceContentHash"))
        == str(current.get("sourceContentHash"))
        and previous.get("pageIndex") == current.get("pageIndex")
        and str(previous.get("parentShardElementId"))
        == str(current.get("parentShardElementId"))
    )


def _flatten_page_window_results(
    window_results: Sequence[Sequence[Mapping[str, Any]]],
) -> list[Mapping[str, Any]]:
    return [result for window in window_results for result in window]


def _ocr2_page_marker(input_row: Mapping[str, Any]) -> str:
    return (
        f"{_DEEPSEEK_OCR2_PAGE_MARKER_PREFIX}"
        f"{input_row.get('pageIndex')}"
        f"{_DEEPSEEK_OCR2_PAGE_MARKER_SUFFIX}"
    )


def _ocr2_region_marker(input_row: Mapping[str, Any]) -> str:
    return (
        f"{_DEEPSEEK_OCR2_REGION_MARKER_PREFIX}"
        f"{input_row.get('pageIndex')}:"
        f"{input_row.get('regionIndex')}:"
        f"{input_row.get('shardElementId')}"
        f"{_DEEPSEEK_OCR2_REGION_MARKER_SUFFIX}"
    )


def _extract_ocr2_page_window_markdown(
    markdown: str,
    input_rows: Sequence[Mapping[str, Any]],
) -> list[str]:
    return _extract_ocr2_marked_sections(
        markdown,
        [_ocr2_page_marker(row) for row in input_rows],
        "page-window",
    )


def _extract_ocr2_region_composite_markdown(
    markdown: str,
    input_rows: Sequence[Mapping[str, Any]],
) -> list[str]:
    return _extract_ocr2_marked_sections(
        markdown,
        [_ocr2_region_marker(row) for row in input_rows],
        "region-composite",
    )


def _extract_ocr2_marked_sections(
    markdown: str,
    markers: Sequence[str],
    label: str,
) -> list[str]:
    if not markdown.strip():
        raise ValueError(f"OCR2 {label} response returned empty text")
    sections = []
    cursor = 0
    for index, marker in enumerate(markers):
        marker_position = markdown.find(marker, cursor)
        if marker_position < 0:
            raise ValueError(f"OCR2 {label} response is missing a section marker")
        content_start = marker_position + len(marker)
        if index + 1 < len(markers):
            next_position = markdown.find(markers[index + 1], content_start)
            if next_position < 0:
                raise ValueError(
                    f"OCR2 {label} response is missing the next section marker"
                )
            content_end = next_position
        else:
            content_end = len(markdown)
        text = markdown[content_start:content_end].strip()
        if not text:
            raise ValueError(f"OCR2 {label} response returned an empty section")
        sections.append(text)
        cursor = content_end
    return sections


def _openrouter_headers() -> dict[str, str]:
    headers = {}
    referer = _env_value(DEEPSEEK_OCR2_OPENROUTER_HTTP_REFERER_ENV, "")
    title = _env_value(DEEPSEEK_OCR2_OPENROUTER_TITLE_ENV, "")
    if referer:
        headers["HTTP-Referer"] = referer
    if title:
        headers["X-OpenRouter-Title"] = title
    return headers


def _chat_completion_url(base_url: str) -> str:
    normalized = base_url.rstrip("/")
    if normalized.endswith("/chat/completions"):
        return normalized
    return f"{normalized}/chat/completions"


def _response_http_status(response: object) -> int | None:
    status = getattr(response, "status", None)
    if isinstance(status, int):
        return status
    code = getattr(response, "code", None)
    if isinstance(code, int):
        return code
    return None


def _should_retry_ocr2_http_error(error: urllib.error.HTTPError, attempt: int) -> bool:
    return (
        error.code in _DEEPSEEK_OCR2_TRANSIENT_HTTP_STATUS
        and attempt < _DEEPSEEK_OCR2_MAX_TRANSIENT_RETRIES
    )


def _is_transient_ocr2_failure(result: Mapping[str, Any]) -> bool:
    if result.get("status") != "failed":
        return False
    error_message = str(result.get("errorMessage") or "")
    return any(
        f"HTTP Error {status}" in error_message
        for status in _DEEPSEEK_OCR2_TRANSIENT_HTTP_STATUS
    )


def _ocr2_retry_delay_seconds(
    attempt: int,
    error: urllib.error.HTTPError | None,
) -> float:
    retry_after = _ocr2_retry_after_seconds(error)
    if retry_after is not None:
        return retry_after
    base_seconds = (
        _DEEPSEEK_OCR2_RATE_LIMIT_RETRY_BASE_SECONDS
        if error is not None and error.code == 429
        else _DEEPSEEK_OCR2_RETRY_BASE_SECONDS
    )
    return min(
        base_seconds * (2**attempt),
        _DEEPSEEK_OCR2_MAX_RETRY_DELAY_SECONDS,
    )


def _ocr2_retry_after_seconds(error: urllib.error.HTTPError | None) -> float | None:
    if error is None:
        return None
    headers = getattr(error, "headers", None)
    if headers is None:
        return None
    value = headers.get("Retry-After")
    if value is None:
        return None
    try:
        seconds = float(value)
    except ValueError:
        return None
    return min(max(seconds, 0.0), _DEEPSEEK_OCR2_MAX_RETRY_DELAY_SECONDS)


def _short_error_message(error: BaseException | None) -> str | None:
    if error is None:
        return None
    message = str(error)
    if len(message) <= 240:
        return message
    return f"{message[:237]}..."


def _extract_openai_message_content(payload: Mapping[str, Any]) -> str:
    choices = payload.get("choices")
    if not isinstance(choices, list) or not choices:
        raise ValueError("OpenAI-compatible response does not contain choices")
    first_choice = choices[0]
    if not isinstance(first_choice, Mapping):
        raise ValueError("OpenAI-compatible response choice is not an object")
    message = first_choice.get("message")
    if not isinstance(message, Mapping):
        raise ValueError("OpenAI-compatible response choice does not contain message")
    content = message.get("content")
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        parts = []
        for part in content:
            if isinstance(part, Mapping) and isinstance(part.get("text"), str):
                parts.append(part["text"])
        if parts:
            return "".join(parts)
    raise ValueError("OpenAI-compatible response message content is not text")


def _positive_int_env(key: str, default: int) -> int:
    try:
        value = int(os.environ.get(key, ""))
    except ValueError:
        return default
    return value if value > 0 else default


def _positive_int_value(value: int | str | None) -> int | None:
    if value is None:
        return None
    try:
        parsed = int(value)
    except (TypeError, ValueError):
        return None
    return parsed if parsed > 0 else None


def _positive_float_env(key: str, default: float) -> float:
    try:
        value = float(os.environ.get(key, ""))
    except ValueError:
        return default
    return value if value > 0 else default


def _env_value(key: str, default: str) -> str:
    value = os.environ.get(key)
    if value is None or not value.strip():
        return default
    return value


def _optional_path_env(key: str) -> Path | None:
    value = os.environ.get(key)
    if value is None or not value.strip():
        return None
    return Path(value)
