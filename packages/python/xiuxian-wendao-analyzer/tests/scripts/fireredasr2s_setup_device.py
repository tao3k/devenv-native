"""Device selection and command construction for FireRedASR2S diagnostics."""

from __future__ import annotations

import json
import shlex
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path

try:
    from .fireredasr2s_setup_config import (
        FIRERED_DEFAULT_DEVICE,
        FIRERED_DEVICE_CHOICES,
        run_command,
        venv_cli,
        venv_python,
    )
except ImportError:
    from fireredasr2s_setup_config import (
        FIRERED_DEFAULT_DEVICE,
        FIRERED_DEVICE_CHOICES,
        run_command,
        venv_cli,
        venv_python,
    )


def build_firered_command(
    *, venv_dir: Path, model_root: Path, resolved_device: str = "cuda"
) -> str:
    """Build the diagnostic command consumed by audio_asr_diagnostic.py."""

    if resolved_device != "cuda":
        raise ValueError(
            "FireRedASR2S diagnostic setup only supports CUDA as an accelerated "
            "runner. CPU fallback is intentionally disabled, and MPS requires a "
            "separate MLX/Metal audio runner."
        )
    use_gpu = "1"
    parts = [
        str(venv_cli(venv_dir)),
        "--asr_model_dir",
        str(model_root / "FireRedASR2-AED"),
        "--vad_model_dir",
        str(model_root / "FireRedVAD" / "VAD"),
        "--lid_model_dir",
        str(model_root / "FireRedLID"),
        "--punc_model_dir",
        str(model_root / "FireRedPunc"),
        "--asr_use_gpu",
        use_gpu,
        "--vad_use_gpu",
        use_gpu,
        "--lid_use_gpu",
        use_gpu,
        "--punc_use_gpu",
        use_gpu,
        "--write_textgrid",
        "0",
        "--write_srt",
        "0",
    ]
    return shlex.join(parts)


def probe_torch_devices(*, python: Path, dry_run: bool) -> dict[str, bool]:
    """Return CUDA/MPS availability for the isolated FireRed environment."""

    if dry_run:
        return {"cuda": False, "mps": False, "mpsBuilt": False}
    completed = run_command(
        [
            python,
            "-c",
            (
                "import json, torch; "
                "mps=getattr(torch.backends, 'mps', None); "
                "print(json.dumps({"
                "'cuda': torch.cuda.is_available(), "
                "'mps': bool(mps and mps.is_available()), "
                "'mpsBuilt': bool(mps and mps.is_built())"
                "}))"
            ),
        ],
        dry_run=False,
    )
    parsed = json.loads(completed.stdout.strip() or "{}")
    return {
        "cuda": bool(parsed.get("cuda")),
        "mps": bool(parsed.get("mps")),
        "mpsBuilt": bool(parsed.get("mpsBuilt")),
    }


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
    devices = probe_torch_devices(python=venv_python(venv_dir), dry_run=dry_run)
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
