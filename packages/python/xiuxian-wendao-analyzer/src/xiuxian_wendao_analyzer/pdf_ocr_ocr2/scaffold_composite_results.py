"""Result builders for OCR2 scaffold composite recognition."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from .results import failed_results, succeeded_markdown_results

if TYPE_CHECKING:
    import urllib.error
    from collections.abc import Mapping, Sequence
    from pathlib import Path

    from .scaffold_composite import ScaffoldCompositeClient


def _missing_image_failure(
    client: ScaffoldCompositeClient,
    input_rows: Sequence[Mapping[str, Any]],
    missing_path: Path,
    image_bytes: int,
    max_tokens: int,
    started: float,
) -> list[Mapping[str, Any]]:
    error = ValueError(f"Hosted VLM/OCR shard image does not exist: {missing_path}")
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
    if str(error).startswith("missing Hosted VLM/OCR region scaffold sidecar"):
        return 0
    return len(input_rows)


def _scaffold_json_chars(error: OSError | ValueError | urllib.error.URLError) -> int:
    response_text = getattr(error, "response_text", "")
    if isinstance(response_text, str):
        return len(response_text)
    return 0
