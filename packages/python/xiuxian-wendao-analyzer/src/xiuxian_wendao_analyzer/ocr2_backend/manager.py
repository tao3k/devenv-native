"""Command helpers for local DeepSeek-OCR-2 OpenAI-compatible backends."""

from __future__ import annotations

from ..local_backend import BackendLaunch
from .manager_launch import build_start_backend_launch, start_backend
from .manager_models import fetch_models
from .manager_types import (
    DEFAULT_OCR2_MODEL_NAME,
    DEFAULT_VLLM_PACKAGE,
    GENERIC_VLLM_MODEL_DIR,
    GENERIC_VLLM_REPO_ID,
    METAL_MLX_MODEL_DIR,
    METAL_MLX_REPO_ID,
    Ocr2BackendAction,
    Ocr2BackendError,
    Ocr2BackendOptions,
)
from .manager_vllm import install_vllm_metal, probe_vllm_metal


def run_ocr2_backend_action(
    action: Ocr2BackendAction,
    options: Ocr2BackendOptions,
) -> int:
    """Run an analyzer-owned OCR2 backend management action.

    # Errors

    Raises `Ocr2BackendError` when the selected action cannot be resolved for
    the current host or when required local artifacts are missing.
    """

    if action == "fetch-models":
        return fetch_models(options)
    if action == "install-vllm-metal":
        return install_vllm_metal()
    if action == "probe-vllm-metal":
        return probe_vllm_metal()
    if action == "start-backend":
        return start_backend(options)
    raise Ocr2BackendError(f"unsupported OCR2 backend action: {action}")


__all__ = [
    "DEFAULT_OCR2_MODEL_NAME",
    "DEFAULT_VLLM_PACKAGE",
    "GENERIC_VLLM_MODEL_DIR",
    "GENERIC_VLLM_REPO_ID",
    "METAL_MLX_MODEL_DIR",
    "METAL_MLX_REPO_ID",
    "BackendLaunch",
    "Ocr2BackendAction",
    "Ocr2BackendError",
    "Ocr2BackendOptions",
    "build_start_backend_launch",
    "fetch_models",
    "install_vllm_metal",
    "probe_vllm_metal",
    "run_ocr2_backend_action",
    "start_backend",
]
