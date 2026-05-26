"""document_extract_perf_script test slice 5."""

from __future__ import annotations

from .support import (
    Path,
    _load_benchmark_module,
)


def test_cargo_perf_probe_forwards_structure_baseline_root(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    report_path = tmp_path / "report.json"
    baseline_root = tmp_path / "baselines"
    captured_env = {}

    def fake_run(command: list[str], *, check: bool, env) -> None:
        assert command[0] == "cargo"
        assert check
        captured_env.update(env)
        report_path.write_text(
            '{"latenciesMs":[1.0],"requestCount":1,"rowCount":1,'
            '"batchCount":1,"arrowIpcBytes":1,"errorRowCount":0,'
            '"statusCounts":{"ok":1},"wallTimeMs":1.0}',
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
        structure_baseline_root=baseline_root,
    )

    benchmark.run_cargo_perf_test(
        args,
        tmp_path / "sample.md",
        tmp_path / "out",
        force=False,
        iterations=1,
        concurrency=1,
        report_path=report_path,
    )

    assert captured_env["WENDAO_DOCUMENT_EXTRACT_PERF_STRUCTURE_BASELINE_ROOT"] == str(
        baseline_root
    )


def test_cargo_perf_probe_can_override_flight_mode_without_self_parity(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    report_path = tmp_path / "report.json"
    captured_env = {}
    commands: list[list[str]] = []

    def fake_run(command: list[str], *, check: bool, env) -> None:
        commands.append(command)
        assert check
        captured_env.update(env)
        report_path.write_text(
            '{"latenciesMs":[1.0],"requestCount":1,"rowCount":1,'
            '"batchCount":1,"arrowIpcBytes":1,"errorRowCount":0,'
            '"statusCounts":{"ok":1},"wallTimeMs":1.0}',
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
        flight_mode="hybrid-page-ocr",
        wait_ms=0,
        structure_baseline_root=tmp_path / "baselines",
    )

    benchmark.run_cargo_perf_test(
        args,
        tmp_path / "sample.pdf",
        tmp_path / "baseline",
        force=True,
        iterations=1,
        concurrency=1,
        report_path=report_path,
        flight_mode="sync",
        include_structure_baseline_root=False,
    )

    assert captured_env["WENDAO_DOCUMENT_EXTRACT_PERF_MODE"] == "sync"
    assert "WENDAO_DOCUMENT_EXTRACT_PERF_STRUCTURE_BASELINE_ROOT" not in captured_env
    assert commands[0][commands[0].index("--features") + 1] == (
        "performance,studio,zhenfa-router,duckdb"
    )


def test_cargo_perf_probe_forwards_rust_audio_planning_env(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    report_path = tmp_path / "report.json"
    sidecar = tmp_path / "speech.jsonl"
    captured_env = {}

    def fake_run(command: list[str], *, check: bool, env) -> None:
        assert command[0] == "cargo"
        assert check
        captured_env.update(env)
        report_path.write_text(
            '{"latenciesMs":[1.0],"requestCount":1,"rowCount":1,'
            '"batchCount":1,"arrowIpcBytes":1,"errorRowCount":0,'
            '"statusCounts":{"ok":1},"wallTimeMs":1.0}',
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
        flight_mode="audio-shards",
        wait_ms=0,
        rust_audio_backend_profile="hosted-audio-transcript-v1",
        rust_audio_chunk_ms=30_000,
        rust_audio_context_before_ms=0,
        rust_audio_context_after_ms=0,
        rust_audio_recovery_split_ms=30_000,
        rust_audio_sample_rate_hz=16_000,
        rust_audio_channels=1,
        rust_audio_format="wav",
        rust_audio_bitrate="96k",
        rust_audio_artifact_cache_dir=tmp_path / "artifacts",
        rust_audio_transcript_admission_dir=tmp_path / "admissions",
        rust_audio_base_workers="auto",
        rust_audio_recovery_workers=None,
        rust_audio_speech_segments_jsonl=sidecar,
        rust_audio_speech_merge_gap_ms=500,
        rust_audio_speech_min_window_ms=2_000,
        rust_audio_speech_max_window_ms=28_000,
        rust_audio_speech_boundary_snap_tolerance_ms=1_000,
        rust_audio_speech_limit_chunks=10_000,
    )

    benchmark.run_cargo_perf_test(
        args,
        tmp_path / "sample.mp3",
        tmp_path / "out",
        force=True,
        iterations=1,
        concurrency=1,
        report_path=report_path,
    )

    assert captured_env["WENDAO_DOCUMENT_EXTRACT_AUDIO_BACKEND_PROFILE"] == (
        "hosted-audio-transcript-v1"
    )
    assert captured_env["WENDAO_DOCUMENT_EXTRACT_AUDIO_CHUNK_MS"] == "30000"
    assert captured_env["WENDAO_DOCUMENT_EXTRACT_AUDIO_CONTEXT_BEFORE_MS"] == "0"
    assert captured_env["WENDAO_DOCUMENT_EXTRACT_AUDIO_CONTEXT_AFTER_MS"] == "0"
    assert captured_env["WENDAO_DOCUMENT_EXTRACT_AUDIO_BITRATE"] == "96k"
    assert captured_env["WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_SEGMENTS_JSONL"] == str(sidecar)
    assert captured_env["WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_MIN_WINDOW_MS"] == "2000"
    assert captured_env["WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_MAX_WINDOW_MS"] == "28000"
    assert captured_env["WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_BOUNDARY_SNAP_TOLERANCE_MS"] == (
        "1000"
    )
