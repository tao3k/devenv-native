"""Run bounded MP3 ASR diagnostics through local Docling and OpenRouter audio."""

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import os
import shutil
import shlex
import subprocess
import sys
import time
import urllib.error
import urllib.request
from collections.abc import Iterable, Mapping, Sequence
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path

DEFAULT_PROMPT = (
    "Please transcribe this audio verbatim as Simplified Chinese. "
    "Do not summarize, translate, or complete inaudible content. "
    "Preserve English technical terms, model names, code names, and person names. "
    "Mark inaudible spans as [inaudible]. Output only the transcript text."
)
DEFAULT_OPENROUTER_URL = "https://openrouter.ai/api/v1/chat/completions"
DEFAULT_OPENROUTER_MODEL = "xiaomi/mimo-v2.5"
DEFAULT_LOCAL_ASR_MODEL = "WHISPER_TINY"
DEFAULT_LOCAL_LANGUAGE = "zh"
DEFAULT_FIREREDASR2S_COMMAND = "fireredasr2s-cli"
DEFAULT_OUTPUT_DIR = "audio_asr_diagnostic"
DEFAULT_AUDIO_SHARD_PROFILE = "audio-shards-v1"
AUDIO_SHARD_MANIFEST_SCHEMA = "xiuxian_wendao.audio_shards.v1"
SUPPORTED_AUDIO_SUFFIXES = {".mp3", ".wav", ".m4a", ".flac", ".aac", ".ogg"}
INAUDIBLE_MARKERS = ("[inaudible]", "[听不清]", "听不清")


@dataclass(frozen=True)
class AudioChunk:
    source: Path
    path: Path
    index: int
    start_seconds: float
    duration_seconds: float
    format: str
    shard_id: str = ""
    cache_key: str = ""
    source_sha256: str = ""
    sample_rate_hz: int = 0
    channels: int = 1
    media_start_seconds: float = 0.0
    media_duration_seconds: float = 0.0
    context_before_seconds: float = 0.0
    context_after_seconds: float = 0.0


@dataclass(frozen=True)
class AsrResult:
    backend: str
    source: str
    chunk: str
    chunk_index: int
    start_seconds: float
    duration_seconds: float
    model: str
    status: str
    wall_seconds: float
    transcript_chars: int
    transcript_path: str
    error: str
    shard_id: str = ""
    shard_cache_key: str = ""
    result_cache_key: str = ""


@dataclass(frozen=True)
class AudioShardManifestItem:
    shardId: str
    sourceId: str
    sourceSha256: str
    chunkIndex: int
    startMs: int
    durationMs: int
    mediaStartMs: int
    mediaDurationMs: int
    contextBeforeMs: int
    contextAfterMs: int
    sampleRateHz: int
    channels: int
    audioFormat: str
    cacheKey: str
    readingOrderKey: str


@dataclass(frozen=True)
class QualityRow:
    backend: str
    source: str
    chunk_index: int
    start_seconds: float
    status: str
    review_status: str
    model: str
    transcript_chars: int
    chinese_ratio: float | None
    inaudible_count: int
    inaudible_per_minute: float
    chars_per_minute: float
    reference_cer: float | None
    transcript_path: str
    error: str


def discover_audio_sources(source_root: Path, *, limit_files: int | None) -> list[Path]:
    """Return supported audio files below ``source_root`` in deterministic order."""

    if source_root.is_file():
        candidates = [source_root]
    else:
        candidates = [
            path
            for path in source_root.rglob("*")
            if path.is_file() and path.suffix.lower() in SUPPORTED_AUDIO_SUFFIXES
        ]
    sources = sorted(candidates, key=lambda path: str(path).lower())
    if limit_files is not None:
        return sources[:limit_files]
    return sources


def resolve_repo_root(start: Path) -> Path:
    """Find the nearest git repository root for default evidence placement."""

    for candidate in [start, *start.parents]:
        if (candidate / ".git").exists():
            return candidate
    return start


def default_output_dir(start: Path) -> Path:
    """Return a timestamped diagnostic output directory below the repo cache."""

    stamp = datetime.now(tz=UTC).strftime("%Y%m%dT%H%M%SZ")
    return (
        resolve_repo_root(start)
        / ".cache"
        / "agent"
        / "evidence"
        / DEFAULT_OUTPUT_DIR
        / stamp
    )


def read_env_file(path: Path) -> dict[str, str]:
    """Read a simple dotenv file without adding a runtime dependency."""

    if not path.exists():
        return {}
    values: dict[str, str] = {}
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        key = key.strip()
        value = normalize_env_value(value)
        if key:
            values[key] = value
    return values


def normalize_env_value(value: str) -> str:
    """Normalize simple dotenv/env values used by the diagnostic."""

    return value.strip().strip("\"'")


def resolve_openrouter_api_key(
    env: Mapping[str, str], *, env_file: Path | None
) -> str | None:
    """Return the standard OpenRouter key from env or dotenv."""

    if env.get("OPENROUTER_API_KEY"):
        return normalize_env_value(env["OPENROUTER_API_KEY"])
    if env_file is None:
        return None
    return read_env_file(env_file).get("OPENROUTER_API_KEY")


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


def safe_file_stem(path: Path) -> str:
    """Return a filesystem-stable stem for evidence files."""

    raw = path.stem.lower()
    chars = [char if char.isalnum() else "-" for char in raw]
    collapsed = "-".join(part for part in "".join(chars).split("-") if part)
    return collapsed or "audio"


def sha256_file(path: Path) -> str:
    """Return SHA-256 for a source file without loading it all at once."""

    hasher = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


def audio_shard_identity(
    *,
    profile: str,
    source_sha256: str,
    chunk_index: int,
    start_ms: int,
    duration_ms: int,
    media_start_ms: int,
    media_duration_ms: int,
    sample_rate_hz: int,
    channels: int,
    audio_format: str,
) -> str:
    """Return the Rust-compatible audio shard identity hash."""

    payload = (
        f"{profile}:{source_sha256}:{chunk_index}:{start_ms}:"
        f"{duration_ms}:{media_start_ms}:{media_duration_ms}:"
        f"{sample_rate_hz}:{channels}:{audio_format.lower()}"
    )
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()


def audio_shard_cache_key(profile: str, shard_id: str) -> str:
    """Return the model-agnostic normalized-audio cache key."""

    return f"{profile}:{shard_id}"


def stable_json_hash(value: object) -> str:
    """Return a stable SHA-256 for JSON-serializable identity data."""

    payload = json.dumps(
        value, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    )
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()


def audio_result_cache_key(
    *,
    shard_cache_key: str,
    task_profile: str,
    backend_id: str,
    backend_config_hash: str,
) -> str:
    """Return the Rust-compatible downstream audio result cache key."""

    payload = f"{shard_cache_key}:{task_profile}:{backend_id}:{backend_config_hash}"
    return f"{task_profile}:{backend_id}:{hashlib.sha256(payload.encode('utf-8')).hexdigest()}"


def backend_model_label(
    backend: str,
    *,
    openrouter_model: str,
    local_asr_model: str,
    local_language: str,
) -> str:
    """Return the backend model label recorded in diagnostic results."""

    if backend == "openrouter-chat-audio":
        return openrouter_model
    if backend == "local-docling":
        return f"docling-asr:{local_asr_model}:{local_language}"
    if backend == "local-whisper":
        return f"openai-whisper:{normalize_whisper_model_name(local_asr_model)}:{local_language}"
    if backend == "local-fireredasr2s":
        return "fireredasr2s-cli"
    return backend


def backend_config_hash(
    backend: str,
    *,
    openrouter_model: str,
    openrouter_base_url: str,
    local_asr_model: str,
    local_language: str,
    fireredasr2s_command: str,
    prompt: str,
    max_tokens: int,
    temperature: float,
    audio_format: str,
) -> str:
    """Return backend configuration hash for cache identity."""

    if backend == "openrouter-chat-audio":
        identity = {
            "audioFormat": audio_format,
            "baseUrl": openrouter_base_url,
            "maxTokens": max_tokens,
            "model": openrouter_model,
            "prompt": prompt,
            "temperature": temperature,
        }
    elif backend == "local-fireredasr2s":
        identity = {"command": fireredasr2s_command}
    else:
        identity = {
            "language": local_language,
            "model": local_asr_model,
        }
    return stable_json_hash(identity)


def result_cache_path(cache_dir: Path, result_cache_key: str) -> Path:
    """Return a filesystem-safe path for one result cache key."""

    key_hash = hashlib.sha256(result_cache_key.encode("utf-8")).hexdigest()
    return cache_dir / f"{key_hash}.json"


def read_result_cache(cache_dir: Path, result_cache_key: str) -> tuple[str, str] | None:
    """Return cached transcript and model label when available."""

    path = result_cache_path(cache_dir, result_cache_key)
    if not path.exists():
        return None
    row = json.loads(path.read_text(encoding="utf-8"))
    transcript = row.get("transcript")
    model = row.get("model")
    if isinstance(transcript, str) and transcript.strip() and isinstance(model, str):
        return transcript, model
    return None


def write_result_cache(
    cache_dir: Path,
    *,
    result_cache_key: str,
    backend: str,
    model: str,
    transcript: str,
) -> None:
    """Persist a successful downstream audio result for reuse."""

    write_json(
        result_cache_path(cache_dir, result_cache_key),
        {
            "backend": backend,
            "model": model,
            "resultCacheKey": result_cache_key,
            "transcript": transcript,
        },
    )


def build_audio_shard_manifest_item(
    source: Path,
    *,
    profile: str,
    source_sha256: str,
    chunk_index: int,
    start_seconds: float,
    duration_seconds: float,
    media_start_seconds: float,
    media_duration_seconds: float,
    sample_rate_hz: int,
    channels: int,
    audio_format: str,
) -> AudioShardManifestItem:
    """Build one backend-independent audio shard manifest item."""

    start_ms = int(round(start_seconds * 1000))
    duration_ms = int(round(duration_seconds * 1000))
    media_start_ms = int(round(media_start_seconds * 1000))
    media_duration_ms = int(round(media_duration_seconds * 1000))
    normalized_format = audio_format.lower()
    shard_id = audio_shard_identity(
        profile=profile,
        source_sha256=source_sha256,
        chunk_index=chunk_index,
        start_ms=start_ms,
        duration_ms=duration_ms,
        media_start_ms=media_start_ms,
        media_duration_ms=media_duration_ms,
        sample_rate_hz=sample_rate_hz,
        channels=channels,
        audio_format=normalized_format,
    )
    return AudioShardManifestItem(
        shardId=shard_id,
        sourceId=str(source),
        sourceSha256=source_sha256,
        chunkIndex=chunk_index,
        startMs=start_ms,
        durationMs=duration_ms,
        mediaStartMs=media_start_ms,
        mediaDurationMs=media_duration_ms,
        contextBeforeMs=start_ms - media_start_ms,
        contextAfterMs=max(
            0, media_start_ms + media_duration_ms - start_ms - duration_ms
        ),
        sampleRateHz=sample_rate_hz,
        channels=channels,
        audioFormat=normalized_format,
        cacheKey=audio_shard_cache_key(profile, shard_id),
        readingOrderKey=f"{chunk_index:06}.{start_ms:012}",
    )


def media_window_for_chunk(
    *,
    start_seconds: float,
    duration_seconds: float,
    context_seconds: float,
    source_duration_seconds: float | None,
) -> tuple[float, float, float, float]:
    """Return actual media window and effective before/after context."""

    media_start = max(0.0, start_seconds - context_seconds)
    logical_end = start_seconds + duration_seconds
    requested_end = logical_end + context_seconds
    media_end = (
        min(requested_end, source_duration_seconds)
        if source_duration_seconds is not None
        else requested_end
    )
    media_duration = max(0.0, media_end - media_start)
    before = max(0.0, start_seconds - media_start)
    after = max(0.0, media_end - logical_end)
    return media_start, media_duration, before, after


def ensure_ffmpeg_on_path(bin_dir: Path) -> None:
    """Expose imageio-ffmpeg as ``ffmpeg`` for Docling/Whisper subprocesses."""

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


def chunk_start_offsets(
    *,
    duration_seconds: float | None,
    chunk_seconds: int,
    limit_chunks: int,
    strategy: str,
    start_offset_seconds: float,
) -> list[float]:
    """Return deterministic start offsets for bounded audio sampling."""

    if limit_chunks <= 0:
        raise ValueError("limit_chunks must be positive")
    if chunk_seconds <= 0:
        raise ValueError("chunk_seconds must be positive")
    if strategy == "head":
        return [
            start_offset_seconds + index * chunk_seconds
            for index in range(limit_chunks)
        ]
    if strategy != "uniform":
        raise ValueError(f"unsupported sample strategy: {strategy}")
    if duration_seconds is None:
        raise ValueError("uniform sampling requires duration_seconds")
    max_start = max(start_offset_seconds, duration_seconds - chunk_seconds)
    if limit_chunks == 1:
        return [min(start_offset_seconds, max_start)]
    span = max(0.0, max_start - start_offset_seconds)
    return [
        min(max_start, start_offset_seconds + span * index / (limit_chunks - 1))
        for index in range(limit_chunks)
    ]


def materialize_audio_chunks(
    source: Path,
    *,
    chunk_dir: Path,
    chunk_seconds: int,
    limit_chunks: int,
    sample_rate: int,
    audio_format: str,
    audio_shard_profile: str = DEFAULT_AUDIO_SHARD_PROFILE,
    chunk_context_seconds: float = 0.0,
    ffmpeg_path: str | None = None,
    sample_strategy: str = "head",
    start_offset_seconds: float = 0.0,
    source_duration_seconds: float | None = None,
    force: bool = False,
) -> list[AudioChunk]:
    """Create bounded mono chunks for fair local/cloud ASR comparison."""

    if limit_chunks <= 0:
        raise ValueError("limit_chunks must be positive")
    if chunk_context_seconds < 0:
        raise ValueError("chunk_context_seconds cannot be negative")
    chunk_dir.mkdir(parents=True, exist_ok=True)
    ffmpeg = ffmpeg_path or resolve_ffmpeg_executable()
    chunks: list[AudioChunk] = []
    source_sha256 = sha256_file(source)
    offsets = chunk_start_offsets(
        duration_seconds=source_duration_seconds,
        chunk_seconds=chunk_seconds,
        limit_chunks=limit_chunks,
        strategy=sample_strategy,
        start_offset_seconds=start_offset_seconds,
    )
    for index, start_seconds in enumerate(offsets):
        (
            media_start_seconds,
            media_duration_seconds,
            context_before_seconds,
            context_after_seconds,
        ) = media_window_for_chunk(
            start_seconds=float(start_seconds),
            duration_seconds=float(chunk_seconds),
            context_seconds=chunk_context_seconds,
            source_duration_seconds=source_duration_seconds,
        )
        manifest_item = build_audio_shard_manifest_item(
            source,
            profile=audio_shard_profile,
            source_sha256=source_sha256,
            chunk_index=index,
            start_seconds=float(start_seconds),
            duration_seconds=float(chunk_seconds),
            media_start_seconds=media_start_seconds,
            media_duration_seconds=media_duration_seconds,
            sample_rate_hz=sample_rate,
            channels=1,
            audio_format=audio_format,
        )
        chunk_path = (
            chunk_dir
            / f"{safe_file_stem(source)}__chunk_{index:04d}.{audio_format.lower()}"
        )
        if force or not chunk_path.exists():
            command = [
                ffmpeg,
                "-hide_banner",
                "-nostdin",
                "-loglevel",
                "error",
                "-y",
                "-ss",
                f"{media_start_seconds:.3f}",
                "-t",
                f"{media_duration_seconds:.3f}",
                "-i",
                str(source),
                "-ac",
                "1",
                "-ar",
                str(sample_rate),
                "-vn",
                str(chunk_path),
            ]
            result = subprocess.run(
                command,
                check=False,
                capture_output=True,
                text=True,
            )
            if result.returncode != 0:
                raise RuntimeError(
                    "ffmpeg audio chunking failed for "
                    f"{source} chunk {index}: {result.stderr.strip()}"
                )
        chunks.append(
            AudioChunk(
                source=source,
                path=chunk_path,
                index=index,
                start_seconds=float(start_seconds),
                duration_seconds=float(chunk_seconds),
                format=audio_format.lower(),
                shard_id=manifest_item.shardId,
                cache_key=manifest_item.cacheKey,
                source_sha256=source_sha256,
                sample_rate_hz=sample_rate,
                channels=1,
                media_start_seconds=media_start_seconds,
                media_duration_seconds=media_duration_seconds,
                context_before_seconds=context_before_seconds,
                context_after_seconds=context_after_seconds,
            )
        )
    return chunks


def _row_value(row: object, field: str) -> object:
    if isinstance(row, Mapping):
        return row.get(field)
    return getattr(row, field, None)


def transcript_from_document_rows(rows: Iterable[object]) -> str:
    """Extract transcript-like content from Docling resource rows."""

    preferred: list[str] = []
    fallback: list[str] = []
    for row in rows:
        content = _row_value(row, "content")
        if not isinstance(content, str) or not content.strip():
            continue
        resource_type = _row_value(row, "resourceType")
        if resource_type in {"audio", "document"}:
            preferred.append(content.strip())
        else:
            fallback.append(content.strip())
    return "\n\n".join(preferred or fallback)


def build_docling_audio_converter(asr_model: str, language: str) -> object:
    """Create a Docling audio converter with explicit ASR model and language."""

    from docling.datamodel import asr_model_specs
    from docling.datamodel.base_models import InputFormat
    from docling.datamodel.pipeline_options import AsrPipelineOptions
    from docling.document_converter import AudioFormatOption, DocumentConverter
    from docling.pipeline.asr_pipeline import AsrPipeline

    if not hasattr(asr_model_specs, asr_model):
        raise ValueError(f"unknown Docling ASR model spec: {asr_model}")
    asr_options = getattr(asr_model_specs, asr_model).model_copy(deep=True)
    asr_options.language = language
    if hasattr(asr_options, "verbose"):
        asr_options.verbose = False
    pipeline_options = AsrPipelineOptions()
    pipeline_options.asr_options = asr_options
    return DocumentConverter(
        format_options={
            InputFormat.AUDIO: AudioFormatOption(
                pipeline_cls=AsrPipeline,
                pipeline_options=pipeline_options,
            )
        }
    )


def normalize_whisper_model_name(model: str) -> str:
    """Convert Docling-style Whisper constants to openai-whisper model names."""

    value = model.strip()
    if value.startswith("WHISPER_"):
        value = value.removeprefix("WHISPER_")
    value = value.lower().replace("_native", "").replace("_mlx", "")
    aliases = {
        "large": "large",
        "turbo": "turbo",
        "large_v3_turbo": "turbo",
    }
    return aliases.get(value, value)


def transcribe_local_whisper(
    chunk: AudioChunk, output_dir: Path, *, model: str, language: str
) -> str:
    """Run openai-whisper directly with explicit language control."""

    import whisper

    ensure_ffmpeg_on_path(output_dir / "_ffmpeg_bin")
    whisper_model = whisper.load_model(normalize_whisper_model_name(model))
    result = whisper_model.transcribe(
        str(chunk.path),
        language=language,
        verbose=False,
        fp16=False,
        word_timestamps=False,
    )
    text = result.get("text", "")
    return text.strip() if isinstance(text, str) else ""


def transcribe_local_docling(
    chunk: AudioChunk, output_dir: Path, *, asr_model: str, language: str
) -> str:
    """Run local Docling ASR for one materialized chunk."""

    from xiuxian_wendao_analyzer.document_extract import extract_document_resources

    ensure_ffmpeg_on_path(output_dir / "_ffmpeg_bin")
    converter = build_docling_audio_converter(asr_model, language)
    rows = extract_document_resources(chunk.path, output_dir, converter=converter)
    return transcript_from_document_rows(rows)


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


def build_openrouter_payload(
    *,
    model: str,
    prompt: str,
    audio_bytes: bytes,
    audio_format: str,
    max_tokens: int,
    temperature: float,
) -> dict[str, object]:
    """Build an OpenRouter chat/audio transcription request."""

    encoded_audio = base64.b64encode(audio_bytes).decode("ascii")
    return {
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": prompt},
                    {
                        "type": "input_audio",
                        "input_audio": {
                            "data": encoded_audio,
                            "format": audio_format,
                        },
                    },
                ],
            }
        ],
        "temperature": temperature,
        "max_tokens": max_tokens,
    }


def _message_content_to_text(content: object) -> str:
    if isinstance(content, str):
        return content
    if isinstance(content, Sequence) and not isinstance(content, (bytes, bytearray)):
        parts: list[str] = []
        for item in content:
            if isinstance(item, Mapping) and isinstance(item.get("text"), str):
                parts.append(item["text"])
        return "\n".join(parts)
    return ""


def extract_openrouter_transcript(response: Mapping[str, object]) -> str:
    """Extract assistant text from an OpenRouter chat completion response."""

    choices = response.get("choices")
    if not isinstance(choices, list) or not choices:
        return ""
    first_choice = choices[0]
    if not isinstance(first_choice, Mapping):
        return ""
    message = first_choice.get("message")
    if not isinstance(message, Mapping):
        return ""
    return _message_content_to_text(message.get("content")).strip()


def transcribe_openrouter(
    chunk: AudioChunk,
    *,
    api_key: str,
    model: str,
    prompt: str,
    base_url: str,
    max_tokens: int,
    temperature: float,
    timeout_seconds: int,
    raw_response_path: Path,
) -> str:
    """Run OpenRouter chat/audio transcription for one chunk."""

    payload = build_openrouter_payload(
        model=model,
        prompt=prompt,
        audio_bytes=chunk.path.read_bytes(),
        audio_format=chunk.format,
        max_tokens=max_tokens,
        temperature=temperature,
    )
    request = urllib.request.Request(
        base_url,
        data=json.dumps(payload).encode("utf-8"),
        headers={
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
            "HTTP-Referer": "https://github.com/tao3k/xiuxian-artisan-workshop",
            "X-Title": "Wendao audio ASR diagnostic",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=timeout_seconds) as response:
            body = response.read().decode("utf-8")
    except urllib.error.HTTPError as exc:
        error_body = exc.read().decode("utf-8", errors="replace")
        raise RuntimeError(f"OpenRouter HTTP {exc.code}: {error_body}") from exc

    raw_response_path.parent.mkdir(parents=True, exist_ok=True)
    raw_response_path.write_text(body, encoding="utf-8")
    parsed = json.loads(body)
    if not isinstance(parsed, Mapping):
        return ""
    return extract_openrouter_transcript(parsed)


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def run_backend(
    backend: str,
    chunk: AudioChunk,
    *,
    output_dir: Path,
    openrouter_api_key: str | None,
    openrouter_model: str,
    openrouter_base_url: str,
    local_asr_model: str,
    local_language: str,
    fireredasr2s_command: str,
    prompt: str,
    max_tokens: int,
    temperature: float,
    timeout_seconds: int,
    result_cache_dir: Path | None,
) -> AsrResult:
    """Run one backend for one chunk and persist its transcript."""

    started = time.perf_counter()
    transcript = ""
    error = ""
    status = "ok"
    model = backend_model_label(
        backend,
        openrouter_model=openrouter_model,
        local_asr_model=local_asr_model,
        local_language=local_language,
    )
    result_cache_key = audio_result_cache_key(
        shard_cache_key=chunk.cache_key,
        task_profile="transcription",
        backend_id=backend,
        backend_config_hash=backend_config_hash(
            backend,
            openrouter_model=openrouter_model,
            openrouter_base_url=openrouter_base_url,
            local_asr_model=local_asr_model,
            local_language=local_language,
            fireredasr2s_command=fireredasr2s_command,
            prompt=prompt,
            max_tokens=max_tokens,
            temperature=temperature,
            audio_format=chunk.format,
        ),
    )
    source_stem = safe_file_stem(chunk.source)
    transcript_path = (
        output_dir
        / "transcripts"
        / backend
        / f"{source_stem}__chunk_{chunk.index:04d}.txt"
    )
    try:
        cached = (
            read_result_cache(result_cache_dir, result_cache_key)
            if result_cache_dir is not None
            else None
        )
        if cached is not None:
            transcript, model = cached
            write_text(transcript_path, transcript)
            wall_seconds = time.perf_counter() - started
            return AsrResult(
                backend=backend,
                source=str(chunk.source),
                chunk=str(chunk.path),
                chunk_index=chunk.index,
                start_seconds=chunk.start_seconds,
                duration_seconds=chunk.duration_seconds,
                model=model,
                status=status,
                wall_seconds=wall_seconds,
                transcript_chars=len(transcript),
                transcript_path=str(transcript_path),
                error=error,
                shard_id=chunk.shard_id,
                shard_cache_key=chunk.cache_key,
                result_cache_key=result_cache_key,
            )
        if backend == "local-docling":
            transcript = transcribe_local_docling(
                chunk,
                output_dir / "local-docling" / source_stem / f"chunk_{chunk.index:04d}",
                asr_model=local_asr_model,
                language=local_language,
            )
        elif backend == "local-whisper":
            transcript = transcribe_local_whisper(
                chunk,
                output_dir / "local-whisper" / source_stem / f"chunk_{chunk.index:04d}",
                model=local_asr_model,
                language=local_language,
            )
        elif backend == "local-fireredasr2s":
            transcript = transcribe_fireredasr2s(
                chunk,
                output_dir
                / "local-fireredasr2s"
                / source_stem
                / f"chunk_{chunk.index:04d}",
                command=fireredasr2s_command,
            )
        elif backend == "openrouter-chat-audio":
            if not openrouter_api_key:
                raise RuntimeError("OPENROUTER_API_KEY is required for OpenRouter ASR")
            raw_response_path = (
                output_dir
                / "raw"
                / backend
                / f"{source_stem}__chunk_{chunk.index:04d}.json"
            )
            transcript = transcribe_openrouter(
                chunk,
                api_key=openrouter_api_key,
                model=openrouter_model,
                prompt=prompt,
                base_url=openrouter_base_url,
                max_tokens=max_tokens,
                temperature=temperature,
                timeout_seconds=timeout_seconds,
                raw_response_path=raw_response_path,
            )
        else:
            raise ValueError(f"unsupported backend: {backend}")
        if not transcript.strip():
            raise RuntimeError("ASR backend returned empty transcript")
        write_text(transcript_path, transcript)
        if result_cache_dir is not None:
            write_result_cache(
                result_cache_dir,
                result_cache_key=result_cache_key,
                backend=backend,
                model=model,
                transcript=transcript,
            )
    except Exception as exc:  # noqa: BLE001 - diagnostic must record per-chunk errors.
        status = "error"
        error = str(exc)
    wall_seconds = time.perf_counter() - started
    return AsrResult(
        backend=backend,
        source=str(chunk.source),
        chunk=str(chunk.path),
        chunk_index=chunk.index,
        start_seconds=chunk.start_seconds,
        duration_seconds=chunk.duration_seconds,
        model=model,
        status=status,
        wall_seconds=wall_seconds,
        transcript_chars=len(transcript),
        transcript_path=str(transcript_path) if transcript else "",
        error=error,
        shard_id=chunk.shard_id,
        shard_cache_key=chunk.cache_key,
        result_cache_key=result_cache_key,
    )


def summarize_results(results: Sequence[AsrResult]) -> dict[str, object]:
    """Build a compact diagnostic summary."""

    by_backend: dict[str, dict[str, object]] = {}
    for result in results:
        item = by_backend.setdefault(
            result.backend,
            {
                "chunks": 0,
                "errors": 0,
                "wallSeconds": 0.0,
                "audioSeconds": 0.0,
                "transcriptChars": 0,
            },
        )
        item["chunks"] = int(item["chunks"]) + 1
        item["errors"] = int(item["errors"]) + (1 if result.status != "ok" else 0)
        item["wallSeconds"] = float(item["wallSeconds"]) + result.wall_seconds
        item["audioSeconds"] = float(item["audioSeconds"]) + result.duration_seconds
        item["transcriptChars"] = int(item["transcriptChars"]) + result.transcript_chars
    for item in by_backend.values():
        audio_seconds = float(item["audioSeconds"])
        item["realTimeFactor"] = (
            float(item["wallSeconds"]) / audio_seconds if audio_seconds else None
        )
    return {
        "resultCount": len(results),
        "errorCount": sum(1 for result in results if result.status != "ok"),
        "byBackend": by_backend,
    }


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def audio_shard_manifest(
    *,
    profile: str,
    sample_strategy: str,
    chunks: Sequence[AudioChunk],
) -> dict[str, object]:
    """Build the model-agnostic audio shard manifest sidecar."""

    return {
        "schema": AUDIO_SHARD_MANIFEST_SCHEMA,
        "profile": profile,
        "sampleStrategy": sample_strategy,
        "items": [
            {
                "shardId": chunk.shard_id,
                "sourceId": str(chunk.source),
                "sourceSha256": chunk.source_sha256,
                "chunkIndex": chunk.index,
                "startMs": int(round(chunk.start_seconds * 1000)),
                "durationMs": int(round(chunk.duration_seconds * 1000)),
                "mediaStartMs": int(round(chunk.media_start_seconds * 1000)),
                "mediaDurationMs": int(round(chunk.media_duration_seconds * 1000)),
                "contextBeforeMs": int(round(chunk.context_before_seconds * 1000)),
                "contextAfterMs": int(round(chunk.context_after_seconds * 1000)),
                "sampleRateHz": chunk.sample_rate_hz,
                "channels": chunk.channels,
                "audioFormat": chunk.format,
                "cacheKey": chunk.cache_key,
                "readingOrderKey": (
                    f"{chunk.index:06}.{int(round(chunk.start_seconds * 1000)):012}"
                ),
            }
            for chunk in chunks
        ],
    }


def normalize_reference_text(text: str) -> str:
    """Normalize text for coarse character error rate comparison."""

    return "".join(char.lower() for char in text if not char.isspace())


def levenshtein_distance(left: str, right: str) -> int:
    """Return Levenshtein distance with two-row dynamic programming."""

    if left == right:
        return 0
    if not left:
        return len(right)
    if not right:
        return len(left)
    previous = list(range(len(right) + 1))
    for left_index, left_char in enumerate(left, start=1):
        current = [left_index]
        for right_index, right_char in enumerate(right, start=1):
            substitution = previous[right_index - 1] + (
                0 if left_char == right_char else 1
            )
            current.append(
                min(previous[right_index] + 1, current[-1] + 1, substitution)
            )
        previous = current
    return previous[-1]


def character_error_rate(candidate: str, reference: str) -> float | None:
    """Return CER against a reference transcript, or ``None`` for empty reference."""

    normalized_reference = normalize_reference_text(reference)
    if not normalized_reference:
        return None
    normalized_candidate = normalize_reference_text(candidate)
    return levenshtein_distance(normalized_candidate, normalized_reference) / len(
        normalized_reference
    )


def load_reference_transcripts(path: Path | None) -> dict[tuple[str, int], str]:
    """Load optional JSONL references keyed by source basename and chunk index."""

    if path is None:
        return {}
    references: dict[tuple[str, int], str] = {}
    for line_number, raw_line in enumerate(
        path.read_text(encoding="utf-8").splitlines(), start=1
    ):
        if not raw_line.strip():
            continue
        row = json.loads(raw_line)
        source = row.get("source")
        chunk_index = row.get("chunkIndex", row.get("chunk_index"))
        text = row.get("text")
        if (
            not isinstance(source, str)
            or not isinstance(chunk_index, int)
            or not isinstance(text, str)
        ):
            raise ValueError(f"invalid reference row at line {line_number}")
        references[(Path(source).name, chunk_index)] = text
    return references


def read_transcript(path: str) -> str:
    """Read a transcript path when present."""

    return Path(path).read_text(encoding="utf-8") if path else ""


def chinese_ratio(text: str) -> float | None:
    """Return the ratio of CJK characters among non-space characters."""

    chars = [char for char in text if not char.isspace()]
    if not chars:
        return None
    chinese = sum(1 for char in chars if "\u4e00" <= char <= "\u9fff")
    return chinese / len(chars)


def inaudible_count(text: str) -> int:
    """Count common inaudible markers in a transcript."""

    lowered = text.lower()
    return sum(lowered.count(marker.lower()) for marker in INAUDIBLE_MARKERS)


def classify_quality(
    result: AsrResult,
    *,
    transcript: str,
    reference_cer: float | None,
    min_chars_per_minute: float,
    min_chinese_ratio: float,
    max_inaudible_per_minute: float,
) -> str:
    """Classify one ASR result for precision review."""

    if result.status != "ok":
        return "failed"
    if reference_cer is not None:
        return "reference-pass" if reference_cer <= 0.15 else "reference-fail"
    duration_minutes = result.duration_seconds / 60 if result.duration_seconds else 0.0
    chars_per_minute = len(transcript) / duration_minutes if duration_minutes else 0.0
    ratio = chinese_ratio(transcript)
    markers_per_minute = (
        inaudible_count(transcript) / duration_minutes if duration_minutes else 0.0
    )
    if chars_per_minute < min_chars_per_minute:
        return "weak-too-short"
    if ratio is not None and ratio < min_chinese_ratio:
        return "weak-language-ratio"
    if markers_per_minute > max_inaudible_per_minute:
        return "weak-inaudible-heavy"
    return "review-needed"


def build_quality_rows(
    results: Sequence[AsrResult],
    *,
    references: Mapping[tuple[str, int], str],
    min_chars_per_minute: float,
    min_chinese_ratio: float,
    max_inaudible_per_minute: float,
) -> list[QualityRow]:
    """Build per-result quality rows from transcripts and optional references."""

    rows: list[QualityRow] = []
    for result in results:
        transcript = read_transcript(result.transcript_path)
        reference = references.get((Path(result.source).name, result.chunk_index))
        cer = character_error_rate(transcript, reference) if reference else None
        duration_minutes = (
            result.duration_seconds / 60 if result.duration_seconds else 0.0
        )
        chars_per_minute = (
            len(transcript) / duration_minutes if duration_minutes else 0.0
        )
        markers = inaudible_count(transcript)
        markers_per_minute = markers / duration_minutes if duration_minutes else 0.0
        rows.append(
            QualityRow(
                backend=result.backend,
                source=result.source,
                chunk_index=result.chunk_index,
                start_seconds=result.start_seconds,
                status=result.status,
                review_status=classify_quality(
                    result,
                    transcript=transcript,
                    reference_cer=cer,
                    min_chars_per_minute=min_chars_per_minute,
                    min_chinese_ratio=min_chinese_ratio,
                    max_inaudible_per_minute=max_inaudible_per_minute,
                ),
                model=result.model,
                transcript_chars=len(transcript),
                chinese_ratio=chinese_ratio(transcript),
                inaudible_count=markers,
                inaudible_per_minute=markers_per_minute,
                chars_per_minute=chars_per_minute,
                reference_cer=cer,
                transcript_path=result.transcript_path,
                error=result.error,
            )
        )
    return rows


def summarize_quality(rows: Sequence[QualityRow]) -> dict[str, object]:
    """Summarize quality rows by backend and review status."""

    by_backend: dict[str, dict[str, object]] = {}
    for row in rows:
        item = by_backend.setdefault(
            row.backend,
            {
                "rows": 0,
                "failed": 0,
                "reviewNeeded": 0,
                "weakRows": 0,
                "referencePass": 0,
                "referenceFail": 0,
                "avgCharsPerMinute": 0.0,
                "avgChineseRatio": 0.0,
                "avgInaudiblePerMinute": 0.0,
            },
        )
        item["rows"] = int(item["rows"]) + 1
        item["failed"] = int(item["failed"]) + (
            1 if row.review_status == "failed" else 0
        )
        item["reviewNeeded"] = int(item["reviewNeeded"]) + (
            1 if row.review_status == "review-needed" else 0
        )
        item["weakRows"] = int(item["weakRows"]) + (
            1 if row.review_status.startswith("weak-") else 0
        )
        item["referencePass"] = int(item["referencePass"]) + (
            1 if row.review_status == "reference-pass" else 0
        )
        item["referenceFail"] = int(item["referenceFail"]) + (
            1 if row.review_status == "reference-fail" else 0
        )
        item["avgCharsPerMinute"] = (
            float(item["avgCharsPerMinute"]) + row.chars_per_minute
        )
        item["avgChineseRatio"] = float(item["avgChineseRatio"]) + (
            row.chinese_ratio or 0.0
        )
        item["avgInaudiblePerMinute"] = (
            float(item["avgInaudiblePerMinute"]) + row.inaudible_per_minute
        )
    for item in by_backend.values():
        row_count = int(item["rows"])
        if row_count:
            item["avgCharsPerMinute"] = float(item["avgCharsPerMinute"]) / row_count
            item["avgChineseRatio"] = float(item["avgChineseRatio"]) / row_count
            item["avgInaudiblePerMinute"] = (
                float(item["avgInaudiblePerMinute"]) / row_count
            )
    return {"qualityByBackend": by_backend}


def write_quality_tsv(path: Path, rows: Sequence[QualityRow]) -> None:
    """Write a compact TSV for human precision review."""

    header = [
        "backend",
        "source",
        "chunkIndex",
        "startSeconds",
        "status",
        "reviewStatus",
        "model",
        "transcriptChars",
        "charsPerMinute",
        "chineseRatio",
        "inaudiblePerMinute",
        "referenceCer",
        "transcriptPath",
        "error",
    ]
    lines = ["\t".join(header)]
    for row in rows:
        values = [
            row.backend,
            row.source,
            str(row.chunk_index),
            f"{row.start_seconds:.3f}",
            row.status,
            row.review_status,
            row.model,
            str(row.transcript_chars),
            f"{row.chars_per_minute:.3f}",
            "" if row.chinese_ratio is None else f"{row.chinese_ratio:.6f}",
            f"{row.inaudible_per_minute:.3f}",
            "" if row.reference_cer is None else f"{row.reference_cer:.6f}",
            row.transcript_path,
            row.error.replace("\t", " ").replace("\n", " "),
        ]
        lines.append("\t".join(values))
    write_text(path, "\n".join(lines) + "\n")


def run_diagnostic(args: argparse.Namespace) -> dict[str, object]:
    """Run the bounded ASR diagnostic and write evidence files."""

    output_dir = args.output_dir or default_output_dir(Path.cwd())
    output_dir.mkdir(parents=True, exist_ok=True)
    source_root = Path(args.source_root)
    sources = discover_audio_sources(source_root, limit_files=args.limit_files)
    if not sources:
        raise RuntimeError(f"no supported audio files found under {source_root}")
    api_key = resolve_openrouter_api_key(os.environ, env_file=args.env_file)
    result_cache_dir = (
        None
        if args.no_result_cache
        else (args.result_cache_dir or output_dir / "result_cache")
    )
    backends = (
        ["local-whisper", "openrouter-chat-audio"]
        if args.backend == "both"
        else (
            [
                "local-docling",
                "local-whisper",
                "local-fireredasr2s",
                "openrouter-chat-audio",
            ]
            if args.backend == "all"
            else (
                ["local-fireredasr2s", "openrouter-chat-audio"]
                if args.backend == "firered-openrouter"
                else [args.backend]
            )
        )
    )
    results: list[AsrResult] = []
    manifest_chunks: list[AudioChunk] = []
    for source in sources:
        duration = None
        if args.sample_strategy == "uniform":
            duration = audio_duration_seconds(source)
        chunks = materialize_audio_chunks(
            source,
            chunk_dir=output_dir / "chunks" / safe_file_stem(source),
            chunk_seconds=args.chunk_seconds,
            limit_chunks=args.limit_chunks,
            sample_rate=args.sample_rate,
            audio_format=args.audio_format,
            chunk_context_seconds=args.chunk_context_seconds,
            sample_strategy=args.sample_strategy,
            start_offset_seconds=args.start_offset_seconds,
            source_duration_seconds=duration,
            force=args.force,
        )
        manifest_chunks.extend(chunks)
        for chunk in chunks:
            for backend in backends:
                results.append(
                    run_backend(
                        backend,
                        chunk,
                        output_dir=output_dir,
                        openrouter_api_key=api_key,
                        openrouter_model=args.openrouter_model,
                        openrouter_base_url=args.openrouter_base_url,
                        local_asr_model=args.local_asr_model,
                        local_language=args.local_language,
                        fireredasr2s_command=args.fireredasr2s_command,
                        prompt=args.prompt,
                        max_tokens=args.max_tokens,
                        temperature=args.temperature,
                        timeout_seconds=args.timeout_seconds,
                        result_cache_dir=result_cache_dir,
                    )
                )

    result_rows = [result.__dict__ for result in results]
    references = load_reference_transcripts(args.reference_jsonl)
    quality_rows = build_quality_rows(
        results,
        references=references,
        min_chars_per_minute=args.min_chars_per_minute,
        min_chinese_ratio=args.min_chinese_ratio,
        max_inaudible_per_minute=args.max_inaudible_per_minute,
    )
    summary = summarize_results(results)
    quality_summary = summarize_quality(quality_rows)
    report = {
        "createdAt": datetime.now(tz=UTC).isoformat(),
        "sourceRoot": str(source_root),
        "outputDir": str(output_dir),
        "sourceCount": len(sources),
        "chunkSeconds": args.chunk_seconds,
        "limitFiles": args.limit_files,
        "limitChunks": args.limit_chunks,
        "sampleStrategy": args.sample_strategy,
        "startOffsetSeconds": args.start_offset_seconds,
        "chunkContextSeconds": args.chunk_context_seconds,
        "audioShardManifestSchema": AUDIO_SHARD_MANIFEST_SCHEMA,
        "audioShardProfile": DEFAULT_AUDIO_SHARD_PROFILE,
        "resultCacheEnabled": result_cache_dir is not None,
        "resultCacheDir": "" if result_cache_dir is None else str(result_cache_dir),
        "openRouterModel": args.openrouter_model,
        "openRouterApiKeyConfigured": bool(api_key),
        "localAsrModel": args.local_asr_model,
        "localLanguage": args.local_language,
        "fireRedAsr2sCommand": args.fireredasr2s_command,
        "referenceConfigured": bool(references),
        **summary,
        **quality_summary,
    }
    write_json(output_dir / "results.json", result_rows)
    write_json(
        output_dir / "audio_shards.json",
        audio_shard_manifest(
            profile=DEFAULT_AUDIO_SHARD_PROFILE,
            sample_strategy=args.sample_strategy,
            chunks=manifest_chunks,
        ),
    )
    write_json(output_dir / "quality.json", [row.__dict__ for row in quality_rows])
    write_quality_tsv(output_dir / "review.tsv", quality_rows)
    write_json(output_dir / "summary.json", report)
    return report


def build_parser() -> argparse.ArgumentParser:
    """Build the diagnostic CLI parser."""

    parser = argparse.ArgumentParser(
        description="Run bounded MP3 ASR diagnostics for local Docling and OpenRouter."
    )
    parser.add_argument("source_root", help="Directory or audio file to diagnose.")
    parser.add_argument(
        "--backend",
        choices=[
            "local-docling",
            "local-whisper",
            "local-fireredasr2s",
            "openrouter-chat-audio",
            "both",
            "firered-openrouter",
            "all",
        ],
        default="both",
    )
    parser.add_argument("--output-dir", type=Path, default=None)
    parser.add_argument("--env-file", type=Path, default=Path(".env"))
    parser.add_argument("--chunk-seconds", type=int, default=60)
    parser.add_argument("--limit-files", type=int, default=2)
    parser.add_argument("--limit-chunks", type=int, default=1)
    parser.add_argument(
        "--sample-strategy", choices=["head", "uniform"], default="head"
    )
    parser.add_argument("--start-offset-seconds", type=float, default=0.0)
    parser.add_argument("--chunk-context-seconds", type=float, default=0.0)
    parser.add_argument("--sample-rate", type=int, default=16000)
    parser.add_argument("--audio-format", choices=["wav", "flac"], default="wav")
    parser.add_argument("--openrouter-model", default=DEFAULT_OPENROUTER_MODEL)
    parser.add_argument("--openrouter-base-url", default=DEFAULT_OPENROUTER_URL)
    parser.add_argument("--local-asr-model", default=DEFAULT_LOCAL_ASR_MODEL)
    parser.add_argument("--local-language", default=DEFAULT_LOCAL_LANGUAGE)
    parser.add_argument("--fireredasr2s-command", default=DEFAULT_FIREREDASR2S_COMMAND)
    parser.add_argument("--prompt", default=DEFAULT_PROMPT)
    parser.add_argument("--max-tokens", type=int, default=4096)
    parser.add_argument("--temperature", type=float, default=0.0)
    parser.add_argument("--timeout-seconds", type=int, default=300)
    parser.add_argument("--result-cache-dir", type=Path, default=None)
    parser.add_argument("--no-result-cache", action="store_true")
    parser.add_argument("--reference-jsonl", type=Path, default=None)
    parser.add_argument("--min-chars-per-minute", type=float, default=40.0)
    parser.add_argument("--min-chinese-ratio", type=float, default=0.35)
    parser.add_argument("--max-inaudible-per-minute", type=float, default=30.0)
    parser.add_argument("--force", action="store_true")
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    """Run the diagnostic command."""

    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        report = run_diagnostic(args)
    except Exception as exc:  # noqa: BLE001 - command should print a concise failure.
        print(f"audio ASR diagnostic failed: {exc}", file=sys.stderr)
        return 1
    print(json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
