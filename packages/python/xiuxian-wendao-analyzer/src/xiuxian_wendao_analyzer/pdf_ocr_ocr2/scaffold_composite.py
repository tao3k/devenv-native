"""Hosted VLM/OCR scaffolded region-composite recognition path."""

from __future__ import annotations

import time
import urllib.error
from pathlib import Path
from typing import TYPE_CHECKING, Any, Protocol

from .http import extract_openai_message_content
from .payloads import region_scaffold_request_payload
from .scaffold import extract_ocr2_scaffold_markdown, load_ocr2_region_scaffolds
from .scaffold_composite_results import (
    _http_failure,
    _missing_image_failure,
    _success,
    _validation_failure,
)

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence


class ScaffoldResponseValidationError(ValueError):
    def __init__(self, message: str, response_text: str) -> None:
        super().__init__(message)
        self.response_text = response_text


class ScaffoldCompositeClient(Protocol):
    _model: str
    _prompt: str

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
        http_attempt_count: int = 1,
        scaffold_applied_count: int = 0,
        scaffold_validation_failure_count: int = 0,
        scaffold_json_chars: int = 0,
        canonical_markdown_chars: int = 0,
    ) -> None: ...


def recognize_region_composite_scaffold(
    client: ScaffoldCompositeClient,
    input_rows: Sequence[Mapping[str, Any]],
) -> list[Mapping[str, Any]]:
    image_paths = [Path(str(input_row["imagePath"])) for input_row in input_rows]
    missing_path = next(
        (image_path for image_path in image_paths if not image_path.is_file()), None
    )
    image_bytes = sum(
        image_path.stat().st_size for image_path in image_paths if image_path.is_file()
    )
    max_tokens = client._max_tokens_for_region_composite(input_rows)
    started = time.perf_counter()
    http_status = None
    if missing_path is not None:
        return _missing_image_failure(
            client,
            input_rows,
            missing_path,
            image_bytes,
            max_tokens,
            started,
        )
    try:
        http_status, response_text, region_texts = _request_scaffold(
            client,
            input_rows,
            image_paths,
            max_tokens,
        )
    except urllib.error.HTTPError as exc:
        return _http_failure(client, input_rows, exc, image_bytes, max_tokens, started)
    except (OSError, ValueError, urllib.error.URLError) as exc:
        return _validation_failure(
            client,
            input_rows,
            exc,
            http_status,
            image_bytes,
            max_tokens,
            started,
        )
    return _success(
        client,
        input_rows,
        response_text,
        region_texts,
        http_status,
        image_bytes,
        max_tokens,
        started,
    )


def _request_scaffold(
    client: ScaffoldCompositeClient,
    input_rows: Sequence[Mapping[str, Any]],
    image_paths: Sequence[Path],
    max_tokens: int,
) -> tuple[int | None, str, list[str]]:
    scaffolds = load_ocr2_region_scaffolds(input_rows)
    payload = region_scaffold_request_payload(
        model=client._model,
        prompt=client._prompt,
        input_rows=input_rows,
        image_paths=image_paths,
        scaffolds=scaffolds,
        max_tokens=max_tokens,
        composite=True,
    )
    http_status, response_payload = client._send_completion_request(payload)
    response_text = extract_openai_message_content(response_payload)
    try:
        region_texts = extract_ocr2_scaffold_markdown(response_text, input_rows)
    except ValueError as exc:
        raise ScaffoldResponseValidationError(str(exc), response_text) from exc
    return (
        http_status,
        response_text,
        region_texts,
    )
