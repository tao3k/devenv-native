"""document_extract_perf_script test slice 11."""

from __future__ import annotations

from .support import Path, _load_benchmark_module, pytest


def test_docling_groundtruth_exact_markdown_and_json_match(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    source = tmp_path / "sample.pdf"
    source.write_bytes(b"%PDF")
    groundtruth_root = tmp_path / "groundtruth"
    output_dir = tmp_path / "out"
    groundtruth_root.mkdir()
    output_dir.mkdir()
    (groundtruth_root / "sample.md").write_text("# Title\n\nBody\n", encoding="utf-8")
    (groundtruth_root / "sample.json").write_text(
        '{"texts":[{"text":"Body"}]}', encoding="utf-8"
    )
    (output_dir / "sample.md").write_text("# Title\n\nBody\n", encoding="utf-8")
    (output_dir / "sample.docling.json").write_text(
        '{"texts":[{"text":"Body"}]}',
        encoding="utf-8",
    )

    report = benchmark.compare_artifact_to_docling_groundtruth(
        source=source,
        output_dir=output_dir,
        groundtruth_root=groundtruth_root,
    )

    assert report["checked"] is True
    assert report["passed"] is True
    assert report["markdownExactMatch"] is True
    assert report["jsonExactMatch"] is True
    assert report["charCoverageRatio"] == 1.0
    assert report["markdownSimilarity"] == 1.0


def test_docling_groundtruth_reads_structure_arrow_for_hybrid_candidate(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    source = tmp_path / "sample.pdf"
    source.write_bytes(b"%PDF")
    groundtruth_root = tmp_path / "groundtruth"
    output_dir = tmp_path / "out"
    groundtruth_root.mkdir()
    output_dir.mkdir()
    (groundtruth_root / "sample.md").write_text(
        "# Title\n\nA much longer upstream body.\n",
        encoding="utf-8",
    )
    _write_structure_arrow(
        output_dir / "_structure.arrow",
        [
            ("000001", "short"),
            ("000000", "# Title"),
        ],
    )

    report = benchmark.compare_artifact_to_docling_groundtruth(
        source=source,
        output_dir=output_dir,
        groundtruth_root=groundtruth_root,
        min_char_coverage=0.98,
        min_similarity=0.98,
    )

    assert report["checked"] is True
    assert report["passed"] is False
    assert report["candidateTextSource"].endswith("_structure.arrow")
    assert report["candidateTextChars"] == len("# Title\n\nshort")
    assert report["failures"]


def test_docling_groundtruth_summary_tracks_missing_and_failures(
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    summary = benchmark.summarize_docling_groundtruth_reports(
        [
            {"checked": True, "passed": True, "markdownSimilarity": 1.0},
            {
                "checked": True,
                "passed": False,
                "markdownSimilarity": 0.7,
                "failures": ["low similarity"],
                "groundtruthStem": "bad",
            },
            {"checked": False, "missingReason": "missing"},
        ]
    )

    assert summary["checked"] is True
    assert summary["checkedCount"] == 2
    assert summary["missingCount"] == 1
    assert summary["failureCount"] == 1
    assert summary["passed"] is False
    assert summary["minMarkdownSimilarity"] == 0.7


def test_run_fixture_probe_can_fail_on_docling_groundtruth_mismatch(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    benchmark = _load_benchmark_module()
    source = tmp_path / "sample.pdf"
    source.write_bytes(b"%PDF")
    groundtruth_root = tmp_path / "groundtruth"
    groundtruth_root.mkdir()
    (groundtruth_root / "sample.md").write_text("expected markdown", encoding="utf-8")

    def fake_run_cargo_perf_test(
        args: object,
        _source: Path,
        output_dir: Path,
        *,
        force: bool,
        iterations: int,
        concurrency: int,
        report_path: Path,
        **_kwargs: object,
    ) -> dict[str, object]:
        _ = args, force, iterations, concurrency
        output_dir.mkdir(parents=True, exist_ok=True)
        (output_dir / "sample.md").write_text("short", encoding="utf-8")
        return {
            "latenciesMs": [1.0],
            "requestCount": 1,
            "rowCount": 1,
            "batchCount": 1,
            "arrowIpcBytes": 1,
            "wallTimeMs": 1.0,
            "concurrency": 1,
            "errorRowCount": 0,
            "statusCounts": {"succeeded": 1},
            "artifactReports": [
                {
                    "source": str(source),
                    "outputDir": str(output_dir),
                    "resourcesArrowExists": True,
                    "resourcesRowCount": 1,
                    "structureArrowExists": True,
                    "structureRowCount": 1,
                    "structureOcrPageBlocks": 1,
                    "structureOcrRegionBlocks": 0,
                    "structureBboxBlocks": 1,
                    "structureReadingOrderSorted": True,
                    "structureOrderSignature": "order",
                    "metricsArrowExists": True,
                    "metricsRowCount": 1,
                    "metricsResultChars": 5,
                    "metricsBboxCount": 1,
                    "metricsRustSchedulerElapsedMs": 1.0,
                    "hybridPageOcrTimingTotalElapsedMs": 1.0,
                    "hybridPageOcrTimingPhaseElapsedMs": {},
                    "hybridPageOcrTimingOcr2RegionShardCount": 0,
                    "hybridPageOcrTimingOcr2RegionRequestCount": 0,
                    "hybridPageOcrTimingOcr2RegionRenderedShardCount": 0,
                    "hybridPageOcrTimingOcr2RegionRenderCacheHitCount": 0,
                    "hybridPageOcrTimingOcr2RegionRenderCacheMissCount": 0,
                    "hybridPageOcrTimingSchedulerTrace": [],
                }
            ],
        }

    monkeypatch.setattr(benchmark, "run_cargo_perf_test", fake_run_cargo_perf_test)
    args = benchmark.argparse.Namespace(
        duplicate_miss_concurrency=0,
        fail_on_error_rows=True,
        fail_on_duplicate_conversions=False,
        fail_on_missing_ocr_metrics=False,
        fail_on_structure_order_mismatch=True,
        fail_on_docling_groundtruth_mismatch=True,
        compare_docling_groundtruth=True,
        docling_groundtruth_root=groundtruth_root,
        docling_groundtruth_min_char_coverage=0.98,
        docling_groundtruth_min_similarity=0.98,
        iterations=1,
        concurrency=1,
        shard_cache_reuse_probe=False,
        artifact_registry_reuse_probe=False,
    )

    with pytest.raises(SystemExit, match="upstream Docling groundtruth"):
        benchmark.run_fixture_probe(args, "pdf", source, tmp_path / "out")


def _write_structure_arrow(path: Path, rows: list[tuple[str, str]]) -> None:
    import pyarrow as pa
    import pyarrow.ipc as ipc

    batch = pa.record_batch(
        [
            pa.array([row[0] for row in rows], type=pa.string()),
            pa.array([row[1] for row in rows], type=pa.string()),
        ],
        names=["readingOrderKey", "content"],
    )
    with path.open("wb") as handle, ipc.new_file(handle, batch.schema) as writer:
        writer.write_batch(batch)
