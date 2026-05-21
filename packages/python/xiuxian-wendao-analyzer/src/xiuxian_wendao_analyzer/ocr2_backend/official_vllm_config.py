"""Runtime configuration for the official DeepSeek-OCR-2 vLLM adapter."""

from __future__ import annotations

import os
import sys
from pathlib import Path

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


def configure_official_vllm_imports() -> None:
    """Write runtime config and expose official OCR2 modules on sys.path."""

    _write_runtime_config()
    sys.path.insert(0, str(RUNTIME_CONFIG_DIR))
    sys.path.insert(1, str(OFFICIAL_VLLM_DIR))
