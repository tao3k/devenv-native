"""Configuration helpers for FireRedASR2S local diagnostics."""

from __future__ import annotations

import shlex
import subprocess
from dataclasses import dataclass
from pathlib import Path

FIRERED_REPO_URL = "https://github.com/FireRedTeam/FireRedASR2S.git"
FIRERED_REPO_REV = "7434958bfe5c6ab1900a26e173e884ba7d6a8fcc"
BOOTSTRAP_DEPENDENCIES = [
    "packaging>=24.0",
    "setuptools>=70,<81",
]
PYTHON_DEPENDENCIES = [
    "torch==2.10.0",
    "torchaudio==2.10.0",
    "transformers==4.51.3",
    "numpy==1.26.1",
    "cn2an==0.5.23",
    "kaldiio==2.18.0",
    "kaldi_native_fbank>=1.18,<2",
    "sentencepiece==0.2.1",
    "soundfile==0.13.1",
    "textgrid==1.6.1",
    "peft>=0.13.2",
    "huggingface_hub[cli]>=0.23",
]
MODEL_REPOS = {
    "FireRedASR2-AED": "FireRedTeam/FireRedASR2-AED",
    "FireRedVAD": "FireRedTeam/FireRedVAD",
    "FireRedLID": "FireRedTeam/FireRedLID",
    "FireRedPunc": "FireRedTeam/FireRedPunc",
}
FIRERED_DEVICE_CHOICES = ("auto", "cuda", "mps")
FIRERED_DEFAULT_DEVICE = "auto"


@dataclass(frozen=True)
class CommandResult:
    command: list[str]
    returncode: int
    stdout: str
    stderr: str


def run_command(
    command: list[str] | tuple[str, ...],
    *,
    cwd: Path | None = None,
    env: dict[str, str] | None = None,
    dry_run: bool = False,
) -> CommandResult:
    """Run a setup command and return captured output."""

    command_list = [str(part) for part in command]
    if dry_run:
        return CommandResult(command_list, 0, "", "")
    result = subprocess.run(
        command_list,
        cwd=cwd,
        env=dict(env) if env is not None else None,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise RuntimeError(
            "command failed: "
            f"{shlex.join(command_list)}\n"
            f"stdout={result.stdout.strip()}\n"
            f"stderr={result.stderr.strip()}"
        )
    return CommandResult(
        command_list,
        result.returncode,
        result.stdout,
        result.stderr,
    )


def resolve_repo_root(start: Path) -> Path:
    """Find the nearest git repository root."""

    for candidate in [start, *start.parents]:
        if (candidate / ".git").exists():
            return candidate
    return start


def default_cache_root(env: dict[str, str], start: Path) -> Path:
    """Return the cache root used for tool checkouts and venvs."""

    return Path(env.get("PRJ_CACHE_HOME") or resolve_repo_root(start) / ".cache")


def default_data_root(env: dict[str, str], start: Path) -> Path:
    """Return the data root used for persistent local model weights."""

    return Path(env.get("PRJ_DATA_HOME") or resolve_repo_root(start) / ".data")


def venv_python(venv_dir: Path) -> Path:
    """Return the Python executable inside a virtual environment."""

    return venv_dir / "bin" / "python"


def venv_cli(venv_dir: Path) -> Path:
    """Return the FireRedASR2S CLI path inside a virtual environment."""

    return venv_dir / "bin" / "fireredasr2s-cli"
