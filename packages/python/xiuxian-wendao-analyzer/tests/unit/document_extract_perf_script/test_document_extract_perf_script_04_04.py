"""document_extract_perf_script test slice 4."""

from __future__ import annotations

import json

from .support import (
    Path,
    _load_benchmark_module,
)


def test_summarize_hosted_vlm_ocr_request_traces(tmp_path: Path) -> None:
    benchmark = _load_benchmark_module()
    log_dir = tmp_path / "logs"
    log_dir.mkdir()
    (log_dir / "python-worker.hosted-vlm-ocr.jsonl").write_text(
        "\n".join(
            [
                json.dumps(
                    {
                        "status": "succeeded",
                        "httpStatus": 200,
                        "startedUnixMs": 1_000,
                        "endedUnixMs": 1_010,
                        "latencyMs": 10.0,
                        "model": "baidu/qianfan-ocr-fast:free",
                        "markdownChars": 100,
                        "imageBytes": 2048,
                        "pageCount": 2,
                        "requestKind": "page-window-canary",
                        "httpAttemptCount": 1,
                        "shardCount": 2,
                        "shardTypeCounts": {"page": 2},
                        "sourcePixelArea": 2000,
                        "renderDpi": 300,
                        "scaffoldMode": "disabled",
                        "imageOptimizationMode": "disabled",
                        "scaffoldAppliedCount": 0,
                        "scaffoldValidationFailureCount": 0,
                        "scaffoldJsonChars": 0,
                        "canonicalMarkdownChars": 0,
                    }
                ),
                json.dumps(
                    {
                        "status": "failed",
                        "httpStatus": 429,
                        "startedUnixMs": 1_005,
                        "endedUnixMs": 1_035,
                        "latencyMs": 30.0,
                        "model": "baidu/qianfan-ocr-fast:free",
                        "markdownChars": 0,
                        "imageBytes": 1024,
                        "requestKind": "region",
                        "httpAttemptCount": 2,
                        "shardCount": 1,
                        "shardTypeCounts": {"region": 1},
                        "sourcePixelArea": 400,
                        "renderDpi": 300,
                        "scaffoldMode": "region-table-json",
                        "imageOptimizationMode": "region-whitespace-trim",
                        "scaffoldAppliedCount": 1,
                        "scaffoldValidationFailureCount": 1,
                        "scaffoldJsonChars": 17,
                        "canonicalMarkdownChars": 0,
                    }
                ),
                "{bad-json",
            ]
        ),
        encoding="utf-8",
    )

    summary = benchmark.summarize_hosted_vlm_ocr_request_traces(log_dir)

    assert summary["traceFileCount"] == 1
    assert summary["requestCount"] == 2
    assert summary["httpAttemptCountTotal"] == 3
    assert summary["successCount"] == 1
    assert summary["failureCount"] == 1
    assert summary["parseErrorCount"] == 1
    assert summary["statusCounts"] == {"failed": 1, "succeeded": 1}
    assert summary["httpStatusCounts"] == {"200": 1, "429": 1}
    assert summary["modelCounts"] == {"baidu/qianfan-ocr-fast:free": 2}
    assert summary["requestKindCounts"] == {
        "page-window-canary": 1,
        "region": 1,
    }
    assert summary["scaffoldModeCounts"] == {
        "disabled": 1,
        "region-table-json": 1,
    }
    assert summary["imageOptimizationModeCounts"] == {
        "disabled": 1,
        "region-whitespace-trim": 1,
    }
    assert summary["shardTypeCounts"] == {"page": 2, "region": 1}
    assert summary["renderDpiCounts"] == {"300": 2}
    assert summary["pageCountTotal"] == 3
    assert summary["shardCountTotal"] == 3
    assert summary["pageShardCount"] == 2
    assert summary["regionShardCount"] == 1
    assert summary["charCountTotal"] == 100
    assert summary["scaffoldAppliedCount"] == 1
    assert summary["scaffoldValidationFailureCount"] == 1
    assert summary["scaffoldJsonCharCountTotal"] == 17
    assert summary["canonicalMarkdownCharCountTotal"] == 0
    assert summary["imageBytesTotal"] == 3072
    assert summary["sourcePixelAreaTotal"] == 2400
    assert summary["latencyMsP50"] == 10.0
    assert summary["latencyMsP95"] == 30.0
    assert summary["latencyMsMax"] == 30.0
    assert summary["requestLatencyMsTotal"] == 40.0
    assert summary["requestWallStartUnixMs"] == 1_000
    assert summary["requestWallEndUnixMs"] == 1_035
    assert summary["requestWallSpanMs"] == 35
    assert summary["requestLatencyOverlapRatio"] == 1.143


def test_openrouter_key_configured_reads_environment(monkeypatch) -> None:
    benchmark = _load_benchmark_module()
    for key in (
        "WENDAO_OPENROUTER_API_KEY",
        "OPENROUTER_API_KEY",
        "WENDAO_HOSTED_VLM_OCR_API_KEY",
        "OPENROUTE_API_KEY",
    ):
        monkeypatch.delenv(key, raising=False)

    assert benchmark._openrouter_key_configured() is False

    monkeypatch.setenv("OPENROUTER_API_KEY", "or-key")

    assert benchmark._openrouter_key_configured() is True

    monkeypatch.delenv("OPENROUTER_API_KEY")
    monkeypatch.setenv("OPENROUTE_API_KEY", "or-legacy-key")

    assert benchmark._openrouter_key_configured() is True


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
