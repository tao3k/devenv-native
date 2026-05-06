"""OCR2 result-row constructors."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from ..pdf_ocr_results import failed_pdf_ocr_shard_result

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence


def succeeded_markdown_result(markdown: str) -> Mapping[str, Any]:
    return {
        "status": "succeeded",
        "text": markdown,
        "textMimeType": "text/markdown",
        "confidence": None,
        "errorMessage": None,
    }


def succeeded_markdown_results(texts: Sequence[str]) -> list[Mapping[str, Any]]:
    return [succeeded_markdown_result(text) for text in texts]


def failed_results(
    input_rows: Sequence[Mapping[str, Any]],
    error: BaseException,
) -> list[Mapping[str, Any]]:
    return [
        failed_pdf_ocr_shard_result(input_row, f"DeepSeek-OCR-2 OCR failed: {error}")
        for input_row in input_rows
    ]
