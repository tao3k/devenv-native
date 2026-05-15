"""OpenAI-compatible chat/audio adapter backed by mlx-qwen3-asr."""

from __future__ import annotations

import os
import sys
import tempfile
import time
import uuid
from pathlib import Path
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence

try:
    from ..audio_openai_protocol import extract_input_audio, extract_text_prompt
except ImportError:
    sys.path.append(str(Path(__file__).resolve().parents[2]))
    from xiuxian_wendao_analyzer.audio_openai_protocol import (
        extract_input_audio,
        extract_text_prompt,
    )

DEFAULT_QWEN3_ASR_MODEL = "Qwen/Qwen3-ASR-1.7B"


class AudioAdapterRequestError(ValueError):
    """Raised when an OpenAI-compatible audio request is invalid."""

    def __init__(self, detail: str) -> None:
        super().__init__(detail)
        self.detail = detail


def create_app() -> object:
    """Create the Qwen3-ASR MLX audio adapter app."""

    from fastapi import Body, FastAPI, HTTPException
    from pydantic import BaseModel

    class ChatCompletionRequest(BaseModel):
        """Minimal OpenAI-compatible chat completion request."""

        model: str | None = None
        messages: list[dict[str, Any]]

    app = FastAPI(title="Wendao local Qwen3-ASR MLX audio adapter")

    @app.get("/v1/models")
    async def list_models() -> dict[str, object]:
        model_name = _served_model_name()
        return {"object": "list", "data": [{"id": model_name, "object": "model"}]}

    request_body_param = Body(...)

    @app.post("/v1/chat/completions")
    async def chat_completions(
        request_body: dict[str, Any] = request_body_param,
    ) -> dict[str, object]:
        request = ChatCompletionRequest.model_validate(request_body)
        try:
            return complete_chat_audio(request.messages, requested_model=request.model)
        except AudioAdapterRequestError as exc:
            raise HTTPException(status_code=400, detail=exc.detail) from exc

    return app


def complete_chat_audio(
    messages: Sequence[Mapping[str, object]],
    *,
    requested_model: str | None = None,
) -> dict[str, object]:
    """Transcribe an OpenAI-compatible chat/audio request."""

    try:
        audio_input = extract_input_audio(messages)
    except ValueError as exc:
        raise AudioAdapterRequestError(str(exc)) from exc
    context = _context_from_request(extract_text_prompt(messages))
    model_path = _model_path()
    with tempfile.TemporaryDirectory(prefix="wendao-qwen3-asr-audio-") as tmpdir:
        audio_path = Path(tmpdir) / f"input.{audio_input.format}"
        audio_path.write_bytes(audio_input.data)
        text, segments = _transcribe_audio(
            audio_path,
            model_path=model_path,
            context=context,
        )
    return _chat_completion_response(
        model=requested_model or _served_model_name(),
        text=text,
        segments=segments,
    )


def _transcribe_audio(
    audio_path: Path,
    *,
    model_path: str,
    context: str = "",
) -> tuple[str, list[dict[str, object]]]:
    import mlx_qwen3_asr

    result = mlx_qwen3_asr.transcribe(
        str(audio_path),
        model=model_path,
        context=context,
        language=os.environ.get("WENDAO_AUDIO_LOCAL_LANGUAGE", "zh"),
        return_timestamps=_env_bool("WENDAO_AUDIO_QWEN3_RETURN_TIMESTAMPS", False),
        return_chunks=_env_bool("WENDAO_AUDIO_QWEN3_RETURN_CHUNKS", False),
        max_new_tokens=_env_optional_int("WENDAO_AUDIO_QWEN3_MAX_NEW_TOKENS"),
        verbose=False,
    )
    text = getattr(result, "text", "")
    return text.strip() if isinstance(text, str) else "", _segments_from_result(result)


def _segments_from_result(result: object) -> list[dict[str, object]]:
    raw_segments = (
        getattr(result, "segments", None)
        or getattr(result, "chunks", None)
        or getattr(result, "timestamps", None)
    )
    if not isinstance(raw_segments, list):
        return []
    segments: list[dict[str, object]] = []
    for item in raw_segments:
        segment = _normalize_segment(item)
        if segment is not None:
            segments.append(segment)
    return segments


def _normalize_segment(value: object) -> dict[str, object] | None:
    if isinstance(value, dict):
        start = _seconds(value.get("startSeconds", value.get("start")))
        end = _seconds(value.get("endSeconds", value.get("end")))
        timestamp = value.get("timestamp")
        if isinstance(timestamp, list | tuple) and len(timestamp) >= 2:
            start = _seconds(timestamp[0]) if start is None else start
            end = _seconds(timestamp[1]) if end is None else end
        text = _segment_text(value)
    elif isinstance(value, list | tuple) and len(value) >= 2:
        start = _seconds(value[0])
        end = _seconds(value[1])
        text = str(value[2]).strip() if len(value) >= 3 else ""
    else:
        return None
    if start is None or end is None or end <= start or not text:
        return None
    return {"startSeconds": start, "endSeconds": end, "text": text}


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


def _segment_text(value: dict[str, object]) -> str:
    for key in ("text", "word", "transcript", "content"):
        text = value.get(key)
        if isinstance(text, str) and text.strip():
            return text.strip()
    return ""


def _context_from_request(prompt: str) -> str:
    configured = os.environ.get("WENDAO_AUDIO_QWEN3_CONTEXT")
    if configured is not None:
        return configured.strip()
    if _env_bool("WENDAO_AUDIO_QWEN3_USE_REQUEST_PROMPT_AS_CONTEXT", True):
        return prompt.strip()
    return ""


def _env_bool(name: str, default: bool) -> bool:
    value = os.environ.get(name)
    if value is None:
        return default
    return value.strip().lower() in {"1", "true", "yes", "on"}


def _env_optional_int(name: str) -> int | None:
    value = os.environ.get(name)
    if value is None:
        return None
    normalized = value.strip().lower()
    if normalized in {"", "none", "null", "disabled"}:
        return None
    return int(normalized)


def _chat_completion_response(
    *,
    model: str,
    text: str,
    segments: list[dict[str, object]] | None = None,
) -> dict[str, object]:
    response: dict[str, object] = {
        "id": f"chatcmpl-{uuid.uuid4().hex}",
        "object": "chat.completion",
        "created": int(time.time()),
        "model": model,
        "choices": [
            {
                "index": 0,
                "message": {"role": "assistant", "content": text},
                "finish_reason": "stop",
            }
        ],
    }
    if segments:
        response["segments"] = segments
    return response


def _model_path() -> str:
    return (
        os.environ.get("WENDAO_AUDIO_LOCAL_MODEL_PATH")
        or os.environ.get("WENDAO_AUDIO_QWEN3_ASR_MODEL")
        or DEFAULT_QWEN3_ASR_MODEL
    )


def _served_model_name() -> str:
    return os.environ.get("WENDAO_AUDIO_LOCAL_MODEL") or "wendao-qwen3-asr-audio"


def main() -> None:
    """Run the Qwen3-ASR MLX audio adapter."""

    import uvicorn

    uvicorn.run(
        create_app(),
        host=os.environ.get("WENDAO_AUDIO_LOCAL_HOST", "127.0.0.1"),
        port=int(os.environ.get("WENDAO_AUDIO_LOCAL_PORT", "8010")),
        log_level=os.environ.get("WENDAO_AUDIO_LOCAL_LOG_LEVEL", "info"),
    )


if __name__ == "__main__":
    main()
