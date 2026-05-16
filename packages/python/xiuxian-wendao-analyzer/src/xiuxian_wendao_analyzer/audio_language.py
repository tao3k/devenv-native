"""Model-neutral audio language hint helpers."""

from __future__ import annotations

PRIMARY_LANGUAGE_UNKNOWN = "unknown"


def normalize_primary_language(value: str | None) -> str:
    """Normalize an optional primary language hint."""

    normalized = (value or "").strip().lower().replace("_", "-")
    return normalized or PRIMARY_LANGUAGE_UNKNOWN


def prompt_with_primary_language(prompt: str, primary_language: str | None) -> str:
    """Append model-neutral language guidance to an audio prompt."""

    normalized = normalize_primary_language(primary_language)
    if normalized == PRIMARY_LANGUAGE_UNKNOWN:
        return prompt
    return (
        f"{prompt}\n\nPRIMARY_LANGUAGE={normalized}. Use this key as a hint "
        "when interpreting the audio. Infer the actual spoken language from the "
        "audio and transcribe in that language; do not translate into another "
        "language."
    )
