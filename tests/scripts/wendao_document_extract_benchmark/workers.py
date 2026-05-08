"""Local Python document worker startup helpers."""

from __future__ import annotations

from dataclasses import dataclass

from .common import (
    Path,
    os,
    subprocess,
)
from .constants import (
    OPENROUTER_LEGACY_PUBLIC_API_KEY_ENV,
    OPENROUTER_PUBLIC_API_KEY_ENV,
    OPENROUTER_STANDARD_API_KEY_ENVS,
)
from .http_status import pick_free_port
from .processes import start_logged_process
from .server_code import fixture_server_code, real_docling_server_code

OPENROUTER_OCR_SMOKE_MODEL = "baidu/qianfan-ocr-fast:free"
HOSTED_VLM_OCR_TRACE_PATH_ENV = "WENDAO_HOSTED_VLM_OCR_TRACE_PATH"
PDF_OCR_PREWARM_ENV_KEYS = frozenset(
    {
        "WENDAO_PDF_OCR_PREWARM_PROFILES",
        "WENDAO_PDF_OCR_PREWARM_SOURCE_PATH",
        "WENDAO_PDF_OCR_PREWARM_PAGE_INDICES",
        "WENDAO_PDF_OCR_PREWARM_PAGE_INDEX",
    }
)


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
    hosted_vlm_ocr_env: dict[str, str] | None = None,
    pdf_ocr_prewarm_endpoint_count: int | None = None,
    log_dir: Path | None = None,
) -> list[PythonWorkerServer]:
    endpoint_count = validate_endpoint_count(endpoint_count)
    if (
        pdf_ocr_prewarm_endpoint_count is not None
        and pdf_ocr_prewarm_endpoint_count < 1
    ):
        raise SystemExit("--pdf-ocr-prewarm-endpoint-count must be at least 1")
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
            hosted_vlm_ocr_env=hosted_vlm_ocr_env_for_worker(
                hosted_vlm_ocr_env,
                worker_index=index,
                prewarm_endpoint_count=pdf_ocr_prewarm_endpoint_count,
                log_dir=log_dir,
                process_name=name,
            ),
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


def hosted_vlm_ocr_env_for_worker(
    hosted_vlm_ocr_env: dict[str, str] | None,
    *,
    worker_index: int,
    prewarm_endpoint_count: int | None,
    log_dir: Path | None,
    process_name: str,
) -> dict[str, str]:
    env = dict(hosted_vlm_ocr_env or {})
    if prewarm_endpoint_count is not None and worker_index >= prewarm_endpoint_count:
        for key in PDF_OCR_PREWARM_ENV_KEYS:
            env.pop(key, None)
    return hosted_vlm_ocr_trace_env(env, log_dir=log_dir, process_name=process_name)


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
    hosted_vlm_ocr_env: dict[str, str] | None = None,
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
    worker_hosted_vlm_ocr_env = hosted_vlm_ocr_trace_env(
        hosted_vlm_ocr_env,
        log_dir=effective_log_dir,
        process_name=process_name,
    )
    process_env = None
    if worker_hosted_vlm_ocr_env:
        process_env = os.environ.copy()
        process_env.update(worker_hosted_vlm_ocr_env)
    return start_logged_process(
        command,
        log_dir=effective_log_dir,
        name=process_name,
        env=process_env,
    )


def hosted_vlm_ocr_trace_env(
    hosted_vlm_ocr_env: dict[str, str] | None,
    *,
    log_dir: Path | None,
    process_name: str,
) -> dict[str, str]:
    env = dict(hosted_vlm_ocr_env or {})
    if log_dir is not None:
        env.setdefault(
            HOSTED_VLM_OCR_TRACE_PATH_ENV,
            str(log_dir / f"{process_name}.hosted-vlm-ocr.jsonl"),
        )
    return env


def hosted_vlm_ocr_process_env(args: object) -> dict[str, str]:
    env = {}
    mappings = {
        "hosted_vlm_ocr_provider": "WENDAO_HOSTED_VLM_OCR_PROVIDER",
        "hosted_vlm_ocr_base_url": "WENDAO_HOSTED_VLM_OCR_BASE_URL",
        "hosted_vlm_ocr_model": "WENDAO_HOSTED_VLM_OCR_MODEL",
        "hosted_vlm_ocr_prompt": "WENDAO_HOSTED_VLM_OCR_PROMPT",
        "hosted_vlm_ocr_max_tokens": "WENDAO_HOSTED_VLM_OCR_MAX_TOKENS",
        "hosted_vlm_ocr_region_max_tokens": ("WENDAO_HOSTED_VLM_OCR_REGION_MAX_TOKENS"),
        "hosted_vlm_ocr_region_composite_size": (
            "WENDAO_HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE"
        ),
        "hosted_vlm_ocr_region_atlas_mode": "WENDAO_HOSTED_VLM_OCR_REGION_ATLAS_MODE",
        "hosted_vlm_ocr_scaffold_mode": "WENDAO_HOSTED_VLM_OCR_SCAFFOLD_MODE",
        "hosted_vlm_ocr_image_optimization": (
            "WENDAO_HOSTED_VLM_OCR_IMAGE_OPTIMIZATION"
        ),
        "hosted_vlm_ocr_timeout_seconds": "WENDAO_HOSTED_VLM_OCR_TIMEOUT_SECONDS",
        "hosted_vlm_ocr_request_concurrency": (
            "WENDAO_HOSTED_VLM_OCR_REQUEST_CONCURRENCY"
        ),
        "hosted_vlm_ocr_speculative_retry_delay_seconds": (
            "WENDAO_HOSTED_VLM_OCR_SPECULATIVE_RETRY_DELAY_SECONDS"
        ),
        "hosted_vlm_ocr_page_window_size": "WENDAO_HOSTED_VLM_OCR_PAGE_WINDOW_SIZE",
        "pdf_ocr_backend_text_page_fallback": (
            "WENDAO_PDF_OCR_BACKEND_TEXT_PAGE_FALLBACK"
        ),
        "pdf_ocr_backend_text_empty_page": "WENDAO_PDF_OCR_BACKEND_TEXT_EMPTY_PAGE",
        "openrouter_model": "WENDAO_OPENROUTER_MODEL",
        "openrouter_http_referer": "WENDAO_OPENROUTER_HTTP_REFERER",
        "openrouter_title": "WENDAO_OPENROUTER_TITLE",
    }
    for attr, key in mappings.items():
        value = getattr(args, attr, None)
        if (
            attr
            in {
                "pdf_ocr_backend_text_page_fallback",
                "pdf_ocr_backend_text_empty_page",
            }
            and value == "disabled"
        ):
            continue
        if value is not None:
            env[key] = str(value)
    prewarm_profiles = getattr(args, "pdf_ocr_prewarm_profile", [])
    if prewarm_profiles:
        env["WENDAO_PDF_OCR_PREWARM_PROFILES"] = ",".join(
            dict.fromkeys(str(profile) for profile in prewarm_profiles)
        )
    prewarm_source_path = getattr(args, "pdf_ocr_prewarm_source_path", None)
    if prewarm_source_path:
        env["WENDAO_PDF_OCR_PREWARM_SOURCE_PATH"] = str(prewarm_source_path)
    prewarm_page_indices = getattr(args, "pdf_ocr_prewarm_page_indices", None)
    if prewarm_page_indices:
        env["WENDAO_PDF_OCR_PREWARM_PAGE_INDICES"] = str(prewarm_page_indices)
    prewarm_page_index = getattr(args, "pdf_ocr_prewarm_page_index", None)
    if prewarm_page_indices is None and prewarm_page_index is not None:
        env["WENDAO_PDF_OCR_PREWARM_PAGE_INDEX"] = str(prewarm_page_index)
    if (
        env.get("WENDAO_HOSTED_VLM_OCR_PROVIDER") == "openrouter"
        and "WENDAO_HOSTED_VLM_OCR_MODEL" not in env
        and "WENDAO_OPENROUTER_MODEL" not in env
    ):
        env["WENDAO_OPENROUTER_MODEL"] = OPENROUTER_OCR_SMOKE_MODEL
    if env.get("WENDAO_HOSTED_VLM_OCR_PROVIDER") == "openrouter":
        forward_legacy_openrouter_api_key(env)
    return env


def forward_legacy_openrouter_api_key(env: dict[str, str]) -> None:
    has_standard_key = any(
        (env.get(key) or os.environ.get(key) or "").strip()
        for key in OPENROUTER_STANDARD_API_KEY_ENVS
    )
    if has_standard_key:
        return
    legacy_key = os.environ.get(OPENROUTER_LEGACY_PUBLIC_API_KEY_ENV, "")
    if not legacy_key.strip():
        return
    env[OPENROUTER_PUBLIC_API_KEY_ENV] = strip_wrapping_quotes(legacy_key.strip())


def strip_wrapping_quotes(value: str) -> str:
    if len(value) >= 2 and value[0] == value[-1] and value[0] in {"'", '"'}:
        return value[1:-1]
    return value


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
