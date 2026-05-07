"""Client configuration for Hosted VLM/OCR OpenAI-compatible workers."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from ..pdf_ocr_contracts import (
    HOSTED_VLM_OCR_API_KEY_ENV,
    HOSTED_VLM_OCR_BASE_URL_ENV,
    HOSTED_VLM_OCR_DEFAULT_API_KEY,
    HOSTED_VLM_OCR_DEFAULT_BASE_URL,
    HOSTED_VLM_OCR_DEFAULT_MAX_TOKENS,
    HOSTED_VLM_OCR_DEFAULT_MODEL,
    HOSTED_VLM_OCR_DEFAULT_PAGE_WINDOW_SIZE,
    HOSTED_VLM_OCR_DEFAULT_PROMPT,
    HOSTED_VLM_OCR_DEFAULT_REGION_COMPOSITE_SIZE,
    HOSTED_VLM_OCR_DEFAULT_REGION_MAX_TOKENS,
    HOSTED_VLM_OCR_DEFAULT_REQUEST_CONCURRENCY,
    HOSTED_VLM_OCR_DEFAULT_TIMEOUT_SECONDS,
    HOSTED_VLM_OCR_MAX_TOKENS_ENV,
    HOSTED_VLM_OCR_MODEL_ENV,
    HOSTED_VLM_OCR_OPENROUTER_BASE_URL,
    HOSTED_VLM_OCR_OPENROUTER_MODEL_ENV,
    HOSTED_VLM_OCR_OPENROUTER_PROVIDER,
    HOSTED_VLM_OCR_OPENROUTER_TEST_MODEL,
    HOSTED_VLM_OCR_PAGE_WINDOW_SIZE_ENV,
    HOSTED_VLM_OCR_PROMPT_ENV,
    HOSTED_VLM_OCR_PROVIDER_ENV,
    HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE_ENV,
    HOSTED_VLM_OCR_REGION_MAX_TOKENS_ENV,
    HOSTED_VLM_OCR_REQUEST_CONCURRENCY_ENV,
    HOSTED_VLM_OCR_TIMEOUT_SECONDS_ENV,
    HOSTED_VLM_OCR_TRACE_PATH_ENV,
)
from .env import (
    env_value,
    openrouter_headers,
    optional_path_env,
    positive_float_env,
    positive_int_env,
    positive_int_value,
    region_atlas_mode_env,
    resolve_openrouter_api_key,
    scaffold_mode_env,
)

if TYPE_CHECKING:
    from collections.abc import Mapping
    from pathlib import Path


@dataclass(frozen=True)
class Ocr2ClientConfig:
    base_url: str
    model: str
    api_key: str
    prompt: str
    max_tokens: int
    region_max_tokens: int
    region_composite_size: int
    region_atlas_mode: str
    timeout_seconds: float
    request_concurrency: int
    page_window_size: int
    scaffold_mode: str
    trace_path: Path | None = None
    extra_headers: Mapping[str, str] | None = None


def ocr2_client_config_from_env(
    *,
    request_concurrency: int | str | None = None,
) -> Ocr2ClientConfig:
    resolved_request_concurrency = positive_int_value(request_concurrency)
    provider = env_value(HOSTED_VLM_OCR_PROVIDER_ENV, "")
    if provider == HOSTED_VLM_OCR_OPENROUTER_PROVIDER:
        return Ocr2ClientConfig(
            base_url=env_value(
                HOSTED_VLM_OCR_BASE_URL_ENV,
                HOSTED_VLM_OCR_OPENROUTER_BASE_URL,
            ),
            model=env_value(
                HOSTED_VLM_OCR_MODEL_ENV,
                env_value(
                    HOSTED_VLM_OCR_OPENROUTER_MODEL_ENV,
                    HOSTED_VLM_OCR_OPENROUTER_TEST_MODEL,
                ),
            ),
            api_key=resolve_openrouter_api_key(),
            prompt=env_value(HOSTED_VLM_OCR_PROMPT_ENV, HOSTED_VLM_OCR_DEFAULT_PROMPT),
            max_tokens=positive_int_env(
                HOSTED_VLM_OCR_MAX_TOKENS_ENV,
                HOSTED_VLM_OCR_DEFAULT_MAX_TOKENS,
            ),
            region_max_tokens=positive_int_env(
                HOSTED_VLM_OCR_REGION_MAX_TOKENS_ENV,
                HOSTED_VLM_OCR_DEFAULT_REGION_MAX_TOKENS,
            ),
            region_composite_size=positive_int_env(
                HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE_ENV,
                HOSTED_VLM_OCR_DEFAULT_REGION_COMPOSITE_SIZE,
            ),
            region_atlas_mode=region_atlas_mode_env(),
            timeout_seconds=positive_float_env(
                HOSTED_VLM_OCR_TIMEOUT_SECONDS_ENV,
                HOSTED_VLM_OCR_DEFAULT_TIMEOUT_SECONDS,
            ),
            request_concurrency=_request_concurrency(resolved_request_concurrency),
            page_window_size=positive_int_env(
                HOSTED_VLM_OCR_PAGE_WINDOW_SIZE_ENV,
                HOSTED_VLM_OCR_DEFAULT_PAGE_WINDOW_SIZE,
            ),
            scaffold_mode=scaffold_mode_env(),
            trace_path=optional_path_env(HOSTED_VLM_OCR_TRACE_PATH_ENV),
            extra_headers=openrouter_headers(),
        )
    if provider and provider != "openai-compatible":
        raise ValueError(
            f"unsupported {HOSTED_VLM_OCR_PROVIDER_ENV}={provider}; "
            "supported values: openai-compatible, openrouter"
        )
    return Ocr2ClientConfig(
        base_url=env_value(
            HOSTED_VLM_OCR_BASE_URL_ENV, HOSTED_VLM_OCR_DEFAULT_BASE_URL
        ),
        model=env_value(HOSTED_VLM_OCR_MODEL_ENV, HOSTED_VLM_OCR_DEFAULT_MODEL),
        api_key=env_value(HOSTED_VLM_OCR_API_KEY_ENV, HOSTED_VLM_OCR_DEFAULT_API_KEY),
        prompt=env_value(HOSTED_VLM_OCR_PROMPT_ENV, HOSTED_VLM_OCR_DEFAULT_PROMPT),
        max_tokens=positive_int_env(
            HOSTED_VLM_OCR_MAX_TOKENS_ENV,
            HOSTED_VLM_OCR_DEFAULT_MAX_TOKENS,
        ),
        region_max_tokens=positive_int_env(
            HOSTED_VLM_OCR_REGION_MAX_TOKENS_ENV,
            HOSTED_VLM_OCR_DEFAULT_REGION_MAX_TOKENS,
        ),
        region_composite_size=positive_int_env(
            HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE_ENV,
            HOSTED_VLM_OCR_DEFAULT_REGION_COMPOSITE_SIZE,
        ),
        region_atlas_mode=region_atlas_mode_env(),
        timeout_seconds=positive_float_env(
            HOSTED_VLM_OCR_TIMEOUT_SECONDS_ENV,
            HOSTED_VLM_OCR_DEFAULT_TIMEOUT_SECONDS,
        ),
        request_concurrency=_request_concurrency(resolved_request_concurrency),
        page_window_size=positive_int_env(
            HOSTED_VLM_OCR_PAGE_WINDOW_SIZE_ENV,
            HOSTED_VLM_OCR_DEFAULT_PAGE_WINDOW_SIZE,
        ),
        scaffold_mode=scaffold_mode_env(),
        trace_path=optional_path_env(HOSTED_VLM_OCR_TRACE_PATH_ENV),
    )


def _request_concurrency(resolved: int | None) -> int:
    return resolved or positive_int_env(
        HOSTED_VLM_OCR_REQUEST_CONCURRENCY_ENV,
        HOSTED_VLM_OCR_DEFAULT_REQUEST_CONCURRENCY,
    )
