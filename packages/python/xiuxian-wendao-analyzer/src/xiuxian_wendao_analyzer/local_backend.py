"""Shared helpers for analyzer-owned local model backends."""

from __future__ import annotations

import os
import platform
from dataclasses import dataclass, field
from pathlib import Path


class LocalBackendError(RuntimeError):
    """Raised when a local backend cannot be resolved or launched."""


@dataclass(frozen=True, slots=True)
class BackendLaunch:
    """Resolved long-running local backend command."""

    runner: str
    command: tuple[str, ...]
    message: str
    env_updates: dict[str, str] = field(default_factory=dict)


@dataclass(frozen=True, slots=True)
class LocalDeviceProbe:
    """Host accelerator availability observed by a local backend manager."""

    platform: str
    machine: str
    torch_mps_available: bool
    torch_mps_built: bool
    torch_cuda_available: bool
    mlx_available: bool

    def to_dict(self) -> dict[str, object]:
        """Return a JSON-serializable probe row."""

        return {
            "platform": self.platform,
            "machine": self.machine,
            "torchMpsAvailable": self.torch_mps_available,
            "torchMpsBuilt": self.torch_mps_built,
            "torchCudaAvailable": self.torch_cuda_available,
            "mlxAvailable": self.mlx_available,
        }


def probe_local_devices() -> LocalDeviceProbe:
    """Probe local accelerator availability without loading model weights."""

    torch_probe = _probe_torch()
    return LocalDeviceProbe(
        platform=platform.system(),
        machine=platform.machine(),
        torch_mps_available=torch_probe["mps"],
        torch_mps_built=torch_probe["mpsBuilt"],
        torch_cuda_available=torch_probe["cuda"],
        mlx_available=_probe_import("mlx"),
    )


def is_macos_apple_silicon() -> bool:
    """Return whether this host is macOS on Apple Silicon."""

    return platform.system() == "Darwin" and platform.machine() in {"arm64", "aarch64"}


def require_macos_apple_silicon(label: str) -> None:
    """Raise when a local Metal backend is requested on an unsupported host."""

    if not is_macos_apple_silicon():
        raise LocalBackendError(f"{label} requires macOS on Apple Silicon.")


def env_value(name: str, default: str) -> str:
    """Return a non-empty environment override or the provided default."""

    value = os.environ.get(name)
    if value is None or not value.strip():
        return default
    return value


def project_data_home() -> Path:
    """Return the project data root used by local backend artifacts."""

    return Path(env_value("PRJ_DATA_HOME", ".data")).resolve()


def project_cache_home() -> Path:
    """Return the project cache root used by local backend build artifacts."""

    return Path(env_value("PRJ_CACHE_HOME", ".cache")).resolve()


def module_path(anchor_file: str, filename: str) -> Path:
    """Return a path beside a manager module."""

    return Path(anchor_file).resolve().parent / filename


def exec_backend_launch(launch: BackendLaunch) -> int:
    """Replace the current process with a resolved local backend command."""

    env = os.environ.copy()
    env.update(launch.env_updates)
    os.execvpe(launch.command[0], list(launch.command), env)
    return 127


def _probe_torch() -> dict[str, bool]:
    try:
        import torch
    except ImportError:
        return {"cuda": False, "mps": False, "mpsBuilt": False}
    mps = getattr(torch.backends, "mps", None)
    return {
        "cuda": bool(torch.cuda.is_available()),
        "mps": bool(mps and mps.is_available()),
        "mpsBuilt": bool(mps and mps.is_built()),
    }


def _probe_import(module_name: str) -> bool:
    try:
        __import__(module_name)
    except ImportError:
        return False
    return True
