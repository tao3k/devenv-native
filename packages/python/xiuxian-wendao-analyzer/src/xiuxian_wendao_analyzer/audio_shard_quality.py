"""Model-neutral transcript quality gates for audio shard workers."""

from __future__ import annotations

import re
from dataclasses import dataclass
from typing import TYPE_CHECKING

from .audio_language import PRIMARY_LANGUAGE_UNKNOWN, normalize_primary_language

if TYPE_CHECKING:
    from collections.abc import Mapping
    from typing import Any


AUDIO_TRANSCRIPT_QUALITY_MAX_CHARS_PER_MINUTE = 900.0
AUDIO_TRANSCRIPT_QUALITY_MAX_REPEATED_NGRAM_RATIO = 0.20
AUDIO_TRANSCRIPT_QUALITY_MAX_LATIN_RATIO_FOR_CHINESE = 0.20
AUDIO_TRANSCRIPT_QUALITY_REPEATED_NGRAM_SIZE = 6

_HOSTED_REFUSAL_RE = re.compile(
    r"\b("
    r"audio file attached|"
    r"do(?:es)?n'?t see an? audio file|"
    r"no audio file (?:is )?(?:attached|accessible)|"
    r"cannot access (?:the )?audio|"
    r"can't access (?:the )?audio|"
    r"unable to transcribe|"
    r"please upload|"
    r"as an ai"
    r")\b",
    re.IGNORECASE,
)

_NO_TRANSCRIBABLE_SPEECH_RE = re.compile(
    r"("
    r"no (?:transcribable )?(?:speech|voice|audio content)|"
    r"nothing (?:to )?transcribe|"
    r"没有(?:可)?转录(?:的)?(?:语音|音频|内容)|"
    r"没有(?:可识别|可听清)(?:的)?(?:语音|音频|内容)"
    r")",
    re.IGNORECASE,
)


@dataclass(frozen=True, slots=True)
class AudioTranscriptQualityOptions:
    """Thresholds for accepting model-produced audio transcript text."""

    enabled: bool = True
    max_chars_per_minute: float = AUDIO_TRANSCRIPT_QUALITY_MAX_CHARS_PER_MINUTE
    max_repeated_ngram_ratio: float = AUDIO_TRANSCRIPT_QUALITY_MAX_REPEATED_NGRAM_RATIO
    max_latin_ratio_for_chinese: float = (
        AUDIO_TRANSCRIPT_QUALITY_MAX_LATIN_RATIO_FOR_CHINESE
    )
    repeated_ngram_size: int = AUDIO_TRANSCRIPT_QUALITY_REPEATED_NGRAM_SIZE


def audio_transcript_quality_failure(
    input_row: Mapping[str, Any],
    text: str,
    *,
    primary_language: str = PRIMARY_LANGUAGE_UNKNOWN,
    options: AudioTranscriptQualityOptions | None = None,
) -> str | None:
    """Return a failure message when transcript text is not precision-safe."""

    options = options or AudioTranscriptQualityOptions()
    if not options.enabled:
        return None
    transcript = text.strip()
    if not transcript:
        return "audio transcript quality gate failed: empty_text"

    reasons: list[str] = []
    chars_per_minute = _chars_per_minute(transcript, input_row)
    if chars_per_minute > options.max_chars_per_minute:
        reasons.append(
            f"chars_per_minute={chars_per_minute:.3f}>{options.max_chars_per_minute:.3f}"
        )

    repeat_ratio = repeated_ngram_ratio(
        transcript,
        ngram_size=options.repeated_ngram_size,
    )
    if repeat_ratio > options.max_repeated_ngram_ratio:
        reasons.append(
            f"repeated_ngram_ratio={repeat_ratio:.6f}>{options.max_repeated_ngram_ratio:.6f}"
        )

    if _HOSTED_REFUSAL_RE.search(transcript):
        reasons.append("hosted_refusal_text")

    if _NO_TRANSCRIBABLE_SPEECH_RE.search(transcript):
        reasons.append("no_transcribable_speech_text")

    if _expects_chinese(primary_language, input_row):
        latin_ratio = _latin_ratio(transcript)
        if latin_ratio > options.max_latin_ratio_for_chinese:
            reasons.append(
                "latin_ratio_for_chinese="
                f"{latin_ratio:.6f}>{options.max_latin_ratio_for_chinese:.6f}"
            )

    if not reasons:
        return None
    return "audio transcript quality gate failed: " + "; ".join(reasons)


def repeated_ngram_ratio(text: str, *, ngram_size: int) -> float:
    """Return the share of repeated normalized character n-grams."""

    normalized = "".join(char.lower() for char in text if not char.isspace())
    if ngram_size <= 0 or len(normalized) < ngram_size:
        return 0.0
    counts: dict[str, int] = {}
    total = 0
    for index in range(0, len(normalized) - ngram_size + 1):
        ngram = normalized[index : index + ngram_size]
        counts[ngram] = counts.get(ngram, 0) + 1
        total += 1
    repeated = sum(count - 1 for count in counts.values() if count > 1)
    return repeated / total if total else 0.0


def _chars_per_minute(text: str, input_row: Mapping[str, Any]) -> float:
    duration_ms = _positive_float(input_row.get("durationMs"))
    if not duration_ms:
        return 0.0
    return len(text) / (duration_ms / 60_000.0)


def _positive_float(value: object) -> float:
    try:
        parsed = float(value)
    except (TypeError, ValueError):
        return 0.0
    return parsed if parsed > 0.0 else 0.0


def _expects_chinese(
    primary_language: str,
    input_row: Mapping[str, Any],
) -> bool:
    configured = normalize_primary_language(primary_language)
    row_value = normalize_primary_language(
        str(input_row.get("preferredLanguages") or "")
    )
    return _language_is_chinese(configured) or _language_is_chinese(row_value)


def _language_is_chinese(value: str) -> bool:
    normalized = normalize_primary_language(value)
    return normalized in {"zh", "zh-cn", "zh-hans", "zh-hant", "chinese", "mandarin"}


def _latin_ratio(text: str) -> float:
    chars = [char for char in text if not char.isspace()]
    if not chars:
        return 0.0
    latin = sum(1 for char in chars if ("a" <= char.lower() <= "z"))
    return latin / len(chars)
