"""OCR2 scaffolded region-composite recognition path."""

from __future__ import annotations

import time
import urllib.error
from pathlib import Path
from typing import TYPE_CHECKING, Any, Protocol

from .http import extract_openai_message_content
from .payloads import region_scaffold_request_payload
from .results import failed_results, succeeded_markdown_results
from .scaffold import extract_ocr2_scaffold_markdown, load_ocr2_region_scaffolds

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


def _missing_image_failure(
    client: ScaffoldCompositeClient,
    input_rows: Sequence[Mapping[str, Any]],
    missing_path: Path,
    image_bytes: int,
    max_tokens: int,
    started: float,
) -> list[Mapping[str, Any]]:
    error = ValueError(f"DeepSeek-OCR-2 shard image does not exist: {missing_path}")
    client._write_trace(
        input_rows[0],
        status="failed",
        started=started,
        http_status=None,
        image_bytes=image_bytes,
        markdown_chars=0,
        error=error,
        input_rows=input_rows,
        max_tokens=max_tokens,
        request_kind="region-composite-scaffold",
        scaffold_applied_count=0,
        scaffold_validation_failure_count=len(input_rows),
    )
    return failed_results(input_rows, error)


def _http_failure(
    client: ScaffoldCompositeClient,
    input_rows: Sequence[Mapping[str, Any]],
    error: urllib.error.HTTPError,
    image_bytes: int,
    max_tokens: int,
    started: float,
) -> list[Mapping[str, Any]]:
    client._write_trace(
        input_rows[0],
        status="failed",
        started=started,
        http_status=error.code,
        image_bytes=image_bytes,
        markdown_chars=0,
        error=error,
        input_rows=input_rows,
        max_tokens=max_tokens,
        request_kind="region-composite-scaffold",
        scaffold_applied_count=len(input_rows),
    )
    return failed_results(input_rows, error)


def _validation_failure(
    client: ScaffoldCompositeClient,
    input_rows: Sequence[Mapping[str, Any]],
    error: OSError | ValueError | urllib.error.URLError,
    http_status: int | None,
    image_bytes: int,
    max_tokens: int,
    started: float,
) -> list[Mapping[str, Any]]:
    client._write_trace(
        input_rows[0],
        status="failed",
        started=started,
        http_status=http_status,
        image_bytes=image_bytes,
        markdown_chars=0,
        error=error,
        input_rows=input_rows,
        max_tokens=max_tokens,
        request_kind="region-composite-scaffold",
        scaffold_applied_count=_scaffold_applied_count(input_rows, error),
        scaffold_validation_failure_count=len(input_rows),
        scaffold_json_chars=_scaffold_json_chars(error),
    )
    return failed_results(input_rows, error)


def _success(
    client: ScaffoldCompositeClient,
    input_rows: Sequence[Mapping[str, Any]],
    response_text: str,
    region_texts: Sequence[str],
    http_status: int | None,
    image_bytes: int,
    max_tokens: int,
    started: float,
) -> list[Mapping[str, Any]]:
    markdown_chars = sum(len(text) for text in region_texts)
    client._write_trace(
        input_rows[0],
        status="succeeded",
        started=started,
        http_status=http_status,
        image_bytes=image_bytes,
        markdown_chars=markdown_chars,
        error=None,
        input_rows=input_rows,
        max_tokens=max_tokens,
        request_kind="region-composite-scaffold",
        scaffold_applied_count=len(input_rows),
        scaffold_json_chars=len(response_text),
        canonical_markdown_chars=markdown_chars,
    )
    return succeeded_markdown_results(region_texts)


def _scaffold_applied_count(
    input_rows: Sequence[Mapping[str, Any]],
    error: OSError | ValueError | urllib.error.URLError,
) -> int:
    if str(error).startswith("missing OCR2 region scaffold sidecar"):
        return 0
    return len(input_rows)


def _scaffold_json_chars(error: OSError | ValueError | urllib.error.URLError) -> int:
    response_text = getattr(error, "response_text", "")
    if isinstance(response_text, str):
        return len(response_text)
    return 0
