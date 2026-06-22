"""PDF OCR Arrow contracts and worker protocol."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any, Protocol

import pyarrow as pa

from .arrow_schema_contracts import ArrowSchemaColumn, build_arrow_schema

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence


PDF_OCR_SHARD_INPUT_SCHEMA_VERSION = "xiuxian_wendao.pdf_ocr_shard_input.v1"

PDF_OCR_SHARD_RESULT_SCHEMA_VERSION = "xiuxian_wendao.pdf_ocr_shard_result.v1"

PDF_OCR_SHARD_INPUT_TABLE = "pdf_ocr_shard_input"

PDF_OCR_SHARD_RESULT_TABLE = "pdf_ocr_shard_result"

PDF_OCR_DEFAULT_PROFILE = "docling-compatible-page-ocr-v1"

PDF_OCR_FAST_TEXT_PROFILE = "docling-fast-text-ocr"

PDF_OCR_BACKEND_TEXT_PROFILE = "docling-backend-text-ocr-v1"

PDF_OCR_HOSTED_VLM_DIRECT_PROFILE = "hosted-vlm-direct-ocr-v1"

PDF_OCR_DOCLING_VLM_DEEPSEEK_OCR_PROFILE = "docling-vlm-deepseek-ocr"


def is_hosted_vlm_direct_profile(profile: str) -> bool:
    """Return true when an OCR profile uses the hosted direct VLM worker."""

    return profile in {
        PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
    }


PDF_OCR_WORKERS_ENV = "WENDAO_PDF_OCR_WORKERS"

PDF_OCR_MAX_WORKERS_ENV = "WENDAO_PDF_OCR_MAX_WORKERS"

HOSTED_VLM_OCR_BASE_URL_ENV = "WENDAO_HOSTED_VLM_OCR_BASE_URL"

HOSTED_VLM_OCR_PROVIDER_ENV = "WENDAO_HOSTED_VLM_OCR_PROVIDER"

HOSTED_VLM_OCR_MODEL_ENV = "WENDAO_HOSTED_VLM_OCR_MODEL"

HOSTED_VLM_OCR_API_KEY_ENV = "WENDAO_HOSTED_VLM_OCR_API_KEY"

HOSTED_VLM_OCR_PROMPT_ENV = "WENDAO_HOSTED_VLM_OCR_PROMPT"

HOSTED_VLM_OCR_MAX_TOKENS_ENV = "WENDAO_HOSTED_VLM_OCR_MAX_TOKENS"

HOSTED_VLM_OCR_REGION_MAX_TOKENS_ENV = "WENDAO_HOSTED_VLM_OCR_REGION_MAX_TOKENS"

HOSTED_VLM_OCR_REGION_PROMPT_MODE_ENV = "WENDAO_HOSTED_VLM_OCR_REGION_PROMPT_MODE"

HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE_ENV = "WENDAO_HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE"

HOSTED_VLM_OCR_REGION_COMPOSITE_MODE_ENV = "WENDAO_HOSTED_VLM_OCR_REGION_COMPOSITE_MODE"

HOSTED_VLM_OCR_REGION_COMPOSITE_MAX_SOURCE_PIXELS_ENV = (
    "WENDAO_HOSTED_VLM_OCR_REGION_COMPOSITE_MAX_SOURCE_PIXELS"
)

HOSTED_VLM_OCR_REGION_COMPOSITE_MAX_IMAGE_BYTES_ENV = (
    "WENDAO_HOSTED_VLM_OCR_REGION_COMPOSITE_MAX_IMAGE_BYTES"
)

HOSTED_VLM_OCR_REGION_ATLAS_MODE_ENV = "WENDAO_HOSTED_VLM_OCR_REGION_ATLAS_MODE"

HOSTED_VLM_OCR_TIMEOUT_SECONDS_ENV = "WENDAO_HOSTED_VLM_OCR_TIMEOUT_SECONDS"

HOSTED_VLM_OCR_REQUEST_CONCURRENCY_ENV = "WENDAO_HOSTED_VLM_OCR_REQUEST_CONCURRENCY"

HOSTED_VLM_OCR_SPECULATIVE_RETRY_DELAY_SECONDS_ENV = (
    "WENDAO_HOSTED_VLM_OCR_SPECULATIVE_RETRY_DELAY_SECONDS"
)

HOSTED_VLM_OCR_SPECULATIVE_RETRY_MIN_SOURCE_PIXELS_ENV = (
    "WENDAO_HOSTED_VLM_OCR_SPECULATIVE_RETRY_MIN_SOURCE_PIXELS"
)

HOSTED_VLM_OCR_SPECULATIVE_RETRY_MIN_IMAGE_BYTES_ENV = (
    "WENDAO_HOSTED_VLM_OCR_SPECULATIVE_RETRY_MIN_IMAGE_BYTES"
)

HOSTED_VLM_OCR_PAGE_WINDOW_SIZE_ENV = "WENDAO_HOSTED_VLM_OCR_PAGE_WINDOW_SIZE"

HOSTED_VLM_OCR_TRACE_PATH_ENV = "WENDAO_HOSTED_VLM_OCR_TRACE_PATH"

HOSTED_VLM_OCR_SCAFFOLD_MODE_ENV = "WENDAO_HOSTED_VLM_OCR_SCAFFOLD_MODE"

HOSTED_VLM_OCR_IMAGE_OPTIMIZATION_ENV = "WENDAO_HOSTED_VLM_OCR_IMAGE_OPTIMIZATION"

HOSTED_VLM_OCR_DEFAULT_BASE_URL = "http://127.0.0.1:8000/v1"

HOSTED_VLM_OCR_OPENROUTER_PROVIDER = "openrouter"

HOSTED_VLM_OCR_OPENROUTER_BASE_URL = "https://openrouter.ai/api/v1"

HOSTED_VLM_OCR_OPENROUTER_API_KEY_ENV = "WENDAO_OPENROUTER_API_KEY"

HOSTED_VLM_OCR_OPENROUTER_PUBLIC_API_KEY_ENV = "OPENROUTER_API_KEY"

HOSTED_VLM_OCR_OPENROUTER_MODEL_ENV = "WENDAO_OPENROUTER_MODEL"

HOSTED_VLM_OCR_OPENROUTER_TEST_MODEL = "baidu/qianfan-ocr-fast"

HOSTED_VLM_OCR_OPENROUTER_HTTP_REFERER_ENV = "WENDAO_OPENROUTER_HTTP_REFERER"

HOSTED_VLM_OCR_OPENROUTER_TITLE_ENV = "WENDAO_OPENROUTER_TITLE"

HOSTED_VLM_OCR_OPENROUTER_PROVIDER_JSON_ENV = "WENDAO_HOSTED_VLM_OCR_OPENROUTER_PROVIDER_JSON"

HOSTED_VLM_OCR_DEFAULT_MODEL = "deepseek-ai/DeepSeek-OCR-2"

HOSTED_VLM_OCR_DEFAULT_API_KEY = "EMPTY"

HOSTED_VLM_OCR_DEFAULT_PROMPT = "<image>\n<|grounding|>Convert the document to markdown."

HOSTED_VLM_OCR_DEFAULT_MAX_TOKENS = 8192

HOSTED_VLM_OCR_DEFAULT_REGION_MAX_TOKENS = 2048

HOSTED_VLM_OCR_DEFAULT_REGION_PROMPT_MODE = "default"

HOSTED_VLM_OCR_COMPACT_REGION_MARKDOWN_PROMPT_MODE = "compact-region-markdown"

HOSTED_VLM_OCR_DEFAULT_REGION_COMPOSITE_SIZE = 1

HOSTED_VLM_OCR_DEFAULT_REGION_COMPOSITE_MODE = "fixed"

HOSTED_VLM_OCR_REGION_COMPOSITE_DISABLED_MODE = "disabled"

HOSTED_VLM_OCR_REGION_COMPOSITE_FIXED_MODE = "fixed"

HOSTED_VLM_OCR_REGION_COMPOSITE_ADAPTIVE_SMALL_REGION_MODE = "adaptive-small-region"

HOSTED_VLM_OCR_DEFAULT_REGION_COMPOSITE_MAX_SOURCE_PIXELS = 2_500_000

HOSTED_VLM_OCR_DEFAULT_REGION_COMPOSITE_MAX_IMAGE_BYTES = 750_000

HOSTED_VLM_OCR_DEFAULT_REGION_ATLAS_MODE = "disabled"

HOSTED_VLM_OCR_REGION_ATLAS_SAME_PAGE_JSON_MODE = "same-page-json"

HOSTED_VLM_OCR_DEFAULT_TIMEOUT_SECONDS = 3600.0

HOSTED_VLM_OCR_DEFAULT_REQUEST_CONCURRENCY = 1

HOSTED_VLM_OCR_DEFAULT_SPECULATIVE_RETRY_DELAY_SECONDS = 0.0

HOSTED_VLM_OCR_DEFAULT_SPECULATIVE_RETRY_MIN_SOURCE_PIXELS = 0

HOSTED_VLM_OCR_DEFAULT_SPECULATIVE_RETRY_MIN_IMAGE_BYTES = 0

HOSTED_VLM_OCR_DEFAULT_PAGE_WINDOW_SIZE = 1

HOSTED_VLM_OCR_DEFAULT_SCAFFOLD_MODE = "disabled"

HOSTED_VLM_OCR_REGION_TABLE_JSON_SCAFFOLD_MODE = "region-table-json"

HOSTED_VLM_OCR_DEFAULT_IMAGE_OPTIMIZATION = "disabled"

HOSTED_VLM_OCR_REGION_WHITESPACE_TRIM_OPTIMIZATION = "region-whitespace-trim"

PDF_OCR_PAGE_BREAK_SENTINEL = "<!-- xiuxian-wendao-pdf-ocr-page-break -->"

PDF_OCR_SHARD_INPUT_SCHEMA = build_arrow_schema(
    PDF_OCR_SHARD_INPUT_TABLE,
    (
        ArrowSchemaColumn("contractVersion", pa.string(), nullable=False),
        ArrowSchemaColumn("sourcePath", pa.string(), nullable=False),
        ArrowSchemaColumn("sourceContentHash", pa.string(), nullable=False),
        ArrowSchemaColumn("pageIndex", pa.int32(), nullable=False),
        ArrowSchemaColumn("imagePath", pa.string(), nullable=False),
        ArrowSchemaColumn("imageMimeType", pa.string(), nullable=False),
        ArrowSchemaColumn("rasterSha256", pa.string(), nullable=False),
        ArrowSchemaColumn("renderProfile", pa.string(), nullable=False),
        ArrowSchemaColumn("ocrProfile", pa.string(), nullable=False),
        ArrowSchemaColumn("ocrEngine", pa.string(), nullable=False),
        ArrowSchemaColumn("preferredLanguages", pa.string(), nullable=False),
        ArrowSchemaColumn("minConfidence", pa.float64(), nullable=False),
        ArrowSchemaColumn("preserveLayout", pa.bool_(), nullable=False),
        ArrowSchemaColumn("rasterWidthPx", pa.int32(), nullable=False),
        ArrowSchemaColumn("rasterHeightPx", pa.int32(), nullable=False),
        ArrowSchemaColumn("renderDpi", pa.int32(), nullable=False),
        ArrowSchemaColumn("rotationDegrees", pa.int32(), nullable=False),
        ArrowSchemaColumn("cropLeft", pa.float64(), nullable=False),
        ArrowSchemaColumn("cropBottom", pa.float64(), nullable=False),
        ArrowSchemaColumn("cropRight", pa.float64(), nullable=False),
        ArrowSchemaColumn("cropTop", pa.float64(), nullable=False),
        ArrowSchemaColumn("pointToPixelScaleX", pa.float64(), nullable=False),
        ArrowSchemaColumn("pointToPixelScaleY", pa.float64(), nullable=False),
        ArrowSchemaColumn("shardElementId", pa.string(), nullable=False),
        ArrowSchemaColumn("shardType", pa.string(), nullable=False),
        ArrowSchemaColumn("regionIndex", pa.int32(), nullable=False),
        ArrowSchemaColumn("parentShardElementId", pa.string(), nullable=False),
        ArrowSchemaColumn("readingOrderKey", pa.string(), nullable=False),
        ArrowSchemaColumn("sourcePagePixelLeft", pa.int32(), nullable=False),
        ArrowSchemaColumn("sourcePagePixelTop", pa.int32(), nullable=False),
        ArrowSchemaColumn("sourcePagePixelRight", pa.int32(), nullable=False),
        ArrowSchemaColumn("sourcePagePixelBottom", pa.int32(), nullable=False),
    ),
)

PDF_OCR_SHARD_RESULT_SCHEMA = build_arrow_schema(
    PDF_OCR_SHARD_RESULT_TABLE,
    (
        ArrowSchemaColumn("contractVersion", pa.string(), nullable=False),
        ArrowSchemaColumn("sourcePath", pa.string(), nullable=False),
        ArrowSchemaColumn("sourceContentHash", pa.string(), nullable=False),
        ArrowSchemaColumn("pageIndex", pa.int32(), nullable=False),
        ArrowSchemaColumn("imagePath", pa.string(), nullable=False),
        ArrowSchemaColumn("imageMimeType", pa.string(), nullable=False),
        ArrowSchemaColumn("rasterSha256", pa.string(), nullable=False),
        ArrowSchemaColumn("renderProfile", pa.string(), nullable=False),
        ArrowSchemaColumn("ocrProfile", pa.string(), nullable=False),
        ArrowSchemaColumn("status", pa.string(), nullable=False),
        ArrowSchemaColumn("text", pa.string(), nullable=True),
        ArrowSchemaColumn("textMimeType", pa.string(), nullable=False),
        ArrowSchemaColumn("confidence", pa.float64(), nullable=True),
        ArrowSchemaColumn("errorMessage", pa.string(), nullable=True),
        ArrowSchemaColumn("shardElementId", pa.string(), nullable=False),
        ArrowSchemaColumn("elementId", pa.string(), nullable=False),
    ),
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
