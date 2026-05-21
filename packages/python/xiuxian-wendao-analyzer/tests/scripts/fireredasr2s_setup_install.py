"""Install and download helpers for FireRedASR2S local diagnostics."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path

try:
    from .fireredasr2s_setup_config import (
        BOOTSTRAP_DEPENDENCIES,
        MODEL_REPOS,
        PYTHON_DEPENDENCIES,
        run_command,
        venv_python,
    )
except ImportError:
    from fireredasr2s_setup_config import (
        BOOTSTRAP_DEPENDENCIES,
        MODEL_REPOS,
        PYTHON_DEPENDENCIES,
        run_command,
        venv_python,
    )


def clone_or_update_repo(
    *,
    repo_dir: Path,
    repo_url: str,
    rev: str,
    dry_run: bool,
) -> None:
    """Clone or update the FireRedASR2S source checkout."""

    if (repo_dir / ".git").exists():
        run_command(
            ["git", "fetch", "--depth", "1", "origin", rev],
            cwd=repo_dir,
            dry_run=dry_run,
        )
        run_command(["git", "checkout", "--detach", rev], cwd=repo_dir, dry_run=dry_run)
        return
    repo_dir.parent.mkdir(parents=True, exist_ok=True)
    run_command(
        ["git", "clone", "--depth", "1", repo_url, str(repo_dir)], dry_run=dry_run
    )
    run_command(
        ["git", "fetch", "--depth", "1", "origin", rev], cwd=repo_dir, dry_run=dry_run
    )
    run_command(["git", "checkout", "--detach", rev], cwd=repo_dir, dry_run=dry_run)


def create_venv(*, venv_dir: Path, python: str, dry_run: bool) -> None:
    """Create the isolated FireRedASR2S virtual environment."""

    if venv_python(venv_dir).exists():
        return
    venv_dir.parent.mkdir(parents=True, exist_ok=True)
    run_command([python, "-m", "venv", str(venv_dir)], dry_run=dry_run)


def install_dependencies(*, venv_dir: Path, repo_dir: Path, dry_run: bool) -> None:
    """Install macOS-friendly FireRedASR2S dependencies into the venv."""

    python = venv_python(venv_dir)
    run_command([python, "-m", "pip", "install", "--upgrade", "pip"], dry_run=dry_run)
    run_command(
        [python, "-m", "pip", "install", *BOOTSTRAP_DEPENDENCIES], dry_run=dry_run
    )
    run_command([python, "-m", "pip", "install", *PYTHON_DEPENDENCIES], dry_run=dry_run)
    run_command(
        [python, "-m", "pip", "install", "-e", str(repo_dir), "--no-deps"],
        dry_run=dry_run,
    )


def download_models(*, venv_dir: Path, model_root: Path, dry_run: bool) -> None:
    """Download the FireRedASR2S AED, VAD, LID, and punctuation weights."""

    huggingface_cli = venv_dir / "bin" / "huggingface-cli"
    for model_name, repo_id in MODEL_REPOS.items():
        target_dir = model_root / model_name
        if target_dir.exists() and any(target_dir.iterdir()):
            continue
        target_dir.mkdir(parents=True, exist_ok=True)
        run_command(
            [
                str(huggingface_cli),
                "download",
                repo_id,
                "--local-dir",
                str(target_dir),
            ],
            dry_run=dry_run,
        )
