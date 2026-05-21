"""Audio diagnostic precision metric helpers."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Sequence

INAUDIBLE_MARKERS = ("[inaudible]", "[听不清]", "听不清")


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


def repeated_ngram_ratio(text: str, *, ngram_size: int = 3) -> float:
    """Return the share of repeated normalized character n-grams."""

    normalized = normalize_reference_text(text)
    if len(normalized) < ngram_size:
        return 0.0
    counts: dict[str, int] = {}
    total = 0
    for index in range(0, len(normalized) - ngram_size + 1):
        ngram = normalized[index : index + ngram_size]
        counts[ngram] = counts.get(ngram, 0) + 1
        total += 1
    repeated = sum(count - 1 for count in counts.values() if count > 1)
    return repeated / total if total else 0.0


def required_term_coverage(
    transcript: str, required_terms: Sequence[str]
) -> tuple[int, list[str], float | None]:
    """Return required-term coverage using normalized substring matching."""

    if not required_terms:
        return 0, [], None
    normalized_transcript = normalize_reference_text(transcript)
    missing = [
        term
        for term in required_terms
        if normalize_reference_text(term) not in normalized_transcript
    ]
    matched = len(required_terms) - len(missing)
    return matched, missing, matched / len(required_terms)
