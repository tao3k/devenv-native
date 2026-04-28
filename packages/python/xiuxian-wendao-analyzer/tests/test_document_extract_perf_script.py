from __future__ import annotations

import importlib.util
import tomllib
from pathlib import Path


def _load_benchmark_module():
    repo_root = Path(__file__).resolve().parents[4]
    script_path = repo_root / "scripts" / "benchmark_wendao_document_extract.py"
    spec = importlib.util.spec_from_file_location(
        "benchmark_wendao_document_extract",
        script_path,
    )
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_docling_real_fixtures_select_all_supported_real_attachment_paths(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    for relative_path in benchmark.DOCLING_REAL_FIXTURE_PATHS.values():
        fixture_path = tmp_path / relative_path
        fixture_path.parent.mkdir(parents=True, exist_ok=True)
        fixture_path.write_bytes(b"fixture")

    fixtures = benchmark.docling_real_fixtures(tmp_path, include_audio=True)
    assert set(fixtures) == set(benchmark.DOCLING_REAL_FIXTURE_PATHS)
    assert fixtures["mets-gbs"].name.endswith(".tar.gz")
    assert fixtures["xbrl-xml"].name == "mlac-20251231.xml"
    assert fixtures["audio"].name == "sample_10s.mp3"


def test_docling_real_fixtures_can_skip_audio(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    for name, relative_path in benchmark.DOCLING_REAL_FIXTURE_PATHS.items():
        if name == "audio":
            continue
        fixture_path = tmp_path / relative_path
        fixture_path.parent.mkdir(parents=True, exist_ok=True)
        fixture_path.write_bytes(b"fixture")

    fixtures = benchmark.docling_real_fixtures(tmp_path, include_audio=False)
    assert "audio" not in fixtures
    assert "webvtt" in fixtures


def test_select_fixtures_filters_named_fixture(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    fixtures = {
        "pdf": tmp_path / "sample.pdf",
        "audio": tmp_path / "sample.mp3",
    }

    selected = benchmark.select_fixtures(fixtures, ["audio"])

    assert selected == {"audio": tmp_path / "sample.mp3"}


def test_document_extras_cover_xbrl_and_audio_asr() -> None:
    package_root = Path(__file__).resolve().parents[1]
    pyproject = tomllib.loads((package_root / "pyproject.toml").read_text())
    optional_dependencies = pyproject["project"]["optional-dependencies"]

    assert "docling[xbrl]>=2.70.0" in optional_dependencies["documents"]
    assert "docling[xbrl]>=2.70.0" in optional_dependencies["documents-audio"]
    assert "openai-whisper>=20250625" in optional_dependencies["documents-audio"]
    assert "imageio-ffmpeg>=0.6.0" in optional_dependencies["documents-audio"]


def test_docling_real_fixture_root_defaults_to_prj_data_home(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    monkeypatch.setenv("PRJ_DATA_HOME", str(tmp_path))

    assert benchmark.resolve_docling_source_root(None) == (
        tmp_path / "docling-real-fixtures"
    ).resolve()


def test_prepare_docling_fixtures_uses_sparse_checkout(monkeypatch, tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    commands: list[list[str]] = []

    def fake_run(command: list[str], *, check: bool) -> None:
        commands.append(command)
        assert check

    monkeypatch.setattr(benchmark.subprocess, "run", fake_run)

    benchmark.prepare_docling_fixtures(
        tmp_path / "docling-real-fixtures",
        repo_url="https://example.test/docling.git",
        git_ref=benchmark.DOCLING_DEFAULT_GIT_REF,
    )

    assert commands[0][:5] == ["git", "clone", "--depth", "1", "--filter=blob:none"]
    assert "--sparse" in commands[0]
    assert commands[1][-2:] == ["--skip-checks", "tests/data"]


def test_rows_per_second_uses_wall_clock_time() -> None:
    benchmark = _load_benchmark_module()

    assert benchmark.rows_per_second(40, 200.0) == 200.0


def test_rust_jobs_status_summary_tracks_pressure() -> None:
    benchmark = _load_benchmark_module()

    summary = benchmark.summarize_rust_jobs_status_samples(
        [
            {
                "queuedJobs": 3,
                "runningJobs": 1,
                "inProcessRunningConversions": 1,
                "inProcessScheduledJobs": 4,
                "availableConversionPermits": 2,
                "maxRunningConversions": 4,
            },
            {
                "queuedJobs": 1,
                "runningJobs": 2,
                "inProcessRunningConversions": 2,
                "inProcessScheduledJobs": 2,
                "availableConversionPermits": 1,
                "maxRunningConversions": 4,
                "lastConversionDurationMs": 120,
                "maxConversionDurationMs": 300,
            },
        ]
    )

    assert summary["sampleCount"] == 2
    assert summary["maxQueuedJobs"] == 3
    assert summary["maxRunningJobs"] == 2
    assert summary["maxInProcessRunningConversions"] == 2
    assert summary["minAvailableConversionPermits"] == 1
    assert summary["lastConversionDurationMs"] == 120
    assert summary["maxConversionDurationMs"] == 300


def test_rust_jobs_status_summary_combines_fixture_phases() -> None:
    benchmark = _load_benchmark_module()

    combined = benchmark.combine_rust_jobs_status_summaries(
        [
            {
                "sampleCount": 2,
                "maxQueuedJobs": 4,
                "maxRunningJobs": 1,
                "maxInProcessRunningConversions": 1,
                "maxInProcessScheduledJobs": 4,
                "minAvailableConversionPermits": 3,
                "maxRunningConversions": 4,
                "lastConversionDurationMs": None,
                "maxConversionDurationMs": None,
            },
            {
                "sampleCount": 1,
                "maxQueuedJobs": 0,
                "maxRunningJobs": 2,
                "maxInProcessRunningConversions": 2,
                "maxInProcessScheduledJobs": 2,
                "minAvailableConversionPermits": 2,
                "maxRunningConversions": 4,
                "lastConversionDurationMs": 80,
                "maxConversionDurationMs": 120,
            },
        ]
    )

    assert combined["sampleCount"] == 3
    assert combined["maxQueuedJobs"] == 4
    assert combined["maxRunningJobs"] == 2
    assert combined["minAvailableConversionPermits"] == 2
    assert combined["lastConversionDurationMs"] == 80


def test_fetch_rust_jobs_status_reads_gateway_payload(monkeypatch) -> None:
    benchmark = _load_benchmark_module()

    class Response:
        def __enter__(self):
            return self

        def __exit__(self, *args) -> None:
            return None

        def read(self) -> bytes:
            return b'{"queuedJobs":2,"runningJobs":1}'

    def fake_urlopen(url: str, timeout: float) -> Response:
        assert url == "http://127.0.0.1:7788/api/document-extract-jobs"
        assert timeout == 1.0
        return Response()

    monkeypatch.setattr(benchmark.urllib.request, "urlopen", fake_urlopen)
    monkeypatch.setattr(benchmark.time, "time", lambda: 42.5)

    status = benchmark.fetch_rust_jobs_status(
        "http://127.0.0.1:7788/",
        require_status=True,
    )

    assert status == {
        "queuedJobs": 2,
        "runningJobs": 1,
        "sampledAtMs": 42500,
    }


def test_fixture_server_code_can_record_converter_count(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()

    code = benchmark.fixture_server_code("127.0.0.1", 50051, tmp_path / "count.txt")

    assert "CONVERTER_COUNT_PATH" in code
    assert "self.calls += 1" in code
    assert "write_text(str(self.calls)" in code


def test_real_docling_server_code_can_record_converter_count(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()

    code = benchmark.real_docling_server_code(
        "127.0.0.1",
        50051,
        tmp_path / "docling-fixtures",
        False,
        tmp_path / "count.txt",
    )

    assert "class CountingConverter" in code
    assert "converter = CountingConverter(converter)" in code
    assert "write_text(str(self.calls)" in code


def test_converter_count_path_reads_external_fake_counter(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    count_path = tmp_path / "count.txt"
    count_path.write_text("9", encoding="utf-8")
    args = benchmark.argparse.Namespace(converter_count_path=count_path)

    assert benchmark.read_converter_count(args) == 9


def test_cargo_perf_probe_uses_minimal_feature_set(monkeypatch, tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    report_path = tmp_path / "report.json"
    commands: list[list[str]] = []

    def fake_run(command: list[str], *, check: bool, env) -> None:
        commands.append(command)
        assert check
        assert env["WENDAO_DOCUMENT_EXTRACT_PERF_ENDPOINT"] == "http://127.0.0.1:50052"
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
        wait_ms=100,
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

    assert "--no-default-features" in commands[0]
    assert commands[0][commands[0].index("--features") + 1] == (
        "performance,studio,zhenfa-router,duckdb"
    )
    assert commands[0][commands[0].index("--test") + 1] == "xiuxian-testing-gate"
    report = benchmark.json.loads(report_path.read_text(encoding="utf-8"))
    assert report["rustJobsStatusSummary"]["sampleCount"] == 0


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
        gateway_features="studio,zhenfa-router,duckdb,builtin-plugins",
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
        "xiuxian-wendao",
        "--no-default-features",
        "--features",
        "studio,zhenfa-router,duckdb,builtin-plugins",
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
    assert env["WENDAO_DOCUMENT_EXTRACT_ENDPOINT"] == "http://127.0.0.1:51051"
    assert env["VALKEY_URL"] == "redis://127.0.0.1:51079/0"
    assert env["XIUXIAN_WENDAO_SEARCH_PLANE_VALKEY_URL"] == (
        "redis://127.0.0.1:51079/0"
    )
    assert env["XIUXIAN_WENDAO_GATEWAY_BOOTSTRAP_BACKGROUND_INDEXING"] == "false"
    config = (tmp_path / "gateway" / "wendao.toml").read_text(encoding="utf-8")
    assert "[search.cache]" in config
    assert 'valkey_url = "redis://127.0.0.1:51079/0"' in config


def test_start_valkey_server_uses_temp_runtime_flags(monkeypatch, tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    calls = []

    class FakePopen:
        def __init__(self, command, **kwargs):
            calls.append((command, kwargs))

    monkeypatch.setattr(benchmark.subprocess, "Popen", FakePopen)

    benchmark.start_valkey_server(host="127.0.0.1", port=51079, temp_root=tmp_path)

    command, kwargs = calls[0]
    assert command[:5] == ["valkey-server", "--bind", "127.0.0.1", "--port", "51079"]
    assert "--appendonly" in command
    assert "no" in command
    assert kwargs["start_new_session"] is True


def test_summary_reports_duplicate_miss_converter_calls() -> None:
    benchmark = _load_benchmark_module()

    summary = benchmark.summarize_results(
        [
            {
                "totalRows": 10,
                "forceErrorRows": 0,
                "cacheErrorRows": 0,
                "requestCount": 2,
                "arrowIpcBytes": 1024,
                "cacheSpeedup": 2.0,
                "duplicateMissConverterCalls": 1,
            }
        ]
    )

    assert summary["totalDuplicateMissConverterCalls"] == 1
    assert summary["maxDuplicateMissConverterCalls"] == 1
    assert summary["rustJobsStatusSummary"]["sampleCount"] == 0
