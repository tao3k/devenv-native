"""Command helpers for local DeepSeek-OCR-2 OpenAI-compatible backends."""

from __future__ import annotations

import os
import platform
import shlex
import shutil
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Literal

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


class Ocr2BackendError(RuntimeError):
    """Raised when local OCR2 backend management cannot proceed."""


@dataclass(frozen=True, slots=True)
class Ocr2BackendOptions:
    """Options shared by analyzer-owned OCR2 backend actions."""

    repo_id: str = ""
    model_dir: str = ""
    model_path: str = ""
    quantization: str = "auto"
    backend_runner: str = "auto"


@dataclass(frozen=True, slots=True)
class BackendLaunch:
    """Resolved long-running backend command."""

    runner: str
    command: tuple[str, ...]
    message: str
    env_updates: dict[str, str] = field(default_factory=dict)


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


def fetch_models(options: Ocr2BackendOptions) -> int:
    """Fetch prebuilt DeepSeek-OCR-2 artifacts from Hugging Face.

    # Errors

    Raises `Ocr2BackendError` when the requested model flavor is unsupported or
    when the selected download does not produce a model weight file.
    """

    data_home = _data_home()
    model_flavor = _resolve_model_flavor()
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
    target_dir = data_home / "models" / resolved_model_dir
    current_link = data_home / "models" / "deepseek-ocr2-current"
    target_dir.mkdir(parents=True, exist_ok=True)

    command = [
        *_hf_command(),
        "download",
        *_include_args(),
        "--local-dir",
        str(target_dir),
        resolved_repo_id,
    ]
    subprocess.run(command, check=True)

    if not _has_weight_file(target_dir):
        raise Ocr2BackendError(
            f"no model weight file was downloaded into {target_dir}. "
            "Set WENDAO_DEEPSEEK_OCR2_HF_INCLUDE only when the patterns match "
            "the selected repo."
        )

    _replace_symlink(current_link, target_dir)
    sys.stdout.write(f"DeepSeek-OCR-2 source: {resolved_repo_id}\n")
    sys.stdout.write(f"DeepSeek-OCR-2 model flavor: {model_flavor}\n")
    sys.stdout.write(f"DeepSeek-OCR-2 artifacts: {target_dir}\n")
    sys.stdout.write(f"Current model link: {current_link}\n")
    return 0


def install_vllm_metal() -> int:
    """Install the local vLLM Metal runtime used by macOS OCR2 probes.

    # Errors

    Raises `Ocr2BackendError` when the current host is not Apple Silicon macOS or
    when the installer completes without producing the expected vLLM binary.
    """

    _require_macos_apple_silicon("vLLM Metal")
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
            str(_cache_home() / "ocr" / "vllm-metal-build"),
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

    _require_macos_apple_silicon("vLLM Metal probe")
    python_bin = _vllm_metal_home() / "bin" / "python"
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
        [str(python_bin), str(_module_path("vllm_metal_probe.py"))],
        check=False,
        env=env,
    )
    return completed.returncode


def start_backend(options: Ocr2BackendOptions) -> int:
    """Start the platform-selected OCR2 OpenAI-compatible backend.

    # Errors

    Raises `Ocr2BackendError` when the selected runner is unsupported or its
    required local runtime/model artifact is missing.
    """

    launch = build_start_backend_launch(options)
    sys.stdout.write(f"{launch.message}\n")
    env = os.environ.copy()
    env.update(launch.env_updates)
    os.execvpe(launch.command[0], list(launch.command), env)
    return 127


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
    model_path = _resolve_model_path(options.model_path)
    _require_default_model_path(model_path)
    host = _env("WENDAO_DEEPSEEK_OCR2_HOST", "127.0.0.1")
    port = _env("WENDAO_DEEPSEEK_OCR2_PORT", "8000")
    served_model_name = _env("WENDAO_DEEPSEEK_OCR2_MODEL", DEFAULT_OCR2_MODEL_NAME)
    quantization = _env("WENDAO_DEEPSEEK_OCR2_VLLM_QUANTIZATION", options.quantization)
    vllm_runner = _env("WENDAO_DEEPSEEK_OCR2_VLLM_RUNNER", "auto")

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
            _env("WENDAO_DEEPSEEK_OCR2_VLLM_PACKAGE", DEFAULT_VLLM_PACKAGE),
        ]
        for package in _split_csv(
            _env("WENDAO_DEEPSEEK_OCR2_VLLM_WITH", "addict,matplotlib")
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
    _require_macos_apple_silicon("metal-vllm runner")
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

    model_path = _resolve_model_path(options.model_path)
    _require_default_model_path(model_path)
    vllm_bin = Path(
        os.environ.get(
            "WENDAO_DEEPSEEK_OCR2_VLLM_METAL_BIN",
            str(_vllm_metal_home() / "bin" / "vllm"),
        )
    )
    if not (vllm_bin.is_file() and os.access(vllm_bin, os.X_OK)):
        raise Ocr2BackendError(
            f"vLLM Metal binary is missing: {vllm_bin}. Run: just install-vllm-metal"
        )

    host = _env("WENDAO_DEEPSEEK_OCR2_HOST", "127.0.0.1")
    port = _env("WENDAO_DEEPSEEK_OCR2_PORT", "8000")
    served_model_name = _env("WENDAO_DEEPSEEK_OCR2_MODEL", DEFAULT_OCR2_MODEL_NAME)
    quantization = _env("WENDAO_DEEPSEEK_OCR2_VLLM_QUANTIZATION", options.quantization)
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
    _require_macos_apple_silicon("mlx-vlm runner")
    model_path = _resolve_model_path(options.model_path)
    _require_default_model_path(model_path)
    python_bin = Path(
        os.environ.get(
            "WENDAO_DEEPSEEK_OCR2_MLX_VLM_PYTHON",
            str(_vllm_metal_home() / "bin" / "python"),
        )
    )
    if not (python_bin.is_file() and os.access(python_bin, os.X_OK)):
        raise Ocr2BackendError(
            f"MLX-VLM Python runtime is missing: {python_bin}. "
            "Run: just install-vllm-metal"
        )

    host = _env("WENDAO_DEEPSEEK_OCR2_HOST", "127.0.0.1")
    port = _env("WENDAO_DEEPSEEK_OCR2_PORT", "8000")
    return BackendLaunch(
        runner="mlx-vlm",
        command=(str(python_bin), str(_module_path("mlx_vlm_openai_adapter.py"))),
        message=f"Starting OCR2 MLX-VLM backend at http://{host}:{port}/v1",
        env_updates={
            "WENDAO_DEEPSEEK_OCR2_MODEL_PATH": str(model_path),
            "VLLM_METAL_USE_MLX": os.environ.get("VLLM_METAL_USE_MLX", "1"),
            "VLLM_MLX_DEVICE": os.environ.get("VLLM_MLX_DEVICE", "gpu"),
        },
    )


def _build_official_vllm_launch(options: Ocr2BackendOptions) -> BackendLaunch:
    model_path = _resolve_model_path(options.model_path)
    if not model_path.exists():
        raise Ocr2BackendError(f"{model_path} is missing. Run: just fetch-models")

    cache_home = _cache_home()
    runtime_dir = Path(os.environ.get("PRJ_RUNTIME_DIR", ".run"))
    repo_url = _env(
        "WENDAO_DEEPSEEK_OCR2_OFFICIAL_REPO_URL",
        "https://github.com/deepseek-ai/DeepSeek-OCR-2.git",
    )
    repo_dir = Path(
        os.environ.get(
            "WENDAO_DEEPSEEK_OCR2_OFFICIAL_REPO_DIR",
            str(cache_home / "ocr" / "DeepSeek-OCR-2"),
        )
    )
    adapter_dir = repo_dir / "DeepSeek-OCR2-master" / "DeepSeek-OCR2-vllm"
    _ensure_official_adapter(repo_url, repo_dir, adapter_dir)

    host = _env("WENDAO_DEEPSEEK_OCR2_HOST", "127.0.0.1")
    port = _env("WENDAO_DEEPSEEK_OCR2_PORT", "8000")
    served_model_name = _env("WENDAO_DEEPSEEK_OCR2_MODEL", DEFAULT_OCR2_MODEL_NAME)
    command = ["uv", "run", "--no-project"]
    adapter_python = os.environ.get("WENDAO_DEEPSEEK_OCR2_ADAPTER_PYTHON")
    if adapter_python:
        command.extend(["--python", adapter_python])
    command.extend(["--with", _env("WENDAO_DEEPSEEK_OCR2_VLLM_PACKAGE", "vllm")])
    for package in _split_csv(
        _env(
            "WENDAO_DEEPSEEK_OCR2_ADAPTER_WITH",
            "addict,matplotlib,einops,easydict,pillow,fastapi,uvicorn",
        )
    ):
        command.extend(["--with", package])
    command.extend(["python", str(_module_path("official_vllm_openai_adapter.py"))])

    runtime_config_dir = runtime_dir / "ocr" / "deepseek-ocr2-official-vllm"
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
        "mlx-vlm" if _is_macos_apple_silicon() else "generic-vllm"
    )


def _resolve_model_flavor() -> str:
    return os.environ.get("WENDAO_DEEPSEEK_OCR2_MODEL_FLAVOR") or (
        "metal-mlx" if _is_macos_apple_silicon() else "generic-vllm"
    )


def _resolve_model_path(model_path: str) -> Path:
    if model_path:
        return Path(model_path)
    return Path(
        os.environ.get(
            "WENDAO_DEEPSEEK_OCR2_MODEL_PATH",
            str(_data_home() / "models" / "deepseek-ocr2-current"),
        )
    )


def _require_default_model_path(model_path: Path) -> None:
    default_path = _data_home() / "models" / "deepseek-ocr2-current"
    if model_path == default_path and not model_path.exists():
        raise Ocr2BackendError(f"{model_path} is missing. Run: just fetch-models")


def _require_macos_apple_silicon(label: str) -> None:
    if not _is_macos_apple_silicon():
        raise Ocr2BackendError(f"{label} requires macOS on Apple Silicon.")


def _is_macos_apple_silicon() -> bool:
    return platform.system() == "Darwin" and platform.machine() in {"arm64", "aarch64"}


def _vllm_metal_home() -> Path:
    return Path(
        os.environ.get(
            "WENDAO_DEEPSEEK_OCR2_VLLM_METAL_HOME",
            str(Path.home() / ".venv-vllm-metal"),
        )
    )


def _data_home() -> Path:
    return Path(os.environ.get("PRJ_DATA_HOME", ".data")).resolve()


def _cache_home() -> Path:
    return Path(os.environ.get("PRJ_CACHE_HOME", ".cache")).resolve()


def _env(name: str, default: str) -> str:
    value = os.environ.get(name)
    if value is None or not value.strip():
        return default
    return value


def _split_csv(value: str) -> list[str]:
    return [item.strip() for item in value.split(",") if item.strip()]


def _include_args() -> list[str]:
    patterns = _split_csv(os.environ.get("WENDAO_DEEPSEEK_OCR2_HF_INCLUDE", ""))
    if not patterns:
        return []
    return ["--include", *patterns]


def _hf_command() -> list[str]:
    if shutil.which("hf"):
        return ["hf"]
    if shutil.which("huggingface-cli"):
        return ["huggingface-cli"]
    return ["uvx", "--from", "huggingface-hub", "hf"]


def _has_weight_file(path: Path) -> bool:
    return any(
        candidate.is_file()
        for pattern in ("*.safetensors", "*.gguf", "*.bin")
        for candidate in path.glob(pattern)
    )


def _replace_symlink(link_path: Path, target_path: Path) -> None:
    link_path.parent.mkdir(parents=True, exist_ok=True)
    if link_path.is_symlink():
        link_path.unlink()
    elif link_path.exists():
        raise Ocr2BackendError(
            f"{link_path} exists and is not a symlink; refusing to replace it."
        )
    link_path.symlink_to(target_path)


def _module_path(filename: str) -> Path:
    return Path(__file__).resolve().parent / filename
