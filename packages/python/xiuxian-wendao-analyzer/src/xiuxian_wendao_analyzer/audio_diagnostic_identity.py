"""Audio diagnostic shard identity and manifest helpers."""

from __future__ import annotations

import hashlib
from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Sequence
    from pathlib import Path

DEFAULT_AUDIO_SHARD_PROFILE = "audio-shards-v1"
AUDIO_SHARD_MANIFEST_SCHEMA = "xiuxian_wendao.audio_shards.v1"
SUPPORTED_AUDIO_SUFFIXES = {".mp3", ".wav", ".m4a", ".flac", ".aac", ".ogg"}
AUDIO_MATERIALIZATION_NORMALIZED_16K_WAV = "normalized-16k-wav"
AUDIO_MATERIALIZATION_NATIVE_RATE_WAV = "native-rate-wav"
AUDIO_MATERIALIZATION_SOURCE_DIRECT = "source-direct"
SUPPORTED_AUDIO_MATERIALIZATION_MODES = {
    AUDIO_MATERIALIZATION_NORMALIZED_16K_WAV,
    AUDIO_MATERIALIZATION_NATIVE_RATE_WAV,
    AUDIO_MATERIALIZATION_SOURCE_DIRECT,
}


@dataclass(frozen=True)
class AudioChunk:
    """One logical audio shard and its materialized media path."""

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
class SpeechSegment:
    """Model-neutral speech window emitted by a VAD or audio planner."""

    source: str
    index: int
    start_seconds: float
    duration_seconds: float
    confidence: float | None = None
    label: str = ""


@dataclass(frozen=True)
class AudioShardManifestItem:
    """Stable audio shard identity row."""

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

    start_ms = round(start_seconds * 1000)
    duration_ms = round(duration_seconds * 1000)
    media_start_ms = round(media_start_seconds * 1000)
    media_duration_ms = round(media_duration_seconds * 1000)
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


def audio_shard_manifest(
    *,
    profile: str,
    sample_strategy: str,
    audio_materialization_mode: str,
    chunks: Sequence[AudioChunk],
) -> dict[str, object]:
    """Build the model-agnostic audio shard manifest sidecar."""

    return {
        "schema": AUDIO_SHARD_MANIFEST_SCHEMA,
        "profile": profile,
        "sampleStrategy": sample_strategy,
        "audioMaterializationMode": audio_materialization_mode,
        "items": [
            {
                "shardId": chunk.shard_id,
                "sourceId": str(chunk.source),
                "sourceSha256": chunk.source_sha256,
                "chunkIndex": chunk.index,
                "startMs": round(chunk.start_seconds * 1000),
                "durationMs": round(chunk.duration_seconds * 1000),
                "mediaStartMs": round(chunk.media_start_seconds * 1000),
                "mediaDurationMs": round(chunk.media_duration_seconds * 1000),
                "contextBeforeMs": round(chunk.context_before_seconds * 1000),
                "contextAfterMs": round(chunk.context_after_seconds * 1000),
                "sampleRateHz": chunk.sample_rate_hz,
                "channels": chunk.channels,
                "audioFormat": chunk.format,
                "cacheKey": chunk.cache_key,
                "readingOrderKey": (
                    f"{chunk.index:06}.{round(chunk.start_seconds * 1000):012}"
                ),
            }
            for chunk in chunks
        ],
    }


def truth_template_rows(chunks: Sequence[AudioChunk]) -> list[dict[str, object]]:
    """Build empty reference rows for manual truth transcription."""

    return [
        {
            "source": chunk.source.name,
            "sourceId": str(chunk.source),
            "sourceSha256": chunk.source_sha256,
            "chunkIndex": chunk.index,
            "shardId": chunk.shard_id,
            "cacheKey": chunk.cache_key,
            "startSeconds": chunk.start_seconds,
            "durationSeconds": chunk.duration_seconds,
            "mediaStartSeconds": chunk.media_start_seconds,
            "mediaDurationSeconds": chunk.media_duration_seconds,
            "audioFormat": chunk.format,
            "text": "",
        }
        for chunk in chunks
    ]
