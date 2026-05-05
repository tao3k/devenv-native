"""document_service test slice 5."""

from __future__ import annotations

import sys
from pathlib import Path

from xiuxian_wendao_analyzer import document_service_cli
from xiuxian_wendao_analyzer.document_service_cli import (
    build_document_extract_argument_parser,
    resolve_ocr2_backend_action,
)
from xiuxian_wendao_analyzer.ocr2_backend import manager
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
        "uv run --package xiuxian-wendao-analyzer wendao-document-extract "
        "--ocr2-fetch-models"
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


def test_ocr2_manager_keeps_backend_contract_defaults(monkeypatch) -> None:
    monkeypatch.setattr(manager.shutil, "which", lambda name: None)

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
    assert (
        "vllm.model_executor.models.deepseek_ocr:NGramPerReqLogitsProcessor"
        in launch.command
    )
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

    mlx_adapter = (package_dir / "mlx_vlm_openai_adapter.py").read_text(
        encoding="utf-8"
    )
    official_adapter = (package_dir / "official_vllm_openai_adapter.py").read_text(
        encoding="utf-8"
    )
    manager_source = (package_dir / "manager.py").read_text(encoding="utf-8")

    assert "from mlx_vlm import generate, load" in mlx_adapter
    assert "apply_chat_template" in mlx_adapter
    assert "trust_remote_code=False" in mlx_adapter
    assert "lifespan=_lifespan" in mlx_adapter
    assert 'ModelRegistry.register_model("DeepseekOCR2ForCausalLM"' in official_adapter
    assert (
        'python", str(_module_path("official_vllm_openai_adapter.py"))'
        in manager_source
    )
    assert 'str(_module_path("mlx_vlm_openai_adapter.py"))' in manager_source
    assert "vllm-omni" not in manager_source
