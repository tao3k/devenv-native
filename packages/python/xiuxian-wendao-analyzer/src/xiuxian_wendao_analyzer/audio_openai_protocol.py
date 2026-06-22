"""OpenAI-compatible chat/audio protocol helpers."""

from __future__ import annotations

import base64
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path

AUDIO_OPENAI_DEFAULT_PROMPT = "Transcribe this audio shard faithfully. Return only transcript text."


@dataclass(frozen=True, slots=True)
class AudioInput:
    """Decoded OpenAI-compatible input audio content."""

    data: bytes
    format: str


def build_chat_audio_payload(
    *,
    model: str,
    audio_path: Path,
    audio_format: str = "wav",
    prompt: str = AUDIO_OPENAI_DEFAULT_PROMPT,
    disable_reasoning: bool = False,
) -> dict[str, Any]:
    """Build an OpenAI-compatible chat completion payload with audio input."""

    audio_data = base64.b64encode(audio_path.read_bytes()).decode("ascii")
    payload: dict[str, Any] = {
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": prompt},
                    {
                        "type": "input_audio",
                        "input_audio": {
                            "data": audio_data,
                            "format": audio_format.strip().lower() or "wav",
                        },
                    },
                ],
            }
        ],
        "stream": False,
    }
    if disable_reasoning:
        payload["reasoning"] = {"effort": "none"}
    return payload


def build_audio_transcription_payload(
    *,
    model: str,
    audio_path: Path,
    audio_format: str = "wav",
    language: str | None = None,
) -> dict[str, Any]:
    """Build an OpenRouter/OpenAI-compatible STT transcription payload."""

    audio_data = base64.b64encode(audio_path.read_bytes()).decode("ascii")
    payload: dict[str, Any] = {
        "model": model,
        "input_audio": {
            "data": audio_data,
            "format": audio_format.strip().lower() or "wav",
        },
    }
    if language and language.strip().lower() != "unknown":
        payload["language"] = language.strip().lower()
    return payload


def extract_input_audio(
    messages: Sequence[Mapping[str, object]],
) -> AudioInput:
    """Decode the first OpenAI-compatible input audio item from messages."""

    for message in messages:
        content = message.get("content")
        if not isinstance(content, Sequence) or isinstance(content, (str, bytes)):
            continue
        for item in content:
            if not isinstance(item, Mapping) or item.get("type") != "input_audio":
                continue
            input_audio = item.get("input_audio")
            if not isinstance(input_audio, Mapping):
                continue
            data = input_audio.get("data")
            audio_format = input_audio.get("format", "wav")
            if not isinstance(data, str) or not isinstance(audio_format, str):
                continue
            try:
                return AudioInput(
                    data=base64.b64decode(data, validate=True),
                    format=audio_format.lower(),
                )
            except ValueError as exc:
                raise ValueError("invalid input_audio data") from exc
    raise ValueError("missing input_audio content")


def extract_text_prompt(messages: Sequence[Mapping[str, object]]) -> str:
    """Extract concatenated text prompt parts from chat messages."""

    parts: list[str] = []
    for message in messages:
        content = message.get("content")
        if isinstance(content, str):
            if content.strip():
                parts.append(content.strip())
            continue
        if not isinstance(content, Sequence) or isinstance(content, bytes):
            continue
        for item in content:
            if not isinstance(item, Mapping):
                continue
            text = item.get("text")
            if item.get("type") == "text" and isinstance(text, str) and text.strip():
                parts.append(text.strip())
    return "\n".join(parts)


def extract_openai_message_content(payload: Mapping[str, Any]) -> str:
    """Extract text content from an OpenAI-compatible chat response."""

    choices = payload.get("choices")
    if not isinstance(choices, list) or not choices:
        raise ValueError("OpenAI-compatible audio response does not contain choices")
    first_choice = choices[0]
    if not isinstance(first_choice, Mapping):
        raise ValueError("OpenAI-compatible audio response choice is not an object")
    message = first_choice.get("message")
    if not isinstance(message, Mapping):
        raise ValueError("OpenAI-compatible audio response choice lacks message")
    content = message.get("content")
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        parts = [
            str(part["text"])
            for part in content
            if isinstance(part, Mapping) and isinstance(part.get("text"), str)
        ]
        if parts:
            return "".join(parts)
    raise ValueError("OpenAI-compatible audio response content is not text")


def extract_audio_transcription_text(payload: Mapping[str, Any]) -> str:
    """Extract text content from a STT transcription response."""

    text = payload.get("text")
    if isinstance(text, str):
        return text
    raise ValueError("OpenAI-compatible audio transcription response lacks text")
