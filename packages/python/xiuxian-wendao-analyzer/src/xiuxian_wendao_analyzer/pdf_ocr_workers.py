"""PDF OCR shard worker implementations."""

from __future__ import annotations

import base64
import json
import os
import threading
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
    DEEPSEEK_OCR2_DEFAULT_PROMPT,
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
    DEEPSEEK_OCR2_PROMPT_ENV,
    DEEPSEEK_OCR2_PROVIDER_ENV,
    DEEPSEEK_OCR2_REQUEST_CONCURRENCY_ENV,
    DEEPSEEK_OCR2_TIMEOUT_SECONDS_ENV,
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
        ocr_profile = _ocr_profile(input_rows[0])
        if ocr_profile == PDF_OCR_DEEPSEEK_OCR2_DIRECT_VLM_PROFILE:
            return [
                (index, result)
                for index, result in zip(
                    indexes,
                    _recognize_deepseek_ocr2_many(input_rows),
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
) -> list[Mapping[str, Any]]:
    try:
        client = _DeepSeekOcr2OpenAiClient.from_env()
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
        timeout_seconds: float,
        request_concurrency: int,
        extra_headers: Mapping[str, str] | None = None,
    ) -> None:
        self._completion_url = _chat_completion_url(base_url)
        self._model = model
        self._api_key = api_key
        self._prompt = prompt
        self._max_tokens = max_tokens
        self._timeout_seconds = timeout_seconds
        self._request_concurrency = request_concurrency
        self._extra_headers = dict(extra_headers or {})

    @classmethod
    def from_env(cls) -> _DeepSeekOcr2OpenAiClient:
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
                timeout_seconds=_positive_float_env(
                    DEEPSEEK_OCR2_TIMEOUT_SECONDS_ENV,
                    DEEPSEEK_OCR2_DEFAULT_TIMEOUT_SECONDS,
                ),
                request_concurrency=_positive_int_env(
                    DEEPSEEK_OCR2_REQUEST_CONCURRENCY_ENV,
                    DEEPSEEK_OCR2_DEFAULT_REQUEST_CONCURRENCY,
                ),
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
            timeout_seconds=_positive_float_env(
                DEEPSEEK_OCR2_TIMEOUT_SECONDS_ENV,
                DEEPSEEK_OCR2_DEFAULT_TIMEOUT_SECONDS,
            ),
            request_concurrency=_positive_int_env(
                DEEPSEEK_OCR2_REQUEST_CONCURRENCY_ENV,
                DEEPSEEK_OCR2_DEFAULT_REQUEST_CONCURRENCY,
            ),
        )

    def recognize_many(
        self,
        input_rows: Sequence[Mapping[str, Any]],
    ) -> list[Mapping[str, Any]]:
        rows = list(input_rows)
        if self._request_concurrency <= 1 or len(rows) <= 1:
            return [self.recognize(input_row) for input_row in rows]
        worker_count = min(self._request_concurrency, len(rows))
        with ThreadPoolExecutor(max_workers=worker_count) as executor:
            futures = [executor.submit(self.recognize, input_row) for input_row in rows]
            return [future.result() for future in futures]

    def recognize(self, input_row: Mapping[str, Any]) -> Mapping[str, Any]:
        image_path = Path(str(input_row["imagePath"]))
        if not image_path.is_file():
            return failed_pdf_ocr_shard_result(
                input_row,
                f"DeepSeek-OCR-2 shard image does not exist: {image_path}",
            )
        try:
            payload = self._request_payload(input_row, image_path)
            request = urllib.request.Request(
                self._completion_url,
                data=json.dumps(payload).encode("utf-8"),
                headers=self._headers(),
                method="POST",
            )
            with urllib.request.urlopen(
                request,
                timeout=self._timeout_seconds,
            ) as response:
                response_payload = json.loads(response.read().decode("utf-8"))
            markdown = _extract_openai_message_content(response_payload)
        except (
            OSError,
            ValueError,
            urllib.error.URLError,
            urllib.error.HTTPError,
        ) as exc:
            return failed_pdf_ocr_shard_result(
                input_row, f"DeepSeek-OCR-2 OCR failed: {exc}"
            )
        if not markdown.strip():
            return failed_pdf_ocr_shard_result(
                input_row,
                "DeepSeek-OCR-2 OCR returned empty text",
            )
        return {
            "status": "succeeded",
            "text": markdown,
            "textMimeType": "text/markdown",
            "confidence": None,
            "errorMessage": None,
        }

    def _request_payload(
        self,
        input_row: Mapping[str, Any],
        image_path: Path,
    ) -> dict[str, Any]:
        image_bytes = image_path.read_bytes()
        image_mime_type = str(input_row.get("imageMimeType") or "image/png")
        image_data_url = (
            f"data:{image_mime_type};base64,"
            f"{base64.b64encode(image_bytes).decode('ascii')}"
        )
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
            "max_tokens": self._max_tokens,
            "temperature": 0,
        }

    def _headers(self) -> dict[str, str]:
        headers = {"Content-Type": "application/json", **self._extra_headers}
        if self._api_key and self._api_key != DEEPSEEK_OCR2_DEFAULT_API_KEY:
            headers["Authorization"] = f"Bearer {self._api_key}"
        return headers


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
