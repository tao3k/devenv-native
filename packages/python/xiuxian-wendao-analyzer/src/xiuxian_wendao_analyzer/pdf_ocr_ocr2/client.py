"""DeepSeek-OCR-2 OpenAI-compatible client."""

from __future__ import annotations

import os
import threading
from contextlib import suppress
from typing import TYPE_CHECKING, Any

from ..pdf_ocr_contracts import DEEPSEEK_OCR2_DEFAULT_API_KEY
from ..pdf_ocr_results import failed_pdf_ocr_shard_result
from . import http as ocr2_http
from .config import Ocr2ClientConfig, ocr2_client_config_from_env
from .http import is_transient_ocr2_failure, send_completion_request
from .page_window import (
    recognize_indexed_window_tasks,
    recognize_page_tasks_once,
    recognize_window,
    try_recognize_page_window,
)
from .region_composite import (
    recognize_region_composite,
    recognize_region_composite_tasks,
)
from .routing import (
    flatten_page_window_results,
    ocr2_page_windows,
    ocr2_region_composite_tasks,
)
from .single import recognize_single
from .trace import write_trace_record

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence

_DEEPSEEK_OCR2_PAGE_WINDOW_PROBE_LOCK = threading.Lock()
_DEEPSEEK_OCR2_PAGE_WINDOW_COMPATIBILITY: dict[tuple[str, int], bool] = {}
_DEEPSEEK_OCR2_GROUP_RETRY_DELAYS_SECONDS = (8.0, 16.0)


def recognize_deepseek_ocr2_many(
    input_rows: Sequence[Mapping[str, Any]],
    *,
    request_concurrency: int | str | None = None,
) -> list[Mapping[str, Any]]:
    try:
        config = ocr2_client_config_from_env(request_concurrency=request_concurrency)
    except ValueError as exc:
        return [
            failed_pdf_ocr_shard_result(input_row, f"DeepSeek-OCR-2 OCR failed: {exc}")
            for input_row in input_rows
        ]
    return _DeepSeekOcr2OpenAiClient(config).recognize_many(input_rows)


class _DeepSeekOcr2OpenAiClient:
    def __init__(self, config: Ocr2ClientConfig) -> None:
        from .http import chat_completion_url

        self._completion_url = chat_completion_url(config.base_url)
        self._model = config.model
        self._api_key = config.api_key
        self._prompt = config.prompt
        self._max_tokens = config.max_tokens
        self._region_max_tokens = config.region_max_tokens
        self._region_composite_size = config.region_composite_size
        self._region_atlas_mode = config.region_atlas_mode
        self._timeout_seconds = config.timeout_seconds
        self._request_concurrency = config.request_concurrency
        self._page_window_size = config.page_window_size
        self._scaffold_mode = config.scaffold_mode
        self._trace_path = config.trace_path
        self._extra_headers = dict(config.extra_headers or {})
        self._disabled_region_canaries: set[str] = set()

    def recognize_many(
        self,
        input_rows: Sequence[Mapping[str, Any]],
    ) -> list[Mapping[str, Any]]:
        rows = list(input_rows)
        if self._region_composite_size > 1:
            region_tasks = ocr2_region_composite_tasks(
                rows, self._region_composite_size
            )
            if any(len(task) > 1 for task in region_tasks):
                return self._retry_transient_failed_results(
                    rows,
                    recognize_region_composite_tasks(self, region_tasks),
                )
        if self._page_window_size <= 1:
            return self._retry_transient_failed_results(
                rows,
                recognize_page_tasks_once(self, rows),
            )
        windows = ocr2_page_windows(rows, self._page_window_size)
        if len(windows) == len(rows) and all(len(window) == 1 for window in windows):
            return self._retry_transient_failed_results(
                rows,
                recognize_page_tasks_once(self, rows),
            )
        return self._recognize_page_windows(rows, windows)

    def recognize_window(
        self,
        input_rows: Sequence[Mapping[str, Any]],
    ) -> list[Mapping[str, Any]]:
        return recognize_window(self, input_rows)

    def recognize_region_composite(
        self,
        input_rows: Sequence[Mapping[str, Any]],
    ) -> list[Mapping[str, Any]]:
        return recognize_region_composite(self, input_rows)

    def recognize(self, input_row: Mapping[str, Any]) -> Mapping[str, Any]:
        return recognize_single(self, input_row)

    def _claim_region_canary(self, request_kind: str) -> bool:
        if request_kind in self._disabled_region_canaries:
            return False
        state_paths = self._region_canary_state_paths(request_kind)
        if state_paths is None:
            return True
        disabled_path, probing_path, supported_path = state_paths
        if disabled_path.exists():
            self._disabled_region_canaries.add(request_kind)
            return False
        if supported_path.exists():
            return True
        probing_path.parent.mkdir(parents=True, exist_ok=True)
        try:
            descriptor = os.open(
                probing_path,
                os.O_CREAT | os.O_EXCL | os.O_WRONLY,
            )
        except FileExistsError:
            return False
        with os.fdopen(descriptor, "w", encoding="utf-8") as handle:
            handle.write(str(os.getpid()))
        return True

    def _mark_region_canary_success(self, request_kind: str) -> None:
        state_paths = self._region_canary_state_paths(request_kind)
        if state_paths is None:
            return
        _, probing_path, supported_path = state_paths
        supported_path.write_text(str(os.getpid()), encoding="utf-8")
        with suppress(FileNotFoundError):
            probing_path.unlink()

    def _disable_region_canary(self, request_kind: str) -> None:
        self._disabled_region_canaries.add(request_kind)
        state_paths = self._region_canary_state_paths(request_kind)
        if state_paths is None:
            return
        disabled_path, probing_path, _ = state_paths
        disabled_path.write_text(str(os.getpid()), encoding="utf-8")
        with suppress(FileNotFoundError):
            probing_path.unlink()

    def _region_canary_state_paths(
        self, request_kind: str
    ) -> tuple[Any, Any, Any] | None:
        if self._trace_path is None:
            return None
        safe_kind = "".join(
            character if character.isalnum() or character in {"-", "_"} else "-"
            for character in request_kind
        )
        state_dir = self._trace_path.parent / "ocr2-region-canaries"
        return (
            state_dir / f"{safe_kind}.disabled",
            state_dir / f"{safe_kind}.probing",
            state_dir / f"{safe_kind}.supported",
        )

    def _recognize_page_windows(
        self,
        rows: Sequence[Mapping[str, Any]],
        windows: Sequence[Sequence[Mapping[str, Any]]],
    ) -> list[Mapping[str, Any]]:
        probe_index = next(
            (index for index, window in enumerate(windows) if len(window) > 1),
            None,
        )
        if probe_index is None:
            return self._retry_transient_failed_results(
                rows,
                recognize_page_tasks_once(self, rows),
            )
        probe_result = self._probe_page_window_compatibility(windows[probe_index])
        if probe_result is None:
            return self._retry_transient_failed_results(
                rows,
                recognize_page_tasks_once(self, rows),
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
        for index, result in recognize_indexed_window_tasks(self, pending_windows):
            window_results[index] = result
        return self._retry_transient_failed_results(
            rows,
            flatten_page_window_results(
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
                return try_recognize_page_window(self, input_rows)
            probe_result = try_recognize_page_window(self, input_rows)
            _DEEPSEEK_OCR2_PAGE_WINDOW_COMPATIBILITY[compatibility_key] = (
                probe_result is not None
            )
            return probe_result

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
                if is_transient_ocr2_failure(result)
            ]
            if not retry_indexes:
                break
            ocr2_http.sleep(delay_seconds)
            retry_rows = [rows[index] for index in retry_indexes]
            retry_results = recognize_page_tasks_once(self, retry_rows)
            for index, result in zip(retry_indexes, retry_results, strict=True):
                output[index] = result
        return output

    def _send_completion_request(
        self, payload: Mapping[str, Any]
    ) -> tuple[int | None, Any]:
        return send_completion_request(
            completion_url=self._completion_url,
            headers=self._headers(),
            timeout_seconds=self._timeout_seconds,
            payload=payload,
        )

    def _max_tokens_for_row(self, input_row: Mapping[str, Any]) -> int:
        if str(input_row.get("shardType") or "") == "region":
            return min(self._max_tokens, self._region_max_tokens)
        return self._max_tokens

    def _max_tokens_for_region_composite(
        self,
        input_rows: Sequence[Mapping[str, Any]],
    ) -> int:
        return min(self._max_tokens, self._region_max_tokens * len(input_rows))

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
        scaffold_applied_count: int = 0,
        scaffold_validation_failure_count: int = 0,
        scaffold_json_chars: int = 0,
        canonical_markdown_chars: int = 0,
    ) -> None:
        write_trace_record(
            trace_path=self._trace_path,
            input_row=input_row,
            model=self._model,
            completion_url=self._completion_url,
            scaffold_mode=self._scaffold_mode,
            default_max_tokens=self._max_tokens,
            status=status,
            started=started,
            http_status=http_status,
            image_bytes=image_bytes,
            markdown_chars=markdown_chars,
            error=error,
            page_count=page_count,
            input_rows=input_rows,
            max_tokens=max_tokens,
            request_kind=request_kind,
            scaffold_applied_count=scaffold_applied_count,
            scaffold_validation_failure_count=scaffold_validation_failure_count,
            scaffold_json_chars=scaffold_json_chars,
            canonical_markdown_chars=canonical_markdown_chars,
        )
