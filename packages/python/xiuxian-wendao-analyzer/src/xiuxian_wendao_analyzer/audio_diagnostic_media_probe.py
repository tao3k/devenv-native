"""Audio diagnostic ffmpeg and ffprobe helpers."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
from collections.abc import Mapping
from pathlib import Path


def resolve_ffmpeg_executable(env: Mapping[str, str] = os.environ) -> str:
    """Return the configured ffmpeg executable, preferring imageio-ffmpeg."""

    if env.get("WENDAO_AUDIO_FFMPEG"):
        return env["WENDAO_AUDIO_FFMPEG"]
    try:
        import imageio_ffmpeg
    except ImportError as exc:  # pragma: no cover - exercised by integration use.
        raise RuntimeError(
            "Audio chunking requires imageio-ffmpeg. Install the analyzer "
            "`documents-audio` extra or set WENDAO_AUDIO_FFMPEG."
        ) from exc
    return str(imageio_ffmpeg.get_ffmpeg_exe())


def resolve_ffprobe_executable(env: Mapping[str, str] = os.environ) -> str:
    """Return an ffprobe executable when available."""

    if env.get("WENDAO_AUDIO_FFPROBE"):
        return env["WENDAO_AUDIO_FFPROBE"]
    found = shutil.which("ffprobe")
    if found:
        return found
    ffmpeg_path = Path(resolve_ffmpeg_executable(env))
    sibling = ffmpeg_path.with_name("ffprobe")
    if sibling.exists():
        return str(sibling)
    raise RuntimeError("ffprobe is required for duration-aware audio sampling")


def ensure_ffmpeg_on_path(bin_dir: Path) -> None:
    """Expose imageio-ffmpeg as ``ffmpeg`` for media subprocesses."""

    if shutil.which("ffmpeg"):
        return
    ffmpeg_path = Path(resolve_ffmpeg_executable())
    bin_dir.mkdir(parents=True, exist_ok=True)
    ffmpeg_link = bin_dir / "ffmpeg"
    if not ffmpeg_link.exists():
        try:
            ffmpeg_link.symlink_to(ffmpeg_path)
        except OSError:
            shutil.copy2(ffmpeg_path, ffmpeg_link)
            ffmpeg_link.chmod(0o755)
    os.environ["PATH"] = str(bin_dir) + os.pathsep + os.environ.get("PATH", "")


def audio_duration_seconds(source: Path, *, ffprobe_path: str | None = None) -> float:
    """Return source audio duration using ffprobe."""

    ffprobe = ffprobe_path or resolve_ffprobe_executable()
    result = subprocess.run(
        [
            ffprobe,
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            str(source),
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise RuntimeError(f"ffprobe failed for {source}: {result.stderr.strip()}")
    return float(result.stdout.strip())


def audio_stream_info(
    source: Path, *, ffprobe_path: str | None = None
) -> tuple[int, int]:
    """Return source sample rate and channel count for the first audio stream."""

    ffprobe = ffprobe_path or resolve_ffprobe_executable()
    result = subprocess.run(
        [
            ffprobe,
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_entries",
            "stream=sample_rate,channels",
            "-of",
            "json",
            str(source),
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise RuntimeError(f"ffprobe failed for {source}: {result.stderr.strip()}")
    parsed = json.loads(result.stdout)
    streams = parsed.get("streams")
    if not isinstance(streams, list) or not streams:
        raise RuntimeError(f"ffprobe found no audio stream for {source}")
    stream = streams[0]
    if not isinstance(stream, Mapping):
        raise RuntimeError(f"ffprobe returned malformed stream data for {source}")
    sample_rate = int(stream.get("sample_rate", 0))
    channels = int(stream.get("channels", 0))
    if sample_rate <= 0 or channels <= 0:
        raise RuntimeError(f"ffprobe returned invalid stream data for {source}")
    return sample_rate, channels
