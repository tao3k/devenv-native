"""Audio diagnostic tests."""

from __future__ import annotations

import argparse

from xiuxian_wendao_analyzer.audio_diagnostic_quality_summary import (
    summarize_timeline_structure,
)

from .support import Path, _load_audio_asr_diagnostic


def test_quality_rows_classify_reference_and_proxy_statuses(tmp_path: Path) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    good_transcript = tmp_path / "good.txt"
    good_transcript.write_text(
        "这是一个中文转写,包含智能家居论坛内容。", encoding="utf-8"
    )
    noisy_transcript = tmp_path / "noisy.txt"
    noisy_transcript.write_text("[inaudible] [inaudible] ok", encoding="utf-8")
    results = [
        diagnostic.AsrResult(
            backend="openrouter-chat-audio",
            source="/tmp/forum.MP3",
            chunk="/tmp/chunk0.wav",
            chunk_index=0,
            start_seconds=0.0,
            duration_seconds=30.0,
            model="xiaomi/mimo-v2.5",
            status="ok",
            wall_seconds=3.0,
            transcript_chars=good_transcript.stat().st_size,
            transcript_path=str(good_transcript),
            error="",
        ),
        diagnostic.AsrResult(
            backend="local-docling",
            source="/tmp/forum.MP3",
            chunk="/tmp/chunk1.wav",
            chunk_index=1,
            start_seconds=30.0,
            duration_seconds=30.0,
            model="docling-asr:DOCLING_AUDIO:zh",
            status="ok",
            wall_seconds=3.0,
            transcript_chars=noisy_transcript.stat().st_size,
            transcript_path=str(noisy_transcript),
            error="",
        ),
    ]

    rows = diagnostic.build_quality_rows(
        results,
        references={("forum.MP3", 0): "这是一个中文转写,包含智能家居论坛内容。"},
        max_reference_cer=0.15,
        required_terms=[],
        min_required_term_recall=1.0,
        min_chars_per_minute=20.0,
        min_chinese_ratio=0.35,
        max_inaudible_per_minute=1.0,
        max_repeated_ngram_ratio=0.35,
    )

    assert rows[0].review_status == "reference-pass"
    assert rows[0].reference_cer == 0
    assert rows[1].review_status == "weak-language-ratio"
    summary = diagnostic.summarize_quality(rows)
    assert summary["qualityByBackend"]["openrouter-chat-audio"]["referencePass"] == 1
    assert summary["qualityByBackend"]["local-docling"]["weakRows"] == 1
    subset = diagnostic.summarize_reference_subset(rows)
    assert subset["referenceSubsetConfigured"] is True
    assert subset["referenceSubsetRows"] == 1
    assert subset["referenceSubsetPassRows"] == 1
    assert subset["referenceSubsetFailRows"] == 0
    assert subset["referenceSubsetMaxObservedCer"] == 0


def test_summarize_results_reports_latency_percentiles() -> None:
    diagnostic = _load_audio_asr_diagnostic()
    results = [
        diagnostic.AsrResult(
            backend="local-openai-audio",
            source="/tmp/forum.MP3",
            chunk=f"/tmp/chunk{index}.wav",
            chunk_index=index,
            start_seconds=float(index * 30),
            duration_seconds=30.0,
            model="wendao-local-audio",
            status="ok",
            wall_seconds=wall_seconds,
            transcript_chars=10,
            transcript_path=f"/tmp/chunk{index}.txt",
            error="",
        )
        for index, wall_seconds in enumerate([0.5, 1.0, 3.0])
    ]

    summary = diagnostic.summarize_results(results)
    row = summary["byBackend"]["local-openai-audio"]

    assert row["chunks"] == 3
    assert row["latencyP50Seconds"] == 1.0
    assert row["latencyP95Seconds"] == 3.0


def test_timeline_structure_reports_contiguous_timestamp_coverage() -> None:
    diagnostic = _load_audio_asr_diagnostic()
    rows = [
        diagnostic.QualityRow(
            backend="local-openai-audio",
            source="/tmp/forum.MP3",
            chunk_index=index,
            start_seconds=float(index * 30),
            duration_seconds=30.0,
            status="ok",
            review_status="review-needed",
            model="wendao-local-audio",
            transcript_chars=80,
            chinese_ratio=0.9,
            inaudible_count=0,
            inaudible_per_minute=0.0,
            chars_per_minute=160.0,
            repeated_ngram_ratio=0.0,
            reference_cer=None,
            required_terms_count=0,
            missing_required_terms="",
            required_term_recall=None,
            transcript_path="/tmp/transcript.txt",
            error="",
        )
        for index in range(3)
    ]

    summary = summarize_timeline_structure(rows)
    backend = summary["timelineStructureByBackend"]["local-openai-audio"]

    assert summary["timelineStructurePassed"] is True
    assert backend["rows"] == 3
    assert backend["coverageSeconds"] == 90.0
    assert backend["expectedSpanSeconds"] == 90.0
    assert backend["coverageRatio"] == 1.0
    assert backend["gapSeconds"] == 0.0
    assert backend["overlapSeconds"] == 0.0


def test_timeline_structure_flags_timestamp_gaps() -> None:
    diagnostic = _load_audio_asr_diagnostic()
    rows = [
        diagnostic.QualityRow(
            backend="openrouter-chat-audio",
            source="/tmp/forum.MP3",
            chunk_index=0,
            start_seconds=0.0,
            duration_seconds=30.0,
            status="ok",
            review_status="review-needed",
            model="xiaomi/mimo-v2.5",
            transcript_chars=80,
            chinese_ratio=0.9,
            inaudible_count=0,
            inaudible_per_minute=0.0,
            chars_per_minute=160.0,
            repeated_ngram_ratio=0.0,
            reference_cer=None,
            required_terms_count=0,
            missing_required_terms="",
            required_term_recall=None,
            transcript_path="/tmp/transcript-0.txt",
            error="",
        ),
        diagnostic.QualityRow(
            backend="openrouter-chat-audio",
            source="/tmp/forum.MP3",
            chunk_index=1,
            start_seconds=45.0,
            duration_seconds=30.0,
            status="ok",
            review_status="review-needed",
            model="xiaomi/mimo-v2.5",
            transcript_chars=80,
            chinese_ratio=0.9,
            inaudible_count=0,
            inaudible_per_minute=0.0,
            chars_per_minute=160.0,
            repeated_ngram_ratio=0.0,
            reference_cer=None,
            required_terms_count=0,
            missing_required_terms="",
            required_term_recall=None,
            transcript_path="/tmp/transcript-1.txt",
            error="",
        ),
    ]

    summary = summarize_timeline_structure(rows)
    backend = summary["timelineStructureByBackend"]["openrouter-chat-audio"]

    assert summary["timelineStructurePassed"] is False
    assert backend["gapSeconds"] == 15.0
    assert backend["coverageRatio"] == 60.0 / 75.0


def test_timeline_structure_allows_planned_speech_segment_gaps() -> None:
    diagnostic = _load_audio_asr_diagnostic()
    rows = [
        diagnostic.QualityRow(
            backend="openrouter-chat-audio",
            source="/tmp/forum.MP3",
            chunk_index=0,
            start_seconds=0.0,
            duration_seconds=10.0,
            status="ok",
            review_status="review-needed",
            model="xiaomi/mimo-v2.5",
            transcript_chars=80,
            chinese_ratio=0.9,
            inaudible_count=0,
            inaudible_per_minute=0.0,
            chars_per_minute=160.0,
            repeated_ngram_ratio=0.0,
            reference_cer=None,
            required_terms_count=0,
            missing_required_terms="",
            required_term_recall=None,
            transcript_path="/tmp/transcript-0.txt",
            error="",
        ),
        diagnostic.QualityRow(
            backend="openrouter-chat-audio",
            source="/tmp/forum.MP3",
            chunk_index=1,
            start_seconds=14.0,
            duration_seconds=8.0,
            status="ok",
            review_status="review-needed",
            model="xiaomi/mimo-v2.5",
            transcript_chars=80,
            chinese_ratio=0.9,
            inaudible_count=0,
            inaudible_per_minute=0.0,
            chars_per_minute=160.0,
            repeated_ngram_ratio=0.0,
            reference_cer=None,
            required_terms_count=0,
            missing_required_terms="",
            required_term_recall=None,
            transcript_path="/tmp/transcript-1.txt",
            error="",
        ),
    ]

    summary = summarize_timeline_structure(rows, allow_planned_gaps=True)
    backend = summary["timelineStructureByBackend"]["openrouter-chat-audio"]

    assert summary["timelineGapPolicy"] == "planned-gaps-allowed"
    assert summary["timelineStructurePassed"] is True
    assert backend["gapSeconds"] == 4.0
    assert backend["overlapSeconds"] == 0.0


def test_required_terms_mark_precision_failure(tmp_path: Path) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    transcript = tmp_path / "transcript.txt"
    transcript.write_text("这里讨论智能家居。", encoding="utf-8")
    result = diagnostic.AsrResult(
        backend="openrouter-chat-audio",
        source="/tmp/forum.MP3",
        chunk="/tmp/chunk0.wav",
        chunk_index=0,
        start_seconds=0.0,
        duration_seconds=30.0,
        model="xiaomi/mimo-v2.5",
        status="ok",
        wall_seconds=3.0,
        transcript_chars=transcript.stat().st_size,
        transcript_path=str(transcript),
        error="",
    )

    rows = diagnostic.build_quality_rows(
        [result],
        references={},
        max_reference_cer=0.15,
        required_terms=["智能家居", "长春"],
        min_required_term_recall=1.0,
        min_chars_per_minute=20.0,
        min_chinese_ratio=0.35,
        max_inaudible_per_minute=1.0,
        max_repeated_ngram_ratio=0.35,
    )

    assert rows[0].review_status == "required-term-miss"
    assert rows[0].required_term_recall == 0.5
    assert rows[0].missing_required_terms == "长春"
    summary = diagnostic.summarize_quality(rows)
    assert summary["qualityByBackend"]["openrouter-chat-audio"]["requiredTermMiss"] == 1


def test_repetition_marks_weak_precision_row(tmp_path: Path) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    transcript = tmp_path / "transcript.txt"
    transcript.write_text("我说我说我说我说我说我说我说我说", encoding="utf-8")
    result = diagnostic.AsrResult(
        backend="local-openai-audio",
        source="/tmp/forum.MP3",
        chunk="/tmp/chunk0.wav",
        chunk_index=0,
        start_seconds=0.0,
        duration_seconds=30.0,
        model="wendao-local-audio",
        status="ok",
        wall_seconds=1.0,
        transcript_chars=transcript.stat().st_size,
        transcript_path=str(transcript),
        error="",
    )

    rows = diagnostic.build_quality_rows(
        [result],
        references={},
        max_reference_cer=0.15,
        required_terms=[],
        min_required_term_recall=1.0,
        min_chars_per_minute=20.0,
        min_chinese_ratio=0.35,
        max_inaudible_per_minute=1.0,
        max_repeated_ngram_ratio=0.2,
    )

    assert rows[0].review_status == "weak-repetition-heavy"
    assert rows[0].repeated_ngram_ratio > 0.2


def test_natural_short_utterance_is_review_not_weak(tmp_path: Path) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    transcript = tmp_path / "transcript.txt"
    transcript.write_text("啊来", encoding="utf-8")
    result = diagnostic.AsrResult(
        backend="local-openai-audio",
        source="/tmp/forum.MP3",
        chunk="/tmp/chunk0.wav",
        chunk_index=0,
        start_seconds=302.88,
        duration_seconds=10.82,
        model="wendao-local-audio",
        status="ok",
        wall_seconds=1.0,
        transcript_chars=transcript.stat().st_size,
        transcript_path=str(transcript),
        error="",
    )

    rows = diagnostic.build_quality_rows(
        [result],
        references={},
        max_reference_cer=0.15,
        required_terms=[],
        min_required_term_recall=1.0,
        min_chars_per_minute=40.0,
        min_chinese_ratio=0.35,
        max_inaudible_per_minute=1.0,
        max_repeated_ngram_ratio=0.35,
    )
    summary = diagnostic.summarize_quality(rows)

    assert rows[0].review_status == "short-utterance-review"
    assert summary["qualityByBackend"]["local-openai-audio"]["weakRows"] == 0
    assert summary["qualityByBackend"]["local-openai-audio"]["shortUtteranceRows"] == 1


def test_recheck_quality_summary_reuses_saved_results(tmp_path: Path) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    transcript = tmp_path / "transcript.txt"
    transcript.write_text("啊来", encoding="utf-8")
    summary_path = tmp_path / "summary.json"
    results_path = tmp_path / "results.json"
    diagnostic.write_json(
        summary_path,
        {
            "byBackend": {},
            "sampleStrategy": "speech-segments",
            "precisionGatePassed": False,
            "precisionGateReason": "reference-not-configured",
        },
    )
    diagnostic.write_json(
        results_path,
        [
            diagnostic.AsrResult(
                backend="local-openai-audio",
                source="/tmp/forum.MP3",
                chunk="/tmp/chunk0.wav",
                chunk_index=0,
                start_seconds=302.88,
                duration_seconds=10.82,
                model="wendao-local-audio",
                status="ok",
                wall_seconds=1.0,
                transcript_chars=transcript.stat().st_size,
                transcript_path=str(transcript),
                error="",
            ).__dict__
        ],
    )

    report = diagnostic.recheck_quality_summary(
        argparse.Namespace(
            recheck_quality_summary_json=summary_path,
            recheck_quality_results_json=None,
            reference_jsonl=None,
            required_terms_file=None,
            max_reference_cer=0.15,
            min_required_term_recall=1.0,
            min_chars_per_minute=40.0,
            min_chinese_ratio=0.35,
            max_inaudible_per_minute=30.0,
            max_repeated_ngram_ratio=0.35,
        )
    )
    quality = report["qualityByBackend"]["local-openai-audio"]

    assert report["qualityRecheckSourceSummaryPath"] == str(summary_path)
    assert quality["weakRows"] == 0
    assert quality["shortUtteranceRows"] == 1
    assert report["timelineStructurePassed"] is True


def test_precision_gate_requires_reference_coverage(tmp_path: Path) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    transcript = tmp_path / "transcript.txt"
    transcript.write_text("智能家居论坛", encoding="utf-8")
    result = diagnostic.AsrResult(
        backend="local-openai-audio",
        source="/tmp/forum.MP3",
        chunk="/tmp/chunk0.wav",
        chunk_index=0,
        start_seconds=0.0,
        duration_seconds=30.0,
        model="wendao-local-audio",
        status="ok",
        wall_seconds=1.0,
        transcript_chars=transcript.stat().st_size,
        transcript_path=str(transcript),
        error="",
    )
    rows = diagnostic.build_quality_rows(
        [result],
        references={},
        max_reference_cer=0.15,
        required_terms=[],
        min_required_term_recall=1.0,
        min_chars_per_minute=20.0,
        min_chinese_ratio=0.35,
        max_inaudible_per_minute=1.0,
        max_repeated_ngram_ratio=0.35,
    )

    summary = diagnostic.summarize_precision_gate(
        rows,
        reference_configured=True,
        max_reference_cer=0.15,
        required_terms_configured=False,
    )

    assert summary["precisionGatePassed"] is False
    assert summary["precisionGateReason"] == "reference-coverage-missing"
    assert summary["referenceMissingRows"] == 1


def test_precision_gate_passes_with_reference_and_required_terms(
    tmp_path: Path,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    transcript = tmp_path / "transcript.txt"
    transcript.write_text("智能家居论坛长春", encoding="utf-8")
    result = diagnostic.AsrResult(
        backend="local-openai-audio",
        source="/tmp/forum.MP3",
        chunk="/tmp/chunk0.wav",
        chunk_index=0,
        start_seconds=0.0,
        duration_seconds=30.0,
        model="wendao-local-audio",
        status="ok",
        wall_seconds=1.0,
        transcript_chars=transcript.stat().st_size,
        transcript_path=str(transcript),
        error="",
    )
    rows = diagnostic.build_quality_rows(
        [result],
        references={("forum.MP3", 0): "智能家居论坛长春"},
        max_reference_cer=0.15,
        required_terms=["智能家居", "长春"],
        min_required_term_recall=1.0,
        min_chars_per_minute=20.0,
        min_chinese_ratio=0.35,
        max_inaudible_per_minute=1.0,
        max_repeated_ngram_ratio=0.35,
    )

    summary = diagnostic.summarize_precision_gate(
        rows,
        reference_configured=True,
        max_reference_cer=0.15,
        required_terms_configured=True,
    )

    assert rows[0].review_status == "reference-pass"
    assert summary["precisionGatePassed"] is True
    assert summary["precisionGateReason"] == "passed"
    assert summary["maxObservedReferenceCer"] == 0
