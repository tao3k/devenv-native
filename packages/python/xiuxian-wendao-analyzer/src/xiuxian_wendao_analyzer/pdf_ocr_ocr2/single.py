"""OCR2 single-shard recognition path."""

from __future__ import annotations

import time
import urllib.error
from pathlib import Path
from typing import TYPE_CHECKING, Any, Protocol

from ..pdf_ocr_contracts import DEEPSEEK_OCR2_REGION_TABLE_JSON_SCAFFOLD_MODE
from ..pdf_ocr_results import failed_pdf_ocr_shard_result
from .http import extract_openai_message_content
from .payloads import region_scaffold_request_payload, request_payload
from .results import succeeded_markdown_result
from .scaffold import extract_ocr2_scaffold_markdown, load_ocr2_region_scaffolds

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence


class SingleShardClient(Protocol):
    _model: str
    _prompt: str
    _scaffold_mode: str

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
        scaffold_applied_count: int = 0,
        scaffold_validation_failure_count: int = 0,
        scaffold_json_chars: int = 0,
        canonical_markdown_chars: int = 0,
    ) -> None: ...


def recognize_single(
    client: SingleShardClient,
    input_row: Mapping[str, Any],
) -> Mapping[str, Any]:
    image_path = Path(str(input_row["imagePath"]))
    if not image_path.is_file():
        return failed_pdf_ocr_shard_result(
            input_row,
            f"DeepSeek-OCR-2 shard image does not exist: {image_path}",
        )
    image_bytes = image_path.stat().st_size
    started = time.perf_counter()
    http_status = None
    max_tokens = client._max_tokens_for_row(input_row)
    if (
        client._scaffold_mode == DEEPSEEK_OCR2_REGION_TABLE_JSON_SCAFFOLD_MODE
        and str(input_row.get("shardType") or "") == "region"
    ):
        return recognize_region_scaffold(
            client, input_row, image_path, image_bytes, started, max_tokens
        )
    try:
        payload = request_payload(
            model=client._model,
            prompt=client._prompt,
            input_row=input_row,
            image_path=image_path,
            max_tokens=max_tokens,
        )
        http_status, response_payload = client._send_completion_request(payload)
        markdown = extract_openai_message_content(response_payload)
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
        return failed_pdf_ocr_shard_result(
            input_row, f"DeepSeek-OCR-2 OCR failed: {exc}"
        )
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
        return failed_pdf_ocr_shard_result(
            input_row, f"DeepSeek-OCR-2 OCR failed: {exc}"
        )
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
            input_row, "DeepSeek-OCR-2 OCR returned empty text"
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
    )
    return succeeded_markdown_result(markdown)


def recognize_region_scaffold(
    client: SingleShardClient,
    input_row: Mapping[str, Any],
    image_path: Path,
    image_bytes: int,
    started: float,
    max_tokens: int,
) -> Mapping[str, Any]:
    http_status = None
    response_text = ""
    try:
        scaffolds = load_ocr2_region_scaffolds([input_row])
        payload = region_scaffold_request_payload(
            model=client._model,
            prompt=client._prompt,
            input_rows=[input_row],
            image_paths=[image_path],
            scaffolds=scaffolds,
            max_tokens=max_tokens,
            composite=False,
        )
        http_status, response_payload = client._send_completion_request(payload)
        response_text = extract_openai_message_content(response_payload)
        markdown = extract_ocr2_scaffold_markdown(response_text, [input_row])[0]
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
            request_kind="region-scaffold",
            scaffold_applied_count=1,
        )
        return failed_pdf_ocr_shard_result(
            input_row, f"DeepSeek-OCR-2 OCR failed: {exc}"
        )
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
            request_kind="region-scaffold",
            scaffold_applied_count=(
                0 if str(exc).startswith("missing OCR2 region scaffold sidecar") else 1
            ),
            scaffold_validation_failure_count=1,
            scaffold_json_chars=len(response_text),
        )
        return failed_pdf_ocr_shard_result(
            input_row, f"DeepSeek-OCR-2 OCR failed: {exc}"
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
        request_kind="region-scaffold",
        scaffold_applied_count=1,
        scaffold_json_chars=len(response_text),
        canonical_markdown_chars=len(markdown),
    )
    return succeeded_markdown_result(markdown)
