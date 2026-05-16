"""Audio diagnostic tests."""

from __future__ import annotations

import json

from .support import Path, _load_audio_asr_diagnostic


def test_precision_gate_rejects_candidate_draft_reference(
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
        reference_candidate_draft_rows=1,
        max_reference_cer=0.15,
        required_terms_configured=False,
    )

    assert rows[0].review_status == "reference-pass"
    assert summary["precisionGatePassed"] is False
    assert summary["precisionGateReason"] == "reference-candidate-draft"
    assert summary["referenceCandidateDraftRows"] == 1


def test_prompt_with_domain_terms_appends_glossary() -> None:
    diagnostic = _load_audio_asr_diagnostic()

    prompt = diagnostic.prompt_with_domain_terms("transcribe", ["智能家居", "Matter"])

    assert "Domain vocabulary" in prompt
    assert "智能家居、Matter" in prompt


def test_prompt_with_primary_language_adds_model_neutral_hint() -> None:
    diagnostic = _load_audio_asr_diagnostic()

    prompt = diagnostic.prompt_with_primary_language("transcribe", "zh-CN")

    assert "PRIMARY_LANGUAGE=zh-cn" in prompt
    assert "Infer the actual spoken language from the audio" in prompt
    assert diagnostic.prompt_with_primary_language("transcribe", "unknown") == (
        "transcribe"
    )


def test_quality_review_tsv_contains_chunk_and_status(tmp_path: Path) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    row = diagnostic.QualityRow(
        backend="openrouter-chat-audio",
        source="/tmp/forum.MP3",
        chunk_index=2,
        start_seconds=60.0,
        duration_seconds=30.0,
        status="ok",
        review_status="review-needed",
        model="xiaomi/mimo-v2.5",
        transcript_chars=80,
        chinese_ratio=0.8,
        inaudible_count=1,
        inaudible_per_minute=2.0,
        chars_per_minute=160.0,
        repeated_ngram_ratio=0.0,
        reference_cer=None,
        required_terms_count=0,
        missing_required_terms="",
        required_term_recall=None,
        transcript_path="/tmp/transcript.txt",
        error="",
    )

    diagnostic.write_quality_tsv(tmp_path / "review.tsv", [row])

    content = (tmp_path / "review.tsv").read_text(encoding="utf-8")
    assert "reviewStatus" in content
    assert "review-needed" in content
    assert "\t2\t60.000\t" in content


def test_transcript_review_tsv_contains_text_for_private_evidence(
    tmp_path: Path,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    transcript = tmp_path / "transcript.txt"
    transcript.write_text("第一行\n第二行\t术语", encoding="utf-8")
    row = diagnostic.QualityRow(
        backend="local-openai-audio",
        source="/tmp/forum.MP3",
        chunk_index=1,
        start_seconds=30.0,
        duration_seconds=30.0,
        status="ok",
        review_status="review-needed",
        model="wendao-local-audio",
        transcript_chars=8,
        chinese_ratio=1.0,
        inaudible_count=0,
        inaudible_per_minute=0.0,
        chars_per_minute=16.0,
        repeated_ngram_ratio=0.0,
        reference_cer=None,
        required_terms_count=0,
        missing_required_terms="",
        required_term_recall=None,
        transcript_path=str(transcript),
        error="",
    )

    diagnostic.write_transcript_review_tsv(tmp_path / "transcript_review.tsv", [row])

    content = (tmp_path / "transcript_review.tsv").read_text(encoding="utf-8")
    assert "text" in content
    assert "forum.MP3" in content
    assert "第一行\\n第二行 术语" in content


def test_transcript_timeline_outputs_timestamped_formats(tmp_path: Path) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    transcript = tmp_path / "transcript.txt"
    transcript.write_text("时间线文本", encoding="utf-8")
    row = diagnostic.QualityRow(
        backend="local-openai-audio",
        source="/tmp/forum.MP3",
        chunk_index=3,
        start_seconds=90.0,
        duration_seconds=30.0,
        status="ok",
        review_status="review-needed",
        model="wendao-local-audio",
        transcript_chars=5,
        chinese_ratio=1.0,
        inaudible_count=0,
        inaudible_per_minute=0.0,
        chars_per_minute=10.0,
        repeated_ngram_ratio=0.0,
        reference_cer=None,
        required_terms_count=0,
        missing_required_terms="",
        required_term_recall=None,
        transcript_path=str(transcript),
        error="",
    )

    diagnostic.write_transcript_timeline_vtt(tmp_path / "timeline.vtt", [row])
    diagnostic.write_transcript_timeline_srt(tmp_path / "timeline.srt", [row])
    diagnostic.write_transcript_timeline_jsonl(tmp_path / "timeline.jsonl", [row])
    diagnostic.write_transcript_timeline_org(tmp_path / "timeline.org", [row])

    assert "00:01:30.000 --> 00:02:00.000" in (tmp_path / "timeline.vtt").read_text(
        encoding="utf-8"
    )
    assert "00:01:30,000 --> 00:02:00,000" in (tmp_path / "timeline.srt").read_text(
        encoding="utf-8"
    )
    timeline_row = json.loads((tmp_path / "timeline.jsonl").read_text(encoding="utf-8"))
    assert timeline_row["startSeconds"] == 90.0
    assert timeline_row["endSeconds"] == 120.0
    assert timeline_row["backend"] == "local-openai-audio"
    assert timeline_row["model"] == "wendao-local-audio"
    org_content = (tmp_path / "timeline.org").read_text(encoding="utf-8")
    assert "* 00:01:30.000 -- 00:02:00.000 forum.MP3 chunk 0003" in org_content
    assert ":MODEL: wendao-local-audio" in org_content
    assert ":INVOCATION_BACKEND: local-openai-audio" in org_content
    assert ":BACKEND: local-openai-audio" not in org_content
    assert ":CHUNK_INDEX: 3" in org_content
    assert "时间线文本" in org_content
