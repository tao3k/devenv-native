"""Local Python document worker startup helpers."""

from __future__ import annotations

from dataclasses import dataclass

from .common import (
    Path,
    os,
    subprocess,
)
from .http_status import pick_free_port
from .processes import start_logged_process
from .server_code import fixture_server_code, real_docling_server_code

OPENROUTER_OCR_SMOKE_MODEL = "baidu/qianfan-ocr-fast:free"


@dataclass(frozen=True)
class PythonWorkerServer:
    process: subprocess.Popen[str]
    host: str
    port: int
    endpoint_url: str


def resolve_local_python_ocr_endpoint_count(args: object) -> int:
    raw_count = getattr(args, "local_python_ocr_endpoint_count", "auto")
    if isinstance(raw_count, int):
        return validate_endpoint_count(raw_count)

    raw_text = str(raw_count).strip().lower()
    if raw_text != "auto":
        try:
            return validate_endpoint_count(int(raw_text))
        except ValueError as exc:
            raise SystemExit(
                "--local-python-ocr-endpoint-count must be a positive integer or `auto`"
            ) from exc

    if getattr(args, "external_endpoint", False):
        return 1
    if not should_auto_fanout_local_ocr_endpoints(args):
        return 1
    return ceil_sqrt(max(os.cpu_count() or 1, 1))


def validate_endpoint_count(endpoint_count: int) -> int:
    if endpoint_count < 1:
        raise SystemExit("--local-python-ocr-endpoint-count must be at least 1")
    return endpoint_count


def should_auto_fanout_local_ocr_endpoints(args: object) -> bool:
    return (
        bool(getattr(args, "real_docling", False))
        and getattr(args, "flight_mode", "") == "hybrid-page-ocr"
        and getattr(args, "pdf_ocr_worker", "") == "docling"
    )


def ceil_sqrt(value: int) -> int:
    if value <= 1:
        return value
    root = 1
    while root * root < value:
        root += 1
    return root


def start_server_pool(
    host: str,
    port: int,
    *,
    endpoint_count: int,
    real_docling: bool,
    real_fixture_root: Path | None,
    include_audio: bool,
    converter_count_path: Path | None,
    pdf_ocr_worker: str = "skip",
    pdf_ocr_workers: str = "auto",
    python_uv_package: str | None = "xiuxian-wendao-analyzer",
    python_uv_extras: list[str] | None = None,
    deepseek_ocr2_env: dict[str, str] | None = None,
    log_dir: Path | None = None,
) -> list[PythonWorkerServer]:
    endpoint_count = validate_endpoint_count(endpoint_count)
    ports = [port]
    while len(ports) < endpoint_count:
        candidate = pick_free_port(host)
        if candidate not in ports:
            ports.append(candidate)

    count_root = None
    if converter_count_path is not None and endpoint_count > 1:
        count_root = converter_count_path
        count_root.mkdir(parents=True, exist_ok=True)

    workers = []
    for index, worker_port in enumerate(ports):
        worker_count_path = converter_count_path
        if count_root is not None:
            worker_count_path = count_root / f"python-worker-{index}.txt"
        name = "python-worker" if endpoint_count == 1 else f"python-worker-{index}"
        process = start_server(
            host,
            worker_port,
            real_docling=real_docling,
            real_fixture_root=real_fixture_root,
            include_audio=include_audio,
            converter_count_path=worker_count_path,
            pdf_ocr_worker=pdf_ocr_worker,
            pdf_ocr_workers=pdf_ocr_workers,
            python_uv_package=python_uv_package,
            python_uv_extras=python_uv_extras,
            deepseek_ocr2_env=deepseek_ocr2_env,
            log_dir=log_dir,
            process_name=name,
        )
        workers.append(
            PythonWorkerServer(
                process=process,
                host=host,
                port=worker_port,
                endpoint_url=f"http://{host}:{worker_port}",
            )
        )
    return workers


def start_server(
    host: str,
    port: int,
    *,
    real_docling: bool,
    real_fixture_root: Path | None,
    include_audio: bool,
    converter_count_path: Path | None,
    pdf_ocr_worker: str = "skip",
    pdf_ocr_workers: str = "auto",
    python_uv_package: str | None = "xiuxian-wendao-analyzer",
    python_uv_extras: list[str] | None = None,
    deepseek_ocr2_env: dict[str, str] | None = None,
    log_dir: Path | None = None,
    process_name: str = "python-worker",
) -> subprocess.Popen[str]:
    if pdf_ocr_worker == "docling" and not real_docling:
        raise SystemExit("--pdf-ocr-worker docling requires --real-docling")
    if real_docling:
        command = python_worker_command(
            real_docling_server_code(
                host,
                port,
                real_fixture_root,
                include_audio,
                converter_count_path,
                pdf_ocr_worker,
                pdf_ocr_workers,
            ),
            uv_package=python_uv_package,
            uv_extras=python_uv_extras,
        )
    else:
        command = python_worker_command(
            fixture_server_code(
                host,
                port,
                converter_count_path,
                pdf_ocr_worker,
                pdf_ocr_workers,
            ),
            uv_package=python_uv_package,
            uv_extras=python_uv_extras,
        )
    effective_log_dir = log_dir or (
        Path(os.environ.get("PRJ_RUNTIME_DIR", ".run"))
        / "document-extract-perf-process-logs"
    )
    process_env = None
    if deepseek_ocr2_env:
        process_env = os.environ.copy()
        process_env.update(deepseek_ocr2_env)
    return start_logged_process(
        command,
        log_dir=effective_log_dir,
        name=process_name,
        env=process_env,
    )


def deepseek_ocr2_process_env(args: object) -> dict[str, str]:
    env = {}
    mappings = {
        "deepseek_ocr2_provider": "WENDAO_DEEPSEEK_OCR2_PROVIDER",
        "deepseek_ocr2_base_url": "WENDAO_DEEPSEEK_OCR2_BASE_URL",
        "deepseek_ocr2_model": "WENDAO_DEEPSEEK_OCR2_MODEL",
        "deepseek_ocr2_prompt": "WENDAO_DEEPSEEK_OCR2_PROMPT",
        "deepseek_ocr2_max_tokens": "WENDAO_DEEPSEEK_OCR2_MAX_TOKENS",
        "deepseek_ocr2_timeout_seconds": "WENDAO_DEEPSEEK_OCR2_TIMEOUT_SECONDS",
        "openrouter_model": "WENDAO_OPENROUTER_MODEL",
        "openrouter_http_referer": "WENDAO_OPENROUTER_HTTP_REFERER",
        "openrouter_title": "WENDAO_OPENROUTER_TITLE",
    }
    for attr, key in mappings.items():
        value = getattr(args, attr, None)
        if value is not None:
            env[key] = str(value)
    if (
        env.get("WENDAO_DEEPSEEK_OCR2_PROVIDER") == "openrouter"
        and "WENDAO_DEEPSEEK_OCR2_MODEL" not in env
        and "WENDAO_OPENROUTER_MODEL" not in env
    ):
        env["WENDAO_OPENROUTER_MODEL"] = OPENROUTER_OCR_SMOKE_MODEL
    return env


def python_worker_command(
    code: str,
    *,
    uv_package: str | None,
    uv_extras: list[str] | None,
) -> list[str]:
    command = ["uv", "run"]
    if uv_package:
        command.extend(["--package", uv_package])
    for extra in uv_extras or []:
        command.extend(["--extra", extra])
    command.extend(["python", "-c", code])
    return command
