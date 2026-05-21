"""document_extract_perf_script test slice 8."""

from __future__ import annotations

from .support import (
    Path,
    _load_benchmark_module,
)


def test_cargo_perf_probe_can_send_distinct_input_manifest(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    report_path = tmp_path / "report.json"
    captured_env = {}

    def fake_run(command: list[str], *, check: bool, env) -> None:
        assert command[0] == "cargo"
        assert check
        captured_env.update(env)
        report_path.write_text(
            '{"latenciesMs":[1.0,2.0],"requestCount":2,"rowCount":2,'
            '"batchCount":1,"arrowIpcBytes":2,"errorRowCount":0,'
            '"statusCounts":{"ok":2},"wallTimeMs":2.0}',
            encoding="utf-8",
        )

    monkeypatch.setattr(benchmark.subprocess, "run", fake_run)
    monkeypatch.setenv("SDKROOT", "/tmp/macos-sdk")
    monkeypatch.setenv("LIBRARY_PATH", "/tmp/macos-sdk/usr/lib")
    args = benchmark.argparse.Namespace(
        benchmark_host="127.0.0.1",
        benchmark_port=50052,
        cargo="cargo",
        cargo_features="performance,studio,zhenfa-router,duckdb",
        flight_mode="async",
        wait_ms=0,
    )

    benchmark.run_cargo_perf_test(
        args,
        tmp_path / "first.md",
        tmp_path / "out",
        force=False,
        iterations=1,
        concurrency=2,
        report_path=report_path,
        inputs={
            "first": tmp_path / "first.md",
            "second": tmp_path / "second.md",
        },
        wait_ms=60000,
    )

    manifest = benchmark.json.loads(
        captured_env["WENDAO_DOCUMENT_EXTRACT_PERF_INPUTS_JSON"]
    )
    assert captured_env["WENDAO_DOCUMENT_EXTRACT_PERF_WAIT_MS"] == "60000"
    assert [item["name"] for item in manifest] == ["first", "second"]
    assert [Path(item["outputDir"]).name for item in manifest] == ["first", "second"]


def test_start_gateway_server_sets_document_extract_and_valkey_env(
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
        gateway_features="cli-bin-support,zhenfa-router,duckdb",
        rust_pdf_ocr_workers="6",
        rust_pdf_ocr_source_range_workers="2",
        rust_pdf_ocr_profile_planner="fast-risk-window",
        rust_pdf_ocr_endpoint=[
            "http://127.0.0.1:52051",
            "http://127.0.0.1:52052/",
        ],
        rust_audio_backend_profile="hosted-audio-transcript-v1",
        rust_audio_chunk_ms=60000,
        rust_audio_context_before_ms=5000,
        rust_audio_context_after_ms=5000,
        rust_audio_recovery_split_ms=15000,
        rust_audio_sample_rate_hz=16000,
        rust_audio_channels=1,
        rust_audio_format="wav",
        rust_audio_base_workers="4",
        rust_audio_recovery_workers="2",
        rust_document_extract_endpoint=[
            "http://127.0.0.1:53051/",
            "http://127.0.0.1:53052",
        ],
    )

    benchmark.start_gateway_server(
        args,
        gateway_port=51080,
        python_host="127.0.0.1",
        python_port=51051,
        valkey_url="redis://127.0.0.1:51079/0",
        temp_root=tmp_path,
    )

    command, kwargs = calls[0]
    assert command[:7] == [
        "cargo",
        "run",
        "-p",
        "xiuxian-wendao-studio",
        "--no-default-features",
        "--features",
        "cli-bin-support,zhenfa-router,duckdb",
    ]
    assert command[-8:] == [
        "--conf",
        str(tmp_path / "gateway" / "wendao.toml"),
        "--root",
        str(tmp_path / "repo"),
        "gateway",
        "start",
        "--port",
        "51080",
    ]
    env = kwargs["env"]
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_OCR_WORKERS"] == "6"
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS"] == "2"
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER"] == "fast-risk-window"
    assert env["WENDAO_DOCUMENT_EXTRACT_AUDIO_BACKEND_PROFILE"] == (
        "hosted-audio-transcript-v1"
    )
    assert env["WENDAO_DOCUMENT_EXTRACT_AUDIO_CHUNK_MS"] == "60000"
    assert env["WENDAO_DOCUMENT_EXTRACT_AUDIO_CONTEXT_BEFORE_MS"] == "5000"
    assert env["WENDAO_DOCUMENT_EXTRACT_AUDIO_CONTEXT_AFTER_MS"] == "5000"
    assert env["WENDAO_DOCUMENT_EXTRACT_AUDIO_RECOVERY_SPLIT_MS"] == "15000"
    assert env["WENDAO_DOCUMENT_EXTRACT_AUDIO_SAMPLE_RATE_HZ"] == "16000"
    assert env["WENDAO_DOCUMENT_EXTRACT_AUDIO_CHANNELS"] == "1"
    assert env["WENDAO_DOCUMENT_EXTRACT_AUDIO_FORMAT"] == "wav"
    assert env["WENDAO_DOCUMENT_EXTRACT_AUDIO_BASE_WORKERS"] == "4"
    assert env["WENDAO_DOCUMENT_EXTRACT_AUDIO_RECOVERY_WORKERS"] == "2"
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_OCR_ENDPOINTS"] == (
        "http://127.0.0.1:52051,http://127.0.0.1:52052"
    )
    assert env["WENDAO_DOCUMENT_EXTRACT_ENDPOINT"] == "http://127.0.0.1:51051"
    assert env["WENDAO_DOCUMENT_EXTRACT_ENDPOINTS"] == (
        "http://127.0.0.1:53051,http://127.0.0.1:53052"
    )
    assert env["WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_ROOT"] == str(
        (tmp_path / "ocr-shard-cache").resolve()
    )
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_SELECTION"] == (
        "shard_fallback_pages"
    )
    assert env["VALKEY_URL"] == "redis://127.0.0.1:51079/0"
    assert env["XIUXIAN_WENDAO_SEARCH_PLANE_VALKEY_URL"] == (
        "redis://127.0.0.1:51079/0"
    )
    assert env["XIUXIAN_WENDAO_GATEWAY_BOOTSTRAP_BACKGROUND_INDEXING"] == "false"
    config = (tmp_path / "gateway" / "wendao.toml").read_text(encoding="utf-8")
    assert "[search.cache]" in config
    assert 'valkey_url = "redis://127.0.0.1:51079/0"' in config


def test_start_rust_provider_server_enables_flight_bin_feature(
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
    args = benchmark.argparse.Namespace(
        cargo="cargo",
        rust_provider_features="cli-bin-support,zhenfa-router,duckdb",
        flight_mode="audio-shards",
        rust_provider_bin=None,
        rust_pdf_ocr_endpoint=[],
        rust_document_extract_endpoint=[],
        rust_audio_backend_profile=None,
        rust_audio_chunk_ms=None,
        rust_audio_context_before_ms=None,
        rust_audio_context_after_ms=None,
        rust_audio_recovery_split_ms=None,
        rust_audio_sample_rate_hz=None,
        rust_audio_channels=None,
        rust_audio_format=None,
        rust_audio_base_workers=None,
        rust_audio_recovery_workers=None,
        ocr_shard_cache_root=None,
        hybrid_pdf_render_selection="shard-fallback-pages",
        pdfium_library_path=None,
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
    features = command[command.index("--features") + 1]
    assert "flight-server-bin-support" in features.split(",")
    assert "document-extract-audio-shards" in features.split(",")
