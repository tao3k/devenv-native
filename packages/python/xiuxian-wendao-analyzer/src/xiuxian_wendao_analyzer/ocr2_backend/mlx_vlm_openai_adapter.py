from __future__ import annotations

import asyncio
import base64
import os
import tempfile
import threading
import time
import uuid
from contextlib import asynccontextmanager
from pathlib import Path
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from collections.abc import AsyncIterator

from fastapi import FastAPI, HTTPException
from mlx_vlm import generate, load
from mlx_vlm.prompt_utils import apply_chat_template
from pydantic import BaseModel, Field
from uvicorn import Config, Server


def _env(name: str, default: str) -> str:
    value = os.environ.get(name)
    if value is None or not value.strip():
        return default
    return value


MODEL_PATH = _env(
    "WENDAO_DEEPSEEK_OCR2_MODEL_PATH", ".data/models/deepseek-ocr2-current"
)
MODEL_NAME = _env("WENDAO_DEEPSEEK_OCR2_MODEL", "deepseek-ai/DeepSeek-OCR-2")
HOST = _env("WENDAO_DEEPSEEK_OCR2_HOST", "127.0.0.1")
PORT = int(_env("WENDAO_DEEPSEEK_OCR2_PORT", "8000"))
DEFAULT_PROMPT = _env(
    "WENDAO_DEEPSEEK_OCR2_PROMPT",
    "<image>\n<|grounding|>Convert the document to markdown.",
)
MAX_TOKENS = int(_env("WENDAO_DEEPSEEK_OCR2_MAX_TOKENS", "8192"))
TEMPERATURE = float(_env("WENDAO_DEEPSEEK_OCR2_TEMPERATURE", "0.0"))

_model: Any | None = None
_processor: Any | None = None
_generation_lock = threading.Lock()


@asynccontextmanager
async def _lifespan(_application: FastAPI) -> AsyncIterator[None]:
    global _model, _processor
    _model, _processor = load(str(Path(MODEL_PATH).resolve()), trust_remote_code=False)
    try:
        yield
    finally:
        _model = None
        _processor = None


app = FastAPI(
    title="Wendao DeepSeek-OCR-2 MLX-VLM OpenAI Adapter",
    lifespan=_lifespan,
)


class ChatCompletionRequest(BaseModel):
    model: str | None = None
    messages: list[dict[str, Any]]
    max_tokens: int | None = Field(default=None, alias="max_tokens")
    temperature: float | None = None


def _strip_image_tokens(prompt: str) -> str:
    lines = [line for line in prompt.splitlines() if line.strip() != "<image>"]
    stripped = "\n".join(lines).strip()
    return stripped or DEFAULT_PROMPT.replace("<image>", "").strip()


def _extract_request_parts(request: ChatCompletionRequest) -> tuple[str, str]:
    text_parts: list[str] = []
    image_url: str | None = None
    for message in request.messages:
        content = message.get("content")
        if isinstance(content, str):
            text_parts.append(content)
            continue
        if not isinstance(content, list):
            continue
        for part in content:
            if not isinstance(part, dict):
                continue
            part_type = part.get("type")
            if part_type == "text":
                text_parts.append(str(part.get("text", "")))
            elif part_type == "image_url":
                image_value = part.get("image_url")
                if isinstance(image_value, dict):
                    image_url = str(image_value.get("url", ""))
                elif isinstance(image_value, str):
                    image_url = image_value

    if not image_url:
        raise HTTPException(
            status_code=400, detail="request is missing image_url content"
        )

    prompt = _strip_image_tokens("\n".join(text_parts).strip() or DEFAULT_PROMPT)
    return prompt, image_url


def _write_data_url_image(image_url: str) -> str:
    if not image_url.startswith("data:"):
        raise HTTPException(
            status_code=400, detail="only data URL images are supported"
        )
    try:
        header, payload = image_url.split(",", 1)
    except ValueError as exc:
        raise HTTPException(status_code=400, detail="invalid image data URL") from exc
    suffix = ".png"
    if "image/jpeg" in header or "image/jpg" in header:
        suffix = ".jpg"
    try:
        image_bytes = base64.b64decode(payload, validate=True)
    except ValueError as exc:
        raise HTTPException(
            status_code=400, detail="invalid base64 image payload"
        ) from exc

    with tempfile.NamedTemporaryFile(delete=False, suffix=suffix) as tmp:
        tmp.write(image_bytes)
        return tmp.name


def _run_generation(
    prompt: str, image_path: str, max_tokens: int, temperature: float
) -> str:
    if _model is None or _processor is None:
        raise RuntimeError("MLX-VLM model is not initialized")
    formatted_prompt = apply_chat_template(
        _processor,
        _model.config,
        prompt,
        num_images=1,
    )
    with _generation_lock:
        result = generate(
            _model,
            _processor,
            formatted_prompt,
            image=[image_path],
            max_tokens=max_tokens,
            temperature=temperature,
            verbose=False,
        )
    return str(result.text).strip()


@app.get("/v1/models")
async def _models() -> dict[str, Any]:
    return {
        "object": "list",
        "data": [{"id": MODEL_NAME, "object": "model", "owned_by": "wendao"}],
    }


@app.post("/v1/chat/completions")
async def _chat_completions(request: ChatCompletionRequest) -> dict[str, Any]:
    prompt, image_url = _extract_request_parts(request)
    image_path = _write_data_url_image(image_url)
    try:
        text = _run_generation(
            prompt,
            image_path,
            request.max_tokens or MAX_TOKENS,
            request.temperature if request.temperature is not None else TEMPERATURE,
        )
    finally:
        Path(image_path).unlink(missing_ok=True)
    if not text:
        raise HTTPException(status_code=502, detail="MLX-VLM returned empty OCR output")

    created = int(time.time())
    return {
        "id": f"chatcmpl-{uuid.uuid4().hex}",
        "object": "chat.completion",
        "created": created,
        "model": request.model or MODEL_NAME,
        "choices": [
            {
                "index": 0,
                "message": {"role": "assistant", "content": text},
                "finish_reason": "stop",
            }
        ],
        "usage": {
            "prompt_tokens": 0,
            "completion_tokens": 0,
            "total_tokens": 0,
        },
    }


def main() -> None:
    config = Config(app=app, host=HOST, port=PORT, log_level="info")
    server = Server(config)
    asyncio.run(server.serve())


if __name__ == "__main__":
    main()
