"""FireRedASR2S audio diagnostic backend helpers."""

from __future__ import annotations

import json
import shlex
import subprocess
from collections.abc import Mapping
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path

    from xiuxian_wendao_analyzer.audio_diagnostic_materialization import AudioChunk


def fireredasr2s_command_parts(command: str) -> list[str]:
    """Split a FireRedASR2S command string for subprocess execution."""

    parts = shlex.split(command)
    if not parts:
        raise ValueError("FireRedASR2S command cannot be empty")
    return parts


def extract_fireredasr2s_text(row: Mapping[str, object]) -> str:
    """Extract text from one FireRedASR2S JSONL row."""

    text = row.get("text")
    if isinstance(text, str) and text.strip():
        return text.strip()
    sentences = row.get("sentences")
    if isinstance(sentences, list):
        parts = [
            sentence.get("text", "").strip()
            for sentence in sentences
            if isinstance(sentence, Mapping)
            and isinstance(sentence.get("text"), str)
            and sentence.get("text", "").strip()
        ]
        return "".join(parts).strip()
    return ""


def transcribe_fireredasr2s(
    chunk: AudioChunk, output_dir: Path, *, command: str
) -> str:
    """Run FireRedASR2S CLI for one already-normalized chunk."""

    output_dir.mkdir(parents=True, exist_ok=True)
    result_path = output_dir / "result.jsonl"
    if result_path.exists():
        result_path.unlink()
    command_parts = fireredasr2s_command_parts(command)
    result = subprocess.run(
        [
            *command_parts,
            "--wav_paths",
            str(chunk.path),
            "--outdir",
            str(output_dir),
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise RuntimeError(
            "FireRedASR2S command failed: "
            f"stdout={result.stdout.strip()} stderr={result.stderr.strip()}"
        )
    candidates = [result_path, *sorted(output_dir.glob("*.jsonl"))]
    for candidate in candidates:
        if not candidate.exists():
            continue
        for raw_line in candidate.read_text(encoding="utf-8").splitlines():
            if not raw_line.strip():
                continue
            parsed = json.loads(raw_line)
            if isinstance(parsed, Mapping):
                text = extract_fireredasr2s_text(parsed)
                if text:
                    return text
    return ""
