from __future__ import annotations

import importlib.util
import tomllib
from pathlib import Path

import pytest


def _load_benchmark_module():
    repo_root = Path(__file__).resolve().parents[5]
    script_path = repo_root / "tests" / "scripts" / "benchmark_wendao_document_extract.py"
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


def test_attachment_classification_covers_docling_real_lanes(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()

    assert benchmark.classify_attachment("pdf", tmp_path / "paper.pdf") == "pdf"
    assert benchmark.classify_attachment("pdf-rtl-01", tmp_path / "rtl.pdf") == "pdf"
    assert benchmark.classify_attachment("docx", tmp_path / "word.docx") == "office"
    assert benchmark.classify_attachment("pptx", tmp_path / "deck.pptx") == "office"
    assert benchmark.classify_attachment("xlsx", tmp_path / "book.xlsx") == "office"
    assert benchmark.classify_attachment("markdown", tmp_path / "wiki.md") == "structured_text"
    assert benchmark.classify_attachment("latex", tmp_path / "paper.tex") == ("structured_text")
    assert benchmark.classify_attachment("html", tmp_path / "wiki.html") == "web"
    assert benchmark.classify_attachment("csv", tmp_path / "rows.csv") == "table_data"
    assert benchmark.classify_attachment("image-png", tmp_path / "page.png") == "image"
    assert benchmark.classify_attachment("jats-xml", tmp_path / "article.xml") == "xml"
    assert benchmark.classify_attachment("mets-gbs", tmp_path / "book.tar.gz") == "archive_document"
    assert (
        benchmark.classify_attachment("docling-json", tmp_path / "docling.json") == "docling_json"
    )
    assert benchmark.classify_attachment("webvtt", tmp_path / "captions.vtt") == ("subtitle")
    assert benchmark.classify_attachment("audio", tmp_path / "sample.mp3") == "audio"
    assert benchmark.classify_attachment("custom", tmp_path / "unknown.bin") == ("unknown")


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


def test_docling_real_fixtures_keep_pdf_corpus_opt_in(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    fixture_paths = {
        **benchmark.DOCLING_REAL_FIXTURE_PATHS,
        **benchmark.DOCLING_REAL_PDF_CORPUS_FIXTURE_PATHS,
    }
    for relative_path in fixture_paths.values():
        fixture_path = tmp_path / relative_path
        fixture_path.parent.mkdir(parents=True, exist_ok=True)
        fixture_path.write_bytes(b"fixture")

    default_fixtures = benchmark.docling_real_fixtures(
        tmp_path,
        include_audio=False,
    )
    corpus_fixtures = benchmark.docling_real_fixtures(
        tmp_path,
        include_audio=False,
        include_pdf_corpus=True,
    )

    assert "pdf-redp5110-sampled" not in default_fixtures
    assert "pdf-redp5110-sampled" in corpus_fixtures
    assert "pdf" in corpus_fixtures
    assert corpus_fixtures["pdf-redp5110-sampled"].name == "redp5110_sampled.pdf"
    assert "audio" not in corpus_fixtures


def test_select_fixtures_filters_named_fixture(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    fixtures = {
        "pdf": tmp_path / "sample.pdf",
        "audio": tmp_path / "sample.mp3",
    }

    selected = benchmark.select_fixtures(fixtures, ["audio"])

    assert selected == {"audio": tmp_path / "sample.mp3"}


def test_parse_extra_fixtures_resolves_existing_files(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    pdf_fixture = tmp_path / "2604.17337.pdf"
    pdf_fixture.write_bytes(b"%PDF")

    fixtures = benchmark.parse_extra_fixtures([f"arxiv-2604-17337={pdf_fixture}"])

    assert fixtures == {"arxiv-2604-17337": pdf_fixture.resolve()}


def test_parse_extra_fixtures_rejects_missing_files(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()

    try:
        benchmark.parse_extra_fixtures([f"arxiv-2604-17337={tmp_path / 'missing.pdf'}"])
    except SystemExit as error:
        assert "Extra fixture path does not exist" in str(error)
    else:
        raise AssertionError("missing extra fixture should fail")


def test_merge_extra_fixtures_rejects_alias_collision(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    pdf_fixture = tmp_path / "sample.pdf"
    pdf_fixture.write_bytes(b"%PDF")

    try:
        benchmark.merge_extra_fixtures(
            {"pdf": tmp_path / "base.pdf"},
            [f"pdf={pdf_fixture}"],
        )
    except SystemExit as error:
        assert "collides with existing fixture" in str(error)
    else:
        raise AssertionError("colliding extra fixture should fail")


def test_prepare_distinct_miss_fixtures_writes_unique_fake_inputs(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        distinct_miss_concurrency=4,
        duplicate_miss_concurrency=0,
        fixture_suite="fake",
        flight_mode="async",
    )

    fixtures = benchmark.prepare_distinct_miss_fixtures(
        args,
        {},
        tmp_path / "distinct-fixtures",
    )

    assert list(fixtures) == [
        "distinct-01-markdown",
        "distinct-02-docx",
        "distinct-03-image",
        "distinct-04-audio",
    ]
    assert len({path.read_bytes() for path in fixtures.values()}) == 4


def test_document_extras_cover_xbrl_and_audio_asr() -> None:
    package_root = Path(__file__).resolve().parents[2]
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

    assert (
        benchmark.resolve_docling_source_root(None)
        == (tmp_path / "docling-real-fixtures").resolve()
    )


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
                "maxPdfOcrWorkers": 8,
                "currentPdfOcrWorkerBudget": 3,
                "availablePdfOcrWorkerPermits": 6,
                "inProcessPdfOcrWorkers": 2,
                "inFlightPdfOcrShards": 5,
                "pdfOcrCacheHits": 10,
                "pdfOcrCacheMisses": 4,
                "pdfOcrLiveRequests": 1,
                "pdfOcrQueueWaitP95Ms": 7,
                "pdfOcrLatencyP95Ms": 80,
                "pdfOcrSourcePdfPageRangeShards": 6,
                "pdfOcrRenderedPageShards": 2,
                "pdfOcrRenderedRegionShards": 1,
                "pdfOcrBudgetIncreaseEvents": 2,
                "pdfOcrBudgetDecreaseEvents": 0,
            },
            {
                "queuedJobs": 1,
                "runningJobs": 2,
                "inProcessRunningConversions": 2,
                "inProcessScheduledJobs": 2,
                "availableConversionPermits": 1,
                "maxRunningConversions": 4,
                "maxPdfOcrWorkers": 8,
                "currentPdfOcrWorkerBudget": 4,
                "availablePdfOcrWorkerPermits": 5,
                "inProcessPdfOcrWorkers": 3,
                "inFlightPdfOcrShards": 2,
                "pdfOcrCacheHits": 12,
                "pdfOcrCacheMisses": 8,
                "pdfOcrLiveRequests": 2,
                "pdfOcrQueueWaitP95Ms": 9,
                "pdfOcrLatencyP95Ms": 120,
                "pdfOcrSourcePdfPageRangeShards": 8,
                "pdfOcrRenderedPageShards": 2,
                "pdfOcrRenderedRegionShards": 3,
                "pdfOcrBudgetIncreaseEvents": 3,
                "pdfOcrBudgetDecreaseEvents": 1,
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
    assert summary["maxPdfOcrWorkers"] == 8
    assert summary["maxCurrentPdfOcrWorkerBudget"] == 4
    assert summary["minAvailablePdfOcrWorkerPermits"] == 5
    assert summary["maxInFlightPdfOcrShards"] == 5
    assert summary["maxPdfOcrCacheHits"] == 12
    assert summary["maxPdfOcrCacheMisses"] == 8
    assert summary["maxPdfOcrLiveRequests"] == 2
    assert summary["maxPdfOcrLatencyP95Ms"] == 120
    assert summary["maxPdfOcrSourcePdfPageRangeShards"] == 8
    assert summary["maxPdfOcrRenderedRegionShards"] == 3
    assert summary["maxPdfOcrBudgetDecreaseEvents"] == 1
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
                "maxPdfOcrWorkers": 8,
                "maxCurrentPdfOcrWorkerBudget": 3,
                "minAvailablePdfOcrWorkerPermits": 6,
                "maxInProcessPdfOcrWorkers": 2,
                "maxInFlightPdfOcrShards": 4,
                "maxPdfOcrCacheHits": 10,
                "maxPdfOcrCacheMisses": 5,
                "maxPdfOcrLiveRequests": 2,
                "maxPdfOcrQueueWaitP95Ms": 20,
                "maxPdfOcrLatencyP95Ms": 300,
                "maxPdfOcrSourcePdfPageRangeShards": 7,
                "maxPdfOcrRenderedPageShards": 2,
                "maxPdfOcrRenderedRegionShards": 1,
                "maxPdfOcrBudgetIncreaseEvents": 1,
                "maxPdfOcrBudgetDecreaseEvents": 0,
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
                "maxPdfOcrWorkers": 8,
                "maxCurrentPdfOcrWorkerBudget": 5,
                "minAvailablePdfOcrWorkerPermits": 3,
                "maxInProcessPdfOcrWorkers": 5,
                "maxInFlightPdfOcrShards": 1,
                "maxPdfOcrCacheHits": 30,
                "maxPdfOcrCacheMisses": 9,
                "maxPdfOcrLiveRequests": 4,
                "maxPdfOcrQueueWaitP95Ms": 40,
                "maxPdfOcrLatencyP95Ms": 500,
                "maxPdfOcrSourcePdfPageRangeShards": 8,
                "maxPdfOcrRenderedPageShards": 3,
                "maxPdfOcrRenderedRegionShards": 2,
                "maxPdfOcrBudgetIncreaseEvents": 2,
                "maxPdfOcrBudgetDecreaseEvents": 1,
                "lastConversionDurationMs": 80,
                "maxConversionDurationMs": 120,
            },
        ]
    )

    assert combined["sampleCount"] == 3
    assert combined["maxQueuedJobs"] == 4
    assert combined["maxRunningJobs"] == 2
    assert combined["minAvailableConversionPermits"] == 2
    assert combined["maxCurrentPdfOcrWorkerBudget"] == 5
    assert combined["minAvailablePdfOcrWorkerPermits"] == 3
    assert combined["maxInProcessPdfOcrWorkers"] == 5
    assert combined["maxPdfOcrCacheHits"] == 30
    assert combined["maxPdfOcrLatencyP95Ms"] == 500
    assert combined["maxPdfOcrBudgetDecreaseEvents"] == 1
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


def test_rust_pdf_ocr_endpoint_pool_normalizes_repeated_endpoints() -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        rust_pdf_ocr_endpoint=[
            " http://127.0.0.1:52051/ ",
            "",
            "http://127.0.0.1:52052",
        ]
    )

    assert benchmark.rust_pdf_ocr_endpoint_pool(args) == (
        "http://127.0.0.1:52051,http://127.0.0.1:52052"
    )


def test_rust_document_extract_endpoint_pool_normalizes_repeated_endpoints() -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        rust_document_extract_endpoint=[
            " http://127.0.0.1:53051/ ",
            "",
            "http://127.0.0.1:53052",
            "http://127.0.0.1:53051",
        ]
    )

    assert benchmark.rust_document_extract_endpoint_pool(args) == (
        "http://127.0.0.1:53051,http://127.0.0.1:53052"
    )


def test_fixture_server_code_can_record_converter_count(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()

    code = benchmark.fixture_server_code(
        "127.0.0.1",
        50051,
        tmp_path / "count.txt",
        "fixture",
    )

    assert "CONVERTER_COUNT_PATH" in code
    assert "self.calls += 1" in code
    assert "write_text(str(self.calls)" in code
    assert "class FixtureOcrWorker" in code


def test_real_docling_server_code_can_record_converter_count(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()

    code = benchmark.real_docling_server_code(
        "127.0.0.1",
        50051,
        tmp_path / "docling-fixtures",
        False,
        tmp_path / "count.txt",
        "docling",
    )

    assert "class CountingConverter" in code
    assert "return CountingConverter(converter)" in code
    assert "def make_converter()" in code
    assert "DoclingPdfOcrShardWorker(" in code
    assert "converter_factory=make_converter" in code
    assert "max_workers='auto'" in code
    assert "write_text(str(self.calls)" in code


def test_python_worker_command_adds_workspace_package_and_extras() -> None:
    benchmark = _load_benchmark_module()

    command = benchmark.python_worker_command(
        "print('worker')",
        uv_package="xiuxian-wendao-analyzer",
        uv_extras=["documents", "documents-audio"],
    )

    assert command == [
        "uv",
        "run",
        "--package",
        "xiuxian-wendao-analyzer",
        "--extra",
        "documents",
        "--extra",
        "documents-audio",
        "python",
        "-c",
        "print('worker')",
    ]


def test_start_server_pool_starts_counted_local_ocr_endpoints(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    calls = []
    extra_ports = iter([52052, 52053])

    class FakeProcess:
        pass

    def fake_pick_free_port(host: str) -> int:
        assert host == "127.0.0.1"
        return next(extra_ports)

    def fake_start_server(host: str, port: int, **kwargs):
        calls.append((host, port, kwargs))
        return FakeProcess()

    monkeypatch.setattr(benchmark._workers, "pick_free_port", fake_pick_free_port)
    monkeypatch.setattr(benchmark._workers, "start_server", fake_start_server)

    workers = benchmark.start_server_pool(
        "127.0.0.1",
        52051,
        endpoint_count=3,
        real_docling=False,
        real_fixture_root=None,
        include_audio=False,
        converter_count_path=tmp_path / "counts",
        pdf_ocr_worker="fixture",
        pdf_ocr_workers="auto",
        python_uv_package="xiuxian-wendao-analyzer",
        python_uv_extras=[],
        log_dir=tmp_path / "logs",
    )

    assert [worker.port for worker in workers] == [52051, 52052, 52053]
    assert [worker.endpoint_url for worker in workers] == [
        "http://127.0.0.1:52051",
        "http://127.0.0.1:52052",
        "http://127.0.0.1:52053",
    ]
    assert [call[2]["process_name"] for call in calls] == [
        "python-worker-0",
        "python-worker-1",
        "python-worker-2",
    ]
    assert [call[2]["converter_count_path"].name for call in calls] == [
        "python-worker-0.txt",
        "python-worker-1.txt",
        "python-worker-2.txt",
    ]


def test_converter_count_path_reads_external_fake_counter(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    count_path = tmp_path / "count.txt"
    count_path.write_text("9", encoding="utf-8")
    args = benchmark.argparse.Namespace(converter_count_path=count_path)

    assert benchmark.read_converter_count(args) == 9


def test_converter_count_path_sums_local_worker_counter_dir(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    count_dir = tmp_path / "counts"
    count_dir.mkdir()
    (count_dir / "python-worker-0.txt").write_text("3", encoding="utf-8")
    (count_dir / "python-worker-1.txt").write_text("4", encoding="utf-8")
    args = benchmark.argparse.Namespace(converter_count_path=count_dir)

    assert benchmark.read_converter_count(args) == 7


def test_artifact_report_summary_tracks_structure_precision() -> None:
    benchmark = _load_benchmark_module()

    summary = benchmark.summarize_artifact_reports(
        [
            {
                "resourcesArrowExists": True,
                "resourcesRowCount": 3,
                "structureArrowExists": True,
                "structureRowCount": 3,
                "structureOcrPageBlocks": 1,
                "structureOcrRegionBlocks": 2,
                "structureBboxBlocks": 2,
                "structureReadingOrderSorted": True,
                "structureOrderSignature": "order-a",
                "structureOrderFirstKey": "000000|000000.000000|000000|a",
                "structureOrderLastKey": "000000|000000.000002|000002|c",
                "structureParity": {
                    "baselineBlockCount": 2,
                    "candidateBlockCount": 3,
                    "baselinePageCount": 1,
                    "candidatePageCount": 1,
                    "baselineTextChars": 80,
                    "candidateTextChars": 120,
                    "protectedBlockCounts": {},
                },
                "structureParityError": None,
                "metricsArrowExists": True,
                "metricsRowCount": 3,
                "metricsResultChars": 120,
                "metricsBboxCount": 2,
                "metricsRustSchedulerElapsedMs": 10.5,
                "documentTimingArrowExists": True,
                "documentTimingRowCount": 3,
                "documentTimingTotalElapsedMs": 20.0,
                "documentTimingPhaseElapsedMs": {
                    "doclingConvert": 15.0,
                    "writeResourcesArrow": 2.0,
                    "total": 20.0,
                },
                "imageAttachmentAudit": {
                    "format": "png",
                    "widthPx": 640,
                    "heightPx": 480,
                    "pixelCount": 307200,
                    "dimensionSource": "png_ihdr",
                    "rustAccelerationCandidate": "image_ocr_cache_candidate",
                },
                "archiveAttachmentAudit": {
                    "archiveFormat": "tar.gz",
                    "memberCount": 10,
                    "regularFileCount": 10,
                    "xmlMemberCount": 1,
                    "imageMemberCount": 3,
                    "totalMemberSizeBytes": 267702,
                    "extensionCounts": {
                        "html": 3,
                        "tif": 3,
                        "txt": 3,
                        "xml": 1,
                    },
                    "largestMemberSizeBytes": 59518,
                    "rustAccelerationCandidate": "mets_gbs_member_manifest_candidate",
                },
                "artifactError": None,
            },
            {
                "resourcesArrowExists": True,
                "resourcesRowCount": 1,
                "structureArrowExists": True,
                "structureRowCount": 1,
                "structureOcrPageBlocks": 0,
                "structureOcrRegionBlocks": 1,
                "structureBboxBlocks": 1,
                "structureReadingOrderSorted": True,
                "structureOrderSignature": "order-b",
                "structureOrderFirstKey": "000001|000001.000000|000000|d",
                "structureOrderLastKey": "000001|000001.000000|000000|d",
                "metricsArrowExists": True,
                "metricsRowCount": 1,
                "metricsResultChars": 40,
                "metricsBboxCount": 1,
                "metricsRustSchedulerElapsedMs": 2.5,
                "documentTimingArrowExists": True,
                "documentTimingRowCount": 2,
                "documentTimingTotalElapsedMs": 5.0,
                "documentTimingPhaseElapsedMs": {
                    "doclingConvert": 4.0,
                    "total": 5.0,
                },
                "artifactError": None,
            },
        ]
    )

    assert summary["resourcesArrowExists"] is True
    assert summary["resourcesRows"] == 4
    assert summary["structureArrowExists"] is True
    assert summary["structureRows"] == 4
    assert summary["structureOcrPageBlocks"] == 1
    assert summary["structureOcrRegionBlocks"] == 3
    assert summary["structureBboxBlocks"] == 3
    assert summary["structureReadingOrderSorted"] is True
    assert summary["structureParityChecked"] is True
    assert summary["structureParityPassed"] is True
    assert summary["structureParityErrorCount"] == 0
    assert summary["metricsArrowExists"] is True
    assert summary["metricsRows"] == 4
    assert summary["metricsResultChars"] == 160
    assert summary["metricsBboxCount"] == 3
    assert summary["metricsRustSchedulerElapsedMs"] == 13.0
    assert summary["documentTimingArrowExists"] is True
    assert summary["documentTimingRows"] == 5
    assert summary["documentTimingTotalElapsedMs"] == 25.0
    assert summary["documentTimingPhaseElapsedMs"] == {
        "doclingConvert": 19.0,
        "total": 25.0,
        "writeResourcesArrow": 2.0,
    }
    assert summary["imageAttachmentAuditCount"] == 1
    assert summary["imageKnownDimensionCount"] == 1
    assert summary["imageFormatCounts"] == {"png": 1}
    assert summary["imageDimensionSourceCounts"] == {"png_ihdr": 1}
    assert summary["imageAccelerationCandidates"] == {"image_ocr_cache_candidate": 1}
    assert summary["maxImageWidthPx"] == 640
    assert summary["maxImageHeightPx"] == 480
    assert summary["maxImagePixelCount"] == 307200
    assert summary["archiveAttachmentAuditCount"] == 1
    assert summary["archiveMemberCount"] == 10
    assert summary["archiveRegularFileCount"] == 10
    assert summary["archiveXmlMemberCount"] == 1
    assert summary["archiveImageMemberCount"] == 3
    assert summary["archiveTotalMemberSizeBytes"] == 267702
    assert summary["archiveFormatCounts"] == {"tar.gz": 1}
    assert summary["archiveExtensionCounts"] == {
        "html": 3,
        "tif": 3,
        "txt": 3,
        "xml": 1,
    }
    assert summary["archiveAccelerationCandidates"] == {
        "mets_gbs_member_manifest_candidate": 1,
    }
    assert summary["maxArchiveLargestMemberSizeBytes"] == 59518
    assert summary["artifactErrorCount"] == 0


def test_structure_order_consistency_compares_force_cache_and_shard_reuse() -> None:
    benchmark = _load_benchmark_module()

    def report(signature: str) -> dict[str, object]:
        return {
            "artifactReports": [
                {
                    "structureArrowExists": True,
                    "structureRowCount": 21,
                    "structureOrderSignature": signature,
                    "structureOrderFirstKey": "000000|000000.000000|000000|page-0",
                    "structureOrderLastKey": "000020|000020.000000|000020|page-20",
                }
            ]
        }

    stable = benchmark.fixture_structure_order_consistency(
        report("same-order"),
        report("same-order"),
        report("same-order"),
    )
    mismatch = benchmark.fixture_structure_order_consistency(
        report("force-order"),
        report("cache-order"),
    )

    assert stable["structureOrderStable"] is True
    assert stable["structureOrderComparedRuns"] == 3
    assert stable["structureOrderMismatchCount"] == 0
    assert stable["structureOrderFirstKey"] == "000000|000000.000000|000000|page-0"
    assert stable["structureOrderLastKey"] == "000020|000020.000000|000020|page-20"
    assert mismatch["structureOrderStable"] is False
    assert mismatch["structureOrderMismatchCount"] == 1


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


def test_cargo_perf_probe_adds_pdf_source_range_for_hybrid_page_ocr(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    report_path = tmp_path / "report.json"
    commands: list[list[str]] = []

    def fake_run(command: list[str], *, check: bool, env) -> None:
        commands.append(command)
        assert check
        assert env["WENDAO_DOCUMENT_EXTRACT_PERF_MODE"] == "hybrid-page-ocr"
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
    )

    benchmark.run_cargo_perf_test(
        args,
        tmp_path / "sample.pdf",
        tmp_path / "out",
        force=False,
        iterations=1,
        concurrency=1,
        report_path=report_path,
    )

    assert commands[0][commands[0].index("--features") + 1] == (
        "performance,studio,zhenfa-router,duckdb,document-extract-pdf-source-range"
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


@pytest.mark.parametrize(
    ("flight_mode", "artifact_registry_reuse_probe", "expected"),
    [
        ("sync", False, False),
        ("sync", True, True),
        ("async", False, True),
        ("hybrid-page-ocr", False, True),
    ],
)
def test_artifact_registry_reuse_probe_routes_through_rust_provider(
    flight_mode: str,
    artifact_registry_reuse_probe: bool,
    expected: bool,
) -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        flight_mode=flight_mode,
        artifact_registry_reuse_probe=artifact_registry_reuse_probe,
    )

    assert benchmark.should_start_local_rust_provider(args) is expected


def test_report_payload_exposes_top_level_precision_speed_summary(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        real_docling=False,
        benchmark_host="127.0.0.1",
        benchmark_port=50052,
        rust_rest_endpoint=None,
        iterations=1,
        concurrency=1,
        flight_mode="sync",
        wait_ms=0,
        pdf_ocr_worker="skip",
        pdf_ocr_workers="auto",
        local_python_ocr_endpoint_count=1,
        rust_pdf_ocr_workers=None,
        rust_pdf_ocr_source_range_workers=None,
        rust_document_extract_endpoint=[],
        rust_pdf_ocr_endpoint=[],
        structure_baseline_root=None,
        shard_cache_reuse_probe=False,
        artifact_registry_reuse_probe=True,
        ocr_shard_cache_root=tmp_path / "ocr-shards",
    )
    result = {
        "fixture": "markdown",
        "attachmentClass": "structured_text",
        "forceRefreshMs": 10.0,
        "artifactRegistryReuseForceMs": 4.0,
        "cacheHitP95Ms": 2.0,
        "wallTimeMs": 3.0,
        "cacheSpeedup": 5.0,
        "forceErrorRows": 0,
        "artifactRegistryReuseErrorRows": 0,
        "cacheErrorRows": 0,
        "totalRows": 2,
        "requestCount": 1,
        "arrowIpcBytes": 64,
        "duplicateMissConverterCalls": None,
        "structureRows": 1,
        "structureReadingOrderSorted": True,
        "structureOrderStable": True,
        "structureOrderMismatchCount": 0,
    }

    payload = benchmark.build_report_payload(
        args,
        real_fixture_root=None,
        results=[result],
        distinct_miss_report=None,
        structure_baseline_report=None,
        ocr_shard_cache_summary={"root": str(tmp_path), "fileCount": 0, "totalBytes": 0},
    )

    assert payload["precisionSpeedSummary"] == payload["summary"]["precisionSpeedSummary"]
    assert payload["precisionSpeedSummary"]["maxArtifactRegistryReuseForceMs"] == 4.0


def test_run_structure_baseline_probe_generates_sync_fixture_baselines(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    calls = []

    def fake_run_cargo_perf_test(
        args,
        source,
        output_dir,
        *,
        force,
        iterations,
        concurrency,
        report_path,
        flight_mode,
        include_structure_baseline_root,
        **_kwargs,
    ):
        calls.append(
            {
                "source": source,
                "output_dir": output_dir,
                "force": force,
                "iterations": iterations,
                "concurrency": concurrency,
                "report_path": report_path,
                "flight_mode": flight_mode,
                "include_structure_baseline_root": include_structure_baseline_root,
            }
        )
        return {
            "errorRowCount": 0,
            "artifactReports": [
                {
                    "resourcesArrowExists": True,
                    "resourcesRowCount": 2,
                    "structureArrowExists": True,
                    "structureRowCount": 2,
                    "structureReadingOrderSorted": True,
                }
            ],
        }

    monkeypatch.setattr(benchmark, "run_cargo_perf_test", fake_run_cargo_perf_test)
    args = benchmark.argparse.Namespace(
        generate_structure_baselines=True,
        fail_on_error_rows=True,
    )
    baseline_root = tmp_path / "baselines"

    report = benchmark.run_structure_baseline_probe(
        args,
        {
            "pdf": tmp_path / "sample.pdf",
            "image": tmp_path / "sample.png",
        },
        baseline_root,
    )

    assert report["enabled"] is True
    assert report["root"] == str(baseline_root)
    assert report["fixtureCount"] == 2
    assert report["totalStructureRows"] == 4
    assert report["allStructureReadingOrderSorted"] is True
    assert [call["output_dir"].name for call in calls] == ["pdf", "image"]
    assert all(call["flight_mode"] == "sync" for call in calls)
    assert all(call["force"] is True for call in calls)
    assert all(call["include_structure_baseline_root"] is False for call in calls)


def test_structure_baseline_root_defaults_to_report_dir_when_generating(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        generate_structure_baselines=True,
        structure_baseline_root=None,
    )

    assert (
        benchmark.resolve_structure_baseline_root(args, tmp_path)
        == (tmp_path / "structure-baselines").resolve()
    )


def test_pdf_render_shard_audit_command_adds_feature_and_fixture_manifest(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        cargo="cargo",
        cargo_features="performance,studio,zhenfa-router,duckdb",
        pdfium_library_path=None,
        prepare_pdfium_runtime=False,
        require_pdfium=False,
        pdf_render_selection="all-pages",
        pdf_render_region=[],
    )

    command, env = benchmark.build_pdf_render_shard_audit_command(
        args,
        {"pdf": tmp_path / "sample.pdf"},
        tmp_path / "reports",
    )

    assert command[:4] == ["cargo", "test", "-p", "xiuxian-wendao"]
    assert command[command.index("--test") + 1] == "xiuxian-testing-gate"
    assert command[command.index("--features") + 1] == (
        "performance,studio,zhenfa-router,duckdb,document-extract-pdf-render"
    )
    assert command[-4:] == [
        "pdf_render_page_render_shard_manifest",
        "--",
        "--ignored",
        "--nocapture",
    ]
    inputs = benchmark.json.loads(env["WENDAO_PDF_RENDER_SHARD_INPUTS_JSON"])
    assert inputs == [{"name": "pdf", "source": str(tmp_path / "sample.pdf")}]
    assert env["WENDAO_PDF_RENDER_SHARD_REPORT_DIR"] == str(tmp_path / "reports")
    assert env["WENDAO_PDF_RENDER_SELECTION"] == "all_pages"


def test_pdf_render_shard_audit_can_pin_pdfium_runtime_path(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    pdfium_library = tmp_path / "libpdfium.dylib"
    pdfium_library.write_bytes(b"pdfium")
    args = benchmark.argparse.Namespace(
        cargo="cargo",
        cargo_features="performance",
        pdfium_library_path=pdfium_library,
        prepare_pdfium_runtime=False,
        require_pdfium=True,
        pdf_render_selection="shard-fallback-pages",
        pdf_render_region=[],
    )

    _command, env = benchmark.build_pdf_render_shard_audit_command(
        args,
        {"pdf": tmp_path / "sample.pdf"},
        tmp_path / "reports",
    )

    assert env["WENDAO_PDFIUM_LIBRARY_PATH"] == str(pdfium_library.resolve())
    assert env["WENDAO_PDF_RENDER_REQUIRE_PDFIUM"] == "1"
    assert env["WENDAO_PDF_RENDER_SELECTION"] == "shard_fallback_pages"


def test_pdf_render_region_shard_audit_emits_region_manifest(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        cargo="cargo",
        cargo_features="performance",
        pdfium_library_path=None,
        prepare_pdfium_runtime=False,
        require_pdfium=False,
        pdf_render_selection="region-shards",
        pdf_render_region=[
            "pdf=0,2,72,80,540,700,000000.000002",
        ],
    )

    _command, env = benchmark.build_pdf_render_shard_audit_command(
        args,
        {"pdf": tmp_path / "sample.pdf"},
        tmp_path / "reports",
    )

    assert env["WENDAO_PDF_RENDER_SELECTION"] == "region_shards"
    regions = benchmark.json.loads(env["WENDAO_PDF_RENDER_REGIONS_JSON"])
    assert regions == [
        {
            "source": str(tmp_path / "sample.pdf"),
            "regions": [
                {
                    "pageIndex": 0,
                    "regionIndex": 2,
                    "regionBox": {
                        "left": 72.0,
                        "bottom": 80.0,
                        "right": 540.0,
                        "top": 700.0,
                    },
                    "readingOrderKey": "000000.000002",
                }
            ],
        }
    ]


def test_pdf_render_region_shard_audit_requires_all_selected_regions(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        cargo="cargo",
        cargo_features="performance",
        pdfium_library_path=None,
        prepare_pdfium_runtime=False,
        require_pdfium=False,
        pdf_render_selection="region-shards",
        pdf_render_region=["pdf=0,0,72,72,540,700"],
    )

    try:
        benchmark.build_pdf_render_shard_audit_command(
            args,
            {
                "pdf": tmp_path / "sample.pdf",
                "other-pdf": tmp_path / "other.pdf",
            },
            tmp_path / "reports",
        )
    except SystemExit as error:
        assert "Missing --pdf-render-region" in str(error)
    else:
        raise AssertionError("missing selected fixture region should fail")


def test_pdf_render_region_rejects_non_region_selection(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        cargo="cargo",
        cargo_features="performance",
        pdfium_library_path=None,
        prepare_pdfium_runtime=False,
        require_pdfium=False,
        pdf_render_selection="all-pages",
        pdf_render_region=["pdf=0,0,72,72,540,700"],
    )

    try:
        benchmark.build_pdf_render_shard_audit_command(
            args,
            {"pdf": tmp_path / "sample.pdf"},
            tmp_path / "reports",
        )
    except SystemExit as error:
        assert "--pdf-render-region requires" in str(error)
    else:
        raise AssertionError("region fixture on page selection should fail")


def test_hybrid_pdf_render_region_env_uses_selected_fixtures(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        hybrid_pdf_render_selection="region-shards",
        pdf_render_region=["pdf=0,4,10,20,110,220,000000.000004"],
        benchmark_fixtures={"pdf": tmp_path / "sample.pdf"},
    )

    env = benchmark.build_hybrid_pdf_render_region_env(args)

    regions = benchmark.json.loads(env["WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_JSON"])
    assert regions == [
        {
            "source": str(tmp_path / "sample.pdf"),
            "regions": [
                {
                    "pageIndex": 0,
                    "regionIndex": 4,
                    "regionBox": {
                        "left": 10.0,
                        "bottom": 20.0,
                        "right": 110.0,
                        "top": 220.0,
                    },
                    "readingOrderKey": "000000.000004",
                }
            ],
        }
    ]


def test_hybrid_pdf_render_region_env_ignores_non_region_selection(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    args = benchmark.argparse.Namespace(
        hybrid_pdf_render_selection="shard-fallback-pages",
        pdf_render_region=["pdf=0,0,10,20,110,220"],
        benchmark_fixtures={"pdf": tmp_path / "sample.pdf"},
    )

    assert benchmark.build_hybrid_pdf_render_region_env(args) == {}


def test_pdfium_asset_selection_covers_primary_platforms() -> None:
    benchmark = _load_benchmark_module()

    assert (
        benchmark.pdfium_asset_name(sys_platform="darwin", machine="arm64")
        == "pdfium-mac-arm64.tgz"
    )
    assert (
        benchmark.pdfium_asset_name(sys_platform="linux", machine="x86_64")
        == "pdfium-linux-x64.tgz"
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


def test_normalize_render_selection_accepts_cli_spelling() -> None:
    benchmark = _load_benchmark_module()

    assert benchmark.normalize_render_selection("shard-fallback-pages") == ("shard_fallback_pages")
    assert benchmark.normalize_render_selection("region-shards") == "region_shards"


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

    manifest = benchmark.json.loads(captured_env["WENDAO_DOCUMENT_EXTRACT_PERF_INPUTS_JSON"])
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
        gateway_features="studio,zhenfa-router,duckdb,builtin-plugins",
        rust_pdf_ocr_workers="6",
        rust_pdf_ocr_source_range_workers="2",
        rust_pdf_ocr_endpoint=[
            "http://127.0.0.1:52051",
            "http://127.0.0.1:52052/",
        ],
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
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_OCR_WORKERS"] == "6"
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS"] == "2"
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
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_SELECTION"] == ("shard_fallback_pages")
    assert env["VALKEY_URL"] == "redis://127.0.0.1:51079/0"
    assert env["XIUXIAN_WENDAO_SEARCH_PLANE_VALKEY_URL"] == ("redis://127.0.0.1:51079/0")
    assert env["XIUXIAN_WENDAO_GATEWAY_BOOTSTRAP_BACKGROUND_INDEXING"] == "false"
    config = (tmp_path / "gateway" / "wendao.toml").read_text(encoding="utf-8")
    assert "[search.cache]" in config
    assert 'valkey_url = "redis://127.0.0.1:51079/0"' in config


def test_start_rust_provider_forwards_hybrid_region_env(
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
        rust_provider_features="studio,zhenfa-router,duckdb,builtin-plugins",
        flight_mode="hybrid-page-ocr",
        hybrid_pdf_render_selection="region-shards",
        pdf_render_region=["pdf=0,1,10,20,110,220"],
        benchmark_fixtures={"pdf": tmp_path / "sample.pdf"},
        pdfium_library_path=None,
        prepare_pdfium_runtime=False,
        rust_pdf_ocr_workers="6",
        rust_pdf_ocr_source_range_workers="2",
        rust_pdf_ocr_endpoint=["http://127.0.0.1:52051"],
        rust_document_extract_endpoint=["http://127.0.0.1:53051"],
    )

    benchmark.start_rust_provider_server(
        args,
        rust_host="127.0.0.1",
        rust_port=51052,
        python_host="127.0.0.1",
        python_port=51051,
        temp_root=tmp_path,
    )

    _command, kwargs = calls[0]
    env = kwargs["env"]
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_OCR_WORKERS"] == "6"
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_OCR_SOURCE_RANGE_WORKERS"] == "2"
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_OCR_ENDPOINTS"] == ("http://127.0.0.1:52051")
    assert env["WENDAO_DOCUMENT_EXTRACT_ENDPOINTS"] == "http://127.0.0.1:53051"
    assert env["WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_ROOT"] == str(
        (tmp_path / "ocr-shard-cache").resolve()
    )
    assert env["WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_SELECTION"] == "region_shards"
    regions = benchmark.json.loads(env["WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_JSON"])
    assert regions[0]["source"] == str(tmp_path / "sample.pdf")
    assert regions[0]["regions"][0]["regionIndex"] == 1


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
    assert summary["precisionSpeedSummary"]["precisionGatePassed"] is True
    assert summary["precisionSpeedSummary"]["errorRows"] == 0


def test_precision_speed_summary_tracks_quality_and_latency() -> None:
    benchmark = _load_benchmark_module()

    summary = benchmark.summarize_results(
        [
            {
                "totalRows": 21,
                "forceErrorRows": 0,
                "cacheErrorRows": 0,
                "shardCacheReuseErrorRows": 0,
                "requestCount": 2,
                "arrowIpcBytes": 2048,
                "cacheSpeedup": 12.5,
                "duplicateMissConverterCalls": 1,
                "artifactErrorCount": 0,
                "structureRows": 21,
                "structureOcrPageBlocks": 21,
                "structureOcrRegionBlocks": 0,
                "structureBboxBlocks": 21,
                "structureReadingOrderSorted": True,
                "structureOrderStable": True,
                "structureOrderMismatchCount": 0,
                "structureParityChecked": True,
                "structureParityPassed": True,
                "structureParityErrorCount": 0,
                "metricsRows": 21,
                "metricsResultChars": 4096,
                "metricsBboxCount": 21,
                "metricsRustSchedulerElapsedMs": 45.5,
                "documentTimingTotalElapsedMs": 950.0,
                "documentTimingOverheadMs": 50.0,
                "documentTimingPhaseElapsedMs": {
                    "doclingConvert": 900.0,
                    "total": 950.0,
                },
                "forceRefreshMs": 1000.0,
                "cacheHitP95Ms": 4.0,
                "shardCacheReuseForceMs": 80.0,
                "artifactRegistryReuseForceMs": 12.0,
                "wallTimeMs": 1005.0,
            }
        ],
        {
            "errorRows": 0,
            "wallTimeMs": 25.0,
            "rustJobsStatusSummary": benchmark.summarize_rust_jobs_status_samples([]),
        },
    )

    precision_speed = summary["precisionSpeedSummary"]
    assert precision_speed["precisionGatePassed"] is True
    assert precision_speed["structureReadingOrderSorted"] is True
    assert precision_speed["structureOrderStable"] is True
    assert precision_speed["structureOrderMismatches"] == 0
    assert precision_speed["structureParityPassed"] is True
    assert precision_speed["ocrPageBlocks"] == 21
    assert precision_speed["bboxBlocks"] == 21
    assert precision_speed["maxForceRefreshMs"] == 1000.0
    assert precision_speed["maxCacheHitP95Ms"] == 4.0
    assert precision_speed["maxShardCacheReuseForceMs"] == 80.0
    assert precision_speed["maxArtifactRegistryReuseForceMs"] == 12.0
    assert precision_speed["totalRustSchedulerElapsedMs"] == 45.5
    assert precision_speed["totalDocumentTimingElapsedMs"] == 950.0
    assert precision_speed["totalDoclingConvertMs"] == 900.0
    assert precision_speed["maxDoclingConvertMs"] == 900.0
    assert precision_speed["maxDoclingConvertShare"] == pytest.approx(900.0 / 950.0)
    assert precision_speed["maxDocumentTimingOverheadMs"] == 50.0
    assert precision_speed["maxDocumentTimingOverheadShare"] == pytest.approx(0.05)
    assert precision_speed["distinctMissWallTimeMs"] == 25.0


def test_attachment_class_summary_groups_precision_and_speed() -> None:
    benchmark = _load_benchmark_module()

    summary = benchmark.summarize_results(
        [
            {
                "fixture": "docx",
                "attachmentClass": "office",
                "totalRows": 10,
                "forceErrorRows": 0,
                "cacheErrorRows": 0,
                "shardCacheReuseErrorRows": 0,
                "requestCount": 2,
                "arrowIpcBytes": 100,
                "cacheSpeedup": 4.0,
                "duplicateMissConverterCalls": 1,
                "artifactErrorCount": 0,
                "structureRows": 4,
                "structureOcrPageBlocks": 0,
                "structureOcrRegionBlocks": 0,
                "structureBboxBlocks": 0,
                "structureReadingOrderSorted": True,
                "structureOrderStable": True,
                "structureOrderMismatchCount": 0,
                "structureParityPassed": None,
                "structureParityErrorCount": 0,
                "metricsRows": 0,
                "metricsResultChars": 0,
                "metricsBboxCount": 0,
                "metricsRustSchedulerElapsedMs": 0.0,
                "documentTimingRows": 3,
                "documentTimingTotalElapsedMs": 18.0,
                "documentTimingOverheadMs": 2.0,
                "documentTimingPhaseElapsedMs": {
                    "doclingConvert": 12.0,
                    "total": 18.0,
                },
                "forceRefreshMs": 20.0,
                "cacheHitP95Ms": 2.0,
                "wallTimeMs": 3.0,
                "resourcesRows": 4,
                "artifactReports": [
                    {
                        "resourceTypeCounts": {
                            "document": 1,
                            "docling_json": 1,
                            "image": 1,
                            "table": 1,
                        },
                        "resourceStatusCounts": {"ok": 4},
                        "structureBlockTypeCounts": {
                            "document": 1,
                            "image": 1,
                            "table": 1,
                        },
                        "metricsStatusCounts": {},
                        "documentTimingStatusCounts": {"ok": 3},
                        "documentTimingPhaseElapsedMs": {
                            "doclingConvert": 12.0,
                            "total": 18.0,
                        },
                    }
                ],
            },
            {
                "fixture": "image-png",
                "attachmentClass": "image",
                "totalRows": 5,
                "forceErrorRows": 0,
                "cacheErrorRows": 0,
                "shardCacheReuseErrorRows": 0,
                "requestCount": 1,
                "arrowIpcBytes": 80,
                "cacheSpeedup": 2.0,
                "duplicateMissConverterCalls": 1,
                "artifactErrorCount": 0,
                "structureRows": 1,
                "structureOcrPageBlocks": 0,
                "structureOcrRegionBlocks": 0,
                "structureBboxBlocks": 0,
                "structureReadingOrderSorted": True,
                "structureOrderStable": True,
                "structureOrderMismatchCount": 0,
                "structureParityPassed": None,
                "structureParityErrorCount": 0,
                "metricsRows": 0,
                "metricsResultChars": 0,
                "metricsBboxCount": 0,
                "metricsRustSchedulerElapsedMs": 0.0,
                "documentTimingRows": 3,
                "documentTimingTotalElapsedMs": 45.0,
                "documentTimingOverheadMs": 5.0,
                "documentTimingPhaseElapsedMs": {
                    "doclingConvert": 40.0,
                    "total": 45.0,
                },
                "forceRefreshMs": 50.0,
                "cacheHitP95Ms": 5.0,
                "wallTimeMs": 6.0,
                "resourcesRows": 3,
                "artifactReports": [
                    {
                        "resourceTypeCounts": {
                            "document": 1,
                            "docling_json": 1,
                            "table": 1,
                        },
                        "resourceStatusCounts": {"ok": 3},
                        "structureBlockTypeCounts": {"document": 1, "table": 1},
                        "metricsStatusCounts": {},
                        "documentTimingStatusCounts": {"ok": 3},
                        "documentTimingPhaseElapsedMs": {
                            "doclingConvert": 40.0,
                            "total": 45.0,
                        },
                        "imageAttachmentAudit": {
                            "format": "png",
                            "widthPx": 640,
                            "heightPx": 480,
                            "pixelCount": 307200,
                            "dimensionSource": "png_ihdr",
                            "rustAccelerationCandidate": ("image_ocr_cache_candidate"),
                        },
                    }
                ],
            },
        ],
    )

    class_summary = {item["attachmentClass"]: item for item in summary["attachmentClassSummary"]}
    assert set(class_summary) == {"image", "office"}
    assert summary["imageAttachmentAuditCount"] == 1
    assert summary["imageKnownDimensionCount"] == 1
    assert summary["imageFormatCounts"] == {"png": 1}
    assert summary["imageDimensionSourceCounts"] == {"png_ihdr": 1}
    assert summary["imageAccelerationCandidates"] == {"image_ocr_cache_candidate": 1}
    assert summary["maxImageWidthPx"] == 640
    assert summary["maxImageHeightPx"] == 480
    assert summary["maxImagePixelCount"] == 307200
    assert class_summary["office"]["fixtureCount"] == 1
    assert class_summary["office"]["fixtures"] == ["docx"]
    assert class_summary["office"]["precisionSpeedSummary"]["precisionGatePassed"] is True
    assert class_summary["office"]["precisionSpeedSummary"]["maxForceRefreshMs"] == 20.0
    assert class_summary["office"]["resourcesRows"] == 4
    assert class_summary["office"]["resourceTypeCounts"] == {
        "docling_json": 1,
        "document": 1,
        "image": 1,
        "table": 1,
    }
    assert class_summary["office"]["structureBlockTypeCounts"] == {
        "document": 1,
        "image": 1,
        "table": 1,
    }
    assert class_summary["office"]["slowestForceFixture"] == {
        "fixture": "docx",
        "latencyMs": 20.0,
    }
    assert class_summary["office"]["documentTimingTotalElapsedMs"] == 18.0
    assert class_summary["office"]["documentTimingOverheadMs"] == 2.0
    assert class_summary["office"]["documentTimingStatusCounts"] == {"ok": 3}
    assert class_summary["office"]["precisionSpeedSummary"][
        "maxDoclingConvertShare"
    ] == pytest.approx(12.0 / 18.0)
    assert class_summary["office"]["precisionSpeedSummary"][
        "maxDocumentTimingOverheadShare"
    ] == pytest.approx(0.1)
    assert class_summary["image"]["structureRows"] == 1
    assert class_summary["image"]["resourceTypeCounts"]["table"] == 1
    assert class_summary["image"]["imageAttachmentAuditCount"] == 1
    assert class_summary["image"]["imageKnownDimensionCount"] == 1
    assert class_summary["image"]["imageFormatCounts"] == {"png": 1}
    assert class_summary["image"]["imageDimensionSourceCounts"] == {"png_ihdr": 1}
    assert class_summary["image"]["imageAccelerationCandidates"] == {"image_ocr_cache_candidate": 1}
    assert class_summary["image"]["maxImageWidthPx"] == 640
    assert class_summary["image"]["maxImageHeightPx"] == 480
    assert class_summary["image"]["maxImagePixelCount"] == 307200
    assert class_summary["image"]["slowestCacheP95Fixture"] == {
        "fixture": "image-png",
        "latencyMs": 5.0,
    }
    assert class_summary["image"]["slowestTimingOverheadFixture"] == {
        "fixture": "image-png",
        "latencyMs": 5.0,
    }
    assert class_summary["image"]["precisionSpeedSummary"]["maxCacheHitP95Ms"] == 5.0
    assert class_summary["image"]["precisionSpeedSummary"]["maxDocumentTimingOverheadMs"] == 5.0
    assert class_summary["image"]["documentTimingPhaseElapsedMs"] == {
        "doclingConvert": 40.0,
        "total": 45.0,
    }
    assert class_summary["image"]["precisionSpeedSummary"]["maxDoclingConvertMs"] == (40.0)
    assert class_summary["image"]["precisionSpeedSummary"][
        "maxDoclingConvertShare"
    ] == pytest.approx(40.0 / 45.0)


def test_summarize_ocr_shard_cache_reports_root_files_and_limits(monkeypatch, tmp_path) -> None:
    benchmark = _load_benchmark_module()
    cache_root = tmp_path / "ocr-shards"
    (cache_root / "aa").mkdir(parents=True)
    (cache_root / "aa" / "one.arrow").write_bytes(b"123")
    (cache_root / "bb").mkdir()
    (cache_root / "bb" / "two.arrow").write_bytes(b"4567")
    monkeypatch.setenv("WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_ROOT", str(cache_root))
    monkeypatch.setenv("WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_BYTES", "100")
    monkeypatch.setenv("WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_ENTRIES", "10")

    summary = benchmark.summarize_ocr_shard_cache()

    assert summary["root"] == str(cache_root.resolve())
    assert summary["fileCount"] == 2
    assert summary["totalBytes"] == 7
    assert summary["maxBytes"] == 100
    assert summary["maxEntries"] == 10


def test_benchmark_ocr_shard_cache_root_defaults_to_temp_for_local_runs(
    monkeypatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    monkeypatch.delenv("WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_ROOT", raising=False)
    args = benchmark.argparse.Namespace(
        ocr_shard_cache_root=None,
        external_endpoint=False,
    )

    assert (
        benchmark.benchmark_ocr_shard_cache_root(args, tmp_path)
        == (tmp_path / "ocr-shard-cache").resolve()
    )


def test_benchmark_ocr_shard_cache_root_honors_explicit_root(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    explicit_root = tmp_path / "explicit-ocr-shards"
    args = benchmark.argparse.Namespace(
        ocr_shard_cache_root=explicit_root,
        external_endpoint=False,
    )

    assert benchmark.benchmark_ocr_shard_cache_root(args, tmp_path) == explicit_root.resolve()


def test_run_fixture_probe_can_measure_cache_reuse_probes(monkeypatch, tmp_path) -> None:
    benchmark = _load_benchmark_module()
    calls = []

    def fake_run_cargo_perf_test(
        args,
        source,
        output_dir,
        *,
        force,
        iterations,
        concurrency,
        report_path,
        **_kwargs,
    ):
        calls.append(
            {
                "source": source,
                "output_dir": output_dir,
                "force": force,
                "iterations": iterations,
                "concurrency": concurrency,
                "report_path": report_path,
            }
        )
        latency_by_report = {
            "force.json": 1000.0,
            "shard-cache-reuse.json": 42.0,
            "artifact-registry-reuse.json": 9.0,
            "cache.json": 4.0,
        }
        latency = latency_by_report[report_path.name]
        return {
            "latenciesMs": [latency],
            "requestCount": 1,
            "rowCount": 21,
            "batchCount": 1,
            "arrowIpcBytes": 117128,
            "wallTimeMs": latency,
            "concurrency": concurrency,
            "errorRowCount": 0,
            "statusCounts": {"succeeded": 21},
            "maxRssKb": None,
            "artifactReports": [
                {
                    "resourcesArrowExists": True,
                    "resourcesRowCount": 21,
                    "structureArrowExists": True,
                    "structureRowCount": 21,
                    "structureOcrPageBlocks": 21,
                    "structureOcrRegionBlocks": 0,
                    "structureBboxBlocks": 21,
                    "structureReadingOrderSorted": True,
                    "structureOrderSignature": "stable-order",
                    "structureOrderFirstKey": "000000|000000.000000|000000|page-0",
                    "structureOrderLastKey": "000020|000020.000000|000020|page-20",
                    "metricsArrowExists": True,
                    "metricsRowCount": 21,
                    "metricsResultChars": 2048,
                    "metricsBboxCount": 21,
                    "metricsRustSchedulerElapsedMs": 40.0,
                }
            ],
        }

    monkeypatch.setattr(benchmark, "run_cargo_perf_test", fake_run_cargo_perf_test)
    args = benchmark.argparse.Namespace(
        duplicate_miss_concurrency=0,
        fail_on_error_rows=True,
        fail_on_duplicate_conversions=False,
        fail_on_structure_order_mismatch=True,
        iterations=1,
        concurrency=1,
        shard_cache_reuse_probe=True,
        artifact_registry_reuse_probe=True,
    )

    result = benchmark.run_fixture_probe(
        args,
        "arxiv",
        tmp_path / "source.pdf",
        tmp_path / "out",
    )

    assert [call["report_path"].name for call in calls] == [
        "force.json",
        "shard-cache-reuse.json",
        "artifact-registry-reuse.json",
        "cache.json",
    ]
    assert calls[1]["output_dir"] == tmp_path / "out" / "shard-cache-reuse"
    assert calls[1]["force"] is True
    assert calls[2]["output_dir"] == tmp_path / "out" / "artifact-registry-reuse"
    assert calls[2]["force"] is False
    assert result["shardCacheReuseEnabled"] is True
    assert result["shardCacheReuseForceMs"] == 42.0
    assert result["shardCacheReuseErrorRows"] == 0
    assert result["artifactRegistryReuseEnabled"] is True
    assert result["artifactRegistryReuseForceMs"] == 9.0
    assert result["artifactRegistryReuseErrorRows"] == 0
    assert result["cacheHitP50Ms"] == 4.0
    assert result["metricsRows"] == 21
    assert result["metricsResultChars"] == 2048
    assert result["metricsBboxCount"] == 21
    assert result["structureOrderStable"] is True
    assert result["structureOrderComparedRuns"] == 4
    assert result["structureOrderMismatchCount"] == 0


def test_run_fixture_probe_can_fail_on_structure_order_mismatch(
    monkeypatch,
    tmp_path,
) -> None:
    benchmark = _load_benchmark_module()

    def fake_run_cargo_perf_test(
        args,
        source,
        output_dir,
        *,
        force,
        iterations,
        concurrency,
        report_path,
        **_kwargs,
    ):
        signature_by_report = {
            "force.json": "force-order",
            "shard-cache-reuse.json": "force-order",
            "cache.json": "cache-order",
        }
        return {
            "latenciesMs": [1.0],
            "requestCount": 1,
            "rowCount": 1,
            "batchCount": 1,
            "arrowIpcBytes": 1,
            "wallTimeMs": 1.0,
            "concurrency": concurrency,
            "errorRowCount": 0,
            "statusCounts": {"succeeded": 1},
            "artifactReports": [
                {
                    "structureArrowExists": True,
                    "structureRowCount": 1,
                    "structureReadingOrderSorted": True,
                    "structureOrderSignature": signature_by_report[report_path.name],
                    "structureOrderFirstKey": "000000|000000.000000|000000|a",
                    "structureOrderLastKey": "000000|000000.000000|000000|a",
                }
            ],
        }

    monkeypatch.setattr(benchmark, "run_cargo_perf_test", fake_run_cargo_perf_test)
    args = benchmark.argparse.Namespace(
        duplicate_miss_concurrency=0,
        fail_on_error_rows=True,
        fail_on_duplicate_conversions=False,
        fail_on_structure_order_mismatch=True,
        iterations=1,
        concurrency=1,
        shard_cache_reuse_probe=True,
        artifact_registry_reuse_probe=False,
    )

    with pytest.raises(SystemExit, match="unstable structure order"):
        benchmark.run_fixture_probe(
            args,
            "arxiv",
            tmp_path / "source.pdf",
            tmp_path / "out",
        )


def test_summary_and_markdown_report_distinct_miss_burst() -> None:
    benchmark = _load_benchmark_module()
    result = {
        "fixture": "small-md",
        "totalRows": 10,
        "forceErrorRows": 0,
        "cacheErrorRows": 0,
        "shardCacheReuseEnabled": True,
        "shardCacheReuseForceMs": 42.0,
        "shardCacheReuseErrorRows": 0,
        "artifactRegistryReuseEnabled": True,
        "artifactRegistryReuseForceMs": 9.0,
        "artifactRegistryReuseErrorRows": 0,
        "requestCount": 2,
        "arrowIpcBytes": 1024,
        "cacheSpeedup": 2.0,
        "duplicateMissConverterCalls": None,
        "rustJobsStatusSummary": benchmark.summarize_rust_jobs_status_samples([]),
        "rows": 5,
        "forceRefreshMs": 10.0,
        "cacheHitP50Ms": 1.0,
        "cacheHitP95Ms": 2.0,
        "wallTimeMs": 3.0,
        "cacheMaxRssKb": None,
        "rustJobsMaxQueuedJobs": None,
        "rustJobsMaxRunningJobs": None,
        "rustJobsMinAvailableConversionPermits": None,
        "metricsRows": 2,
        "metricsResultChars": 80,
        "metricsBboxCount": 2,
        "metricsRustSchedulerElapsedMs": 12.0,
        "documentTimingRows": 3,
        "documentTimingTotalElapsedMs": 30.0,
        "documentTimingOverheadMs": 8.0,
        "documentTimingPhaseElapsedMs": {
            "doclingConvert": 20.0,
            "total": 30.0,
        },
        "structureParityChecked": True,
        "structureParityPassed": True,
        "structureParityErrorCount": 0,
        "structureOrderStable": True,
        "structureOrderMismatchCount": 0,
        "artifactReports": [
            {
                "resourceTypeCounts": {"document": 1, "table": 1},
                "resourceStatusCounts": {"ok": 2},
                "structureBlockTypeCounts": {"document": 1, "table": 1},
                "metricsStatusCounts": {"succeeded": 2},
                "documentTimingStatusCounts": {"ok": 3},
                "documentTimingPhaseElapsedMs": {
                    "doclingConvert": 20.0,
                    "total": 30.0,
                },
                "imageAttachmentAudit": {
                    "format": "png",
                    "widthPx": 640,
                    "heightPx": 480,
                    "pixelCount": 307200,
                    "dimensionSource": "png_ihdr",
                    "rustAccelerationCandidate": "image_ocr_cache_candidate",
                },
                "archiveAttachmentAudit": {
                    "archiveFormat": "tar.gz",
                    "memberCount": 10,
                    "regularFileCount": 10,
                    "xmlMemberCount": 1,
                    "imageMemberCount": 3,
                    "totalMemberSizeBytes": 267702,
                    "extensionCounts": {
                        "html": 3,
                        "tif": 3,
                        "txt": 3,
                        "xml": 1,
                    },
                    "largestMemberSizeBytes": 59518,
                    "rustAccelerationCandidate": "mets_gbs_member_manifest_candidate",
                },
            }
        ],
    }
    distinct_report = {
        "enabled": True,
        "fixtures": ["distinct-01", "distinct-02"],
        "fixtureCount": 2,
        "requestCount": 2,
        "converterCalls": 2,
        "errorRows": 0,
        "wallTimeMs": 25.0,
        "rustJobsStatusSummary": {
            "sampleCount": 3,
            "maxQueuedJobs": 2,
            "maxRunningJobs": 2,
            "maxInProcessRunningConversions": 2,
            "maxInProcessScheduledJobs": 2,
            "minAvailableConversionPermits": 2,
            "maxRunningConversions": 4,
            "lastConversionDurationMs": 20,
            "maxConversionDurationMs": 21,
        },
    }

    summary = benchmark.summarize_results([result], distinct_report)

    assert summary["distinctMissFixtureCount"] == 2
    assert summary["distinctMissConverterCalls"] == 2
    assert summary["totalErrorRows"] == 0
    assert summary["rustJobsStatusSummary"]["maxRunningJobs"] == 2
    assert summary["totalDocumentTimingRows"] == 3
    assert summary["totalDocumentTimingElapsedMs"] == 30.0
    assert summary["totalDocumentTimingOverheadMs"] == 8.0
    assert summary["imageAttachmentAuditCount"] == 1
    assert summary["imageKnownDimensionCount"] == 1
    assert summary["imageFormatCounts"] == {"png": 1}
    assert summary["imageDimensionSourceCounts"] == {"png_ihdr": 1}
    assert summary["imageAccelerationCandidates"] == {"image_ocr_cache_candidate": 1}
    assert summary["maxImageWidthPx"] == 640
    assert summary["maxImageHeightPx"] == 480
    assert summary["maxImagePixelCount"] == 307200
    assert summary["archiveAttachmentAuditCount"] == 1
    assert summary["archiveMemberCount"] == 10
    assert summary["archiveXmlMemberCount"] == 1
    assert summary["archiveImageMemberCount"] == 3
    assert summary["archiveFormatCounts"] == {"tar.gz": 1}
    assert summary["archiveExtensionCounts"] == {
        "html": 3,
        "tif": 3,
        "txt": 3,
        "xml": 1,
    }
    assert summary["archiveAccelerationCandidates"] == {
        "mets_gbs_member_manifest_candidate": 1,
    }
    assert summary["maxArchiveLargestMemberSizeBytes"] == 59518
    assert summary["documentTimingPhaseElapsedMs"] == {
        "doclingConvert": 20.0,
        "total": 30.0,
    }
    assert summary["precisionSpeedSummary"]["maxForceRefreshMs"] == 10.0
    assert summary["precisionSpeedSummary"]["maxCacheHitP95Ms"] == 2.0
    assert summary["precisionSpeedSummary"]["totalDoclingConvertMs"] == 20.0
    assert summary["precisionSpeedSummary"]["maxDoclingConvertMs"] == 20.0
    assert summary["precisionSpeedSummary"]["maxDoclingConvertShare"] == pytest.approx(20.0 / 30.0)
    assert summary["precisionSpeedSummary"]["maxDocumentTimingOverheadShare"] == pytest.approx(0.8)
    assert summary["precisionSpeedSummary"]["precisionGatePassed"] is True
    assert summary["precisionSpeedSummary"]["structureOrderStable"] is True
    assert summary["attachmentClassSummary"][0]["attachmentClass"] == "unknown"
    assert summary["attachmentClassSummary"][0]["archiveAttachmentAuditCount"] == 1
    assert summary["attachmentClassSummary"][0]["archiveMemberCount"] == 10
    assert summary["attachmentClassSummary"][0]["archiveFormatCounts"] == {"tar.gz": 1}
    assert summary["attachmentClassSummary"][0]["archiveAccelerationCandidates"] == {
        "mets_gbs_member_manifest_candidate": 1,
    }

    markdown = benchmark.render_markdown(
        {
            "schema": benchmark.REPORT_SCHEMA,
            "mode": "fixture",
            "endpoint": "http://127.0.0.1:50052",
            "rustRestEndpoint": None,
            "iterations": 1,
            "concurrency": 1,
            "flightMode": "async",
            "waitMs": 0,
            "pdfOcrWorker": "skip",
            "pdfOcrWorkers": "auto",
            "rustPdfOcrWorkers": None,
            "rustPdfOcrSourceRangeWorkers": "2",
            "structureBaselineRoot": "/tmp/baselines",
            "pdfOcrProfile": "skip",
            "shardCacheReuseProbe": True,
            "artifactRegistryReuseProbe": True,
            "ocrShardCache": {
                "root": "/tmp/ocr-shards",
                "fileCount": 2,
                "totalBytes": 7,
                "maxBytes": 100,
            },
            "summary": summary,
            "results": [result],
            "distinctMiss": distinct_report,
            "structureBaseline": {
                "enabled": True,
                "fixtureCount": 1,
                "totalErrorRows": 0,
            },
        }
    )
    assert "## Distinct Cold Miss Burst" in markdown
    assert "## Attachment Class Summary" in markdown
    assert "document=1, table=1" in markdown
    assert "image_ocr_cache_candidate=1" in markdown
    assert "small-md:10.000" in markdown
    assert "distinct-01" in markdown
    assert "Shard reuse force ms" in markdown
    assert "Artifact-registry reuse probe" in markdown
    assert "Artifact reuse ms" in markdown
    assert "9.000" in markdown
    assert "42.000" in markdown
    assert "OCR shard cache" in markdown
    assert "files=2" in markdown
    assert "Metrics sidecar" in markdown
    assert "chars=80" in markdown
    assert "Document timing sidecar" in markdown
    assert "Image audit summary" in markdown
    assert "knownDims=1" in markdown
    assert "dimensionSources=png_ihdr=1" in markdown
    assert "Archive audit summary" in markdown
    assert "members=10" in markdown
    assert "suffixes=html=3, tif=3, txt=3, xml=1" in markdown
    assert "mets_gbs_member_manifest_candidate=1" in markdown
    assert "doclingConvert=20.000" in markdown
    assert "overheadMs=8.000" in markdown
    assert "maxDoclingConvertMs=20.000" in markdown
    assert "maxDoclingShare=66.7%" in markdown
    assert "maxTimingOverheadMs=8.000" in markdown
    assert "maxBoundaryOverheadShare=80.0%" in markdown
    assert "Rust PDF OCR source-range workers" in markdown
    assert "Structure parity" in markdown
    assert "Structure order stable across runs" in markdown
    assert "Structure baseline generation" in markdown
    assert "Precision-speed summary" in markdown
    assert "orderStable=True" in markdown
    assert "maxForceMs=10.000" in markdown
