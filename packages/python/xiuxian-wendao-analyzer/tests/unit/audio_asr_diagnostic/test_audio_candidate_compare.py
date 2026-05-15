"""Audio diagnostic tests."""

from __future__ import annotations

import json

from .support import Path, _load_audio_asr_diagnostic


def test_compare_audio_candidate_summaries_prefers_lower_cer(
    tmp_path: Path,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    qwen_summary = tmp_path / "qwen" / "summary.json"
    xiaomi_summary = tmp_path / "xiaomi" / "summary.json"
    diagnostic.write_json(
        qwen_summary,
        _summary_payload(
            backend="local-openai-audio",
            model="wendao-qwen3-asr-audio",
            precision_passed=True,
            max_cer=0.04,
            wall_seconds=90.0,
        ),
    )
    diagnostic.write_json(
        xiaomi_summary,
        _summary_payload(
            backend="openrouter-chat-audio",
            model="xiaomi/mimo-v2.5",
            precision_passed=True,
            max_cer=0.08,
            wall_seconds=30.0,
        ),
    )

    report = diagnostic.compare_audio_candidate_summaries(
        [qwen_summary, xiaomi_summary]
    )

    assert report["eligiblePrecisionCandidateCount"] == 2
    assert report["eligibleTimelineCandidateCount"] == 2
    assert report["eligibleQualityCandidateCount"] == 2
    assert report["eligiblePromotionCandidateCount"] == 2
    assert report["promotionCandidate"] == ("local-openai-audio:wendao-qwen3-asr-audio")
    assert report["rankedCandidates"] == [
        "local-openai-audio:wendao-qwen3-asr-audio",
        "openrouter-chat-audio:xiaomi/mimo-v2.5",
    ]


def test_compare_audio_candidate_summaries_reports_no_precision_candidate(
    tmp_path: Path,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    summary_path = tmp_path / "summary.json"
    diagnostic.write_json(
        summary_path,
        _summary_payload(
            backend="openrouter-chat-audio",
            model="xiaomi/mimo-v2.5",
            precision_passed=False,
            precision_reason="reference-not-configured",
            max_cer=None,
            wall_seconds=20.0,
        ),
    )

    report = diagnostic.compare_audio_candidate_summaries([summary_path])

    assert report["eligiblePrecisionCandidateCount"] == 0
    assert report["eligiblePromotionCandidateCount"] == 0
    assert report["promotionCandidate"] == ""
    assert report["promotionReason"] == "no-precision-timeline-quality-candidate"


def test_compare_audio_candidate_summaries_prefers_diagnostic_wall_time(
    tmp_path: Path,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    qwen_summary = tmp_path / "qwen" / "summary.json"
    xiaomi_summary = tmp_path / "xiaomi" / "summary.json"
    diagnostic.write_json(
        qwen_summary,
        _summary_payload(
            backend="local-openai-audio",
            model="wendao-qwen3-asr-audio",
            precision_passed=True,
            max_cer=0.04,
            wall_seconds=200.0,
            diagnostic_wall_seconds=25.0,
        ),
    )
    diagnostic.write_json(
        xiaomi_summary,
        _summary_payload(
            backend="openrouter-chat-audio",
            model="xiaomi/mimo-v2.5",
            precision_passed=True,
            max_cer=0.04,
            wall_seconds=20.0,
            diagnostic_wall_seconds=40.0,
        ),
    )

    report = diagnostic.compare_audio_candidate_summaries(
        [qwen_summary, xiaomi_summary]
    )
    candidate = report["candidates"][0]

    assert report["promotionCandidate"] == "local-openai-audio:wendao-qwen3-asr-audio"
    assert candidate["wallSeconds"] == 25.0
    assert candidate["requestWallSeconds"] == 200.0


def test_compare_audio_candidate_summaries_rejects_timeline_gaps(
    tmp_path: Path,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    summary_path = tmp_path / "summary.json"
    diagnostic.write_json(
        summary_path,
        _summary_payload(
            backend="openrouter-chat-audio",
            model="xiaomi/mimo-v2.5",
            precision_passed=True,
            max_cer=0.03,
            wall_seconds=20.0,
            timeline_passed=False,
            timeline_gap_seconds=15.0,
        ),
    )

    report = diagnostic.compare_audio_candidate_summaries([summary_path])
    candidate = report["candidates"][0]

    assert report["eligiblePrecisionCandidateCount"] == 1
    assert report["eligibleTimelineCandidateCount"] == 0
    assert report["eligibleQualityCandidateCount"] == 1
    assert report["eligiblePromotionCandidateCount"] == 0
    assert report["promotionCandidate"] == ""
    assert candidate["timelineStructurePassed"] is False
    assert candidate["timelineGapSeconds"] == 15.0


def test_compare_audio_candidate_summaries_rejects_repetition_proxy(
    tmp_path: Path,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    summary_path = tmp_path / "summary.json"
    diagnostic.write_json(
        summary_path,
        _summary_payload(
            backend="openrouter-chat-audio",
            model="xiaomi/mimo-v2.5",
            precision_passed=True,
            max_cer=0.03,
            wall_seconds=20.0,
            weak_rows=3,
            avg_repeated_ngram_ratio=0.56,
        ),
    )

    report = diagnostic.compare_audio_candidate_summaries([summary_path])
    candidate = report["candidates"][0]

    assert report["eligiblePrecisionCandidateCount"] == 1
    assert report["eligibleTimelineCandidateCount"] == 1
    assert report["eligibleQualityCandidateCount"] == 0
    assert report["eligiblePromotionCandidateCount"] == 0
    assert candidate["qualityProxyPassed"] is False
    assert candidate["qualityProxyReason"] == "weak-quality-rows"
    assert candidate["weakRows"] == 3


def test_compare_audio_candidate_summaries_reports_short_utterance_rows(
    tmp_path: Path,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    summary_path = tmp_path / "summary.json"
    diagnostic.write_json(
        summary_path,
        _summary_payload(
            backend="local-openai-audio",
            model="wendao-qwen3-asr-audio",
            precision_passed=True,
            max_cer=0.03,
            wall_seconds=20.0,
            short_utterance_rows=1,
        ),
    )

    report = diagnostic.compare_audio_candidate_summaries([summary_path])
    candidate = report["candidates"][0]

    assert report["eligibleQualityCandidateCount"] == 1
    assert candidate["qualityProxyPassed"] is True
    assert candidate["shortUtteranceRows"] == 1
    assert candidate["weakRows"] == 0


def test_compare_audio_candidate_summaries_cli_mode_writes_report(
    tmp_path: Path,
    capsys,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    summary_path = tmp_path / "summary.json"
    report_path = tmp_path / "comparison.json"
    diagnostic.write_json(
        summary_path,
        _summary_payload(
            backend="openrouter-chat-audio",
            model="xiaomi/mimo-v2.5",
            precision_passed=True,
            max_cer=0.03,
            wall_seconds=18.0,
        ),
    )

    exit_code = diagnostic.main(
        [
            "--compare-summary-json",
            str(summary_path),
            "--comparison-report-json",
            str(report_path),
        ]
    )

    assert exit_code == 0
    stdout_report = json.loads(capsys.readouterr().out)
    file_report = json.loads(report_path.read_text(encoding="utf-8"))
    assert (
        stdout_report["promotionCandidate"] == "openrouter-chat-audio:xiaomi/mimo-v2.5"
    )
    assert stdout_report["timelineStructureRequired"] is True
    assert stdout_report["qualityProxyRequired"] is True
    assert file_report == stdout_report


def _summary_payload(
    *,
    backend: str,
    model: str,
    precision_passed: bool,
    max_cer: float | None,
    wall_seconds: float,
    precision_reason: str = "passed",
    timeline_passed: bool = True,
    timeline_gap_seconds: float = 0.0,
    weak_rows: int = 0,
    short_utterance_rows: int = 0,
    avg_repeated_ngram_ratio: float = 0.01,
    avg_inaudible_per_minute: float = 0.0,
    diagnostic_wall_seconds: float | None = None,
) -> dict[str, object]:
    payload: dict[str, object] = {
        "byBackend": {
            backend: {
                "chunks": 5,
                "errors": 0,
                "wallSeconds": wall_seconds,
                "transcriptChars": 1000,
                "latencyP50Seconds": wall_seconds / 5,
                "latencyP95Seconds": wall_seconds / 4,
            }
        },
        "qualityByBackend": {
            backend: {
                "avgChineseRatio": 0.9,
                "avgInaudiblePerMinute": avg_inaudible_per_minute,
                "avgRepeatedNgramRatio": avg_repeated_ngram_ratio,
                "shortUtteranceRows": short_utterance_rows,
                "weakRows": weak_rows,
            }
        },
        "timelineStructurePassed": timeline_passed,
        "timelineStructureByBackend": {
            backend: {
                "passed": timeline_passed,
                "coverageRatio": 1.0 if timeline_passed else 0.8,
                "gapSeconds": timeline_gap_seconds,
                "overlapSeconds": 0.0,
            }
        },
        "hostedAudioModel": model,
        "precisionGatePassed": precision_passed,
        "precisionGateReason": precision_reason,
        "maxObservedReferenceCer": max_cer,
        "referenceCoverageRows": 5,
        "referenceFailRows": 0,
        "failedRows": 0,
        "requiredTermMissRows": 0,
    }
    if diagnostic_wall_seconds is not None:
        payload["diagnosticWallSeconds"] = diagnostic_wall_seconds
    return payload
