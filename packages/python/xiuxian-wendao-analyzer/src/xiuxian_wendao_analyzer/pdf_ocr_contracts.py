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

HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE_ENV = "WENDAO_HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE"

HOSTED_VLM_OCR_REGION_ATLAS_MODE_ENV = "WENDAO_HOSTED_VLM_OCR_REGION_ATLAS_MODE"

HOSTED_VLM_OCR_TIMEOUT_SECONDS_ENV = "WENDAO_HOSTED_VLM_OCR_TIMEOUT_SECONDS"

HOSTED_VLM_OCR_REQUEST_CONCURRENCY_ENV = "WENDAO_HOSTED_VLM_OCR_REQUEST_CONCURRENCY"

HOSTED_VLM_OCR_SPECULATIVE_RETRY_DELAY_SECONDS_ENV = (
    "WENDAO_HOSTED_VLM_OCR_SPECULATIVE_RETRY_DELAY_SECONDS"
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

HOSTED_VLM_OCR_OPENROUTER_TEST_MODEL = "baidu/qianfan-ocr-fast:free"

HOSTED_VLM_OCR_OPENROUTER_HTTP_REFERER_ENV = "WENDAO_OPENROUTER_HTTP_REFERER"

HOSTED_VLM_OCR_OPENROUTER_TITLE_ENV = "WENDAO_OPENROUTER_TITLE"

HOSTED_VLM_OCR_DEFAULT_MODEL = "deepseek-ai/DeepSeek-OCR-2"

HOSTED_VLM_OCR_DEFAULT_API_KEY = "EMPTY"

HOSTED_VLM_OCR_DEFAULT_PROMPT = (
    "<image>\n<|grounding|>Convert the document to markdown."
)

HOSTED_VLM_OCR_DEFAULT_MAX_TOKENS = 8192

HOSTED_VLM_OCR_DEFAULT_REGION_MAX_TOKENS = 2048

HOSTED_VLM_OCR_DEFAULT_REGION_COMPOSITE_SIZE = 1

HOSTED_VLM_OCR_DEFAULT_REGION_ATLAS_MODE = "disabled"

HOSTED_VLM_OCR_REGION_ATLAS_SAME_PAGE_JSON_MODE = "same-page-json"

HOSTED_VLM_OCR_DEFAULT_TIMEOUT_SECONDS = 3600.0

HOSTED_VLM_OCR_DEFAULT_REQUEST_CONCURRENCY = 1

HOSTED_VLM_OCR_DEFAULT_SPECULATIVE_RETRY_DELAY_SECONDS = 0.0

HOSTED_VLM_OCR_DEFAULT_PAGE_WINDOW_SIZE = 1

HOSTED_VLM_OCR_DEFAULT_SCAFFOLD_MODE = "disabled"

HOSTED_VLM_OCR_REGION_TABLE_JSON_SCAFFOLD_MODE = "region-table-json"

HOSTED_VLM_OCR_DEFAULT_IMAGE_OPTIMIZATION = "disabled"

HOSTED_VLM_OCR_REGION_WHITESPACE_TRIM_OPTIMIZATION = "region-whitespace-trim"

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
