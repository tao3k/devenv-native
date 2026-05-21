"""Path and host helpers for OCR2 backend management."""

from __future__ import annotations

import os
import shutil
from pathlib import Path

from ..local_backend import (
    is_macos_apple_silicon,
    project_cache_home,
    project_data_home,
)
from .manager_types import Ocr2BackendError


def resolve_model_flavor() -> str:
    """Resolve the configured OCR2 model artifact flavor."""

    return os.environ.get("WENDAO_DEEPSEEK_OCR2_MODEL_FLAVOR") or (
        "metal-mlx" if is_macos_apple_silicon() else "generic-vllm"
    )


def resolve_model_path(model_path: str) -> Path:
    """Resolve the local OCR2 model path."""

    if model_path:
        return Path(model_path)
    return Path(
        os.environ.get(
            "WENDAO_DEEPSEEK_OCR2_MODEL_PATH",
            str(data_home() / "models" / "deepseek-ocr2-current"),
        )
    )


def require_default_model_path(model_path: Path) -> None:
    """Require the default downloaded model symlink when it is selected."""

    default_path = data_home() / "models" / "deepseek-ocr2-current"
    if model_path == default_path and not model_path.exists():
        raise Ocr2BackendError(f"{model_path} is missing. Run: just fetch-models")


def require_macos_apple_silicon(label: str) -> None:
    """Require an Apple Silicon macOS host for Metal-specific actions."""

    if not is_macos_apple_silicon():
        raise Ocr2BackendError(f"{label} requires macOS on Apple Silicon.")


def vllm_metal_home() -> Path:
    """Return the configured vLLM Metal virtualenv path."""

    return Path(
        os.environ.get(
            "WENDAO_DEEPSEEK_OCR2_VLLM_METAL_HOME",
            str(Path.home() / ".venv-vllm-metal"),
        )
    )


def data_home() -> Path:
    """Return the project data home."""

    return project_data_home()


def cache_home() -> Path:
    """Return the project cache home."""

    return project_cache_home()


def split_csv(value: str) -> list[str]:
    """Split a comma-separated environment value."""

    return [item.strip() for item in value.split(",") if item.strip()]


def include_args() -> list[str]:
    """Return Hugging Face include arguments from the environment."""

    patterns = split_csv(os.environ.get("WENDAO_DEEPSEEK_OCR2_HF_INCLUDE", ""))
    if not patterns:
        return []
    return ["--include", *patterns]


def hf_command() -> list[str]:
    """Resolve the Hugging Face CLI command."""

    if shutil.which("hf"):
        return ["hf"]
    if shutil.which("huggingface-cli"):
        return ["huggingface-cli"]
    return ["uvx", "--from", "huggingface-hub", "hf"]


def has_weight_file(path: Path) -> bool:
    """Return whether a model directory contains recognized weight files."""

    return any(
        candidate.is_file()
        for pattern in ("*.safetensors", "*.gguf", "*.bin")
        for candidate in path.glob(pattern)
    )


def replace_symlink(link_path: Path, target_path: Path) -> None:
    """Replace a model-current symlink without overwriting real files."""

    link_path.parent.mkdir(parents=True, exist_ok=True)
    if link_path.is_symlink():
        link_path.unlink()
    elif link_path.exists():
        raise Ocr2BackendError(
            f"{link_path} exists and is not a symlink; refusing to replace it."
        )
    link_path.symlink_to(target_path)
