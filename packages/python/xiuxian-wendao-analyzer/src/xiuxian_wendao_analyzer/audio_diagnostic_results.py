"""Audio diagnostic result identity, cache, and summary helpers."""

from __future__ import annotations

import hashlib
import json
import math
from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Sequence
    from pathlib import Path

OPENAI_COMPATIBLE_AUDIO_BACKENDS = {"openrouter-audio", "local-openai-audio"}


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
    segments_path: str = ""
    segment_count: int = 0


def stable_json_hash(value: object) -> str:
    """Return a stable SHA-256 for JSON-serializable identity data."""

    payload = json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
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

    if backend in OPENAI_COMPATIBLE_AUDIO_BACKENDS:
        return openrouter_model
    if backend == "local-docling":
        return f"docling-asr:{local_asr_model}:{local_language}"
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

    if backend in OPENAI_COMPATIBLE_AUDIO_BACKENDS:
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


def read_result_cache(
    cache_dir: Path, result_cache_key: str
) -> tuple[str, str, list[dict[str, object]]] | None:
    """Return cached transcript and model label when available."""

    path = result_cache_path(cache_dir, result_cache_key)
    if not path.exists():
        return None
    row = json.loads(path.read_text(encoding="utf-8"))
    transcript = row.get("transcript")
    model = row.get("model")
    raw_segments = row.get("segments")
    segments = raw_segments if isinstance(raw_segments, list) else []
    if isinstance(transcript, str) and transcript.strip() and isinstance(model, str):
        return (
            transcript,
            model,
            [segment for segment in segments if isinstance(segment, dict)],
        )
    return None


def write_json(path: Path, value: object) -> None:
    """Write stable JSON, creating parent directories."""

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def write_result_cache(
    cache_dir: Path,
    *,
    result_cache_key: str,
    backend: str,
    model: str,
    transcript: str,
    segments: list[dict[str, object]] | None = None,
) -> None:
    """Persist a successful downstream audio result for reuse."""

    write_json(
        result_cache_path(cache_dir, result_cache_key),
        {
            "backend": backend,
            "model": model,
            "resultCacheKey": result_cache_key,
            "segments": [] if segments is None else segments,
            "transcript": transcript,
        },
    )


def summarize_results(results: Sequence[AsrResult]) -> dict[str, object]:
    """Build a compact diagnostic summary."""

    by_backend: dict[str, dict[str, object]] = {}
    backend_latencies: dict[str, list[float]] = {}
    for result in results:
        item = by_backend.setdefault(
            result.backend,
            {
                "chunks": 0,
                "errors": 0,
                "requestCumulativeSeconds": 0.0,
                "audioSeconds": 0.0,
                "transcriptChars": 0,
                "segmentCount": 0,
            },
        )
        item["chunks"] = int(item["chunks"]) + 1
        item["errors"] = int(item["errors"]) + (1 if result.status != "ok" else 0)
        item["requestCumulativeSeconds"] = (
            float(item["requestCumulativeSeconds"]) + result.wall_seconds
        )
        item["audioSeconds"] = float(item["audioSeconds"]) + result.duration_seconds
        item["transcriptChars"] = int(item["transcriptChars"]) + result.transcript_chars
        item["segmentCount"] = int(item["segmentCount"]) + result.segment_count
        backend_latencies.setdefault(result.backend, []).append(result.wall_seconds)
    for item in by_backend.values():
        audio_seconds = float(item["audioSeconds"])
        item["requestCumulativeRealTimeFactor"] = (
            float(item["requestCumulativeSeconds"]) / audio_seconds if audio_seconds else None
        )
    for backend, latencies in backend_latencies.items():
        item = by_backend[backend]
        item["latencyP50Seconds"] = percentile(latencies, 0.5)
        item["latencyP95Seconds"] = percentile(latencies, 0.95)
    return {
        "resultCount": len(results),
        "errorCount": sum(1 for result in results if result.status != "ok"),
        "byBackend": by_backend,
    }


def percentile(values: Sequence[float], quantile: float) -> float | None:
    """Return a nearest-rank percentile for compact diagnostics."""

    if not values:
        return None
    ordered = sorted(values)
    index = max(0, min(len(ordered) - 1, math.ceil(len(ordered) * quantile) - 1))
    return ordered[index]
