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
from .features import (
    cargo_features_for_provider_mode,
    cargo_features_with_feature,
    normalize_render_selection,
)
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
    rust_pdf_docling_page_range_chunk_plan = getattr(
        args,
        "rust_pdf_docling_page_range_chunk_plan",
        None,
    )
    if rust_pdf_docling_page_range_chunk_plan:
        env["WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_PLAN"] = str(
            rust_pdf_docling_page_range_chunk_plan
        )
    rust_pdf_docling_page_range_profile = getattr(
        args,
        "rust_pdf_docling_page_range_profile",
        None,
    )
    if (
        rust_pdf_docling_page_range_profile
        and rust_pdf_docling_page_range_profile != "full"
    ):
        env["WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_PROFILE"] = str(
            rust_pdf_docling_page_range_profile
        )
    rust_pdf_docling_page_range_hedge_delay_ms = getattr(
        args,
        "rust_pdf_docling_page_range_hedge_delay_ms",
        None,
    )
    if (
        rust_pdf_docling_page_range_hedge_delay_ms
        and rust_pdf_docling_page_range_hedge_delay_ms > 0
    ):
        env["WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_HEDGE_DELAY_MS"] = str(
            rust_pdf_docling_page_range_hedge_delay_ms
        )
    rust_pdf_docling_page_range_structure_cost_budget = getattr(
        args,
        "rust_pdf_docling_page_range_structure_cost_budget",
        None,
    )
    if (
        rust_pdf_docling_page_range_structure_cost_budget
        and rust_pdf_docling_page_range_structure_cost_budget > 0
    ):
        env["WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_STRUCTURE_COST_BUDGET"] = (
            str(rust_pdf_docling_page_range_structure_cost_budget)
        )
    rust_pdf_docling_text_shortcut_promotion = getattr(
        args,
        "rust_pdf_docling_text_shortcut_promotion",
        None,
    )
    if (
        rust_pdf_docling_text_shortcut_promotion
        and rust_pdf_docling_text_shortcut_promotion != "range-fill"
    ):
        env["WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_TEXT_SHORTCUT_PROMOTION"] = str(
            rust_pdf_docling_text_shortcut_promotion
        )
    rust_pdf_local_backend_text = getattr(args, "rust_pdf_local_backend_text", None)
    if rust_pdf_local_backend_text and rust_pdf_local_backend_text != "disabled":
        env["WENDAO_DOCUMENT_EXTRACT_PDF_LOCAL_BACKEND_TEXT"] = str(
            rust_pdf_local_backend_text
        )
    rust_pdf_local_backend_text_empty = getattr(
        args,
        "rust_pdf_local_backend_text_empty",
        None,
    )
    if (
        rust_pdf_local_backend_text_empty
        and rust_pdf_local_backend_text_empty != "dispatch-python"
    ):
        env["WENDAO_DOCUMENT_EXTRACT_PDF_LOCAL_BACKEND_TEXT_EMPTY"] = str(
            rust_pdf_local_backend_text_empty
        )
    pdf_ocr_backend_text_empty_page = getattr(
        args,
        "pdf_ocr_backend_text_empty_page",
        None,
    )
    if (
        pdf_ocr_backend_text_empty_page
        and pdf_ocr_backend_text_empty_page != "disabled"
    ):
        env["WENDAO_DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_EMPTY_PAGE"] = str(
            pdf_ocr_backend_text_empty_page
        )
    rust_pdf_local_fast_text = getattr(args, "rust_pdf_local_fast_text", None)
    if rust_pdf_local_fast_text and rust_pdf_local_fast_text != "disabled":
        env["WENDAO_DOCUMENT_EXTRACT_PDF_LOCAL_FAST_TEXT"] = str(
            rust_pdf_local_fast_text
        )
    rust_pdf_fast_text_source_range_split = getattr(
        args,
        "rust_pdf_fast_text_source_range_split",
        None,
    )
    if (
        rust_pdf_fast_text_source_range_split
        and rust_pdf_fast_text_source_range_split != "disabled"
    ):
        env["WENDAO_DOCUMENT_EXTRACT_PDF_FAST_TEXT_SOURCE_RANGE_SPLIT"] = str(
            rust_pdf_fast_text_source_range_split
        )
    rust_pdf_fast_text_endpoint_affinity = getattr(
        args,
        "rust_pdf_fast_text_endpoint_affinity",
        None,
    )
    if (
        rust_pdf_fast_text_endpoint_affinity
        and rust_pdf_fast_text_endpoint_affinity != "disabled"
    ):
        env["WENDAO_DOCUMENT_EXTRACT_PDF_FAST_TEXT_ENDPOINT_AFFINITY"] = str(
            rust_pdf_fast_text_endpoint_affinity
        )
    rust_pdf_backend_text_topup = getattr(args, "rust_pdf_backend_text_topup", None)
    if rust_pdf_backend_text_topup and rust_pdf_backend_text_topup != "profile":
        env["WENDAO_DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_TOPUP"] = str(
            rust_pdf_backend_text_topup
        )
    rust_pdf_failed_page_recovery = getattr(args, "rust_pdf_failed_page_recovery", None)
    if rust_pdf_failed_page_recovery and rust_pdf_failed_page_recovery != "disabled":
        env["WENDAO_DOCUMENT_EXTRACT_PDF_FAILED_PAGE_RECOVERY"] = str(
            rust_pdf_failed_page_recovery
        )
    rust_pdf_ocr_profile_planner = getattr(args, "rust_pdf_ocr_profile_planner", None)
    if rust_pdf_ocr_profile_planner and rust_pdf_ocr_profile_planner != "disabled":
        env["WENDAO_DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER"] = str(
            rust_pdf_ocr_profile_planner
        )
    rust_pdf_hosted_vlm_render_dpi = getattr(
        args, "rust_pdf_hosted_vlm_render_dpi", None
    )
    if rust_pdf_hosted_vlm_render_dpi and rust_pdf_hosted_vlm_render_dpi >= 300:
        env["WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_RENDER_DPI"] = str(
            rust_pdf_hosted_vlm_render_dpi
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
    rust_pdf_hosted_vlm_region_planner = getattr(
        args, "rust_pdf_hosted_vlm_region_planner", None
    )
    if (
        rust_pdf_hosted_vlm_region_planner
        and rust_pdf_hosted_vlm_region_planner != "disabled"
    ):
        env["WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER"] = str(
            rust_pdf_hosted_vlm_region_planner
        )
    rust_pdf_hosted_vlm_region_target_pixels = getattr(
        args,
        "rust_pdf_hosted_vlm_region_target_pixels",
        None,
    )
    if (
        rust_pdf_hosted_vlm_region_target_pixels is not None
        and rust_pdf_hosted_vlm_region_target_pixels > 0
    ):
        env["WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_TARGET_PIXELS"] = str(
            rust_pdf_hosted_vlm_region_target_pixels
        )
    rust_pdf_hosted_vlm_region_max_slices = getattr(
        args,
        "rust_pdf_hosted_vlm_region_max_slices",
        None,
    )
    if (
        rust_pdf_hosted_vlm_region_max_slices
        and rust_pdf_hosted_vlm_region_max_slices > 0
    ):
        env["WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_MAX_SLICES"] = str(
            rust_pdf_hosted_vlm_region_max_slices
        )
    rust_pdf_hosted_vlm_region_pipeline = getattr(
        args,
        "rust_pdf_hosted_vlm_region_pipeline",
        None,
    )
    if (
        rust_pdf_hosted_vlm_region_pipeline
        and rust_pdf_hosted_vlm_region_pipeline != "disabled"
    ):
        env["WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PIPELINE"] = str(
            rust_pdf_hosted_vlm_region_pipeline
        )
    rust_pdf_hosted_vlm_region_render_ahead = getattr(
        args,
        "rust_pdf_hosted_vlm_region_render_ahead",
        None,
    )
    if (
        rust_pdf_hosted_vlm_region_render_ahead
        and rust_pdf_hosted_vlm_region_render_ahead > 1
    ):
        env["WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_AHEAD"] = str(
            rust_pdf_hosted_vlm_region_render_ahead
        )
    rust_pdf_hosted_vlm_region_render_chunk = getattr(
        args,
        "rust_pdf_hosted_vlm_region_render_chunk",
        None,
    )
    if (
        rust_pdf_hosted_vlm_region_render_chunk
        and rust_pdf_hosted_vlm_region_render_chunk != "page"
    ):
        env["WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_CHUNK"] = str(
            rust_pdf_hosted_vlm_region_render_chunk
        )
    rust_pdf_region_render_mode = getattr(args, "rust_pdf_region_render_mode", None)
    if rust_pdf_region_render_mode and rust_pdf_region_render_mode != "default":
        env["WENDAO_DOCUMENT_EXTRACT_PDF_REGION_RENDER_MODE"] = str(
            rust_pdf_region_render_mode
        )
    rust_pdf_hosted_vlm_region_dispatch_chunk_size = getattr(
        args,
        "rust_pdf_hosted_vlm_region_dispatch_chunk_size",
        None,
    )
    if (
        rust_pdf_hosted_vlm_region_dispatch_chunk_size
        and rust_pdf_hosted_vlm_region_dispatch_chunk_size > 1
    ):
        env["WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_DISPATCH_CHUNK_SIZE"] = str(
            rust_pdf_hosted_vlm_region_dispatch_chunk_size
        )
    rust_pdf_ocr_scheduler_lane_fairness = getattr(
        args,
        "rust_pdf_ocr_scheduler_lane_fairness",
        None,
    )
    if (
        rust_pdf_ocr_scheduler_lane_fairness
        and rust_pdf_ocr_scheduler_lane_fairness != "disabled"
    ):
        env["WENDAO_DOCUMENT_EXTRACT_PDF_OCR_SCHEDULER_LANE_FAIRNESS"] = str(
            rust_pdf_ocr_scheduler_lane_fairness
        )
    hosted_vlm_ocr_scaffold_mode = getattr(args, "hosted_vlm_ocr_scaffold_mode", None)
    if hosted_vlm_ocr_scaffold_mode and hosted_vlm_ocr_scaffold_mode != "disabled":
        env["WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_SCAFFOLD_MODE"] = str(
            hosted_vlm_ocr_scaffold_mode
        )
    hosted_vlm_ocr_region_composite_size = getattr(
        args,
        "hosted_vlm_ocr_region_composite_size",
        None,
    )
    if (
        hosted_vlm_ocr_region_composite_size
        and hosted_vlm_ocr_region_composite_size > 1
    ):
        env["WENDAO_HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE"] = str(
            hosted_vlm_ocr_region_composite_size
        )
    hosted_vlm_ocr_region_composite_mode = getattr(
        args,
        "hosted_vlm_ocr_region_composite_mode",
        None,
    )
    if (
        hosted_vlm_ocr_region_composite_mode
        and hosted_vlm_ocr_region_composite_mode != "fixed"
    ):
        env["WENDAO_HOSTED_VLM_OCR_REGION_COMPOSITE_MODE"] = str(
            hosted_vlm_ocr_region_composite_mode
        )
    hosted_vlm_ocr_region_composite_max_source_pixels = getattr(
        args,
        "hosted_vlm_ocr_region_composite_max_source_pixels",
        None,
    )
    if hosted_vlm_ocr_region_composite_max_source_pixels:
        env["WENDAO_HOSTED_VLM_OCR_REGION_COMPOSITE_MAX_SOURCE_PIXELS"] = str(
            hosted_vlm_ocr_region_composite_max_source_pixels
        )
    hosted_vlm_ocr_region_composite_max_image_bytes = getattr(
        args,
        "hosted_vlm_ocr_region_composite_max_image_bytes",
        None,
    )
    if hosted_vlm_ocr_region_composite_max_image_bytes:
        env["WENDAO_HOSTED_VLM_OCR_REGION_COMPOSITE_MAX_IMAGE_BYTES"] = str(
            hosted_vlm_ocr_region_composite_max_image_bytes
        )
    ocr_endpoint_pool = rust_pdf_ocr_endpoint_pool(args)
    if ocr_endpoint_pool:
        env["WENDAO_DOCUMENT_EXTRACT_PDF_OCR_ENDPOINTS"] = ocr_endpoint_pool


def apply_rust_audio_env(args: argparse.Namespace, env: dict[str, str]) -> None:
    mappings = {
        "rust_audio_backend_profile": "WENDAO_DOCUMENT_EXTRACT_AUDIO_BACKEND_PROFILE",
        "rust_audio_chunk_ms": "WENDAO_DOCUMENT_EXTRACT_AUDIO_CHUNK_MS",
        "rust_audio_context_before_ms": (
            "WENDAO_DOCUMENT_EXTRACT_AUDIO_CONTEXT_BEFORE_MS"
        ),
        "rust_audio_context_after_ms": (
            "WENDAO_DOCUMENT_EXTRACT_AUDIO_CONTEXT_AFTER_MS"
        ),
        "rust_audio_recovery_split_ms": (
            "WENDAO_DOCUMENT_EXTRACT_AUDIO_RECOVERY_SPLIT_MS"
        ),
        "rust_audio_sample_rate_hz": "WENDAO_DOCUMENT_EXTRACT_AUDIO_SAMPLE_RATE_HZ",
        "rust_audio_channels": "WENDAO_DOCUMENT_EXTRACT_AUDIO_CHANNELS",
        "rust_audio_format": "WENDAO_DOCUMENT_EXTRACT_AUDIO_FORMAT",
        "rust_audio_artifact_cache_dir": (
            "WENDAO_DOCUMENT_EXTRACT_AUDIO_ARTIFACT_CACHE_DIR"
        ),
        "rust_audio_transcript_admission_dir": (
            "WENDAO_DOCUMENT_EXTRACT_AUDIO_TRANSCRIPT_ADMISSION_DIR"
        ),
        "rust_audio_base_workers": "WENDAO_DOCUMENT_EXTRACT_AUDIO_BASE_WORKERS",
        "rust_audio_recovery_workers": (
            "WENDAO_DOCUMENT_EXTRACT_AUDIO_RECOVERY_WORKERS"
        ),
        "rust_audio_speech_segments_jsonl": (
            "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_SEGMENTS_JSONL"
        ),
        "rust_audio_speech_merge_gap_ms": (
            "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_MERGE_GAP_MS"
        ),
        "rust_audio_speech_min_window_ms": (
            "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_MIN_WINDOW_MS"
        ),
        "rust_audio_speech_limit_chunks": (
            "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_LIMIT_CHUNKS"
        ),
    }
    for attr, key in mappings.items():
        value = getattr(args, attr, None)
        if value is not None:
            env[key] = str(value)


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
    if getattr(args, "require_pdfium", False):
        env["WENDAO_PDF_RENDER_REQUIRE_PDFIUM"] = "1"
    apply_rust_pdf_ocr_env(args, env)
    apply_rust_audio_env(args, env)
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
        provider_features = cargo_features_with_feature(
            cargo_features_for_provider_mode(args.rust_provider_features, args),
            "flight-server-bin-support",
        )
        command = [
            args.cargo,
            "run",
            "-p",
            "xiuxian-wendao-studio",
            "--no-default-features",
            "--features",
            provider_features,
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
    if getattr(args, "require_pdfium", False):
        env["WENDAO_PDF_RENDER_REQUIRE_PDFIUM"] = "1"
    apply_rust_pdf_ocr_env(args, env)
    apply_rust_audio_env(args, env)
    gateway_args = [
        "--conf",
        str(config_path),
        "--root",
        str(resolve_project_root()),
        "gateway",
        "start",
        "--port",
        str(gateway_port),
    ]
    rust_provider_bin = getattr(args, "rust_provider_bin", None)
    if rust_provider_bin is not None:
        command = [str(rust_provider_bin), *gateway_args]
    else:
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
            *gateway_args,
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
