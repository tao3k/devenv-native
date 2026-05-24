"""document_extract_perf_script test slice 8."""

from __future__ import annotations

from .support import (
    Path,
    _load_benchmark_module,
)


def test_find_pdfium_library_prefers_lib_directory(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    nested = tmp_path / "nested" / "libpdfium.dylib"
    preferred = tmp_path / "lib" / "libpdfium.dylib"
    nested.parent.mkdir(parents=True)
    preferred.parent.mkdir(parents=True)
    nested.write_bytes(b"nested")
    preferred.write_bytes(b"preferred")

    assert benchmark.find_pdfium_library(tmp_path, "libpdfium.dylib") == preferred


def test_pdf_render_shard_features_are_not_duplicated() -> None:
    benchmark = _load_benchmark_module()

    assert (
        benchmark.cargo_features_with_pdf_render("performance document-extract-pdf-render")
        == "performance,document-extract-pdf-render"
    )


def test_hybrid_source_range_features_do_not_pull_pdfium() -> None:
    benchmark = _load_benchmark_module()

    assert (
        benchmark.cargo_features_for_flight_mode("performance studio", "hybrid-page-ocr")
        == "performance,studio,document-extract-pdf-source-range"
    )


def test_audio_shards_perf_probe_keeps_studio_feature_out_of_core_probe() -> None:
    benchmark = _load_benchmark_module()

    assert (
        benchmark.cargo_features_for_flight_mode("performance studio", "audio-shards")
        == "performance studio"
    )


def test_audio_shards_provider_mode_enables_studio_audio_feature() -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(flight_mode="audio-shards")

    assert (
        benchmark.cargo_features_for_provider_mode(
            "cli-bin-support,zhenfa-router,duckdb",
            args,
        )
        == "performance,cli-bin-support,zhenfa-router,duckdb,document-extract-audio-shards"
    )


def test_normalize_render_selection_accepts_cli_spelling() -> None:
    benchmark = _load_benchmark_module()

    assert benchmark.normalize_render_selection("shard-fallback-pages") == ("shard_fallback_pages")
    assert benchmark.normalize_render_selection("region-shards") == "region_shards"


def test_auto_local_ocr_endpoint_count_uses_machine_profile(monkeypatch) -> None:
    benchmark = _load_benchmark_module()
    monkeypatch.setattr(benchmark.os, "cpu_count", lambda: 12)
    args = benchmark.argparse.Namespace(
        local_python_ocr_endpoint_count="auto",
        external_endpoint=False,
        real_docling=True,
        flight_mode="hybrid-page-ocr",
        pdf_ocr_worker="docling",
    )

    assert benchmark.resolve_local_python_ocr_endpoint_count(args) == 4


def test_parse_args_marks_default_port_as_implicit(monkeypatch) -> None:
    benchmark = _load_benchmark_module()
    monkeypatch.setattr(benchmark._args.sys, "argv", ["benchmark"])

    args = benchmark.parse_args()

    assert args.port == 50051
    assert args.port_was_explicit is False


def test_parse_args_marks_cli_port_as_explicit(monkeypatch) -> None:
    benchmark = _load_benchmark_module()
    monkeypatch.setattr(benchmark._args.sys, "argv", ["benchmark", "--port", "62051"])

    args = benchmark.parse_args()

    assert args.port == 62051
    assert args.port_was_explicit is True


def test_parse_args_accepts_rust_audio_artifact_cache_dir(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    cache_dir = tmp_path / "audio-artifacts"
    monkeypatch.setattr(
        benchmark._args.sys,
        "argv",
        ["benchmark", "--rust-audio-artifact-cache-dir", str(cache_dir)],
    )

    args = benchmark.parse_args()

    assert args.rust_audio_artifact_cache_dir == cache_dir


def test_reset_process_log_dir_removes_stale_trace_files(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    process_log_dir = tmp_path / "process-logs"
    process_log_dir.mkdir()
    stale_trace = process_log_dir / "python-worker-0.hosted-vlm-ocr.jsonl"
    stale_trace.write_text('{"requestKind":"region"}\n', encoding="utf-8")
    stale_subdir = process_log_dir / "old"
    stale_subdir.mkdir()

    benchmark.reset_process_log_dir(process_log_dir)

    assert process_log_dir.is_dir()
    assert list(process_log_dir.iterdir()) == []


def test_auto_local_ocr_endpoint_count_keeps_non_hybrid_modes_single_endpoint(
    monkeypatch,
) -> None:
    benchmark = _load_benchmark_module()
    monkeypatch.setattr(benchmark.os, "cpu_count", lambda: 12)
    args = benchmark.argparse.Namespace(
        local_python_ocr_endpoint_count="auto",
        external_endpoint=False,
        real_docling=True,
        flight_mode="sync",
        pdf_ocr_worker="docling",
    )

    assert benchmark.resolve_local_python_ocr_endpoint_count(args) == 1


def test_explicit_local_ocr_endpoint_count_overrides_auto() -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(local_python_ocr_endpoint_count="3")

    assert benchmark.resolve_local_python_ocr_endpoint_count(args) == 3


def test_invalid_local_ocr_endpoint_count_is_rejected() -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(local_python_ocr_endpoint_count="0")

    try:
        benchmark.resolve_local_python_ocr_endpoint_count(args)
    except SystemExit as exc:
        assert "--local-python-ocr-endpoint-count" in str(exc)
    else:
        raise AssertionError("expected invalid endpoint count to exit")


def test_local_rust_provider_port_uses_free_port_unless_explicit(
    monkeypatch,
) -> None:
    benchmark = _load_benchmark_module()
    monkeypatch.setattr(benchmark._cli, "pick_free_port", lambda host: 62052)

    assert (
        benchmark.resolve_local_rust_provider_port(
            benchmark.argparse.Namespace(host="127.0.0.1", rust_provider_port=None)
        )
        == 62052
    )
    assert (
        benchmark.resolve_local_rust_provider_port(
            benchmark.argparse.Namespace(host="127.0.0.1", rust_provider_port=63052)
        )
        == 63052
    )


def test_auto_document_extract_full_threads_caps_docling_structure_recovery() -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        document_extract_full_threads="auto",
        real_docling=True,
        flight_mode="hybrid-page-ocr",
        rust_pdf_ocr_profile_planner="docling-structure-recovery",
    )

    assert benchmark.resolve_document_extract_full_threads(args) == 1


def test_auto_document_extract_full_threads_leaves_other_modes_unset() -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        document_extract_full_threads="auto",
        real_docling=True,
        flight_mode="hybrid-page-ocr",
        rust_pdf_ocr_profile_planner="fast-risk-window",
    )

    assert benchmark.resolve_document_extract_full_threads(args) is None


def test_explicit_document_extract_full_threads_overrides_auto() -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(document_extract_full_threads="2")

    assert benchmark.resolve_document_extract_full_threads(args) == 2
