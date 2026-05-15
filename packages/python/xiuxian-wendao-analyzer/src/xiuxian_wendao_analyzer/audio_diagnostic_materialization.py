"""Audio diagnostic materialization helpers."""

from __future__ import annotations

import subprocess
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Sequence
    from pathlib import Path

from xiuxian_wendao_analyzer.audio_diagnostic_identity import (
    AUDIO_MATERIALIZATION_NATIVE_RATE_WAV,
    AUDIO_MATERIALIZATION_NORMALIZED_16K_WAV,
    AUDIO_MATERIALIZATION_SOURCE_DIRECT,
    DEFAULT_AUDIO_SHARD_PROFILE,
    SUPPORTED_AUDIO_MATERIALIZATION_MODES,
    AudioChunk,
    SpeechSegment,
    build_audio_shard_manifest_item,
    safe_file_stem,
    sha256_file,
)
from xiuxian_wendao_analyzer.audio_diagnostic_media_probe import (
    audio_duration_seconds,
    resolve_ffmpeg_executable,
)
from xiuxian_wendao_analyzer.audio_diagnostic_windows import (
    chunk_windows,
    media_window_for_chunk,
)


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
    speech_segments: Sequence[SpeechSegment] | None = None,
    speech_segment_merge_gap_seconds: float = 0.0,
    speech_segment_min_window_seconds: float = 0.0,
    speech_segment_short_merge_gap_seconds: float | None = None,
    speech_segment_max_window_seconds: float | None = None,
    audio_materialization_mode: str = AUDIO_MATERIALIZATION_NORMALIZED_16K_WAV,
    source_sample_rate_hz: int | None = None,
    source_channels: int | None = None,
    force: bool = False,
) -> list[AudioChunk]:
    """Create bounded mono chunks for fair local/cloud ASR comparison."""

    if audio_materialization_mode not in SUPPORTED_AUDIO_MATERIALIZATION_MODES:
        raise ValueError(
            f"unsupported audio materialization mode: {audio_materialization_mode}"
        )
    if limit_chunks <= 0:
        raise ValueError("limit_chunks must be positive")
    if chunk_context_seconds < 0:
        raise ValueError("chunk_context_seconds cannot be negative")
    chunk_dir.mkdir(parents=True, exist_ok=True)
    chunks: list[AudioChunk] = []
    source_sha256 = sha256_file(source)
    if audio_materialization_mode == AUDIO_MATERIALIZATION_SOURCE_DIRECT:
        if sample_strategy != "head" or limit_chunks != 1 or start_offset_seconds != 0:
            raise ValueError(
                "source-direct materialization supports only one full-source head shard"
            )
        duration = source_duration_seconds or audio_duration_seconds(source)
        sample_rate_hz = source_sample_rate_hz or 0
        channels = source_channels or 0
        manifest_item = build_audio_shard_manifest_item(
            source,
            profile=audio_shard_profile,
            source_sha256=source_sha256,
            chunk_index=0,
            start_seconds=0.0,
            duration_seconds=duration,
            media_start_seconds=0.0,
            media_duration_seconds=duration,
            sample_rate_hz=sample_rate_hz,
            channels=channels,
            audio_format=source.suffix.lower().lstrip(".") or audio_format,
        )
        return [
            AudioChunk(
                source=source,
                path=source,
                index=0,
                start_seconds=0.0,
                duration_seconds=duration,
                format=manifest_item.audioFormat,
                shard_id=manifest_item.shardId,
                cache_key=manifest_item.cacheKey,
                source_sha256=source_sha256,
                sample_rate_hz=sample_rate_hz,
                channels=channels,
                media_start_seconds=0.0,
                media_duration_seconds=duration,
                context_before_seconds=0.0,
                context_after_seconds=0.0,
            )
        ]
    ffmpeg = ffmpeg_path or resolve_ffmpeg_executable()
    target_sample_rate = (
        source_sample_rate_hz or sample_rate
        if audio_materialization_mode == AUDIO_MATERIALIZATION_NATIVE_RATE_WAV
        else sample_rate
    )
    target_audio_format = "wav"
    windows = chunk_windows(
        duration_seconds=source_duration_seconds,
        chunk_seconds=chunk_seconds,
        limit_chunks=limit_chunks,
        sample_strategy=sample_strategy,
        start_offset_seconds=start_offset_seconds,
        speech_segments=speech_segments,
        speech_segment_merge_gap_seconds=speech_segment_merge_gap_seconds,
        speech_segment_min_window_seconds=speech_segment_min_window_seconds,
        speech_segment_short_merge_gap_seconds=speech_segment_short_merge_gap_seconds,
        speech_segment_max_window_seconds=speech_segment_max_window_seconds,
    )
    for index, (start_seconds, logical_duration_seconds) in enumerate(windows):
        (
            media_start_seconds,
            media_duration_seconds,
            context_before_seconds,
            context_after_seconds,
        ) = media_window_for_chunk(
            start_seconds=float(start_seconds),
            duration_seconds=float(logical_duration_seconds),
            context_seconds=chunk_context_seconds,
            source_duration_seconds=source_duration_seconds,
        )
        manifest_item = build_audio_shard_manifest_item(
            source,
            profile=audio_shard_profile,
            source_sha256=source_sha256,
            chunk_index=index,
            start_seconds=float(start_seconds),
            duration_seconds=float(logical_duration_seconds),
            media_start_seconds=media_start_seconds,
            media_duration_seconds=media_duration_seconds,
            sample_rate_hz=target_sample_rate,
            channels=1,
            audio_format=target_audio_format,
        )
        chunk_path = (
            chunk_dir
            / f"{safe_file_stem(source)}__chunk_{index:04d}.{target_audio_format}"
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
                str(target_sample_rate),
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
                duration_seconds=float(logical_duration_seconds),
                format=target_audio_format,
                shard_id=manifest_item.shardId,
                cache_key=manifest_item.cacheKey,
                source_sha256=source_sha256,
                sample_rate_hz=target_sample_rate,
                channels=1,
                media_start_seconds=media_start_seconds,
                media_duration_seconds=media_duration_seconds,
                context_before_seconds=context_before_seconds,
                context_after_seconds=context_after_seconds,
            )
        )
    return chunks
