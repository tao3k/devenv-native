"""Hosted VLM/OCR page-window recognition path."""

from __future__ import annotations

import time
import urllib.error
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from typing import TYPE_CHECKING, Any, Protocol

from .http import extract_openai_message_content
from .markers import extract_ocr2_page_window_markdown
from .payloads import window_request_payload
from .results import succeeded_markdown_results

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence


class PageWindowClient(Protocol):
    _max_tokens: int
    _model: str
    _prompt: str
    _request_concurrency: int

    def recognize(self, input_row: Mapping[str, Any]) -> Mapping[str, Any]: ...

    def _send_completion_request(
        self, payload: Mapping[str, Any]
    ) -> tuple[int | None, Any]: ...

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
    ) -> None: ...


def recognize_page_tasks_once(
    client: PageWindowClient,
    rows: Sequence[Mapping[str, Any]],
) -> list[Mapping[str, Any]]:
    if client._request_concurrency <= 1 or len(rows) <= 1:
        return [client.recognize(input_row) for input_row in rows]
    worker_count = min(client._request_concurrency, len(rows))
    with ThreadPoolExecutor(max_workers=worker_count) as executor:
        futures = [executor.submit(client.recognize, input_row) for input_row in rows]
        return [future.result() for future in futures]


def recognize_indexed_window_tasks(
    client: PageWindowClient,
    windows: Sequence[tuple[int, Sequence[Mapping[str, Any]]]],
) -> list[tuple[int, list[Mapping[str, Any]]]]:
    if client._request_concurrency <= 1 or len(windows) <= 1:
        return [(index, recognize_window(client, window)) for index, window in windows]
    worker_count = min(client._request_concurrency, len(windows))
    with ThreadPoolExecutor(max_workers=worker_count) as executor:
        futures = [
            executor.submit(recognize_window, client, window)
            for _index, window in windows
        ]
        return [
            (index, future.result())
            for (index, _window), future in zip(windows, futures, strict=True)
        ]


def recognize_window(
    client: PageWindowClient,
    input_rows: Sequence[Mapping[str, Any]],
) -> list[Mapping[str, Any]]:
    rows = list(input_rows)
    if len(rows) <= 1:
        return [client.recognize(row) for row in rows]
    batch_result = try_recognize_page_window(client, rows)
    if batch_result is not None:
        return batch_result
    return [client.recognize(row) for row in rows]


def try_recognize_page_window(
    client: PageWindowClient,
    input_rows: Sequence[Mapping[str, Any]],
) -> list[Mapping[str, Any]] | None:
    image_paths = [Path(str(input_row["imagePath"])) for input_row in input_rows]
    missing_path = next(
        (image_path for image_path in image_paths if not image_path.is_file()), None
    )
    if missing_path is not None:
        return None
    image_bytes = sum(image_path.stat().st_size for image_path in image_paths)
    started = time.perf_counter()
    http_status = None
    try:
        payload = window_request_payload(
            model=client._model,
            prompt=client._prompt,
            input_rows=input_rows,
            image_paths=image_paths,
            max_tokens=client._max_tokens,
        )
        http_status, response_payload = client._send_completion_request(payload)
        markdown = extract_openai_message_content(response_payload)
        page_texts = extract_ocr2_page_window_markdown(markdown, input_rows)
    except urllib.error.HTTPError as exc:
        client._write_trace(
            input_rows[0],
            status="failed",
            started=started,
            http_status=exc.code,
            image_bytes=image_bytes,
            markdown_chars=0,
            error=exc,
            page_count=len(input_rows),
            input_rows=input_rows,
            max_tokens=client._max_tokens,
        )
        return None
    except (OSError, ValueError, urllib.error.URLError) as exc:
        client._write_trace(
            input_rows[0],
            status="failed",
            started=started,
            http_status=http_status,
            image_bytes=image_bytes,
            markdown_chars=0,
            error=exc,
            page_count=len(input_rows),
            input_rows=input_rows,
            max_tokens=client._max_tokens,
        )
        return None
    client._write_trace(
        input_rows[0],
        status="succeeded",
        started=started,
        http_status=http_status,
        image_bytes=image_bytes,
        markdown_chars=sum(len(text) for text in page_texts),
        error=None,
        page_count=len(input_rows),
        input_rows=input_rows,
        max_tokens=client._max_tokens,
    )
    return succeeded_markdown_results(page_texts)
