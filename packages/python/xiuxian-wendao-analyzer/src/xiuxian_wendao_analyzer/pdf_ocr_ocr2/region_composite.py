"""OCR2 region-composite recognition path."""

from __future__ import annotations

import time
import urllib.error
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from typing import TYPE_CHECKING, Any, Protocol

from ..pdf_ocr_contracts import DEEPSEEK_OCR2_REGION_TABLE_JSON_SCAFFOLD_MODE
from .http import extract_openai_message_content
from .markers import extract_ocr2_region_composite_markdown
from .payloads import region_composite_request_payload
from .results import succeeded_markdown_results
from .routing import flatten_page_window_results
from .scaffold_composite import recognize_region_composite_scaffold

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence


class RegionCompositeClient(Protocol):
    _model: str
    _prompt: str
    _request_concurrency: int
    _scaffold_mode: str

    def recognize(self, input_row: Mapping[str, Any]) -> Mapping[str, Any]: ...

    def _max_tokens_for_region_composite(
        self,
        input_rows: Sequence[Mapping[str, Any]],
    ) -> int: ...

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


def recognize_region_composite_tasks(
    client: RegionCompositeClient,
    tasks: Sequence[Sequence[Mapping[str, Any]]],
) -> list[Mapping[str, Any]]:
    if client._request_concurrency <= 1 or len(tasks) <= 1:
        return flatten_page_window_results(
            [recognize_region_composite(client, task) for task in tasks]
        )
    worker_count = min(client._request_concurrency, len(tasks))
    with ThreadPoolExecutor(max_workers=worker_count) as executor:
        futures = [
            executor.submit(recognize_region_composite, client, task) for task in tasks
        ]
        return flatten_page_window_results([future.result() for future in futures])


def recognize_region_composite(
    client: RegionCompositeClient,
    input_rows: Sequence[Mapping[str, Any]],
) -> list[Mapping[str, Any]]:
    rows = list(input_rows)
    if len(rows) <= 1:
        return [client.recognize(row) for row in rows]
    if client._scaffold_mode == DEEPSEEK_OCR2_REGION_TABLE_JSON_SCAFFOLD_MODE:
        return recognize_region_composite_scaffold(client, rows)
    composite_result = try_recognize_region_composite(client, rows)
    if composite_result is not None:
        return composite_result
    return [client.recognize(row) for row in rows]


def try_recognize_region_composite(
    client: RegionCompositeClient,
    input_rows: Sequence[Mapping[str, Any]],
) -> list[Mapping[str, Any]] | None:
    image_paths = [Path(str(input_row["imagePath"])) for input_row in input_rows]
    missing_path = next(
        (image_path for image_path in image_paths if not image_path.is_file()), None
    )
    if missing_path is not None:
        return None
    image_bytes = sum(image_path.stat().st_size for image_path in image_paths)
    max_tokens = client._max_tokens_for_region_composite(input_rows)
    started = time.perf_counter()
    http_status = None
    try:
        payload = region_composite_request_payload(
            model=client._model,
            prompt=client._prompt,
            input_rows=input_rows,
            image_paths=image_paths,
            max_tokens=max_tokens,
        )
        http_status, response_payload = client._send_completion_request(payload)
        markdown = extract_openai_message_content(response_payload)
        region_texts = extract_ocr2_region_composite_markdown(markdown, input_rows)
    except urllib.error.HTTPError as exc:
        client._write_trace(
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
        client._write_trace(
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
    client._write_trace(
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
    return succeeded_markdown_results(region_texts)
