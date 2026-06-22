"""Argument parser for the document extraction service CLI."""

from __future__ import annotations

import argparse

from .audio_shard_workers import (
    AUDIO_BACKEND_DOCLING,
    AUDIO_BACKEND_HOSTED,
    AUDIO_BACKEND_SKIP,
    AUDIO_WORKER_ENV,
)


def build_document_extract_argument_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Wendao document extraction Arrow Flight service")
    parser.add_argument("--host", default="0.0.0.0", help="Bind host")
    parser.add_argument("--port", type=int, default=50051, help="Bind port")
    parser.add_argument(
        "--pdf-ocr-worker",
        choices=("skip", "docling"),
        default="skip",
        help="OCR worker used by the internal /analysis/pdf-ocr-shards exchange",
    )
    parser.add_argument(
        "--pdf-ocr-workers",
        default="auto",
        help=(
            "Maximum Docling OCR shard workers for direct local requests. "
            "Rust providers may override this per request with "
            "x-wendao-pdf-ocr-workers."
        ),
    )
    parser.add_argument(
        "--audio-worker",
        choices=(AUDIO_BACKEND_SKIP, AUDIO_BACKEND_DOCLING, AUDIO_BACKEND_HOSTED),
        default=None,
        help=(
            "Audio worker used by the internal /analysis/audio-shards exchange. "
            f"Defaults to {AUDIO_WORKER_ENV} or skip for direct CLI runs; "
            "the managed Wendao analyzer service passes hosted and selects "
            "OpenRouter by default."
        ),
    )
    parser.add_argument(
        "--audio-workers",
        default="auto",
        help=(
            "Maximum audio shard workers for direct local/hosted requests. "
            "Rust providers may override this per request with "
            "x-wendao-audio-workers."
        ),
    )
    audio_actions = parser.add_mutually_exclusive_group()
    audio_actions.add_argument(
        "--audio-probe-local-backend",
        action="store_const",
        const="probe-local",
        dest="audio_backend_action",
        help="Probe analyzer-owned local audio backend readiness.",
    )
    audio_actions.add_argument(
        "--audio-start-backend",
        action="store_const",
        const="start-backend",
        dest="audio_backend_action",
        help="Start the platform-selected OpenAI-compatible local audio backend.",
    )
    parser.set_defaults(audio_backend_action=None)
    parser.add_argument(
        "--audio-backend-runner",
        choices=("auto", "qwen3-asr-mlx", "fireredasr2s"),
        default="auto",
        help="Local audio backend runner used by --audio-start-backend.",
    )
    parser.add_argument(
        "--audio-backend-model-path",
        default="",
        help="Local audio model path or MLX repo id used by --audio-start-backend.",
    )
    parser.add_argument(
        "--audio-backend-host",
        default="",
        help="Bind host for --audio-start-backend.",
    )
    parser.add_argument(
        "--audio-backend-port",
        default="",
        help="Bind port for --audio-start-backend.",
    )
    ocr2_actions = parser.add_mutually_exclusive_group()
    ocr2_actions.add_argument(
        "--ocr2-fetch-models",
        action="store_const",
        const="fetch-models",
        dest="ocr2_backend_action",
        help="Fetch prebuilt DeepSeek-OCR-2 artifacts through the analyzer-owned backend manager.",
    )
    ocr2_actions.add_argument(
        "--ocr2-install-vllm-metal",
        action="store_const",
        const="install-vllm-metal",
        dest="ocr2_backend_action",
        help="Install the local vLLM Metal runtime used by macOS OCR2 probes.",
    )
    ocr2_actions.add_argument(
        "--ocr2-probe-vllm-metal",
        action="store_const",
        const="probe-vllm-metal",
        dest="ocr2_backend_action",
        help="Probe local vLLM Metal readiness without loading OCR2 weights.",
    )
    ocr2_actions.add_argument(
        "--ocr2-start-backend",
        action="store_const",
        const="start-backend",
        dest="ocr2_backend_action",
        help="Start the platform-selected OpenAI-compatible DeepSeek-OCR-2 backend.",
    )
    parser.set_defaults(ocr2_backend_action=None)
    parser.add_argument(
        "--ocr2-repo-id",
        default="",
        help="Hugging Face repo id for --ocr2-fetch-models.",
    )
    parser.add_argument(
        "--ocr2-model-dir",
        default="",
        help="Model directory name under PRJ_DATA_HOME/models for --ocr2-fetch-models.",
    )
    parser.add_argument(
        "--ocr2-model-path",
        default="",
        help="Local model path used by --ocr2-start-backend.",
    )
    parser.add_argument(
        "--ocr2-quantization",
        default="auto",
        help="vLLM quantization mode used by --ocr2-start-backend.",
    )
    parser.add_argument(
        "--ocr2-backend-runner",
        choices=("auto", "mlx-vlm", "metal-vllm", "generic-vllm", "official-vllm"),
        default="auto",
        help="Local OCR2 backend runner used by --ocr2-start-backend.",
    )
    return parser
