"""Client configuration for DeepSeek-OCR-2 OpenAI-compatible workers."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from ..pdf_ocr_contracts import (
    DEEPSEEK_OCR2_API_KEY_ENV,
    DEEPSEEK_OCR2_BASE_URL_ENV,
    DEEPSEEK_OCR2_DEFAULT_API_KEY,
    DEEPSEEK_OCR2_DEFAULT_BASE_URL,
    DEEPSEEK_OCR2_DEFAULT_MAX_TOKENS,
    DEEPSEEK_OCR2_DEFAULT_MODEL,
    DEEPSEEK_OCR2_DEFAULT_PAGE_WINDOW_SIZE,
    DEEPSEEK_OCR2_DEFAULT_PROMPT,
    DEEPSEEK_OCR2_DEFAULT_REGION_COMPOSITE_SIZE,
    DEEPSEEK_OCR2_DEFAULT_REGION_MAX_TOKENS,
    DEEPSEEK_OCR2_DEFAULT_REQUEST_CONCURRENCY,
    DEEPSEEK_OCR2_DEFAULT_TIMEOUT_SECONDS,
    DEEPSEEK_OCR2_MAX_TOKENS_ENV,
    DEEPSEEK_OCR2_MODEL_ENV,
    DEEPSEEK_OCR2_OPENROUTER_BASE_URL,
    DEEPSEEK_OCR2_OPENROUTER_MODEL_ENV,
    DEEPSEEK_OCR2_OPENROUTER_PROVIDER,
    DEEPSEEK_OCR2_OPENROUTER_TEST_MODEL,
    DEEPSEEK_OCR2_PAGE_WINDOW_SIZE_ENV,
    DEEPSEEK_OCR2_PROMPT_ENV,
    DEEPSEEK_OCR2_PROVIDER_ENV,
    DEEPSEEK_OCR2_REGION_COMPOSITE_SIZE_ENV,
    DEEPSEEK_OCR2_REGION_MAX_TOKENS_ENV,
    DEEPSEEK_OCR2_REQUEST_CONCURRENCY_ENV,
    DEEPSEEK_OCR2_TIMEOUT_SECONDS_ENV,
    DEEPSEEK_OCR2_TRACE_PATH_ENV,
)
from .env import (
    env_value,
    openrouter_headers,
    optional_path_env,
    positive_float_env,
    positive_int_env,
    positive_int_value,
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
    provider = env_value(DEEPSEEK_OCR2_PROVIDER_ENV, "")
    if provider == DEEPSEEK_OCR2_OPENROUTER_PROVIDER:
        return Ocr2ClientConfig(
            base_url=env_value(
                DEEPSEEK_OCR2_BASE_URL_ENV,
                DEEPSEEK_OCR2_OPENROUTER_BASE_URL,
            ),
            model=env_value(
                DEEPSEEK_OCR2_MODEL_ENV,
                env_value(
                    DEEPSEEK_OCR2_OPENROUTER_MODEL_ENV,
                    DEEPSEEK_OCR2_OPENROUTER_TEST_MODEL,
                ),
            ),
            api_key=resolve_openrouter_api_key(),
            prompt=env_value(DEEPSEEK_OCR2_PROMPT_ENV, DEEPSEEK_OCR2_DEFAULT_PROMPT),
            max_tokens=positive_int_env(
                DEEPSEEK_OCR2_MAX_TOKENS_ENV,
                DEEPSEEK_OCR2_DEFAULT_MAX_TOKENS,
            ),
            region_max_tokens=positive_int_env(
                DEEPSEEK_OCR2_REGION_MAX_TOKENS_ENV,
                DEEPSEEK_OCR2_DEFAULT_REGION_MAX_TOKENS,
            ),
            region_composite_size=positive_int_env(
                DEEPSEEK_OCR2_REGION_COMPOSITE_SIZE_ENV,
                DEEPSEEK_OCR2_DEFAULT_REGION_COMPOSITE_SIZE,
            ),
            timeout_seconds=positive_float_env(
                DEEPSEEK_OCR2_TIMEOUT_SECONDS_ENV,
                DEEPSEEK_OCR2_DEFAULT_TIMEOUT_SECONDS,
            ),
            request_concurrency=_request_concurrency(resolved_request_concurrency),
            page_window_size=positive_int_env(
                DEEPSEEK_OCR2_PAGE_WINDOW_SIZE_ENV,
                DEEPSEEK_OCR2_DEFAULT_PAGE_WINDOW_SIZE,
            ),
            scaffold_mode=scaffold_mode_env(),
            trace_path=optional_path_env(DEEPSEEK_OCR2_TRACE_PATH_ENV),
            extra_headers=openrouter_headers(),
        )
    if provider and provider != "openai-compatible":
        raise ValueError(
            f"unsupported {DEEPSEEK_OCR2_PROVIDER_ENV}={provider}; "
            "supported values: openai-compatible, openrouter"
        )
    return Ocr2ClientConfig(
        base_url=env_value(DEEPSEEK_OCR2_BASE_URL_ENV, DEEPSEEK_OCR2_DEFAULT_BASE_URL),
        model=env_value(DEEPSEEK_OCR2_MODEL_ENV, DEEPSEEK_OCR2_DEFAULT_MODEL),
        api_key=env_value(DEEPSEEK_OCR2_API_KEY_ENV, DEEPSEEK_OCR2_DEFAULT_API_KEY),
        prompt=env_value(DEEPSEEK_OCR2_PROMPT_ENV, DEEPSEEK_OCR2_DEFAULT_PROMPT),
        max_tokens=positive_int_env(
            DEEPSEEK_OCR2_MAX_TOKENS_ENV,
            DEEPSEEK_OCR2_DEFAULT_MAX_TOKENS,
        ),
        region_max_tokens=positive_int_env(
            DEEPSEEK_OCR2_REGION_MAX_TOKENS_ENV,
            DEEPSEEK_OCR2_DEFAULT_REGION_MAX_TOKENS,
        ),
        region_composite_size=positive_int_env(
            DEEPSEEK_OCR2_REGION_COMPOSITE_SIZE_ENV,
            DEEPSEEK_OCR2_DEFAULT_REGION_COMPOSITE_SIZE,
        ),
        timeout_seconds=positive_float_env(
            DEEPSEEK_OCR2_TIMEOUT_SECONDS_ENV,
            DEEPSEEK_OCR2_DEFAULT_TIMEOUT_SECONDS,
        ),
        request_concurrency=_request_concurrency(resolved_request_concurrency),
        page_window_size=positive_int_env(
            DEEPSEEK_OCR2_PAGE_WINDOW_SIZE_ENV,
            DEEPSEEK_OCR2_DEFAULT_PAGE_WINDOW_SIZE,
        ),
        scaffold_mode=scaffold_mode_env(),
        trace_path=optional_path_env(DEEPSEEK_OCR2_TRACE_PATH_ENV),
    )


def _request_concurrency(resolved: int | None) -> int:
    return resolved or positive_int_env(
        DEEPSEEK_OCR2_REQUEST_CONCURRENCY_ENV,
        DEEPSEEK_OCR2_DEFAULT_REQUEST_CONCURRENCY,
    )
