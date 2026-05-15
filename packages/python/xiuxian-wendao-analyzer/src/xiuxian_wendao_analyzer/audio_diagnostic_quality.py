"""Audio diagnostic quality gates and transcript review reports."""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from xiuxian_wendao_analyzer.audio_diagnostic_metrics import (
    character_error_rate,
    chinese_ratio,
    inaudible_count,
    repeated_ngram_ratio,
    required_term_coverage,
)
from xiuxian_wendao_analyzer.audio_diagnostic_quality_inputs import (
    read_transcript,
)


@dataclass(frozen=True)
class QualityRow:
    """Quality review row for one audio transcript result."""

    backend: str
    source: str
    chunk_index: int
    start_seconds: float
    duration_seconds: float
    status: str
    review_status: str
    model: str
    transcript_chars: int
    chinese_ratio: float | None
    inaudible_count: int
    inaudible_per_minute: float
    chars_per_minute: float
    repeated_ngram_ratio: float
    reference_cer: float | None
    required_terms_count: int
    missing_required_terms: str
    required_term_recall: float | None
    transcript_path: str
    error: str
    segments_path: str = ""
    segment_count: int = 0


def _result_value(result: Any, field: str) -> Any:
    return result.get(field) if isinstance(result, Mapping) else getattr(result, field)


def _is_natural_short_utterance(
    *,
    transcript: str,
    min_chinese_ratio: float,
    max_inaudible_per_minute: float,
    max_repeated_ngram_ratio: float,
    duration_minutes: float,
) -> bool:
    stripped = transcript.strip()
    if not stripped:
        return False
    ratio = chinese_ratio(stripped)
    markers_per_minute = (
        inaudible_count(stripped) / duration_minutes if duration_minutes else 0.0
    )
    return (
        ratio is not None
        and ratio >= min_chinese_ratio
        and markers_per_minute <= max_inaudible_per_minute
        and repeated_ngram_ratio(stripped) <= max_repeated_ngram_ratio
    )


def classify_quality(
    result: Any,
    *,
    transcript: str,
    reference_cer: float | None,
    max_reference_cer: float,
    required_term_recall: float | None,
    min_required_term_recall: float,
    min_chars_per_minute: float,
    min_chinese_ratio: float,
    max_inaudible_per_minute: float,
    max_repeated_ngram_ratio: float,
) -> str:
    """Classify one ASR result for precision review."""

    if _result_value(result, "status") != "ok":
        return "failed"
    if (
        required_term_recall is not None
        and required_term_recall < min_required_term_recall
    ):
        return "required-term-miss"
    if reference_cer is not None:
        return (
            "reference-pass" if reference_cer <= max_reference_cer else "reference-fail"
        )
    duration = float(_result_value(result, "duration_seconds"))
    duration_minutes = duration / 60 if duration else 0.0
    chars_per_minute = len(transcript) / duration_minutes if duration_minutes else 0.0
    ratio = chinese_ratio(transcript)
    markers_per_minute = (
        inaudible_count(transcript) / duration_minutes if duration_minutes else 0.0
    )
    repeat_ratio = repeated_ngram_ratio(transcript)
    if chars_per_minute < min_chars_per_minute:
        if _is_natural_short_utterance(
            transcript=transcript,
            min_chinese_ratio=min_chinese_ratio,
            max_inaudible_per_minute=max_inaudible_per_minute,
            max_repeated_ngram_ratio=max_repeated_ngram_ratio,
            duration_minutes=duration_minutes,
        ):
            return "short-utterance-review"
        return "weak-too-short"
    if ratio is not None and ratio < min_chinese_ratio:
        return "weak-language-ratio"
    if markers_per_minute > max_inaudible_per_minute:
        return "weak-inaudible-heavy"
    if repeat_ratio > max_repeated_ngram_ratio:
        return "weak-repetition-heavy"
    return "review-needed"


def build_quality_rows(
    results: Sequence[Any],
    *,
    references: Mapping[tuple[str, int], str],
    max_reference_cer: float,
    required_terms: Sequence[str],
    min_required_term_recall: float,
    min_chars_per_minute: float,
    min_chinese_ratio: float,
    max_inaudible_per_minute: float,
    max_repeated_ngram_ratio: float,
) -> list[QualityRow]:
    """Build per-result quality rows from transcripts and optional references."""

    rows: list[QualityRow] = []
    for result in results:
        transcript_path = str(_result_value(result, "transcript_path"))
        transcript = read_transcript(transcript_path)
        source = str(_result_value(result, "source"))
        chunk_index = int(_result_value(result, "chunk_index"))
        reference = references.get((Path(source).name, chunk_index))
        cer = character_error_rate(transcript, reference) if reference else None
        duration = float(_result_value(result, "duration_seconds"))
        duration_minutes = duration / 60 if duration else 0.0
        chars_per_minute = (
            len(transcript) / duration_minutes if duration_minutes else 0.0
        )
        markers = inaudible_count(transcript)
        markers_per_minute = markers / duration_minutes if duration_minutes else 0.0
        repeat_ratio = repeated_ngram_ratio(transcript)
        _, missing_terms, term_recall = required_term_coverage(
            transcript, required_terms
        )
        rows.append(
            QualityRow(
                backend=str(_result_value(result, "backend")),
                source=source,
                chunk_index=chunk_index,
                start_seconds=float(_result_value(result, "start_seconds")),
                duration_seconds=duration,
                status=str(_result_value(result, "status")),
                review_status=classify_quality(
                    result,
                    transcript=transcript,
                    reference_cer=cer,
                    max_reference_cer=max_reference_cer,
                    required_term_recall=term_recall,
                    min_required_term_recall=min_required_term_recall,
                    min_chars_per_minute=min_chars_per_minute,
                    min_chinese_ratio=min_chinese_ratio,
                    max_inaudible_per_minute=max_inaudible_per_minute,
                    max_repeated_ngram_ratio=max_repeated_ngram_ratio,
                ),
                model=str(_result_value(result, "model")),
                transcript_chars=len(transcript),
                chinese_ratio=chinese_ratio(transcript),
                inaudible_count=markers,
                inaudible_per_minute=markers_per_minute,
                chars_per_minute=chars_per_minute,
                repeated_ngram_ratio=repeat_ratio,
                reference_cer=cer,
                required_terms_count=len(required_terms),
                missing_required_terms="|".join(missing_terms),
                required_term_recall=term_recall,
                transcript_path=transcript_path,
                error=str(_result_value(result, "error")),
                segments_path=str(getattr(result, "segments_path", "")),
                segment_count=int(getattr(result, "segment_count", 0)),
            )
        )
    return rows
