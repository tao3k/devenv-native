"""document_extract_perf_script test slice 9."""

from __future__ import annotations

from .support import (
    _load_benchmark_module,
)


def test_summarize_ocr_shard_cache_reports_root_files_and_limits(
    monkeypatch, tmp_path
) -> None:
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
