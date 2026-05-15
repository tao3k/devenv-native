"""Model artifact management for local OCR2 backends."""

from __future__ import annotations

import os
import subprocess
import sys

from .manager_paths import (
    data_home,
    has_weight_file,
    hf_command,
    include_args,
    replace_symlink,
    resolve_model_flavor,
)
from .manager_types import (
    GENERIC_VLLM_MODEL_DIR,
    GENERIC_VLLM_REPO_ID,
    METAL_MLX_MODEL_DIR,
    METAL_MLX_REPO_ID,
    Ocr2BackendError,
    Ocr2BackendOptions,
)


def fetch_models(options: Ocr2BackendOptions) -> int:
    """Fetch prebuilt DeepSeek-OCR-2 artifacts from Hugging Face.

    # Errors

    Raises `Ocr2BackendError` when the requested model flavor is unsupported or
    when the selected download does not produce a model weight file.
    """

    model_flavor = resolve_model_flavor()
    if model_flavor == "metal-mlx":
        default_repo_id = METAL_MLX_REPO_ID
        default_model_dir = METAL_MLX_MODEL_DIR
    elif model_flavor == "generic-vllm":
        default_repo_id = GENERIC_VLLM_REPO_ID
        default_model_dir = GENERIC_VLLM_MODEL_DIR
    else:
        raise Ocr2BackendError(
            "unsupported WENDAO_DEEPSEEK_OCR2_MODEL_FLAVOR="
            f"{model_flavor}. Supported values: metal-mlx, generic-vllm"
        )

    resolved_repo_id = (
        options.repo_id
        or os.environ.get("WENDAO_DEEPSEEK_OCR2_HF_REPO")
        or default_repo_id
    )
    resolved_model_dir = (
        options.model_dir
        or os.environ.get("WENDAO_DEEPSEEK_OCR2_MODEL_DIR")
        or default_model_dir
    )
    target_dir = data_home() / "models" / resolved_model_dir
    current_link = data_home() / "models" / "deepseek-ocr2-current"
    target_dir.mkdir(parents=True, exist_ok=True)

    command = [
        *hf_command(),
        "download",
        *include_args(),
        "--local-dir",
        str(target_dir),
        resolved_repo_id,
    ]
    subprocess.run(command, check=True)

    if not has_weight_file(target_dir):
        raise Ocr2BackendError(
            f"no model weight file was downloaded into {target_dir}. "
            "Set WENDAO_DEEPSEEK_OCR2_HF_INCLUDE only when the patterns match "
            "the selected repo."
        )

    replace_symlink(current_link, target_dir)
    sys.stdout.write(f"DeepSeek-OCR-2 source: {resolved_repo_id}\n")
    sys.stdout.write(f"DeepSeek-OCR-2 model flavor: {model_flavor}\n")
    sys.stdout.write(f"DeepSeek-OCR-2 artifacts: {target_dir}\n")
    sys.stdout.write(f"Current model link: {current_link}\n")
    return 0
