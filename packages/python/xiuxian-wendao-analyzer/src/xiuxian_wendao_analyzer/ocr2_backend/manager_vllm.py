"""vLLM Metal installation and readiness probes for OCR2."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

from ..local_backend import module_path
from .manager_paths import cache_home, require_macos_apple_silicon, vllm_metal_home
from .manager_types import Ocr2BackendError


def install_vllm_metal() -> int:
    """Install the local vLLM Metal runtime used by macOS OCR2 probes.

    # Errors

    Raises `Ocr2BackendError` when the current host is not Apple Silicon macOS or
    when the installer completes without producing the expected vLLM binary.
    """

    require_macos_apple_silicon("vLLM Metal")
    install_dir = Path(
        os.environ.get(
            "WENDAO_DEEPSEEK_OCR2_VLLM_METAL_HOME",
            str(Path.home() / ".venv-vllm-metal"),
        )
    )
    vllm_bin = install_dir / "bin" / "vllm"
    if vllm_bin.is_file() and os.access(vllm_bin, os.X_OK):
        sys.stdout.write(f"vLLM Metal is already installed at {install_dir}\n")
        return 0

    install_url = os.environ.get(
        "WENDAO_DEEPSEEK_OCR2_VLLM_METAL_INSTALL_URL",
        "https://raw.githubusercontent.com/vllm-project/vllm-metal/main/install.sh",
    )
    build_dir = Path(
        os.environ.get(
            "WENDAO_DEEPSEEK_OCR2_VLLM_METAL_BUILD_DIR",
            str(cache_home() / "ocr" / "vllm-metal-build"),
        )
    )
    installer = build_dir / "install.sh"
    install_dir.parent.mkdir(parents=True, exist_ok=True)
    build_dir.mkdir(parents=True, exist_ok=True)

    env = os.environ.copy()
    env["UV_PYTHON_DOWNLOADS"] = os.environ.get(
        "WENDAO_DEEPSEEK_OCR2_VLLM_METAL_UV_PYTHON_DOWNLOADS",
        env.get("UV_PYTHON_DOWNLOADS", "automatic"),
    )
    env["UV_PYTHON_PREFERENCE"] = os.environ.get(
        "WENDAO_DEEPSEEK_OCR2_VLLM_METAL_UV_PYTHON_PREFERENCE",
        env.get("UV_PYTHON_PREFERENCE", "managed"),
    )
    env["CC"] = os.environ.get(
        "WENDAO_DEEPSEEK_OCR2_VLLM_METAL_CC",
        env.get("CC", "/usr/bin/clang"),
    )
    env["CXX"] = os.environ.get(
        "WENDAO_DEEPSEEK_OCR2_VLLM_METAL_CXX",
        env.get("CXX", "/usr/bin/clang++"),
    )
    env["PATH"] = (
        f"/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:{env.get('PATH', '')}"
    )

    sys.stdout.write(f"Installing vLLM Metal into {install_dir}\n")
    subprocess.run(["curl", "-fsSL", install_url, "-o", str(installer)], check=True)
    subprocess.run(["bash", str(installer)], cwd=build_dir, env=env, check=True)

    if not (vllm_bin.is_file() and os.access(vllm_bin, os.X_OK)):
        raise Ocr2BackendError(
            f"vLLM Metal install completed but {vllm_bin} is missing."
        )
    sys.stdout.write(f"vLLM Metal installed at {install_dir}\n")
    return 0


def probe_vllm_metal() -> int:
    """Probe local vLLM Metal readiness without loading OCR2 weights.

    # Errors

    Raises `Ocr2BackendError` when the current host is not Apple Silicon macOS or
    when the vLLM Metal Python runtime is missing.
    """

    require_macos_apple_silicon("vLLM Metal probe")
    python_bin = vllm_metal_home() / "bin" / "python"
    if not (python_bin.is_file() and os.access(python_bin, os.X_OK)):
        raise Ocr2BackendError(
            f"vLLM Metal Python runtime is missing: {python_bin}. "
            "Run: just install-vllm-metal"
        )

    env = os.environ.copy()
    env.setdefault("VLLM_METAL_USE_MLX", "1")
    env.setdefault("VLLM_MLX_DEVICE", "gpu")
    env.setdefault("VLLM_METAL_MULTIMODAL_MODE", "multimodal-native")
    completed = subprocess.run(
        [str(python_bin), str(module_path(__file__, "vllm_metal_probe.py"))],
        check=False,
        env=env,
    )
    return completed.returncode
