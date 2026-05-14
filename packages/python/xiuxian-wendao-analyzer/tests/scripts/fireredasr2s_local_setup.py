"""Provision a local FireRedASR2S diagnostic backend in an isolated venv."""

from __future__ import annotations

import argparse
import json
import os
import shlex
import subprocess
import sys
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from datetime import UTC, datetime
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


@dataclass(frozen=True)
class CommandResult:
    command: list[str]
    returncode: int
    stdout: str
    stderr: str


def run_command(
    command: Sequence[str],
    *,
    cwd: Path | None = None,
    env: Mapping[str, str] | None = None,
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


def default_cache_root(env: Mapping[str, str], start: Path) -> Path:
    """Return the cache root used for tool checkouts and venvs."""

    return Path(env.get("PRJ_CACHE_HOME") or resolve_repo_root(start) / ".cache")


def default_data_root(env: Mapping[str, str], start: Path) -> Path:
    """Return the data root used for persistent local model weights."""

    return Path(env.get("PRJ_DATA_HOME") or resolve_repo_root(start) / ".data")


def venv_python(venv_dir: Path) -> Path:
    """Return the Python executable inside a virtual environment."""

    return venv_dir / "bin" / "python"


def venv_cli(venv_dir: Path) -> Path:
    """Return the FireRedASR2S CLI path inside a virtual environment."""

    return venv_dir / "bin" / "fireredasr2s-cli"


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


def build_firered_command(*, venv_dir: Path, model_root: Path) -> str:
    """Build the diagnostic command consumed by audio_asr_diagnostic.py."""

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
        "0",
        "--vad_use_gpu",
        "0",
        "--lid_use_gpu",
        "0",
        "--punc_use_gpu",
        "0",
        "--write_textgrid",
        "0",
        "--write_srt",
        "0",
    ]
    return shlex.join(parts)


def write_summary(path: Path, summary: Mapping[str, object]) -> None:
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

    command = build_firered_command(venv_dir=venv_dir, model_root=model_root)
    summary = {
        "createdAt": datetime.now(tz=UTC).isoformat(),
        "repoUrl": args.repo_url,
        "repoRev": args.repo_rev,
        "repoDir": str(repo_dir),
        "venvDir": str(venv_dir),
        "modelRoot": str(model_root),
        "downloadModels": args.download_models,
        "dryRun": args.dry_run,
        "fireRedAsr2sCommand": command,
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
    parser.add_argument("--download-models", action="store_true")
    parser.add_argument("--skip-repo", action="store_true")
    parser.add_argument("--skip-deps", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--summary-json", type=Path, default=None)
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    """Run the setup helper."""

    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        summary = run_setup(args)
    except Exception as exc:  # noqa: BLE001 - diagnostic helper reports concise errors.
        print(f"FireRedASR2S setup failed: {exc}", file=sys.stderr)
        return 1
    print(json.dumps(summary, ensure_ascii=False, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
