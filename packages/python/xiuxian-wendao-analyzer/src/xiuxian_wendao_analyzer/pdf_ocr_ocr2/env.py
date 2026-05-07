"""Environment resolution for DeepSeek-OCR-2 worker clients."""

from __future__ import annotations

import os
from pathlib import Path

from ..pdf_ocr_contracts import (
    DEEPSEEK_OCR2_API_KEY_ENV,
    DEEPSEEK_OCR2_DEFAULT_API_KEY,
    DEEPSEEK_OCR2_DEFAULT_REGION_ATLAS_MODE,
    DEEPSEEK_OCR2_DEFAULT_SCAFFOLD_MODE,
    DEEPSEEK_OCR2_OPENROUTE_COMPAT_API_KEY_ENV,
    DEEPSEEK_OCR2_OPENROUTER_API_KEY_ENV,
    DEEPSEEK_OCR2_OPENROUTER_HTTP_REFERER_ENV,
    DEEPSEEK_OCR2_OPENROUTER_PUBLIC_API_KEY_ENV,
    DEEPSEEK_OCR2_OPENROUTER_TITLE_ENV,
    DEEPSEEK_OCR2_REGION_ATLAS_MODE_ENV,
    DEEPSEEK_OCR2_REGION_ATLAS_SAME_PAGE_JSON_MODE,
    DEEPSEEK_OCR2_REGION_TABLE_JSON_SCAFFOLD_MODE,
    DEEPSEEK_OCR2_SCAFFOLD_MODE_ENV,
)


def resolve_openrouter_api_key() -> str:
    api_key = env_value(
        DEEPSEEK_OCR2_API_KEY_ENV,
        env_value(
            DEEPSEEK_OCR2_OPENROUTER_API_KEY_ENV,
            env_value(
                DEEPSEEK_OCR2_OPENROUTER_PUBLIC_API_KEY_ENV,
                env_value(
                    DEEPSEEK_OCR2_OPENROUTE_COMPAT_API_KEY_ENV,
                    DEEPSEEK_OCR2_DEFAULT_API_KEY,
                ),
            ),
        ),
    )
    if not api_key or api_key == DEEPSEEK_OCR2_DEFAULT_API_KEY:
        raise ValueError(
            "OpenRouter OCR provider requires WENDAO_OPENROUTER_API_KEY, "
            "OPENROUTER_API_KEY, OPENROUTE_API_KEY, or "
            "WENDAO_DEEPSEEK_OCR2_API_KEY"
        )
    return api_key


def openrouter_headers() -> dict[str, str]:
    headers = {}
    referer = env_value(DEEPSEEK_OCR2_OPENROUTER_HTTP_REFERER_ENV, "")
    title = env_value(DEEPSEEK_OCR2_OPENROUTER_TITLE_ENV, "")
    if referer:
        headers["HTTP-Referer"] = referer
    if title:
        headers["X-OpenRouter-Title"] = title
    return headers


def positive_int_env(key: str, default: int) -> int:
    try:
        value = int(os.environ.get(key, ""))
    except ValueError:
        return default
    return value if value > 0 else default


def positive_int_value(value: int | str | None) -> int | None:
    if value is None:
        return None
    try:
        parsed = int(value)
    except (TypeError, ValueError):
        return None
    return parsed if parsed > 0 else None


def positive_float_env(key: str, default: float) -> float:
    try:
        value = float(os.environ.get(key, ""))
    except ValueError:
        return default
    return value if value > 0 else default


def scaffold_mode_env() -> str:
    value = (
        os.environ.get(
            DEEPSEEK_OCR2_SCAFFOLD_MODE_ENV, DEEPSEEK_OCR2_DEFAULT_SCAFFOLD_MODE
        )
        .strip()
        .replace("_", "-")
        .lower()
    )
    if not value:
        return DEEPSEEK_OCR2_DEFAULT_SCAFFOLD_MODE
    if value in {
        DEEPSEEK_OCR2_DEFAULT_SCAFFOLD_MODE,
        DEEPSEEK_OCR2_REGION_TABLE_JSON_SCAFFOLD_MODE,
    }:
        return value
    raise ValueError(
        f"unsupported {DEEPSEEK_OCR2_SCAFFOLD_MODE_ENV}={value}; "
        "supported values: disabled, region-table-json"
    )


def region_atlas_mode_env() -> str:
    value = (
        os.environ.get(
            DEEPSEEK_OCR2_REGION_ATLAS_MODE_ENV,
            DEEPSEEK_OCR2_DEFAULT_REGION_ATLAS_MODE,
        )
        .strip()
        .replace("_", "-")
        .lower()
    )
    if not value:
        return DEEPSEEK_OCR2_DEFAULT_REGION_ATLAS_MODE
    if value in {
        DEEPSEEK_OCR2_DEFAULT_REGION_ATLAS_MODE,
        DEEPSEEK_OCR2_REGION_ATLAS_SAME_PAGE_JSON_MODE,
    }:
        return value
    raise ValueError(
        f"unsupported {DEEPSEEK_OCR2_REGION_ATLAS_MODE_ENV}={value}; "
        "supported values: disabled, same-page-json"
    )


def env_value(key: str, default: str) -> str:
    value = os.environ.get(key)
    if value is None or not value.strip():
        return default
    return value


def optional_path_env(key: str) -> Path | None:
    value = os.environ.get(key)
    if value is None or not value.strip():
        return None
    return Path(value)
