"""Shared types for analyzer-owned OCR2 backend management."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

from ..local_backend import LocalBackendError

Ocr2BackendAction = Literal[
    "fetch-models",
    "install-vllm-metal",
    "probe-vllm-metal",
    "start-backend",
]

DEFAULT_OCR2_MODEL_NAME = "deepseek-ai/DeepSeek-OCR-2"
GENERIC_VLLM_REPO_ID = "richarddavison/DeepSeek-OCR-2-FP8"
GENERIC_VLLM_MODEL_DIR = "deepseek-ocr2-fp8"
METAL_MLX_REPO_ID = "mlx-community/DeepSeek-OCR-2-bf16"
METAL_MLX_MODEL_DIR = "deepseek-ocr2-mlx-bf16"
DEFAULT_VLLM_PACKAGE = "vllm>=0.20.1"


class Ocr2BackendError(LocalBackendError):
    """Raised when local OCR2 backend management cannot proceed."""


@dataclass(frozen=True, slots=True)
class Ocr2BackendOptions:
    """Options shared by analyzer-owned OCR2 backend actions."""

    repo_id: str = ""
    model_dir: str = ""
    model_path: str = ""
    quantization: str = "auto"
    backend_runner: str = "auto"
