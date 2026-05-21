"""Audio diagnostic backend dispatch helpers."""

from __future__ import annotations

import time
from typing import TYPE_CHECKING

from xiuxian_wendao_analyzer.audio_diagnostic_backend_support import (
    absolute_segments,
    build_asr_result,
    coerce_transcription,
)
from xiuxian_wendao_analyzer.audio_diagnostic_docling import transcribe_local_docling
from xiuxian_wendao_analyzer.audio_diagnostic_firered import (
    transcribe_fireredasr2s,
)
from xiuxian_wendao_analyzer.audio_diagnostic_identity import safe_file_stem
from xiuxian_wendao_analyzer.audio_diagnostic_openrouter import (
    transcribe_openrouter,
)
from xiuxian_wendao_analyzer.audio_diagnostic_reporting import write_jsonl, write_text
from xiuxian_wendao_analyzer.audio_diagnostic_results import (
    OPENAI_COMPATIBLE_AUDIO_BACKENDS,
    AsrResult,
    audio_result_cache_key,
    backend_config_hash,
    backend_model_label,
    read_result_cache,
    write_result_cache,
)

if TYPE_CHECKING:
    from pathlib import Path

    from xiuxian_wendao_analyzer.audio_diagnostic_materialization import AudioChunk


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
    segments: list[dict[str, object]] = []
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
    segments_path = (
        output_dir
        / "segments"
        / backend
        / f"{source_stem}__chunk_{chunk.index:04d}.jsonl"
    )
    try:
        cached = (
            read_result_cache(result_cache_dir, result_cache_key)
            if result_cache_dir is not None
            else None
        )
        if cached is not None:
            transcript, model, cached_segments = cached
            segments = absolute_segments(chunk, cached_segments)
            write_text(transcript_path, transcript)
            if segments:
                write_jsonl(segments_path, segments)
            wall_seconds = time.perf_counter() - started
            return build_asr_result(
                backend=backend,
                chunk=chunk,
                model=model,
                status=status,
                wall_seconds=wall_seconds,
                transcript=transcript,
                transcript_path=transcript_path,
                error=error,
                result_cache_key=result_cache_key,
                segments_path=segments_path,
                segments=segments,
            )
        if backend == "local-docling":
            transcript = transcribe_local_docling(
                chunk,
                output_dir / "local-docling" / source_stem / f"chunk_{chunk.index:04d}",
                asr_model=local_asr_model,
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
        elif backend in OPENAI_COMPATIBLE_AUDIO_BACKENDS:
            if backend == "openrouter-chat-audio" and not openrouter_api_key:
                raise RuntimeError("OPENROUTER_API_KEY is required for OpenRouter ASR")
            raw_response_path = (
                output_dir
                / "raw"
                / backend
                / f"{source_stem}__chunk_{chunk.index:04d}.json"
            )
            transcript, raw_segments = coerce_transcription(
                transcribe_openrouter(
                    chunk,
                    api_key=openrouter_api_key or "EMPTY",
                    model=openrouter_model,
                    prompt=prompt,
                    base_url=openrouter_base_url,
                    max_tokens=max_tokens,
                    temperature=temperature,
                    timeout_seconds=timeout_seconds,
                    raw_response_path=raw_response_path,
                )
            )
            segments = absolute_segments(chunk, raw_segments)
        else:
            raise ValueError(f"unsupported backend: {backend}")
        if not transcript.strip():
            raise RuntimeError("ASR backend returned empty transcript")
        write_text(transcript_path, transcript)
        if segments:
            write_jsonl(segments_path, segments)
        if result_cache_dir is not None:
            write_result_cache(
                result_cache_dir,
                result_cache_key=result_cache_key,
                backend=backend,
                model=model,
                transcript=transcript,
                segments=(
                    raw_segments if backend in OPENAI_COMPATIBLE_AUDIO_BACKENDS else []
                ),
            )
    except Exception as exc:
        status = "error"
        error = str(exc)
    wall_seconds = time.perf_counter() - started
    return build_asr_result(
        backend=backend,
        chunk=chunk,
        model=model,
        status=status,
        wall_seconds=wall_seconds,
        transcript=transcript,
        transcript_path=transcript_path,
        error=error,
        result_cache_key=result_cache_key,
        segments_path=segments_path,
        segments=segments,
    )
