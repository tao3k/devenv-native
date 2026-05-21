"""CLI runner helpers for the FireRedASR2S local setup script."""

from __future__ import annotations

import argparse
import json
import os
from datetime import UTC, datetime
from pathlib import Path

try:
    from .fireredasr2s_setup_config import (
        FIRERED_DEFAULT_DEVICE,
        FIRERED_DEVICE_CHOICES,
        FIRERED_REPO_REV,
        FIRERED_REPO_URL,
        default_cache_root,
        default_data_root,
    )
    from .fireredasr2s_setup_device import build_firered_command, resolve_firered_device
    from .fireredasr2s_setup_install import (
        clone_or_update_repo,
        create_venv,
        download_models,
        install_dependencies,
    )
except ImportError:
    from fireredasr2s_setup_config import (
        FIRERED_DEFAULT_DEVICE,
        FIRERED_DEVICE_CHOICES,
        FIRERED_REPO_REV,
        FIRERED_REPO_URL,
        default_cache_root,
        default_data_root,
    )
    from fireredasr2s_setup_device import build_firered_command, resolve_firered_device
    from fireredasr2s_setup_install import (
        clone_or_update_repo,
        create_venv,
        download_models,
        install_dependencies,
    )


def write_summary(path: Path, summary: dict[str, object]) -> None:
    """Write setup evidence for repeatable diagnostics."""

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(summary, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def run_setup(args: argparse.Namespace) -> dict[str, object]:
    """Provision FireRedASR2S and return the diagnostic command summary."""

    start = Path.cwd()
    cache_root = default_cache_root(os.environ, start)
    data_root = default_data_root(os.environ, start)
    repo_dir = args.repo_dir or cache_root / "agent" / "tools" / "FireRedASR2S"
    venv_dir = args.venv_dir or cache_root / "agent" / "tools" / "fireredasr2s-venv"
    model_root = args.model_root or data_root / "models" / "fireredasr2s"

    if not args.skip_repo:
        clone_or_update_repo(
            repo_dir=repo_dir,
            repo_url=args.repo_url,
            rev=args.repo_rev,
            dry_run=args.dry_run,
        )
    if not args.skip_deps:
        create_venv(venv_dir=venv_dir, python=args.python, dry_run=args.dry_run)
        install_dependencies(venv_dir=venv_dir, repo_dir=repo_dir, dry_run=args.dry_run)
    if args.download_models:
        download_models(venv_dir=venv_dir, model_root=model_root, dry_run=args.dry_run)

    requested_device = args.device or os.environ.get(
        "WENDAO_AUDIO_LOCAL_DEVICE",
        FIRERED_DEFAULT_DEVICE,
    )
    resolved_device, device_note, device_probe = resolve_firered_device(
        requested_device,
        venv_dir=venv_dir,
        dry_run=args.dry_run,
    )
    summary = {
        "createdAt": datetime.now(tz=UTC).isoformat(),
        "repoUrl": args.repo_url,
        "repoRev": args.repo_rev,
        "repoDir": str(repo_dir),
        "venvDir": str(venv_dir),
        "modelRoot": str(model_root),
        "downloadModels": args.download_models,
        "dryRun": args.dry_run,
        "requestedDevice": requested_device,
        "resolvedDevice": resolved_device,
        "deviceProbe": device_probe,
        "deviceNote": device_note,
        "fireRedAsr2sCommand": build_firered_command(
            venv_dir=venv_dir,
            model_root=model_root,
            resolved_device=resolved_device,
        ),
    }
    if args.summary_json is not None:
        write_summary(args.summary_json, summary)
    return summary


def build_parser() -> argparse.ArgumentParser:
    """Build the FireRedASR2S setup parser."""

    parser = argparse.ArgumentParser(
        description="Provision local FireRedASR2S for analyzer ASR diagnostics."
    )
    parser.add_argument("--repo-url", default=FIRERED_REPO_URL)
    parser.add_argument("--repo-rev", default=FIRERED_REPO_REV)
    parser.add_argument("--repo-dir", type=Path, default=None)
    parser.add_argument("--venv-dir", type=Path, default=None)
    parser.add_argument("--model-root", type=Path, default=None)
    parser.add_argument("--python", default="python3.11")
    parser.add_argument(
        "--device",
        choices=FIRERED_DEVICE_CHOICES,
        default=FIRERED_DEFAULT_DEVICE,
    )
    parser.add_argument("--download-models", action="store_true")
    parser.add_argument("--skip-repo", action="store_true")
    parser.add_argument("--skip-deps", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--summary-json", type=Path, default=None)
    return parser
