"""Hosted VLM/OCR region-composite recognition path."""

from __future__ import annotations

import time
import urllib.error
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from queue import Empty, Queue
from threading import Thread
from typing import TYPE_CHECKING, Any, Protocol

from ..pdf_ocr_contracts import (
    HOSTED_VLM_OCR_DEFAULT_IMAGE_OPTIMIZATION,
    HOSTED_VLM_OCR_REGION_ATLAS_SAME_PAGE_JSON_MODE,
    HOSTED_VLM_OCR_REGION_COMPOSITE_FIXED_MODE,
    HOSTED_VLM_OCR_REGION_TABLE_JSON_SCAFFOLD_MODE,
)
from .http import extract_openai_message_content
from .image_payload import hosted_vlm_image_payload
from .markers import extract_ocr2_region_composite_markdown
from .payload_helpers import image_bytes_data_url
from .payloads import region_composite_request_payload
from .region_atlas import try_recognize_region_atlas
from .results import succeeded_markdown_results
from .routing import flatten_page_window_results
from .scaffold_composite import recognize_region_composite_scaffold

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence


class RegionCompositeClient(Protocol):
    _model: str
    _prompt: str
    _request_concurrency: int
    _region_atlas_mode: str
    _region_composite_mode: str
    _scaffold_mode: str
    _image_optimization_mode: str
    _speculative_retry_delay_seconds: float

    def recognize(self, input_row: Mapping[str, Any]) -> Mapping[str, Any]: ...

    def _max_tokens_for_region_composite(
        self,
        input_rows: Sequence[Mapping[str, Any]],
    ) -> int: ...

    def _send_completion_request(
        self, payload: Mapping[str, Any]
    ) -> tuple[int | None, Any]: ...

    def _claim_region_canary(self, request_kind: str) -> bool: ...

    def _mark_region_canary_success(self, request_kind: str) -> None: ...

    def _disable_region_canary(self, request_kind: str) -> None: ...

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
        http_attempt_count: int = 1,
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
    if client._scaffold_mode == HOSTED_VLM_OCR_REGION_TABLE_JSON_SCAFFOLD_MODE:
        return recognize_region_composite_scaffold(client, rows)
    if client._region_atlas_mode == HOSTED_VLM_OCR_REGION_ATLAS_SAME_PAGE_JSON_MODE:
        if client._claim_region_canary("region-atlas"):
            atlas_result = try_recognize_region_atlas(client, rows)
            if atlas_result is not None:
                client._mark_region_canary_success("region-atlas")
                return atlas_result
            client._disable_region_canary("region-atlas")
        return [client.recognize(row) for row in rows]
    if client._region_composite_mode == HOSTED_VLM_OCR_REGION_COMPOSITE_FIXED_MODE:
        composite_result = try_recognize_region_composite(client, rows)
        if composite_result is not None:
            return composite_result
        return [client.recognize(row) for row in rows]
    if client._claim_region_canary("region-composite"):
        composite_result = try_recognize_region_composite(client, rows)
        if composite_result is not None:
            client._mark_region_canary_success("region-composite")
            return composite_result
        client._disable_region_canary("region-composite")
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
    image_data_urls, image_bytes = region_composite_image_data_urls(
        client,
        input_rows,
        image_paths,
    )
    max_tokens = client._max_tokens_for_region_composite(input_rows)
    started = time.perf_counter()
    http_status = None
    hedged = False
    try:
        payload = region_composite_request_payload(
            model=client._model,
            prompt=client._prompt,
            input_rows=input_rows,
            image_paths=image_paths,
            max_tokens=max_tokens,
            image_data_url_values=image_data_urls,
        )
        http_status, region_texts, hedged = send_region_composite_texts_request(
            client,
            payload,
            input_rows,
        )
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
            http_attempt_count=2 if hedged else 1,
        )
        return None
    except Exception as exc:
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
            http_attempt_count=2 if hedged else 1,
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
        request_kind="region-composite-hedged" if hedged else "region-composite",
        http_attempt_count=2 if hedged else 1,
    )
    return succeeded_markdown_results(region_texts)


def region_composite_image_data_urls(
    client: RegionCompositeClient,
    input_rows: Sequence[Mapping[str, Any]],
    image_paths: Sequence[Path],
) -> tuple[list[str], int]:
    image_optimization_mode = getattr(
        client,
        "_image_optimization_mode",
        HOSTED_VLM_OCR_DEFAULT_IMAGE_OPTIMIZATION,
    )
    data_urls = []
    image_bytes_total = 0
    for input_row, image_path in zip(input_rows, image_paths, strict=True):
        payload = hosted_vlm_image_payload(
            input_row,
            image_path,
            image_optimization_mode=image_optimization_mode,
        )
        data_urls.append(
            image_bytes_data_url(payload.image_bytes, payload.image_mime_type)
        )
        image_bytes_total += len(payload.image_bytes)
    return data_urls, image_bytes_total


def send_region_composite_texts_request(
    client: RegionCompositeClient,
    payload: Mapping[str, Any],
    input_rows: Sequence[Mapping[str, Any]],
) -> tuple[int | None, list[str], bool]:
    if client._speculative_retry_delay_seconds <= 0:
        http_status, response_payload = client._send_completion_request(payload)
        markdown = extract_openai_message_content(response_payload)
        return (
            http_status,
            extract_ocr2_region_composite_markdown(markdown, input_rows),
            False,
        )

    return send_hedged_region_composite_texts_request(
        client,
        payload,
        input_rows,
        delay_seconds=client._speculative_retry_delay_seconds,
    )


def send_hedged_region_composite_texts_request(
    client: RegionCompositeClient,
    payload: Mapping[str, Any],
    input_rows: Sequence[Mapping[str, Any]],
    *,
    delay_seconds: float,
) -> tuple[int | None, list[str], bool]:
    outcomes: Queue[tuple[int | None, list[str] | None, Exception | None]] = Queue()

    def send() -> None:
        try:
            http_status, response_payload = client._send_completion_request(payload)
            markdown = extract_openai_message_content(response_payload)
            outcomes.put(
                (
                    http_status,
                    extract_ocr2_region_composite_markdown(markdown, input_rows),
                    None,
                )
            )
        except Exception as exc:
            outcomes.put((None, None, exc))

    Thread(target=send, daemon=True).start()
    try:
        http_status, region_texts, error = outcomes.get(timeout=delay_seconds)
    except Empty:
        Thread(target=send, daemon=True).start()
        return first_successful_hedged_region_composite_outcome(outcomes)
    if error is not None:
        raise error
    return http_status, list(region_texts or []), False


def first_successful_hedged_region_composite_outcome(
    outcomes: Queue[tuple[int | None, list[str] | None, Exception | None]],
) -> tuple[int | None, list[str], bool]:
    first_error: Exception | None = None
    for _ in range(2):
        http_status, region_texts, error = outcomes.get()
        if error is None:
            return http_status, list(region_texts or []), True
        if first_error is None:
            first_error = error
    if first_error is not None:
        raise first_error
    raise ValueError(
        "Hosted VLM/OCR hedged region-composite request produced no result"
    )
