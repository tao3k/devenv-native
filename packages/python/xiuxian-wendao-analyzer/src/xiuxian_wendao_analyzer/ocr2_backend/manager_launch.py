"""Launch command builders for local OCR2 backends."""

from __future__ import annotations

import os
import shlex
import shutil
import subprocess
import sys
from pathlib import Path

from ..local_backend import (
    BackendLaunch,
    env_value,
    exec_backend_launch,
    is_macos_apple_silicon,
    module_path,
)
from .manager_paths import (
    cache_home,
    require_default_model_path,
    require_macos_apple_silicon,
    resolve_model_path,
    split_csv,
    vllm_metal_home,
)
from .manager_types import (
    DEFAULT_OCR2_MODEL_NAME,
    DEFAULT_VLLM_PACKAGE,
    Ocr2BackendError,
    Ocr2BackendOptions,
)


def start_backend(options: Ocr2BackendOptions) -> int:
    """Start the platform-selected OCR2 OpenAI-compatible backend.

    # Errors

    Raises `Ocr2BackendError` when the selected runner is unsupported or its
    required local runtime/model artifact is missing.
    """

    launch = build_start_backend_launch(options)
    sys.stdout.write(f"{launch.message}\n")
    return exec_backend_launch(launch)


def build_start_backend_launch(options: Ocr2BackendOptions) -> BackendLaunch:
    """Resolve the command used to serve a local OCR2 backend.

    # Errors

    Raises `Ocr2BackendError` when the selected runner is unsupported or its
    required local runtime/model artifact is missing.
    """

    runner = _resolve_backend_runner(options.backend_runner)
    if runner == "official-vllm":
        return _build_official_vllm_launch(options)
    if runner == "mlx-vlm":
        return _build_mlx_vlm_launch(options)
    if runner == "metal-vllm":
        return _build_metal_vllm_launch(options)
    if runner in {"generic-vllm", "vllm"}:
        return _build_generic_vllm_launch(options)
    raise Ocr2BackendError(
        "unsupported WENDAO_DEEPSEEK_OCR2_BACKEND_RUNNER="
        f"{runner}. Supported values: mlx-vlm, metal-vllm, generic-vllm, official-vllm"
    )


def _build_generic_vllm_launch(options: Ocr2BackendOptions) -> BackendLaunch:
    model_path = resolve_model_path(options.model_path)
    require_default_model_path(model_path)
    host = env_value("WENDAO_DEEPSEEK_OCR2_HOST", "127.0.0.1")
    port = env_value("WENDAO_DEEPSEEK_OCR2_PORT", "8000")
    served_model_name = env_value("WENDAO_DEEPSEEK_OCR2_MODEL", DEFAULT_OCR2_MODEL_NAME)
    quantization = env_value(
        "WENDAO_DEEPSEEK_OCR2_VLLM_QUANTIZATION", options.quantization
    )
    vllm_runner = env_value("WENDAO_DEEPSEEK_OCR2_VLLM_RUNNER", "auto")

    if shutil.which("vllm") and vllm_runner != "uv":
        command = [
            "vllm",
            "serve",
            str(model_path),
            "--host",
            host,
            "--port",
            port,
            "--served-model-name",
            served_model_name,
        ]
    else:
        command = [
            "uv",
            "run",
            "--no-project",
            "--with",
            env_value("WENDAO_DEEPSEEK_OCR2_VLLM_PACKAGE", DEFAULT_VLLM_PACKAGE),
        ]
        for package in split_csv(
            env_value("WENDAO_DEEPSEEK_OCR2_VLLM_WITH", "addict,matplotlib")
        ):
            command.extend(["--with", package])
        command.extend(
            [
                "vllm",
                "serve",
                str(model_path),
                "--host",
                host,
                "--port",
                port,
                "--served-model-name",
                served_model_name,
            ]
        )

    _extend_common_vllm_args(command, quantization)
    return BackendLaunch(
        runner="generic-vllm",
        command=tuple(command),
        message=f"Starting OCR2 backend at http://{host}:{port}/v1",
    )


def _build_metal_vllm_launch(options: Ocr2BackendOptions) -> BackendLaunch:
    require_macos_apple_silicon("metal-vllm runner")
    if (
        os.environ.get("WENDAO_DEEPSEEK_OCR2_VLLM_METAL_ALLOW_UNSUPPORTED_VLM", "0")
        != "1"
    ):
        raise Ocr2BackendError(
            "vLLM Metal is selected for macOS, but DeepSeek-OCR-2 is a "
            "vision-language OCR profile. Current vLLM Metal support is "
            "text-only, so OCR2 VLM serving is blocked on this backend. Use "
            "an external OpenAI-compatible OCR2 endpoint, or set "
            "WENDAO_DEEPSEEK_OCR2_VLLM_METAL_ALLOW_UNSUPPORTED_VLM=1 for an "
            "explicit frontier probe after vLLM Metal adds/validates "
            "multimodal support."
        )

    model_path = resolve_model_path(options.model_path)
    require_default_model_path(model_path)
    vllm_bin = Path(
        os.environ.get(
            "WENDAO_DEEPSEEK_OCR2_VLLM_METAL_BIN",
            str(vllm_metal_home() / "bin" / "vllm"),
        )
    )
    if not (vllm_bin.is_file() and os.access(vllm_bin, os.X_OK)):
        raise Ocr2BackendError(
            f"vLLM Metal binary is missing: {vllm_bin}. Run: just install-vllm-metal"
        )

    host = env_value("WENDAO_DEEPSEEK_OCR2_HOST", "127.0.0.1")
    port = env_value("WENDAO_DEEPSEEK_OCR2_PORT", "8000")
    served_model_name = env_value("WENDAO_DEEPSEEK_OCR2_MODEL", DEFAULT_OCR2_MODEL_NAME)
    quantization = env_value(
        "WENDAO_DEEPSEEK_OCR2_VLLM_QUANTIZATION", options.quantization
    )
    command = [
        str(vllm_bin),
        "serve",
        str(model_path),
        "--host",
        host,
        "--port",
        port,
        "--served-model-name",
        served_model_name,
    ]
    _extend_common_vllm_args(command, quantization)
    return BackendLaunch(
        runner="metal-vllm",
        command=tuple(command),
        message=f"Starting OCR2 vLLM Metal backend at http://{host}:{port}/v1",
        env_updates={
            "VLLM_METAL_USE_MLX": os.environ.get("VLLM_METAL_USE_MLX", "1"),
            "VLLM_MLX_DEVICE": os.environ.get("VLLM_MLX_DEVICE", "gpu"),
            "VLLM_METAL_MULTIMODAL_MODE": os.environ.get(
                "VLLM_METAL_MULTIMODAL_MODE",
                "multimodal-native",
            ),
        },
    )


def _build_mlx_vlm_launch(options: Ocr2BackendOptions) -> BackendLaunch:
    require_macos_apple_silicon("mlx-vlm runner")
    model_path = resolve_model_path(options.model_path)
    require_default_model_path(model_path)
    python_bin = Path(
        os.environ.get(
            "WENDAO_DEEPSEEK_OCR2_MLX_VLM_PYTHON",
            str(vllm_metal_home() / "bin" / "python"),
        )
    )
    if not (python_bin.is_file() and os.access(python_bin, os.X_OK)):
        raise Ocr2BackendError(
            f"MLX-VLM Python runtime is missing: {python_bin}. "
            "Run: just install-vllm-metal"
        )

    host = env_value("WENDAO_DEEPSEEK_OCR2_HOST", "127.0.0.1")
    port = env_value("WENDAO_DEEPSEEK_OCR2_PORT", "8000")
    return BackendLaunch(
        runner="mlx-vlm",
        command=(
            str(python_bin),
            str(module_path(__file__, "mlx_vlm_openai_adapter.py")),
        ),
        message=f"Starting OCR2 MLX-VLM backend at http://{host}:{port}/v1",
        env_updates={
            "WENDAO_DEEPSEEK_OCR2_MODEL_PATH": str(model_path),
            "VLLM_METAL_USE_MLX": os.environ.get("VLLM_METAL_USE_MLX", "1"),
            "VLLM_MLX_DEVICE": os.environ.get("VLLM_MLX_DEVICE", "gpu"),
        },
    )


def _build_official_vllm_launch(options: Ocr2BackendOptions) -> BackendLaunch:
    model_path = resolve_model_path(options.model_path)
    if not model_path.exists():
        raise Ocr2BackendError(f"{model_path} is missing. Run: just fetch-models")

    repo_url = env_value(
        "WENDAO_DEEPSEEK_OCR2_OFFICIAL_REPO_URL",
        "https://github.com/deepseek-ai/DeepSeek-OCR-2.git",
    )
    repo_dir = Path(
        os.environ.get(
            "WENDAO_DEEPSEEK_OCR2_OFFICIAL_REPO_DIR",
            str(cache_home() / "ocr" / "DeepSeek-OCR-2"),
        )
    )
    adapter_dir = repo_dir / "DeepSeek-OCR2-master" / "DeepSeek-OCR2-vllm"
    _ensure_official_adapter(repo_url, repo_dir, adapter_dir)

    host = env_value("WENDAO_DEEPSEEK_OCR2_HOST", "127.0.0.1")
    port = env_value("WENDAO_DEEPSEEK_OCR2_PORT", "8000")
    served_model_name = env_value("WENDAO_DEEPSEEK_OCR2_MODEL", DEFAULT_OCR2_MODEL_NAME)
    command = ["uv", "run", "--no-project"]
    adapter_python = os.environ.get("WENDAO_DEEPSEEK_OCR2_ADAPTER_PYTHON")
    if adapter_python:
        command.extend(["--python", adapter_python])
    command.extend(["--with", env_value("WENDAO_DEEPSEEK_OCR2_VLLM_PACKAGE", "vllm")])
    for package in split_csv(
        env_value(
            "WENDAO_DEEPSEEK_OCR2_ADAPTER_WITH",
            "addict,matplotlib,einops,easydict,pillow,fastapi,uvicorn",
        )
    ):
        command.extend(["--with", package])
    command.extend(
        ["python", str(module_path(__file__, "official_vllm_openai_adapter.py"))]
    )

    runtime_config_dir = Path(os.environ.get("PRJ_RUNTIME_DIR", ".run"))
    runtime_config_dir /= "ocr" / "deepseek-ocr2-official-vllm"
    return BackendLaunch(
        runner="official-vllm",
        command=tuple(command),
        message=(
            f"Starting official OCR2 vLLM adapter at http://{host}:{port}/v1\n"
            f"Official adapter dir: {adapter_dir}"
        ),
        env_updates={
            "WENDAO_DEEPSEEK_OCR2_MODEL_PATH": str(model_path),
            "WENDAO_DEEPSEEK_OCR2_OFFICIAL_VLLM_DIR": str(adapter_dir),
            "WENDAO_DEEPSEEK_OCR2_RUNTIME_CONFIG_DIR": str(runtime_config_dir),
            "WENDAO_DEEPSEEK_OCR2_HOST": host,
            "WENDAO_DEEPSEEK_OCR2_PORT": port,
            "WENDAO_DEEPSEEK_OCR2_MODEL": served_model_name,
        },
    )


def _ensure_official_adapter(repo_url: str, repo_dir: Path, adapter_dir: Path) -> None:
    if adapter_dir.is_dir():
        return
    repo_dir.parent.mkdir(parents=True, exist_ok=True)
    if (repo_dir / ".git").is_dir():
        subprocess.run(
            ["git", "-C", str(repo_dir), "fetch", "--depth", "1", "origin", "main"],
            check=True,
        )
        subprocess.run(
            ["git", "-C", str(repo_dir), "checkout", "FETCH_HEAD"], check=True
        )
    else:
        if repo_dir.exists():
            shutil.rmtree(repo_dir)
        subprocess.run(
            ["git", "clone", "--depth", "1", repo_url, str(repo_dir)], check=True
        )
    if not adapter_dir.is_dir():
        raise Ocr2BackendError(
            f"official DeepSeek-OCR-2 vLLM adapter not found: {adapter_dir}"
        )


def _extend_common_vllm_args(command: list[str], quantization: str) -> None:
    if os.environ.get("WENDAO_DEEPSEEK_OCR2_VLLM_TRUST_REMOTE_CODE", "1") == "1":
        command.append("--trust-remote-code")
    if quantization and quantization not in {"none", "auto"}:
        command.extend(["--quantization", quantization])
    load_format = os.environ.get("WENDAO_DEEPSEEK_OCR2_VLLM_LOAD_FORMAT")
    if load_format:
        command.extend(["--load-format", load_format])
    extra_args = os.environ.get("WENDAO_DEEPSEEK_OCR2_VLLM_EXTRA_ARGS")
    if extra_args:
        command.extend(shlex.split(extra_args))
    elif os.environ.get("WENDAO_DEEPSEEK_OCR2_VLLM_OCR2_DEFAULT_ARGS", "1") == "1":
        command.extend(
            [
                "--logits_processors",
                "vllm.model_executor.models.deepseek_ocr:NGramPerReqLogitsProcessor",
                "--no-enable-prefix-caching",
                "--mm-processor-cache-gb",
                "0",
            ]
        )


def _resolve_backend_runner(requested: str) -> str:
    if requested and requested != "auto":
        return requested
    return os.environ.get("WENDAO_DEEPSEEK_OCR2_BACKEND_RUNNER") or (
        "mlx-vlm" if is_macos_apple_silicon() else "generic-vllm"
    )
