"""Provision a local FireRedASR2S diagnostic backend in an isolated venv."""

from __future__ import annotations

import json
import shlex
import sys
from pathlib import Path

try:
    from .fireredasr2s_setup_config import (
        FIRERED_DEFAULT_DEVICE,
        FIRERED_DEVICE_CHOICES,
        CommandResult,
        run_command,
    )
    from .fireredasr2s_setup_device import (
        build_firered_command as _build_firered_command,
    )
    from .fireredasr2s_setup_device import (
        probe_torch_devices as _probe_torch_devices,
    )
    from .fireredasr2s_setup_install import (
        download_models as _download_models,
    )
    from .fireredasr2s_setup_runner import (
        build_parser,
        run_setup,
        write_summary,
    )
except ImportError:
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    from fireredasr2s_setup_config import (
        FIRERED_DEFAULT_DEVICE,
        FIRERED_DEVICE_CHOICES,
        CommandResult,
        run_command,
    )
    from fireredasr2s_setup_device import (
        build_firered_command as _build_firered_command,
    )
    from fireredasr2s_setup_device import (
        probe_torch_devices as _probe_torch_devices,
    )
    from fireredasr2s_setup_install import (
        download_models as _download_models,
    )
    from fireredasr2s_setup_runner import (
        build_parser,
        run_setup,
        write_summary,
    )


def build_firered_command(
    *, venv_dir: Path, model_root: Path, resolved_device: str = "cuda"
) -> str:
    """Build the diagnostic command consumed by audio_asr_diagnostic.py."""

    return _build_firered_command(
        venv_dir=venv_dir,
        model_root=model_root,
        resolved_device=resolved_device,
    )


def probe_torch_devices(*, python: Path, dry_run: bool) -> dict[str, bool]:
    """Return CUDA/MPS availability for the isolated FireRed environment."""

    return _probe_torch_devices(python=python, dry_run=dry_run)


def resolve_firered_device(
    requested_device: str,
    *,
    venv_dir: Path,
    dry_run: bool,
) -> tuple[str, str, dict[str, bool]]:
    """Resolve FireRedASR2S compute device without pretending MPS is supported."""

    requested = (requested_device or FIRERED_DEFAULT_DEVICE).strip().lower()
    if requested not in FIRERED_DEVICE_CHOICES:
        raise ValueError(f"unsupported FireRedASR2S device: {requested_device}")
    devices = probe_torch_devices(python=venv_dir / "bin" / "python", dry_run=dry_run)
    if requested == "cpu":
        raise RuntimeError("FireRedASR2S CPU fallback is intentionally disabled")
    if requested == "cuda":
        if not dry_run and not devices["cuda"]:
            raise RuntimeError(
                "FireRedASR2S CUDA was requested but CUDA is unavailable"
            )
        return "cuda", "CUDA requested explicitly.", devices
    if requested == "mps":
        raise RuntimeError(
            "FireRedASR2S cannot use MPS/Metal through its upstream CLI because "
            "the runner calls torch .cuda() directly. Use a true MLX/Metal audio "
            "runner for the OCR2-style local backend architecture."
        )
    if devices["cuda"]:
        return "cuda", "Auto selected CUDA for FireRedASR2S.", devices
    if devices["mps"]:
        raise RuntimeError(
            "Auto detected MPS/Metal, but FireRedASR2S cannot use it because "
            "the upstream runner only exposes CUDA-style .cuda() acceleration. "
            "Use a true MLX/Metal audio runner instead of CPU fallback."
        )
    raise RuntimeError(
        "FireRedASR2S has no supported accelerated device on this host. CPU "
        "fallback is intentionally disabled."
    )


def download_models(*, venv_dir: Path, model_root: Path, dry_run: bool) -> None:
    """Download FireRedASR2S model weights."""

    if run_command is _download_models.__globals__["run_command"]:
        _download_models(venv_dir=venv_dir, model_root=model_root, dry_run=dry_run)
        return
    from fireredasr2s_setup_config import MODEL_REPOS

    huggingface_cli = venv_dir / "bin" / "huggingface-cli"
    for model_name, repo_id in MODEL_REPOS.items():
        target_dir = model_root / model_name
        if target_dir.exists() and any(target_dir.iterdir()):
            continue
        target_dir.mkdir(parents=True, exist_ok=True)
        run_command(
            [str(huggingface_cli), "download", repo_id, "--local-dir", str(target_dir)],
            dry_run=dry_run,
        )


__all__ = [
    "CommandResult",
    "build_firered_command",
    "build_parser",
    "download_models",
    "main",
    "probe_torch_devices",
    "resolve_firered_device",
    "run_command",
    "run_setup",
    "shlex",
    "write_summary",
]


def main(argv: list[str] | tuple[str, ...] | None = None) -> int:
    """Run the setup helper."""

    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        summary = run_setup(args)
    except Exception as exc:
        print(f"FireRedASR2S setup failed: {exc}", file=sys.stderr)
        return 1
    print(json.dumps(summary, ensure_ascii=False, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
