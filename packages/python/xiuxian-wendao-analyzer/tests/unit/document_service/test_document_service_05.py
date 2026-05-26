"""document_service test slice 5."""

from __future__ import annotations

import json
import sys
from pathlib import Path

from xiuxian_wendao_analyzer import document_service_cli
from xiuxian_wendao_analyzer.audio_backend import manager as audio_manager
from xiuxian_wendao_analyzer.audio_backend.manager import AudioBackendOptions
from xiuxian_wendao_analyzer.document_service_cli import (
    build_document_extract_argument_parser,
    resolve_audio_backend_action,
    resolve_ocr2_backend_action,
)
from xiuxian_wendao_analyzer.document_service_startup_log import (
    document_extract_startup_log_payload,
    write_document_extract_startup_log,
)
from xiuxian_wendao_analyzer.ocr2_backend import manager_launch
from xiuxian_wendao_analyzer.ocr2_backend.manager import (
    DEFAULT_VLLM_PACKAGE,
    GENERIC_VLLM_REPO_ID,
    METAL_MLX_REPO_ID,
    Ocr2BackendOptions,
    build_start_backend_launch,
)


def _project_root() -> Path:
    for parent in Path(__file__).resolve().parents:
        if (parent / "justfile").is_file():
            return parent
    raise AssertionError("project root not found")


def test_ocr2_justfile_recipes_delegate_to_analyzer_cli() -> None:
    justfile = (_project_root() / "justfile").read_text(encoding="utf-8")

    assert (
        "uv run --package xiuxian-wendao-analyzer wendao-document-extract --ocr2-fetch-models"
    ) in justfile
    assert "--ocr2-start-backend" in justfile
    assert "--ocr2-install-vllm-metal" in justfile
    assert "--ocr2-probe-vllm-metal" in justfile
    assert "bash scripts/ocr" not in justfile
    assert "huggingface-cli download" not in justfile
    assert "vllm serve" not in justfile


def test_ocr2_backend_cli_flags_parse_as_actions() -> None:
    parser = build_document_extract_argument_parser()
    help_text = parser.format_help()

    assert "--ocr2-fetch-models" in help_text
    assert "--ocr2-start-backend" in help_text
    assert "--ocr2-install-vllm-metal" in help_text
    assert "--ocr2-probe-vllm-metal" in help_text

    fetch_args = parser.parse_args(
        [
            "--ocr2-fetch-models",
            "--ocr2-repo-id",
            "owner/model",
            "--ocr2-model-dir",
            "ocr2",
        ]
    )
    assert resolve_ocr2_backend_action(fetch_args) == "fetch-models"
    assert fetch_args.ocr2_repo_id == "owner/model"
    assert fetch_args.ocr2_model_dir == "ocr2"

    start_args = parser.parse_args(
        [
            "--ocr2-start-backend",
            "--ocr2-model-path",
            "models/ocr2",
            "--ocr2-quantization",
            "awq",
            "--ocr2-backend-runner",
            "generic-vllm",
        ]
    )
    assert resolve_ocr2_backend_action(start_args) == "start-backend"
    assert start_args.ocr2_model_path == "models/ocr2"
    assert start_args.ocr2_quantization == "awq"
    assert start_args.ocr2_backend_runner == "generic-vllm"


def test_audio_backend_cli_flags_parse_as_actions() -> None:
    parser = build_document_extract_argument_parser()
    help_text = parser.format_help()

    assert "--audio-probe-local-backend" in help_text
    assert "--audio-start-backend" in help_text

    probe_args = parser.parse_args(
        ["--audio-probe-local-backend", "--audio-backend-runner", "qwen3-asr-mlx"]
    )
    assert resolve_audio_backend_action(probe_args) == "probe-local"
    assert probe_args.audio_backend_runner == "qwen3-asr-mlx"

    start_args = parser.parse_args(
        [
            "--audio-start-backend",
            "--audio-backend-runner",
            "qwen3-asr-mlx",
            "--audio-backend-model-path",
            "Qwen/Qwen3-ASR-1.7B",
            "--audio-backend-host",
            "127.0.0.1",
            "--audio-backend-port",
            "8010",
        ]
    )
    assert resolve_audio_backend_action(start_args) == "start-backend"
    assert start_args.audio_backend_model_path == "Qwen/Qwen3-ASR-1.7B"


def test_document_extract_startup_log_reports_redacted_runtime_config(
    monkeypatch,
) -> None:
    monkeypatch.setenv("WENDAO_AUDIO_HOSTED_PROVIDER", "openrouter")
    monkeypatch.setenv("WENDAO_AUDIO_HOSTED_MODEL", "qwen/qwen3-asr-flash")
    monkeypatch.setenv("OPENROUTER_API_KEY", "secret-token")
    monkeypatch.setenv("WENDAO_HOSTED_VLM_OCR_PROVIDER", "openrouter")
    monkeypatch.setenv("WENDAO_OPENROUTER_MODEL", "baidu/qianfan-ocr-fast")
    monkeypatch.setenv("WENDAO_HOSTED_VLM_OCR_TRACE_PATH", "/tmp/ocr-trace.jsonl")
    args = build_document_extract_argument_parser().parse_args(
        [
            "--host",
            "127.0.0.1",
            "--port",
            "50051",
            "--pdf-ocr-worker",
            "docling",
            "--pdf-ocr-workers",
            "auto",
            "--audio-worker",
            "hosted",
            "--audio-workers",
            "auto",
        ]
    )

    payload = document_extract_startup_log_payload(
        args,
        location="grpc://127.0.0.1:50051",
        prewarmed_converter_ready=False,
    )

    assert payload["schema"] == "xiuxian_wendao.analyzer_document_extract_startup.v1"
    assert payload["location"] == "grpc://127.0.0.1:50051"
    assert payload["pdfOcr"]["worker"] == "docling"
    assert payload["pdfOcr"]["hostedVlm"]["provider"] == "openrouter"
    assert payload["pdfOcr"]["hostedVlm"]["openRouterModel"] == "baidu/qianfan-ocr-fast"
    assert payload["pdfOcr"]["hostedVlm"]["apiKeyConfigured"] is True
    assert payload["pdfOcr"]["hostedVlm"]["tracePathConfigured"] is True
    assert payload["audio"]["worker"] == "hosted-audio-transcript-v1"
    assert payload["audio"]["hosted"]["active"] is True
    assert payload["audio"]["hosted"]["provider"] == "openrouter"
    assert payload["audio"]["hosted"]["model"] == "qwen/qwen3-asr-flash"
    assert payload["audio"]["hosted"]["apiKeyConfigured"] is True
    assert "secret-token" not in json.dumps(payload)


def test_document_extract_startup_log_writes_single_parseable_line() -> None:
    class Buffer:
        def __init__(self) -> None:
            self.text = ""
            self.flushed = False

        def write(self, value: str) -> None:
            self.text += value

        def flush(self) -> None:
            self.flushed = True

    buffer = Buffer()
    args = build_document_extract_argument_parser().parse_args([])

    write_document_extract_startup_log(
        buffer,
        args,
        location="grpc://0.0.0.0:50051",
        prewarmed_converter_ready=True,
    )

    prefix, payload = buffer.text.strip().split(" ", 1)
    assert prefix == "WENDAO_ANALYZER_STARTUP"
    assert json.loads(payload)["prewarm"]["converterReady"] is True
    assert buffer.flushed is True


def test_audio_backend_launch_uses_qwen3_asr_adapter_on_macos(monkeypatch) -> None:
    monkeypatch.setattr(audio_manager, "is_macos_apple_silicon", lambda: True)
    monkeypatch.setattr(audio_manager.shutil, "which", lambda name: "/usr/bin/ffmpeg")

    launch = audio_manager.build_start_backend_launch(
        AudioBackendOptions(
            model_path="Qwen/Qwen3-ASR-1.7B",
            backend_runner="qwen3-asr-mlx",
            host="127.0.0.1",
            port="8010",
        )
    )

    assert launch.runner == "qwen3-asr-mlx"
    command = " ".join(launch.command)
    assert "uv run --no-project --with mlx-qwen3-asr" in command
    assert "qwen3_asr_mlx_openai_adapter.py" in command
    assert launch.env_updates["WENDAO_AUDIO_LOCAL_DEVICE"] == "metal"
    assert launch.env_updates["WENDAO_AUDIO_LOCAL_MODEL"] == "qwen3-asr-1.7b-mlx"
    assert launch.env_updates["WENDAO_AUDIO_LOCAL_MODEL_PATH"] == ("Qwen/Qwen3-ASR-1.7B")


def test_audio_backend_launch_requires_ffmpeg_for_qwen3_asr_mlx(
    monkeypatch,
) -> None:
    monkeypatch.setattr(audio_manager, "is_macos_apple_silicon", lambda: True)
    monkeypatch.setattr(audio_manager.shutil, "which", lambda name: None)

    try:
        audio_manager.build_start_backend_launch(
            AudioBackendOptions(backend_runner="qwen3-asr-mlx")
        )
    except audio_manager.AudioBackendError as exc:
        assert "requires ffmpeg on PATH" in str(exc)
        assert "direnv/devenv" in str(exc)
    else:
        raise AssertionError("Qwen3-ASR MLX launch should require ffmpeg")


def test_audio_backend_rejects_firered_as_metal_runner() -> None:
    try:
        audio_manager.build_start_backend_launch(AudioBackendOptions(backend_runner="fireredasr2s"))
    except audio_manager.AudioBackendError as exc:
        assert "CUDA-only" in str(exc)
    else:
        raise AssertionError("FireRedASR2S should not be a Metal audio backend")


def test_ocr2_cli_action_runs_backend_manager_without_starting_service(
    monkeypatch,
) -> None:
    calls: list[tuple[str, Ocr2BackendOptions]] = []

    def fake_run(action: str, options: Ocr2BackendOptions) -> int:
        calls.append((action, options))
        return 42

    monkeypatch.setattr(document_service_cli, "run_ocr2_backend_action", fake_run)
    monkeypatch.setattr(
        sys,
        "argv",
        [
            "wendao-document-extract",
            "--ocr2-start-backend",
            "--ocr2-model-path",
            "models/current",
            "--ocr2-quantization",
            "awq",
            "--ocr2-backend-runner",
            "generic-vllm",
        ],
    )

    assert document_service_cli.document_extract_service_main() == 42
    assert calls == [
        (
            "start-backend",
            Ocr2BackendOptions(
                model_path="models/current",
                quantization="awq",
                backend_runner="generic-vllm",
            ),
        )
    ]


def test_audio_cli_action_runs_backend_manager_without_starting_service(
    monkeypatch,
) -> None:
    calls: list[tuple[str, AudioBackendOptions]] = []

    def fake_run(action: str, options: AudioBackendOptions) -> int:
        calls.append((action, options))
        return 43

    monkeypatch.setattr(document_service_cli, "run_audio_backend_action", fake_run)
    monkeypatch.setattr(
        sys,
        "argv",
        [
            "wendao-document-extract",
            "--audio-start-backend",
            "--audio-backend-runner",
            "qwen3-asr-mlx",
            "--audio-backend-model-path",
            "Qwen/Qwen3-ASR-1.7B",
            "--audio-backend-host",
            "127.0.0.1",
            "--audio-backend-port",
            "8010",
        ],
    )

    assert document_service_cli.document_extract_service_main() == 43
    assert calls == [
        (
            "start-backend",
            AudioBackendOptions(
                model_path="Qwen/Qwen3-ASR-1.7B",
                backend_runner="qwen3-asr-mlx",
                host="127.0.0.1",
                port="8010",
            ),
        )
    ]


def test_ocr2_manager_keeps_backend_contract_defaults(monkeypatch) -> None:
    monkeypatch.setattr(manager_launch.shutil, "which", lambda name: None)

    launch = build_start_backend_launch(
        Ocr2BackendOptions(
            model_path="models/deepseek-ocr2-current",
            quantization="awq",
            backend_runner="generic-vllm",
        )
    )
    command = " ".join(launch.command)

    assert GENERIC_VLLM_REPO_ID == "richarddavison/DeepSeek-OCR-2-FP8"
    assert METAL_MLX_REPO_ID == "mlx-community/DeepSeek-OCR-2-bf16"
    assert DEFAULT_VLLM_PACKAGE == "vllm>=0.20.1"
    assert launch.runner == "generic-vllm"
    assert "uv run --no-project --with vllm>=0.20.1" in command
    assert "vllm serve models/deepseek-ocr2-current" in command
    assert "--trust-remote-code" in launch.command
    assert "--quantization" in launch.command
    assert "awq" in launch.command
    assert "vllm.model_executor.models.deepseek_ocr:NGramPerReqLogitsProcessor" in launch.command
    assert "--no-enable-prefix-caching" in launch.command
    assert "--mm-processor-cache-gb" in launch.command


def test_ocr2_backend_modules_replace_root_scripts() -> None:
    project_root = _project_root()
    package_dir = (
        project_root
        / "packages/python/xiuxian-wendao-analyzer/src/xiuxian_wendao_analyzer/ocr2_backend"
    )

    assert not (project_root / "scripts/ocr/fetch_deepseek_ocr2_model.sh").exists()
    assert (package_dir / "manager.py").is_file()
    assert (package_dir / "mlx_vlm_openai_adapter.py").is_file()
    assert (package_dir / "official_vllm_openai_adapter.py").is_file()

    mlx_adapter = (package_dir / "mlx_vlm_openai_adapter.py").read_text(encoding="utf-8")
    official_adapter = (package_dir / "official_vllm_openai_adapter.py").read_text(encoding="utf-8")
    manager_source = (package_dir / "manager.py").read_text(encoding="utf-8")
    launch_source = (package_dir / "manager_launch.py").read_text(encoding="utf-8")

    assert "from mlx_vlm import generate, load" in mlx_adapter
    assert "apply_chat_template" in mlx_adapter
    assert "trust_remote_code=False" in mlx_adapter
    assert "lifespan=_lifespan" in mlx_adapter
    assert 'ModelRegistry.register_model("DeepseekOCR2ForCausalLM"' in official_adapter
    assert "from ..local_backend import (" in launch_source
    assert "exec_backend_launch" in launch_source
    assert 'module_path(__file__, "official_vllm_openai_adapter.py")' in launch_source
    assert 'module_path(__file__, "mlx_vlm_openai_adapter.py")' in launch_source
    assert "vllm-omni" not in manager_source
    assert "vllm-omni" not in launch_source
