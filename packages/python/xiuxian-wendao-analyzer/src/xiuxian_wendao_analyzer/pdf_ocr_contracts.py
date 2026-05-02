"""PDF OCR Arrow contracts and worker protocol."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any, Protocol

import pyarrow as pa

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence


PDF_OCR_SHARD_INPUT_SCHEMA_VERSION = "xiuxian_wendao.pdf_ocr_shard_input.v1"

PDF_OCR_SHARD_RESULT_SCHEMA_VERSION = "xiuxian_wendao.pdf_ocr_shard_result.v1"

PDF_OCR_DEFAULT_PROFILE = "docling-compatible-page-ocr-v1"

PDF_OCR_FAST_TEXT_PROFILE = "docling-fast-text-ocr"

PDF_OCR_WORKERS_ENV = "WENDAO_PDF_OCR_WORKERS"

PDF_OCR_MAX_WORKERS_ENV = "WENDAO_PDF_OCR_MAX_WORKERS"

PDF_OCR_PAGE_BREAK_SENTINEL = "<!-- xiuxian-wendao-pdf-ocr-page-break -->"

PDF_OCR_SHARD_INPUT_SCHEMA = pa.schema(
    [
        pa.field("contractVersion", pa.string(), nullable=False),
        pa.field("sourcePath", pa.string(), nullable=False),
        pa.field("sourceContentHash", pa.string(), nullable=False),
        pa.field("pageIndex", pa.int32(), nullable=False),
        pa.field("imagePath", pa.string(), nullable=False),
        pa.field("imageMimeType", pa.string(), nullable=False),
        pa.field("rasterSha256", pa.string(), nullable=False),
        pa.field("renderProfile", pa.string(), nullable=False),
        pa.field("ocrProfile", pa.string(), nullable=False),
        pa.field("ocrEngine", pa.string(), nullable=False),
        pa.field("preferredLanguages", pa.string(), nullable=False),
        pa.field("minConfidence", pa.float64(), nullable=False),
        pa.field("preserveLayout", pa.bool_(), nullable=False),
        pa.field("rasterWidthPx", pa.int32(), nullable=False),
        pa.field("rasterHeightPx", pa.int32(), nullable=False),
        pa.field("renderDpi", pa.int32(), nullable=False),
        pa.field("rotationDegrees", pa.int32(), nullable=False),
        pa.field("cropLeft", pa.float64(), nullable=False),
        pa.field("cropBottom", pa.float64(), nullable=False),
        pa.field("cropRight", pa.float64(), nullable=False),
        pa.field("cropTop", pa.float64(), nullable=False),
        pa.field("pointToPixelScaleX", pa.float64(), nullable=False),
        pa.field("pointToPixelScaleY", pa.float64(), nullable=False),
        pa.field("shardElementId", pa.string(), nullable=False),
        pa.field("shardType", pa.string(), nullable=False),
        pa.field("regionIndex", pa.int32(), nullable=False),
        pa.field("parentShardElementId", pa.string(), nullable=False),
        pa.field("readingOrderKey", pa.string(), nullable=False),
        pa.field("sourcePagePixelLeft", pa.int32(), nullable=False),
        pa.field("sourcePagePixelTop", pa.int32(), nullable=False),
        pa.field("sourcePagePixelRight", pa.int32(), nullable=False),
        pa.field("sourcePagePixelBottom", pa.int32(), nullable=False),
    ],
)

PDF_OCR_SHARD_RESULT_SCHEMA = pa.schema(
    [
        pa.field("contractVersion", pa.string(), nullable=False),
        pa.field("sourcePath", pa.string(), nullable=False),
        pa.field("sourceContentHash", pa.string(), nullable=False),
        pa.field("pageIndex", pa.int32(), nullable=False),
        pa.field("imagePath", pa.string(), nullable=False),
        pa.field("imageMimeType", pa.string(), nullable=False),
        pa.field("rasterSha256", pa.string(), nullable=False),
        pa.field("renderProfile", pa.string(), nullable=False),
        pa.field("ocrProfile", pa.string(), nullable=False),
        pa.field("status", pa.string(), nullable=False),
        pa.field("text", pa.string(), nullable=True),
        pa.field("textMimeType", pa.string(), nullable=False),
        pa.field("confidence", pa.float64(), nullable=True),
        pa.field("errorMessage", pa.string(), nullable=True),
        pa.field("shardElementId", pa.string(), nullable=False),
        pa.field("elementId", pa.string(), nullable=False),
    ],
)


class PdfOcrShardWorkerProtocol(Protocol):
    """Protocol implemented by injected OCR shard workers."""

    def recognize(
        self,
        inputs: Sequence[Mapping[str, Any]],
        *,
        max_workers: int | str | None = None,
    ) -> Sequence[Mapping[str, Any]]:
        """Return OCR result rows for input shard rows."""
