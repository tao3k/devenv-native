"""OpenAI-compatible audio diagnostic request helpers."""

from __future__ import annotations

import base64
import json
import urllib.error
import urllib.request
from collections.abc import Mapping, Sequence
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path

    from xiuxian_wendao_analyzer.audio_diagnostic_materialization import AudioChunk


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
        parsed = _json_mapping_from_text(content)
        if parsed is not None:
            return _transcript_from_mapping(parsed)
        return content
    if isinstance(content, Sequence) and not isinstance(content, (bytes, bytearray)):
        parts: list[str] = []
        for item in content:
            if isinstance(item, Mapping) and isinstance(item.get("text"), str):
                parts.append(item["text"])
        return "\n".join(parts)
    return ""


def _json_mapping_from_text(text: str) -> Mapping[str, object] | None:
    stripped = text.strip()
    if not stripped.startswith("{"):
        return None
    try:
        parsed = json.loads(stripped)
    except json.JSONDecodeError:
        return None
    return parsed if isinstance(parsed, Mapping) else None


def _transcript_from_mapping(value: Mapping[str, object]) -> str:
    for key in ("text", "transcript", "content"):
        text = value.get(key)
        if isinstance(text, str) and text.strip():
            return text
    return ""


def extract_openrouter_transcript(response: Mapping[str, object]) -> str:
    """Extract assistant text from an OpenRouter chat completion response."""

    direct_text = _transcript_from_mapping(response)
    if direct_text:
        return direct_text.strip()
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


def _seconds(value: object) -> float | None:
    if isinstance(value, bool):
        return None
    if isinstance(value, int | float):
        return float(value)
    if isinstance(value, str):
        try:
            return float(value)
        except ValueError:
            return None
    return None


def _segment_text(value: Mapping[str, object]) -> str:
    for key in ("text", "word", "transcript", "content"):
        text = value.get(key)
        if isinstance(text, str) and text.strip():
            return text.strip()
    return ""


def _normalize_segment(value: object) -> dict[str, object] | None:
    if not isinstance(value, Mapping):
        return None
    start = _seconds(value.get("startSeconds"))
    if start is None:
        start = _seconds(value.get("start"))
    if start is None and (start_ms := _seconds(value.get("startMs"))) is not None:
        start = start_ms / 1000
    end = _seconds(value.get("endSeconds"))
    if end is None:
        end = _seconds(value.get("end"))
    if end is None and (end_ms := _seconds(value.get("endMs"))) is not None:
        end = end_ms / 1000
    duration = _seconds(value.get("durationSeconds"))
    if (
        duration is None
        and (duration_ms := _seconds(value.get("durationMs"))) is not None
    ):
        duration = duration_ms / 1000
    if end is None and start is not None and duration is not None:
        end = start + duration
    text = _segment_text(value)
    if start is None or end is None or end <= start or not text:
        return None
    return {"startSeconds": start, "endSeconds": end, "text": text}


def _segments_from_mapping(value: Mapping[str, object]) -> list[dict[str, object]]:
    for key in ("segments", "words"):
        segments = value.get(key)
        if isinstance(segments, Sequence) and not isinstance(
            segments, (str, bytes, bytearray)
        ):
            return [
                normalized
                for segment in segments
                if (normalized := _normalize_segment(segment)) is not None
            ]
    return []


def extract_openrouter_segments(
    response: Mapping[str, object],
) -> list[dict[str, object]]:
    """Extract timestamped transcription segments when the backend returns them."""

    direct_segments = _segments_from_mapping(response)
    if direct_segments:
        return direct_segments
    choices = response.get("choices")
    if not isinstance(choices, list) or not choices:
        return []
    first_choice = choices[0]
    if not isinstance(first_choice, Mapping):
        return []
    message = first_choice.get("message")
    if not isinstance(message, Mapping):
        return []
    content = message.get("content")
    if isinstance(content, str):
        parsed = _json_mapping_from_text(content)
        return [] if parsed is None else _segments_from_mapping(parsed)
    if isinstance(content, Sequence) and not isinstance(content, (bytes, bytearray)):
        segments: list[dict[str, object]] = []
        for item in content:
            if not isinstance(item, Mapping):
                continue
            if item_segments := _segments_from_mapping(item):
                segments.extend(item_segments)
            elif isinstance(item.get("text"), str):
                parsed = _json_mapping_from_text(str(item["text"]))
                if parsed is not None:
                    segments.extend(_segments_from_mapping(parsed))
        return segments
    return []


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
) -> tuple[str, list[dict[str, object]]]:
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
        return "", []
    return extract_openrouter_transcript(parsed), extract_openrouter_segments(parsed)
