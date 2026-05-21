from __future__ import annotations

import importlib
import importlib.metadata
import importlib.util
import json
import os
import platform
import subprocess
from pathlib import Path


def _package_version(name: str) -> str:
    try:
        return importlib.metadata.version(name)
    except importlib.metadata.PackageNotFoundError:
        return "missing"


def _module_state(name: str) -> str:
    if importlib.util.find_spec(name) is None:
        return "missing"
    try:
        importlib.import_module(name)
    except Exception as exc:
        return f"import-error:{exc.__class__.__name__}"
    return "present"


def _command_output(command: list[str]) -> str:
    timeout_seconds = float(
        os.environ.get("WENDAO_DEEPSEEK_OCR2_VLLM_METAL_PROBE_TIMEOUT_SECONDS", "120")
    )
    try:
        completed = subprocess.run(
            command,
            check=False,
            capture_output=True,
            text=True,
            timeout=timeout_seconds,
        )
    except Exception as exc:
        return f"error:{exc.__class__.__name__}"
    output = (completed.stdout or completed.stderr).strip().splitlines()
    if not output:
        return f"exit={completed.returncode}"
    return f"exit={completed.returncode} {output[0]}"


def main() -> int:
    vllm_bin = Path(os.environ.get("WENDAO_DEEPSEEK_OCR2_VLLM_METAL_BIN", ""))
    if not str(vllm_bin):
        vllm_home = Path(
            os.environ.get(
                "WENDAO_DEEPSEEK_OCR2_VLLM_METAL_HOME",
                str(Path.home() / ".venv-vllm-metal"),
            )
        )
        vllm_bin = vllm_home / "bin" / "vllm"

    is_macos_apple_silicon = platform.system() == "Darwin" and platform.machine() in {
        "arm64",
        "aarch64",
    }

    print(f"platform={platform.system()}-{platform.machine()}")
    print(f"macosAppleSilicon={str(is_macos_apple_silicon).lower()}")
    print(f"vllmBin={vllm_bin}")
    print(f"vllmCli={_command_output([str(vllm_bin), '--version'])}")
    print(f"package.vllm={_package_version('vllm')}")
    print(f"package.vllm-metal={_package_version('vllm-metal')}")
    print(f"package.mlx={_package_version('mlx')}")
    print(f"package.mlx-vlm={_package_version('mlx-vlm')}")
    print(f"module.vllm_metal={_module_state('vllm_metal')}")
    print(
        "module.vllm.model_executor.models.deepseek_ocr="
        f"{_module_state('vllm.model_executor.models.deepseek_ocr')}"
    )

    model_path = Path(
        os.environ.get(
            "WENDAO_DEEPSEEK_OCR2_MODEL_PATH",
            ".data/models/deepseek-ocr2-current",
        )
    )
    model_config_path = model_path / "config.json"
    if model_config_path.exists():
        config = json.loads(model_config_path.read_text(encoding="utf-8"))
        width = config.get("vision_config", {}).get("width")
        print(f"ocr2VisionWidthKind={type(width).__name__}")
        if isinstance(width, dict):
            print("ocr2MlxVlmCompatibility=blocked-deepencoderv2-vision-config")
        else:
            print("ocr2MlxVlmCompatibility=unknown")
    else:
        print("ocr2VisionWidthKind=missing-config")
        print("ocr2MlxVlmCompatibility=unknown")

    print(f"VLLM_METAL_USE_MLX={os.environ.get('VLLM_METAL_USE_MLX', 'unset')}")
    print(f"VLLM_MLX_DEVICE={os.environ.get('VLLM_MLX_DEVICE', 'unset')}")
    print(
        "VLLM_METAL_MULTIMODAL_MODE="
        f"{os.environ.get('VLLM_METAL_MULTIMODAL_MODE', 'unset')}"
    )

    allow_unsupported_vlm = (
        os.environ.get("WENDAO_DEEPSEEK_OCR2_VLLM_METAL_ALLOW_UNSUPPORTED_VLM", "0")
        == "1"
    )
    print("ocr2VlmDefault=blocked-by-vllm-metal-text-only-support")
    if allow_unsupported_vlm:
        print("ocr2VlmOverride=enabled")
        return 0
    print("ocr2VlmOverride=disabled")
    return 3


if __name__ == "__main__":
    raise SystemExit(main())
