"""Command helpers for local OpenAI-compatible audio backends."""

from __future__ import annotations

import json
import sys
from dataclasses import dataclass
from typing import Literal

from ..local_backend import (
    BackendLaunch,
    LocalBackendError,
    env_value,
    exec_backend_launch,
    is_macos_apple_silicon,
    module_path,
    probe_local_devices,
)

AudioBackendAction = Literal["probe-local", "start-backend"]
AudioBackendRunner = Literal["auto", "qwen3-asr-mlx", "fireredasr2s"]

DEFAULT_AUDIO_MODEL_NAME = "wendao-local-audio"
DEFAULT_QWEN3_ASR_PACKAGE = "mlx-qwen3-asr"


class AudioBackendError(LocalBackendError):
    """Raised when local audio backend management cannot proceed."""


@dataclass(frozen=True, slots=True)
class AudioBackendOptions:
    """Options shared by analyzer-owned audio backend actions."""

    model_path: str = ""
    backend_runner: str = "auto"
    host: str = ""
    port: str = ""


@dataclass(frozen=True, slots=True)
class AudioBackendProbe:
    """Local accelerator and runner readiness report."""

    runner: str
    platform: str
    machine: str
    torch_mps_available: bool
    torch_mps_built: bool
    torch_cuda_available: bool
    mlx_available: bool
    selected_runner: str
    metal_usable: bool
    runner_usable: bool
    note: str

    def to_dict(self) -> dict[str, object]:
        """Return a JSON-serializable probe row."""

        return {
            "runner": self.runner,
            "platform": self.platform,
            "machine": self.machine,
            "torchMpsAvailable": self.torch_mps_available,
            "torchMpsBuilt": self.torch_mps_built,
            "torchCudaAvailable": self.torch_cuda_available,
            "mlxAvailable": self.mlx_available,
            "selectedRunner": self.selected_runner,
            "metalUsable": self.metal_usable,
            "runnerUsable": self.runner_usable,
            "note": self.note,
        }


def run_audio_backend_action(
    action: AudioBackendAction,
    options: AudioBackendOptions,
) -> int:
    """Run an analyzer-owned audio backend management action.

    # Errors

    Raises `AudioBackendError` when the selected action cannot be resolved for
    the current host or when required local artifacts are missing.
    """

    if action == "probe-local":
        probe = probe_local_audio_backend(options.backend_runner)
        sys.stdout.write(json.dumps(probe.to_dict(), indent=2, sort_keys=True) + "\n")
        return 0 if probe.runner_usable else 1
    if action == "start-backend":
        launch = build_start_backend_launch(options)
        sys.stdout.write(f"{launch.message}\n")
        return exec_backend_launch(launch)
    raise AudioBackendError(f"unsupported audio backend action: {action}")


def build_start_backend_launch(options: AudioBackendOptions) -> BackendLaunch:
    """Resolve the command used to serve a local audio backend.

    # Errors

    Raises `AudioBackendError` when the selected runner is unsupported or its
    required local runtime/model artifact is missing.
    """

    runner = _resolve_backend_runner(options.backend_runner)
    if runner == "qwen3-asr-mlx":
        return _build_qwen3_asr_mlx_launch(options)
    if runner == "fireredasr2s":
        raise AudioBackendError(
            "FireRedASR2S is CUDA-only in the current upstream CLI and is not a "
            "macOS Metal audio backend. Use qwen3-asr-mlx on Apple Silicon or "
            "a separate hosted OpenAI-compatible FireRed service."
        )
    raise AudioBackendError(
        f"unsupported WENDAO_AUDIO_BACKEND_RUNNER={runner}. "
        "Supported values: qwen3-asr-mlx, fireredasr2s"
    )


def probe_local_audio_backend(requested_runner: str = "auto") -> AudioBackendProbe:
    """Probe local audio backend readiness without loading model weights."""

    runner = _resolve_backend_runner(requested_runner)
    device_probe = probe_local_devices()
    metal_usable = runner == "qwen3-asr-mlx" and is_macos_apple_silicon()
    if runner == "qwen3-asr-mlx" and not is_macos_apple_silicon():
        note = f"{runner} requires macOS on Apple Silicon for the local Metal path."
    elif runner == "qwen3-asr-mlx" and not device_probe.mlx_available:
        note = (
            "mlx is not installed in this environment; start-backend uses uv --with "
            f"to provision {runner} for the adapter process."
        )
    elif runner == "qwen3-asr-mlx":
        note = "Qwen3-ASR MLX is the selected Metal Chinese ASR runner."
    elif runner == "fireredasr2s":
        note = (
            "FireRedASR2S is CUDA-only and not used for macOS Metal audio."
            if not device_probe.torch_cuda_available
            else "FireRedASR2S CUDA runner is available."
        )
    else:
        note = "No usable local audio runner selected."
    return AudioBackendProbe(
        runner=requested_runner or "auto",
        platform=device_probe.platform,
        machine=device_probe.machine,
        torch_mps_available=device_probe.torch_mps_available,
        torch_mps_built=device_probe.torch_mps_built,
        torch_cuda_available=device_probe.torch_cuda_available,
        mlx_available=device_probe.mlx_available,
        selected_runner=runner,
        metal_usable=metal_usable,
        runner_usable=metal_usable
        or (runner == "fireredasr2s" and device_probe.torch_cuda_available),
        note=note,
    )


def _build_qwen3_asr_mlx_launch(options: AudioBackendOptions) -> BackendLaunch:
    if not is_macos_apple_silicon():
        raise AudioBackendError(
            "Qwen3-ASR MLX audio runner requires macOS on Apple Silicon."
        )
    host = env_value("WENDAO_AUDIO_LOCAL_HOST", options.host or "127.0.0.1")
    port = env_value("WENDAO_AUDIO_LOCAL_PORT", options.port or "8010")
    model_path = env_value("WENDAO_AUDIO_LOCAL_MODEL_PATH", options.model_path)
    command = [
        "uv",
        "run",
        "--no-project",
        "--with",
        env_value("WENDAO_AUDIO_QWEN3_ASR_PACKAGE", DEFAULT_QWEN3_ASR_PACKAGE),
        "--with",
        "fastapi",
        "--with",
        "uvicorn",
        "python",
        str(module_path(__file__, "qwen3_asr_mlx_openai_adapter.py")),
    ]
    return BackendLaunch(
        runner="qwen3-asr-mlx",
        command=tuple(command),
        message=f"Starting local Qwen3-ASR MLX audio backend at http://{host}:{port}/v1",
        env_updates={
            "WENDAO_AUDIO_LOCAL_HOST": host,
            "WENDAO_AUDIO_LOCAL_PORT": port,
            "WENDAO_AUDIO_LOCAL_MODEL": env_value(
                "WENDAO_AUDIO_LOCAL_MODEL",
                "wendao-qwen3-asr-audio",
            ),
            "WENDAO_AUDIO_LOCAL_MODEL_PATH": model_path,
            "WENDAO_AUDIO_LOCAL_DEVICE": "metal",
        },
    )


def _resolve_backend_runner(requested: str) -> str:
    runner = requested or "auto"
    if runner != "auto":
        return runner
    env_runner = env_value("WENDAO_AUDIO_BACKEND_RUNNER", "")
    if env_runner:
        return env_runner
    return "qwen3-asr-mlx" if is_macos_apple_silicon() else "fireredasr2s"
