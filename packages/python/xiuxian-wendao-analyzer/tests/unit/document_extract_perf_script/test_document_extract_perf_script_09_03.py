"""document_extract_perf_script test slice 9."""

from __future__ import annotations

from .support import (
    Path,
    _load_benchmark_module,
)


def test_start_rust_provider_defaults_document_extract_pool_to_local_worker(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    calls = []

    class FakePopen:
        def __init__(self, command, **kwargs):
            calls.append((command, kwargs))

    monkeypatch.setattr(benchmark.subprocess, "Popen", FakePopen)
    monkeypatch.setenv("SDKROOT", "/tmp/macos-sdk")
    monkeypatch.setenv("LIBRARY_PATH", "/tmp/macos-sdk/usr/lib")
    monkeypatch.setenv("PRJ_ROOT", str(tmp_path / "repo"))
    args = benchmark.argparse.Namespace(
        cargo="cargo",
        rust_provider_bin=None,
        rust_provider_features="cli-bin-support,zhenfa-router,duckdb",
        flight_mode="async",
        hybrid_pdf_render_selection="shard-fallback-pages",
        pdf_render_region=[],
        benchmark_fixtures={},
        pdfium_library_path=None,
        prepare_pdfium_runtime=False,
        rust_pdf_ocr_workers=None,
        rust_pdf_ocr_source_range_workers=None,
        rust_pdf_ocr_profile_planner="disabled",
        rust_pdf_hosted_vlm_render_dpi=None,
        rust_pdf_ocr_region_context_ratio=None,
        rust_pdf_hosted_vlm_region_planner=None,
        rust_pdf_ocr_endpoint=[],
        rust_document_extract_endpoint=[],
    )

    benchmark.start_rust_provider_server(
        args,
        rust_host="127.0.0.1",
        rust_port=51052,
        python_host="127.0.0.1",
        python_port=51051,
        temp_root=tmp_path,
    )

    env = calls[0][1]["env"]
    assert env["WENDAO_DOCUMENT_EXTRACT_ENDPOINTS"] == "http://127.0.0.1:51051"


def test_start_rust_provider_forwards_audio_speech_sidecar_controls(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    calls = []

    class FakePopen:
        def __init__(self, command, **kwargs):
            calls.append((command, kwargs))

    monkeypatch.setattr(benchmark.subprocess, "Popen", FakePopen)
    monkeypatch.setenv("SDKROOT", "/tmp/macos-sdk")
    monkeypatch.setenv("LIBRARY_PATH", "/tmp/macos-sdk/usr/lib")
    monkeypatch.setenv("PRJ_ROOT", str(tmp_path / "repo"))
    sidecar = tmp_path / "speech.jsonl"
    args = benchmark.argparse.Namespace(
        cargo="cargo",
        rust_provider_bin=None,
        rust_provider_features="cli-bin-support,zhenfa-router,duckdb",
        flight_mode="audio-shards",
        hybrid_pdf_render_selection="shard-fallback-pages",
        pdf_render_region=[],
        benchmark_fixtures={},
        pdfium_library_path=None,
        prepare_pdfium_runtime=False,
        rust_pdf_ocr_workers=None,
        rust_pdf_ocr_source_range_workers=None,
        rust_pdf_ocr_profile_planner="disabled",
        rust_pdf_hosted_vlm_render_dpi=None,
        rust_pdf_ocr_region_context_ratio=None,
        rust_pdf_hosted_vlm_region_planner=None,
        rust_pdf_ocr_endpoint=[],
        rust_document_extract_endpoint=[],
        rust_audio_backend_profile="hosted-audio-transcript-v1",
        rust_audio_chunk_ms=60000,
        rust_audio_context_before_ms=None,
        rust_audio_context_after_ms=None,
        rust_audio_recovery_split_ms=15000,
        rust_audio_sample_rate_hz=None,
        rust_audio_channels=None,
        rust_audio_format=None,
        rust_audio_artifact_cache_dir=None,
        rust_audio_base_workers=None,
        rust_audio_recovery_workers=None,
        rust_audio_speech_segments_jsonl=sidecar,
        rust_audio_speech_merge_gap_ms=700,
        rust_audio_speech_min_window_ms=5000,
        rust_audio_speech_limit_chunks=12,
    )

    benchmark.start_rust_provider_server(
        args,
        rust_host="127.0.0.1",
        rust_port=51052,
        python_host="127.0.0.1",
        python_port=51051,
        temp_root=tmp_path,
    )

    env = calls[0][1]["env"]
    assert env["WENDAO_DOCUMENT_EXTRACT_AUDIO_BACKEND_PROFILE"] == (
        "hosted-audio-transcript-v1"
    )
    assert env["WENDAO_DOCUMENT_EXTRACT_AUDIO_CHUNK_MS"] == "60000"
    assert env["WENDAO_DOCUMENT_EXTRACT_AUDIO_RECOVERY_SPLIT_MS"] == "15000"
    assert env["WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_SEGMENTS_JSONL"] == str(sidecar)
    assert env["WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_MERGE_GAP_MS"] == "700"
    assert env["WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_MIN_WINDOW_MS"] == "5000"
    assert env["WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_LIMIT_CHUNKS"] == "12"


def test_start_rust_provider_does_not_forward_hosted_vlm_dpi_downgrade(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    calls = []

    class FakePopen:
        def __init__(self, command, **kwargs):
            calls.append((command, kwargs))

    monkeypatch.setattr(benchmark.subprocess, "Popen", FakePopen)
    monkeypatch.setenv("SDKROOT", "/tmp/macos-sdk")
    monkeypatch.setenv("LIBRARY_PATH", "/tmp/macos-sdk/usr/lib")
    monkeypatch.setenv("PRJ_ROOT", str(tmp_path / "repo"))
    args = benchmark.argparse.Namespace(
        cargo="cargo",
        rust_provider_bin=None,
        rust_provider_features="cli-bin-support,zhenfa-router,duckdb",
        flight_mode="hybrid-page-ocr",
        hybrid_pdf_render_selection="shard-fallback-pages",
        pdf_render_region=[],
        benchmark_fixtures={},
        pdfium_library_path=None,
        prepare_pdfium_runtime=False,
        rust_pdf_ocr_workers=None,
        rust_pdf_ocr_source_range_workers=None,
        rust_pdf_ocr_profile_planner="hosted-vlm-risk-window",
        rust_pdf_hosted_vlm_render_dpi=180,
        rust_pdf_ocr_region_context_ratio=None,
        rust_pdf_hosted_vlm_region_planner=None,
        rust_pdf_ocr_endpoint=[],
        rust_document_extract_endpoint=[],
    )

    benchmark.start_rust_provider_server(
        args,
        rust_host="127.0.0.1",
        rust_port=51052,
        python_host="127.0.0.1",
        python_port=51051,
        temp_root=tmp_path,
    )

    env = calls[0][1]["env"]
    assert "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_RENDER_DPI" not in env


def test_start_rust_provider_hosted_vlm_planner_enables_pdf_render_feature(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    calls = []

    class FakePopen:
        def __init__(self, command, **kwargs):
            calls.append((command, kwargs))

    monkeypatch.setattr(benchmark.subprocess, "Popen", FakePopen)
    monkeypatch.setenv("SDKROOT", "/tmp/macos-sdk")
    monkeypatch.setenv("LIBRARY_PATH", "/tmp/macos-sdk/usr/lib")
    monkeypatch.setenv("PRJ_ROOT", str(tmp_path / "repo"))
    args = benchmark.argparse.Namespace(
        cargo="cargo",
        rust_provider_bin=None,
        rust_provider_features="cli-bin-support,zhenfa-router,duckdb",
        flight_mode="hybrid-page-ocr",
        hybrid_pdf_render_selection="shard-fallback-pages",
        pdf_render_region=[],
        benchmark_fixtures={},
        pdfium_library_path=None,
        prepare_pdfium_runtime=False,
        rust_pdf_ocr_workers=None,
        rust_pdf_ocr_source_range_workers=None,
        rust_pdf_ocr_profile_planner="hosted-vlm-risk-window",
        rust_pdf_hosted_vlm_render_dpi=None,
        rust_pdf_ocr_region_context_ratio=None,
        rust_pdf_hosted_vlm_region_planner=None,
        rust_pdf_ocr_endpoint=[],
        rust_document_extract_endpoint=[],
    )

    benchmark.start_rust_provider_server(
        args,
        rust_host="127.0.0.1",
        rust_port=51052,
        python_host="127.0.0.1",
        python_port=51051,
        temp_root=tmp_path,
    )

    command, _kwargs = calls[0]
    assert "document-extract-pdf-render" in command[6].split(",")
    assert (
        calls[0][1]["env"]["WENDAO_DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER"]
        == "hosted-vlm-risk-window"
    )
