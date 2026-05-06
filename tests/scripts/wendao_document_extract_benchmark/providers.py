"""Local Rust provider and gateway startup helpers."""

from __future__ import annotations

from .common import (
    Path,
    argparse,
    json,
    subprocess,
    textwrap,
)
from .constants import OCR_SHARD_CACHE_ROOT_ENV
from .features import cargo_features_for_provider_mode, normalize_render_selection
from .pdf_render import build_hybrid_pdf_render_region_env, resolve_pdfium_library_path
from .processes import start_logged_process
from .runtime import resolve_project_root, rust_process_env


def rust_pdf_ocr_endpoint_pool(args: argparse.Namespace) -> str | None:
    endpoints = [
        endpoint.strip().rstrip("/")
        for endpoint in getattr(args, "rust_pdf_ocr_endpoint", [])
        if endpoint.strip()
    ]
    if not endpoints:
        return None
    return ",".join(endpoints)


def rust_document_extract_endpoint_pool(args: argparse.Namespace) -> str | None:
    endpoints = [
        endpoint.strip().rstrip("/")
        for endpoint in getattr(args, "rust_document_extract_endpoint", [])
        if endpoint.strip()
    ]
    if not endpoints:
        return None
    return ",".join(dict.fromkeys(endpoints))


def apply_rust_pdf_ocr_env(args: argparse.Namespace, env: dict[str, str]) -> None:
    rust_pdf_ocr_workers = getattr(args, "rust_pdf_ocr_workers", None)
    if rust_pdf_ocr_workers:
        env["WENDAO_DOCUMENT_EXTRACT_PDF_OCR_WORKERS"] = str(rust_pdf_ocr_workers)
    rust_pdf_ocr_source_range_workers = getattr(
        args,
        "rust_pdf_ocr_source_range_workers",
        None,
    )
    if rust_pdf_ocr_source_range_workers:
        env["WENDAO_DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS"] = str(
            rust_pdf_ocr_source_range_workers
        )
    rust_pdf_ocr_profile_planner = getattr(args, "rust_pdf_ocr_profile_planner", None)
    if rust_pdf_ocr_profile_planner and rust_pdf_ocr_profile_planner != "disabled":
        env["WENDAO_DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER"] = str(
            rust_pdf_ocr_profile_planner
        )
    rust_pdf_ocr2_render_dpi = getattr(args, "rust_pdf_ocr2_render_dpi", None)
    if rust_pdf_ocr2_render_dpi and rust_pdf_ocr2_render_dpi >= 300:
        env["WENDAO_DOCUMENT_EXTRACT_PDF_OCR2_RENDER_DPI"] = str(
            rust_pdf_ocr2_render_dpi
        )
    rust_pdf_ocr_region_context_ratio = getattr(
        args,
        "rust_pdf_ocr_region_context_ratio",
        None,
    )
    if rust_pdf_ocr_region_context_ratio is not None:
        env["WENDAO_DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO"] = str(
            rust_pdf_ocr_region_context_ratio
        )
    rust_pdf_ocr2_region_planner = getattr(args, "rust_pdf_ocr2_region_planner", None)
    if rust_pdf_ocr2_region_planner and rust_pdf_ocr2_region_planner != "disabled":
        env["WENDAO_DOCUMENT_EXTRACT_PDF_OCR2_REGION_PLANNER"] = str(
            rust_pdf_ocr2_region_planner
        )
    ocr_endpoint_pool = rust_pdf_ocr_endpoint_pool(args)
    if ocr_endpoint_pool:
        env["WENDAO_DOCUMENT_EXTRACT_PDF_OCR_ENDPOINTS"] = ocr_endpoint_pool


def start_rust_provider_server(
    args: argparse.Namespace,
    *,
    rust_host: str,
    rust_port: int,
    python_host: str,
    python_port: int,
    temp_root: Path,
    log_dir: Path | None = None,
) -> subprocess.Popen[str]:
    provider_root = temp_root / "rust-provider"
    provider_root.mkdir(parents=True, exist_ok=True)
    local_document_extract_endpoint = f"http://{python_host}:{python_port}"
    env = rust_process_env()
    pdfium_library_path = resolve_pdfium_library_path(args)
    ocr_shard_cache_root = getattr(
        args,
        "ocr_shard_cache_root",
        (temp_root / "ocr-shard-cache").resolve(),
    )
    env.update(
        {
            "WENDAO_DOCUMENT_EXTRACT_ENDPOINT": f"http://{python_host}:{python_port}",
            "WENDAO_DOCUMENT_EXTRACT_JOB_DB": str(provider_root / "jobs.duckdb"),
            "WENDAO_DOCUMENT_EXTRACT_ARTIFACT_ROOT": str(provider_root / "artifacts"),
            OCR_SHARD_CACHE_ROOT_ENV: str(ocr_shard_cache_root),
            "WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_SELECTION": normalize_render_selection(
                getattr(args, "hybrid_pdf_render_selection", "shard-fallback-pages")
            ),
        }
    )
    env["WENDAO_DOCUMENT_EXTRACT_ENDPOINTS"] = (
        rust_document_extract_endpoint_pool(args) or local_document_extract_endpoint
    )
    env.update(build_hybrid_pdf_render_region_env(args))
    if pdfium_library_path is not None:
        env["WENDAO_PDFIUM_LIBRARY_PATH"] = str(pdfium_library_path)
    apply_rust_pdf_ocr_env(args, env)
    rust_provider_bin = getattr(args, "rust_provider_bin", None)
    provider_args = [
        f"{rust_host}:{rust_port}",
        "alpha/repo",
        str(resolve_project_root()),
        "--schema-version=v2",
    ]
    if rust_provider_bin is not None:
        command = [str(rust_provider_bin), *provider_args]
    else:
        command = [
            args.cargo,
            "run",
            "-p",
            "xiuxian-wendao-studio",
            "--no-default-features",
            "--features",
            cargo_features_for_provider_mode(args.rust_provider_features, args),
            "--bin",
            "wendao_search_flight_server",
            "--",
            *provider_args,
        ]
    return start_logged_process(
        command,
        log_dir=log_dir or temp_root / "process-logs",
        name="rust-provider",
        env=env,
    )


def start_valkey_server(
    *,
    host: str,
    port: int,
    temp_root: Path,
    log_dir: Path | None = None,
) -> subprocess.Popen[str]:
    valkey_root = temp_root / "valkey"
    valkey_root.mkdir(parents=True, exist_ok=True)
    command = [
        "valkey-server",
        "--bind",
        host,
        "--port",
        str(port),
        "--dir",
        str(valkey_root),
        "--save",
        "",
        "--appendonly",
        "no",
        "--daemonize",
        "no",
        "--protected-mode",
        "no",
    ]
    return start_logged_process(
        command, log_dir=log_dir or temp_root / "process-logs", name="valkey"
    )


def start_gateway_server(
    args: argparse.Namespace,
    *,
    gateway_port: int,
    python_host: str,
    python_port: int,
    valkey_url: str,
    temp_root: Path,
    log_dir: Path | None = None,
) -> subprocess.Popen[str]:
    gateway_root = temp_root / "gateway"
    gateway_root.mkdir(parents=True, exist_ok=True)
    config_path = write_gateway_benchmark_config(gateway_root, valkey_url=valkey_url)
    local_document_extract_endpoint = f"http://{python_host}:{python_port}"
    env = rust_process_env()
    pdfium_library_path = resolve_pdfium_library_path(args)
    ocr_shard_cache_root = getattr(
        args,
        "ocr_shard_cache_root",
        (temp_root / "ocr-shard-cache").resolve(),
    )
    env.update(
        {
            "WENDAO_DOCUMENT_EXTRACT_ENDPOINT": f"http://{python_host}:{python_port}",
            "WENDAO_DOCUMENT_EXTRACT_JOB_DB": str(gateway_root / "jobs.duckdb"),
            "WENDAO_DOCUMENT_EXTRACT_ARTIFACT_ROOT": str(gateway_root / "artifacts"),
            OCR_SHARD_CACHE_ROOT_ENV: str(ocr_shard_cache_root),
            "WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_SELECTION": normalize_render_selection(
                getattr(args, "hybrid_pdf_render_selection", "shard-fallback-pages")
            ),
            "VALKEY_URL": valkey_url,
            "REDIS_URL": valkey_url,
            "XIUXIAN_WENDAO_SEARCH_PLANE_VALKEY_URL": valkey_url,
            "XIUXIAN_WENDAO_KNOWLEDGE_VALKEY_URL": valkey_url,
            "XIUXIAN_WENDAO_GATEWAY_BOOTSTRAP_BACKGROUND_INDEXING": "false",
        }
    )
    env["WENDAO_DOCUMENT_EXTRACT_ENDPOINTS"] = (
        rust_document_extract_endpoint_pool(args) or local_document_extract_endpoint
    )
    env.update(build_hybrid_pdf_render_region_env(args))
    if pdfium_library_path is not None:
        env["WENDAO_PDFIUM_LIBRARY_PATH"] = str(pdfium_library_path)
    apply_rust_pdf_ocr_env(args, env)
    command = [
        args.cargo,
        "run",
        "-p",
        "xiuxian-wendao-studio",
        "--no-default-features",
        "--features",
        cargo_features_for_provider_mode(args.gateway_features, args),
        "--bin",
        "wendao",
        "--",
        "--conf",
        str(config_path),
        "--root",
        str(resolve_project_root()),
        "gateway",
        "start",
        "--port",
        str(gateway_port),
    ]
    return start_logged_process(
        command,
        log_dir=log_dir or temp_root / "process-logs",
        name="gateway",
        env=env,
    )


def write_gateway_benchmark_config(config_root: Path, *, valkey_url: str) -> Path:
    config_path = config_root / "wendao.toml"
    quoted_valkey_url = json.dumps(valkey_url)
    config_path.write_text(
        textwrap.dedent(
            f"""
            [gateway]
            bind = "127.0.0.1"
            webhook_enabled = false

            [gateway.runtime]
            studio_request_timeout_secs = 300

            [search.cache]
            valkey_url = {quoted_valkey_url}

            [link_graph.cache]
            valkey_url = {quoted_valkey_url}
            key_prefix = "xiuxian_wendao:document_extract_perf"
            """
        ).lstrip(),
        encoding="utf-8",
    )
    return config_path
