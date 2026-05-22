"""Hosted VLM/OCR single-shard recognition path."""

from __future__ import annotations

import time
import urllib.error
from dataclasses import dataclass
from pathlib import Path
from queue import Empty, Queue
from threading import Thread
from typing import TYPE_CHECKING, Any, Protocol

from ..pdf_ocr_contracts import HOSTED_VLM_OCR_REGION_TABLE_JSON_SCAFFOLD_MODE
from ..pdf_ocr_results import failed_pdf_ocr_shard_result
from .http import extract_openai_message_content
from .image_payload import hosted_vlm_image_payload
from .payloads import image_bytes_data_url, request_payload
from .results import succeeded_markdown_result
from .single_scaffold import recognize_region_scaffold
from .trace import row_source_pixel_area

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence


class SingleShardClient(Protocol):
    _model: str
    _prompt: str
    _region_prompt_mode: str
    _image_optimization_mode: str
    _scaffold_mode: str
    _speculative_retry_delay_seconds: float
    _speculative_retry_min_source_pixels: int
    _speculative_retry_min_image_bytes: int

    def _max_tokens_for_row(self, input_row: Mapping[str, Any]) -> int: ...

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
        http_attempt_count: int = 1,
        scaffold_applied_count: int = 0,
        scaffold_validation_failure_count: int = 0,
        scaffold_json_chars: int = 0,
        canonical_markdown_chars: int = 0,
        hedge_winner: str | None = None,
        hedge_delay_seconds: float | None = None,
        hedge_primary_latency_ms: float | None = None,
        hedge_secondary_latency_ms: float | None = None,
    ) -> None: ...


@dataclass(frozen=True)
class SingleMarkdownRequestResult:
    http_status: int | None
    markdown: str
    hedged: bool
    hedge_winner: str | None = None
    hedge_delay_seconds: float | None = None
    hedge_primary_latency_ms: float | None = None
    hedge_secondary_latency_ms: float | None = None


def recognize_single(
    client: SingleShardClient,
    input_row: Mapping[str, Any],
) -> Mapping[str, Any]:
    image_path = Path(str(input_row["imagePath"]))
    if not image_path.is_file():
        return failed_pdf_ocr_shard_result(
            input_row,
            f"Hosted VLM/OCR shard image does not exist: {image_path}",
        )
    started = time.perf_counter()
    http_status = None
    image_bytes = image_path.stat().st_size
    max_tokens = client._max_tokens_for_row(input_row)
    if (
        client._scaffold_mode == HOSTED_VLM_OCR_REGION_TABLE_JSON_SCAFFOLD_MODE
        and str(input_row.get("shardType") or "") == "region"
    ):
        return recognize_region_scaffold(
            client, input_row, image_path, image_bytes, started, max_tokens
        )
    try:
        image_payload = hosted_vlm_image_payload(
            input_row,
            image_path,
            image_optimization_mode=client._image_optimization_mode,
        )
        payload = request_payload(
            model=client._model,
            prompt=client._prompt,
            input_row=input_row,
            image_path=image_path,
            max_tokens=max_tokens,
            image_data_url_value=image_bytes_data_url(
                image_payload.image_bytes,
                image_payload.image_mime_type,
            ),
            region_prompt_mode=client._region_prompt_mode,
        )
        image_bytes = len(image_payload.image_bytes)
        request_result = send_single_markdown_request(
            client, payload, input_row, image_bytes=image_bytes
        )
        http_status = request_result.http_status
        markdown = request_result.markdown
    except urllib.error.HTTPError as exc:
        client._write_trace(
            input_row,
            status="failed",
            started=started,
            http_status=exc.code,
            image_bytes=image_bytes,
            markdown_chars=0,
            error=exc,
            max_tokens=max_tokens,
        )
        return failed_pdf_ocr_shard_result(input_row, f"Hosted VLM/OCR failed: {exc}")
    except (OSError, ValueError, urllib.error.URLError) as exc:
        client._write_trace(
            input_row,
            status="failed",
            started=started,
            http_status=http_status,
            image_bytes=image_bytes,
            markdown_chars=0,
            error=exc,
            max_tokens=max_tokens,
        )
        return failed_pdf_ocr_shard_result(input_row, f"Hosted VLM/OCR failed: {exc}")
    if not markdown.strip():
        error = ValueError("empty OCR text")
        client._write_trace(
            input_row,
            status="failed",
            started=started,
            http_status=http_status,
            image_bytes=image_bytes,
            markdown_chars=0,
            error=error,
            max_tokens=max_tokens,
        )
        return failed_pdf_ocr_shard_result(
            input_row, "Hosted VLM/OCR returned empty text"
        )
    client._write_trace(
        input_row,
        status="succeeded",
        started=started,
        http_status=http_status,
        image_bytes=image_bytes,
        markdown_chars=len(markdown),
        error=None,
        max_tokens=max_tokens,
        request_kind="region-hedged" if request_result.hedged else None,
        http_attempt_count=2 if request_result.hedged else 1,
        hedge_winner=request_result.hedge_winner,
        hedge_delay_seconds=request_result.hedge_delay_seconds,
        hedge_primary_latency_ms=request_result.hedge_primary_latency_ms,
        hedge_secondary_latency_ms=request_result.hedge_secondary_latency_ms,
    )
    return succeeded_markdown_result(markdown)


def send_single_markdown_request(
    client: SingleShardClient,
    payload: Mapping[str, Any],
    input_row: Mapping[str, Any],
    *,
    image_bytes: int,
) -> SingleMarkdownRequestResult:
    if (
        str(input_row.get("shardType") or "") != "region"
        or client._speculative_retry_delay_seconds <= 0
        or not should_speculatively_retry_region(client, input_row, image_bytes)
    ):
        http_status, response_payload = client._send_completion_request(payload)
        markdown = extract_openai_message_content(response_payload)
        return SingleMarkdownRequestResult(http_status, markdown, False)

    return send_hedged_single_markdown_request(
        client,
        payload,
        delay_seconds=client._speculative_retry_delay_seconds,
    )


def should_speculatively_retry_region(
    client: SingleShardClient,
    input_row: Mapping[str, Any],
    image_bytes: int,
) -> bool:
    min_source_pixels = max(0, client._speculative_retry_min_source_pixels)
    if min_source_pixels > 0 and row_source_pixel_area(input_row) < min_source_pixels:
        return False
    min_image_bytes = max(0, client._speculative_retry_min_image_bytes)
    return min_image_bytes <= 0 or image_bytes >= min_image_bytes


def send_hedged_single_markdown_request(
    client: SingleShardClient,
    payload: Mapping[str, Any],
    *,
    delay_seconds: float,
) -> SingleMarkdownRequestResult:
    outcomes: Queue[tuple[str, int | None, str | None, Exception | None, float]] = (
        Queue()
    )

    def send(attempt: str) -> None:
        started = time.perf_counter()
        try:
            http_status, response_payload = client._send_completion_request(payload)
            markdown = require_non_empty_markdown(
                extract_openai_message_content(response_payload)
            )
            latency_ms = round((time.perf_counter() - started) * 1000.0, 3)
            outcomes.put((attempt, http_status, markdown, None, latency_ms))
        except Exception as exc:
            latency_ms = round((time.perf_counter() - started) * 1000.0, 3)
            outcomes.put((attempt, None, None, exc, latency_ms))

    Thread(target=send, args=("primary",), daemon=True).start()
    try:
        _attempt, http_status, markdown, error, _latency_ms = outcomes.get(
            timeout=delay_seconds
        )
    except Empty:
        Thread(target=send, args=("hedge",), daemon=True).start()
        return first_successful_hedged_outcome(outcomes, delay_seconds=delay_seconds)
    if error is not None:
        raise error
    return SingleMarkdownRequestResult(http_status, str(markdown), False)


def first_successful_hedged_outcome(
    outcomes: Queue[tuple[str, int | None, str | None, Exception | None, float]],
    *,
    delay_seconds: float,
) -> SingleMarkdownRequestResult:
    first_error: Exception | None = None
    attempt_latencies: dict[str, float] = {}
    for _ in range(2):
        attempt, http_status, markdown, error, latency_ms = outcomes.get()
        attempt_latencies[attempt] = latency_ms
        if error is None:
            return SingleMarkdownRequestResult(
                http_status,
                str(markdown),
                True,
                hedge_winner=attempt,
                hedge_delay_seconds=delay_seconds,
                hedge_primary_latency_ms=attempt_latencies.get("primary"),
                hedge_secondary_latency_ms=attempt_latencies.get("hedge"),
            )
        if first_error is None:
            first_error = error
    if first_error is not None:
        raise first_error
    raise ValueError("Hosted VLM/OCR hedged request produced no result")


def require_non_empty_markdown(markdown: str) -> str:
    if not markdown.strip():
        raise ValueError("empty OCR text")
    return markdown
