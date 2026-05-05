from __future__ import annotations

import asyncio
import base64
import os
import sys
import time
import uuid
from contextlib import asynccontextmanager
from io import BytesIO
from pathlib import Path
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from collections.abc import AsyncIterator

os.environ.setdefault("VLLM_USE_V1", "0")


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
PROMPT = _env(
    "WENDAO_DEEPSEEK_OCR2_PROMPT",
    "<image>\n<|grounding|>Convert the document to markdown.",
)
MAX_TOKENS = int(_env("WENDAO_DEEPSEEK_OCR2_MAX_TOKENS", "8192"))
DTYPE = _env("WENDAO_DEEPSEEK_OCR2_VLLM_DTYPE", "bfloat16")
MAX_MODEL_LEN = int(_env("WENDAO_DEEPSEEK_OCR2_VLLM_MAX_MODEL_LEN", "8192"))
GPU_MEMORY_UTILIZATION = float(
    _env("WENDAO_DEEPSEEK_OCR2_VLLM_GPU_MEMORY_UTILIZATION", "0.75")
)
TENSOR_PARALLEL_SIZE = int(_env("WENDAO_DEEPSEEK_OCR2_VLLM_TENSOR_PARALLEL_SIZE", "1"))
TEMPERATURE = float(_env("WENDAO_DEEPSEEK_OCR2_TEMPERATURE", "0.0"))
CROP_MODE = _env("WENDAO_DEEPSEEK_OCR2_CROP_MODE", "1") != "0"
SKIP_REPEAT = _env("WENDAO_DEEPSEEK_OCR2_SKIP_REPEAT", "1") != "0"

OFFICIAL_VLLM_DIR = Path(
    _env(
        "WENDAO_DEEPSEEK_OCR2_OFFICIAL_VLLM_DIR",
        ".cache/ocr/DeepSeek-OCR-2/DeepSeek-OCR2-master/DeepSeek-OCR2-vllm",
    )
).resolve()
RUNTIME_CONFIG_DIR = Path(
    _env(
        "WENDAO_DEEPSEEK_OCR2_RUNTIME_CONFIG_DIR",
        ".run/ocr/deepseek-ocr2-official-vllm",
    )
).resolve()


def _write_runtime_config() -> None:
    RUNTIME_CONFIG_DIR.mkdir(parents=True, exist_ok=True)
    config_py = RUNTIME_CONFIG_DIR / "config.py"
    config_py.write_text(
        "\n".join(
            [
                "BASE_SIZE = 1024",
                "IMAGE_SIZE = 768",
                f"CROP_MODE = {CROP_MODE!r}",
                "MIN_CROPS = 2",
                "MAX_CROPS = 6",
                "MAX_CONCURRENCY = 100",
                "NUM_WORKERS = 64",
                "PRINT_NUM_VIS_TOKENS = False",
                "SKIP_REPEAT = True",
                f"MODEL_PATH = {str(Path(MODEL_PATH).resolve())!r}",
                "INPUT_PATH = ''",
                "OUTPUT_PATH = ''",
                f"PROMPT = {PROMPT!r}",
                "from transformers import AutoTokenizer",
                "TOKENIZER = AutoTokenizer.from_pretrained(MODEL_PATH, trust_remote_code=True)",
                "",
            ]
        ),
        encoding="utf-8",
    )


_write_runtime_config()
sys.path.insert(0, str(RUNTIME_CONFIG_DIR))
sys.path.insert(1, str(OFFICIAL_VLLM_DIR))

from deepseek_ocr2 import DeepseekOCR2ForCausalLM  # noqa: E402
from fastapi import FastAPI, HTTPException  # noqa: E402
from PIL import Image, ImageOps  # noqa: E402
from process.image_process import DeepseekOCR2Processor  # noqa: E402
from process.ngram_norepeat import NoRepeatNGramLogitsProcessor  # noqa: E402
from pydantic import BaseModel, Field  # noqa: E402
from uvicorn import Config, Server  # noqa: E402
from vllm import AsyncLLMEngine, SamplingParams  # noqa: E402
from vllm.engine.arg_utils import AsyncEngineArgs  # noqa: E402
from vllm.model_executor.models.registry import ModelRegistry  # noqa: E402

ModelRegistry.register_model("DeepseekOCR2ForCausalLM", DeepseekOCR2ForCausalLM)

_engine: AsyncLLMEngine | None = None
_processor: DeepseekOCR2Processor | None = None


def _engine_args() -> AsyncEngineArgs:
    return AsyncEngineArgs(
        model=str(Path(MODEL_PATH).resolve()),
        hf_overrides={"architectures": ["DeepseekOCR2ForCausalLM"]},
        dtype=DTYPE,
        max_model_len=MAX_MODEL_LEN,
        enforce_eager=False,
        trust_remote_code=True,
        tensor_parallel_size=TENSOR_PARALLEL_SIZE,
        gpu_memory_utilization=GPU_MEMORY_UTILIZATION,
    )


@asynccontextmanager
async def _lifespan(_application: FastAPI) -> AsyncIterator[None]:
    global _engine, _processor
    _engine = AsyncLLMEngine.from_engine_args(_engine_args())
    _processor = DeepseekOCR2Processor()
    try:
        yield
    finally:
        _engine = None
        _processor = None


app = FastAPI(
    title="Wendao DeepSeek-OCR-2 OpenAI Adapter",
    lifespan=_lifespan,
)


class ChatCompletionRequest(BaseModel):
    model: str | None = None
    messages: list[dict[str, Any]]
    max_tokens: int | None = Field(default=None, alias="max_tokens")
    temperature: float | None = None


@app.get("/v1/models")
async def _models() -> dict[str, Any]:
    return {
        "object": "list",
        "data": [{"id": MODEL_NAME, "object": "model", "owned_by": "wendao"}],
    }


@app.post("/v1/chat/completions")
async def _chat_completions(request: ChatCompletionRequest) -> dict[str, Any]:
    if _engine is None or _processor is None:
        raise HTTPException(status_code=503, detail="OCR2 engine is not ready")
    prompt, image = _extract_prompt_and_image(request.messages)
    if not prompt:
        prompt = PROMPT
    engine_request: dict[str, Any]
    if image is not None and "<image>" in prompt:
        image_features = _processor.tokenize_with_images(
            images=[image],
            bos=True,
            eos=True,
            cropping=CROP_MODE,
        )
        engine_request = {
            "prompt": prompt,
            "multi_modal_data": {"image": image_features},
        }
    else:
        engine_request = {"prompt": prompt}

    logits_processors = []
    if SKIP_REPEAT:
        logits_processors.append(
            NoRepeatNGramLogitsProcessor(
                ngram_size=20,
                window_size=90,
                whitelist_token_ids={128821, 128822},
            )
        )
    sampling_params = SamplingParams(
        temperature=TEMPERATURE if request.temperature is None else request.temperature,
        max_tokens=request.max_tokens or MAX_TOKENS,
        logits_processors=logits_processors,
        skip_special_tokens=False,
    )
    output_text = ""
    async for request_output in _engine.generate(
        engine_request,
        sampling_params,
        f"ocr2-{uuid.uuid4()}",
    ):
        if request_output.outputs:
            output_text = request_output.outputs[0].text
    created = int(time.time())
    return {
        "id": f"chatcmpl-{uuid.uuid4().hex}",
        "object": "chat.completion",
        "created": created,
        "model": request.model or MODEL_NAME,
        "choices": [
            {
                "index": 0,
                "message": {"role": "assistant", "content": output_text},
                "finish_reason": "stop",
            }
        ],
    }


def _extract_prompt_and_image(
    messages: list[dict[str, Any]],
) -> tuple[str, Image.Image | None]:
    text_parts: list[str] = []
    image: Image.Image | None = None
    for message in messages:
        content = message.get("content")
        if isinstance(content, str):
            text_parts.append(content)
            continue
        if not isinstance(content, list):
            continue
        for item in content:
            if not isinstance(item, dict):
                continue
            if item.get("type") == "text":
                text_parts.append(str(item.get("text") or ""))
            elif item.get("type") == "image_url":
                image_url = item.get("image_url")
                if isinstance(image_url, dict):
                    image = _decode_data_url(str(image_url.get("url") or ""))
    return "\n".join(part for part in text_parts if part.strip()), image


def _decode_data_url(value: str) -> Image.Image:
    prefix = "base64,"
    if prefix not in value:
        raise HTTPException(
            status_code=400, detail="only base64 data image URLs are supported"
        )
    image_bytes = base64.b64decode(value.split(prefix, 1)[1], validate=True)
    return ImageOps.exif_transpose(Image.open(BytesIO(image_bytes))).convert("RGB")


async def _serve() -> None:
    server = Server(Config(app, host=HOST, port=PORT, log_level="info"))
    await server.serve()


if __name__ == "__main__":
    asyncio.run(_serve())
