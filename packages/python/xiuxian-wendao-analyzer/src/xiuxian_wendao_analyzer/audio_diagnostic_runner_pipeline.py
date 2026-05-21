"""Audio diagnostic execution pipeline helpers."""

from __future__ import annotations

from concurrent.futures import ThreadPoolExecutor, as_completed
from typing import TYPE_CHECKING

from xiuxian_wendao_analyzer.audio_diagnostic_backends import run_backend
from xiuxian_wendao_analyzer.audio_diagnostic_explicit_windows import (
    load_explicit_windows,
)
from xiuxian_wendao_analyzer.audio_diagnostic_identity import (
    AUDIO_MATERIALIZATION_NATIVE_RATE_WAV,
    AUDIO_MATERIALIZATION_SOURCE_DIRECT,
    AudioChunk,
    safe_file_stem,
)
from xiuxian_wendao_analyzer.audio_diagnostic_materialization import (
    materialize_audio_chunks,
)
from xiuxian_wendao_analyzer.audio_diagnostic_media_probe import (
    audio_duration_seconds,
    audio_stream_info,
)
from xiuxian_wendao_analyzer.audio_diagnostic_results import (
    OPENAI_COMPATIBLE_AUDIO_BACKENDS,
    AsrResult,
)
from xiuxian_wendao_analyzer.audio_diagnostic_windows import load_speech_segments

if TYPE_CHECKING:
    import argparse
    from pathlib import Path


def selected_audio_backends(backend: str) -> list[str]:
    """Expand CLI backend presets to concrete backend ids."""

    if backend == "both":
        return ["local-docling", "openrouter-chat-audio"]
    if backend == "all":
        return [
            "local-docling",
            "local-fireredasr2s",
            "local-openai-audio",
            "openrouter-chat-audio",
        ]
    if backend == "firered-openrouter":
        return ["local-fireredasr2s", "openrouter-chat-audio"]
    return [backend]


def backend_flags(backends: list[str]) -> tuple[bool, bool]:
    """Return hosted and OpenAI-compatible backend flags."""

    hosted_audio_enabled = "openrouter-chat-audio" in backends
    openai_compatible_audio_enabled = bool(
        OPENAI_COMPATIBLE_AUDIO_BACKENDS.intersection(backends)
    )
    return hosted_audio_enabled, openai_compatible_audio_enabled


def materialize_diagnostic_sources(
    args: argparse.Namespace,
    *,
    sources: list[Path],
    output_dir: Path,
) -> tuple[list[AudioChunk], int, int]:
    """Materialize audio chunks for all diagnostic sources."""

    manifest_chunks: list[AudioChunk] = []
    speech_segment_row_count = 0
    explicit_window_row_count = 0
    for source in sources:
        duration = None
        source_sample_rate_hz = None
        source_channels = None
        if args.sample_strategy in {
            "uniform",
            "full-coverage",
            "speech-segments",
            "explicit-windows",
        } or (args.audio_materialization_mode == AUDIO_MATERIALIZATION_SOURCE_DIRECT):
            duration = audio_duration_seconds(source)
        if args.audio_materialization_mode in {
            AUDIO_MATERIALIZATION_NATIVE_RATE_WAV,
            AUDIO_MATERIALIZATION_SOURCE_DIRECT,
        }:
            source_sample_rate_hz, source_channels = audio_stream_info(source)
        speech_segments_jsonl = getattr(args, "speech_segments_jsonl", None)
        speech_segments = load_speech_segments(speech_segments_jsonl, source=source)
        speech_segment_row_count += len(speech_segments)
        explicit_windows_json = getattr(args, "explicit_windows_json", None)
        explicit_windows = load_explicit_windows(explicit_windows_json, source=source)
        explicit_window_row_count += len(explicit_windows)
        manifest_chunks.extend(
            materialize_audio_chunks(
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
                speech_segments=speech_segments,
                explicit_windows=explicit_windows,
                speech_segment_merge_gap_seconds=(
                    getattr(args, "speech_segment_merge_gap_seconds", 0.0)
                ),
                speech_segment_min_window_seconds=(
                    getattr(args, "speech_segment_min_window_seconds", 0.0)
                ),
                speech_segment_short_merge_gap_seconds=getattr(
                    args, "speech_segment_short_merge_gap_seconds", None
                ),
                speech_segment_max_window_seconds=(
                    getattr(args, "speech_segment_max_window_seconds", None)
                ),
                audio_materialization_mode=args.audio_materialization_mode,
                source_sample_rate_hz=source_sample_rate_hz,
                source_channels=source_channels,
                force=args.force,
            )
        )
    return manifest_chunks, speech_segment_row_count, explicit_window_row_count


def run_diagnostic_backends(
    args: argparse.Namespace,
    *,
    chunks: list[AudioChunk],
    backends: list[str],
    output_dir: Path,
    api_key: str | None,
    prompt: str,
    result_cache_dir: Path | None,
) -> list[AsrResult]:
    """Run all selected backends over all materialized chunks."""

    tasks = [(chunk, backend) for chunk in chunks for backend in backends]
    results: list[AsrResult | None] = [None] * len(tasks)
    hosted_request_concurrency = max(
        1, int(getattr(args, "hosted_request_concurrency", 1))
    )
    if hosted_request_concurrency > 1 and all(
        backend in OPENAI_COMPATIBLE_AUDIO_BACKENDS for _chunk, backend in tasks
    ):
        with ThreadPoolExecutor(max_workers=hosted_request_concurrency) as executor:
            futures = {
                executor.submit(
                    _run_backend_for_chunk,
                    args,
                    chunk,
                    backend,
                    output_dir=output_dir,
                    api_key=api_key,
                    prompt=prompt,
                    result_cache_dir=result_cache_dir,
                ): index
                for index, (chunk, backend) in enumerate(tasks)
            }
            for future in as_completed(futures):
                results[futures[future]] = future.result()
        return [result for result in results if result is not None]

    ordered_results: list[AsrResult] = []
    for chunk in chunks:
        for backend in backends:
            ordered_results.append(
                _run_backend_for_chunk(
                    args,
                    chunk,
                    backend,
                    output_dir=output_dir,
                    api_key=api_key,
                    prompt=prompt,
                    result_cache_dir=result_cache_dir,
                )
            )
    return ordered_results


def _run_backend_for_chunk(
    args: argparse.Namespace,
    chunk: AudioChunk,
    backend: str,
    *,
    output_dir: Path,
    api_key: str | None,
    prompt: str,
    result_cache_dir: Path | None,
) -> AsrResult:
    return run_backend(
        backend,
        chunk,
        output_dir=output_dir,
        openrouter_api_key=api_key,
        openrouter_model=args.openrouter_model,
        openrouter_base_url=args.openrouter_base_url,
        local_asr_model=args.local_asr_model,
        local_language=args.local_language,
        fireredasr2s_command=args.fireredasr2s_command,
        prompt=prompt,
        max_tokens=args.max_tokens,
        temperature=args.temperature,
        timeout_seconds=args.timeout_seconds,
        result_cache_dir=result_cache_dir,
    )
