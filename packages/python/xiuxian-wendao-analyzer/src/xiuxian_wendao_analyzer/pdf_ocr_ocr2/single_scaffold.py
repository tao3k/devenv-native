"""Hosted VLM/OCR scaffolded single-region recognition."""

from __future__ import annotations

import urllib.error
from typing import TYPE_CHECKING, Any

from ..pdf_ocr_results import failed_pdf_ocr_shard_result
from .http import extract_openai_message_content
from .payloads import region_scaffold_request_payload
from .results import succeeded_markdown_result
from .scaffold import extract_ocr2_scaffold_markdown, load_ocr2_region_scaffolds

if TYPE_CHECKING:
    from collections.abc import Mapping
    from pathlib import Path

    from .single import SingleShardClient


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
            request_kind="region-scaffold",
            scaffold_applied_count=(
                0
                if str(exc).startswith("missing Hosted VLM/OCR region scaffold sidecar")
                else 1
            ),
            scaffold_validation_failure_count=1,
            scaffold_json_chars=len(response_text),
        )
        return failed_pdf_ocr_shard_result(input_row, f"Hosted VLM/OCR failed: {exc}")
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
