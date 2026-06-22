"""Audio diagnostic tests."""

from __future__ import annotations

import json

from .support import Path, _load_audio_asr_diagnostic


def test_precision_gate_rejects_candidate_draft_reference(
    tmp_path: Path,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    transcript = tmp_path / "transcript.txt"
    transcript.write_text("通用术语测试地点", encoding="utf-8")
    result = diagnostic.AsrResult(
        backend="local-openai-audio",
        source="/tmp/forum.MP3",
        chunk="/tmp/chunk0.wav",
        chunk_index=0,
        start_seconds=0.0,
        duration_seconds=30.0,
        model="qwen3-asr-1.7b-mlx",
        status="ok",
        wall_seconds=1.0,
        transcript_chars=transcript.stat().st_size,
        transcript_path=str(transcript),
        error="",
    )
    rows = diagnostic.build_quality_rows(
        [result],
        references={("forum.MP3", 0): "通用术语测试地点"},
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

    prompt = diagnostic.prompt_with_domain_terms("transcribe", ["通用术语", "Matter"])

    assert "Domain vocabulary" in prompt
    assert "通用术语、Matter" in prompt


def test_prompt_with_primary_language_adds_model_neutral_hint() -> None:
    diagnostic = _load_audio_asr_diagnostic()

    prompt = diagnostic.prompt_with_primary_language("transcribe", "zh-CN")

    assert "PRIMARY_LANGUAGE=zh-cn" in prompt
    assert "Infer the actual spoken language from the audio" in prompt
    assert diagnostic.prompt_with_primary_language("transcribe", "unknown") == ("transcribe")


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
        model="qwen3-asr-1.7b-mlx",
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
    assert timeline_row["model"] == "qwen3-asr-1.7b-mlx"
    org_content = (tmp_path / "timeline.org").read_text(encoding="utf-8")
    assert "* 00:01:30.000 -- 00:02:00.000 forum.MP3 chunk 0003" in org_content
    assert ":MODEL: qwen3-asr-1.7b-mlx" in org_content
    assert ":INVOCATION_BACKEND: local-openai-audio" in org_content
    assert ":BACKEND: local-openai-audio" not in org_content
    assert ":CHUNK_INDEX: 3" in org_content
    assert "时间线文本" in org_content


def test_transcript_timeline_collapses_character_timestamp_segments(
    tmp_path: Path,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    transcript = tmp_path / "transcript.txt"
    transcript.write_text("中文时间轴测试文本", encoding="utf-8")
    segments = tmp_path / "segments.jsonl"
    segments.write_text(
        "\n".join(
            json.dumps(
                {
                    "startSeconds": index * 0.25,
                    "endSeconds": (index + 1) * 0.25,
                    "text": text,
                },
                ensure_ascii=False,
            )
            for index, text in enumerate(["中", "文", "时", "间", "轴", "测"])
        ),
        encoding="utf-8",
    )
    row = diagnostic.QualityRow(
        backend="local-openai-audio",
        source="/tmp/forum.MP3",
        chunk_index=4,
        start_seconds=120.0,
        duration_seconds=30.0,
        status="ok",
        review_status="review-needed",
        model="qwen3-asr-1.7b-mlx",
        transcript_chars=10,
        chinese_ratio=1.0,
        inaudible_count=0,
        inaudible_per_minute=0.0,
        chars_per_minute=20.0,
        repeated_ngram_ratio=0.0,
        reference_cer=None,
        required_terms_count=0,
        missing_required_terms="",
        required_term_recall=None,
        transcript_path=str(transcript),
        error="",
        segments_path=str(segments),
        segment_count=6,
    )

    timeline_rows = diagnostic.timeline_review_rows([row])

    assert timeline_rows == [
        {
            "backend": "local-openai-audio",
            "model": "qwen3-asr-1.7b-mlx",
            "source": "forum.MP3",
            "chunkIndex": 4,
            "startSeconds": 0.0,
            "endSeconds": 1.5,
            "status": "ok",
            "reviewStatus": "review-needed",
            "text": "中文时间轴测试文本",
        }
    ]


def test_transcript_timeline_coalesces_character_level_qwen_segments(
    tmp_path: Path,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    transcript = tmp_path / "transcript.txt"
    transcript.write_text("中文语音时间轴合并测试文本", encoding="utf-8")
    segments = tmp_path / "segments.jsonl"
    segment_rows = [
        {"startSeconds": 1.0, "endSeconds": 1.2, "text": "中"},
        {"startSeconds": 1.2, "endSeconds": 1.4, "text": "文"},
        {"startSeconds": 1.4, "endSeconds": 1.6, "text": "语"},
        {"startSeconds": 1.6, "endSeconds": 1.8, "text": "音"},
        {"startSeconds": 1.8, "endSeconds": 2.0, "text": "时间"},
    ]
    segments.write_text(
        "\n".join(json.dumps(row, ensure_ascii=False) for row in segment_rows),
        encoding="utf-8",
    )
    row = diagnostic.QualityRow(
        backend="local-openai-audio",
        source="/tmp/forum.MP3",
        chunk_index=0,
        start_seconds=0.0,
        duration_seconds=30.0,
        status="ok",
        review_status="review-needed",
        model="qwen3-asr-1.7b-mlx",
        transcript_chars=18,
        chinese_ratio=1.0,
        inaudible_count=0,
        inaudible_per_minute=0.0,
        chars_per_minute=36.0,
        repeated_ngram_ratio=0.0,
        reference_cer=None,
        required_terms_count=0,
        missing_required_terms="",
        required_term_recall=None,
        transcript_path=str(transcript),
        error="",
        segments_path=str(segments),
        segment_count=len(segment_rows),
    )

    timeline_rows = diagnostic.timeline_review_rows([row])

    assert timeline_rows == [
        {
            "backend": "local-openai-audio",
            "model": "qwen3-asr-1.7b-mlx",
            "source": "forum.MP3",
            "chunkIndex": 0,
            "startSeconds": 1.0,
            "endSeconds": 2.0,
            "status": "ok",
            "reviewStatus": "review-needed",
            "text": "中文语音时间轴合并测试文本",
        }
    ]


def test_transcript_timeline_keeps_short_segments_without_text_alignment(
    tmp_path: Path,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    transcript = tmp_path / "transcript.txt"
    transcript.write_text("测试文本不应合并", encoding="utf-8")
    segments = tmp_path / "segments.jsonl"
    segment_rows = [
        {"startSeconds": 10.0, "endSeconds": 10.4, "text": "嗯"},
        {"startSeconds": 10.4, "endSeconds": 10.8, "text": "好"},
        {"startSeconds": 10.8, "endSeconds": 11.2, "text": "对"},
        {"startSeconds": 11.2, "endSeconds": 11.6, "text": "是"},
        {"startSeconds": 11.6, "endSeconds": 12.0, "text": "的"},
    ]
    segments.write_text(
        "\n".join(json.dumps(row, ensure_ascii=False) for row in segment_rows),
        encoding="utf-8",
    )
    row = diagnostic.QualityRow(
        backend="local-openai-audio",
        source="/tmp/forum.MP3",
        chunk_index=5,
        start_seconds=150.0,
        duration_seconds=30.0,
        status="ok",
        review_status="review-needed",
        model="qwen3-asr-1.7b-mlx",
        transcript_chars=9,
        chinese_ratio=1.0,
        inaudible_count=0,
        inaudible_per_minute=0.0,
        chars_per_minute=18.0,
        repeated_ngram_ratio=0.0,
        reference_cer=None,
        required_terms_count=0,
        missing_required_terms="",
        required_term_recall=None,
        transcript_path=str(transcript),
        error="",
        segments_path=str(segments),
        segment_count=len(segment_rows),
    )

    timeline_rows = diagnostic.timeline_review_rows([row])

    assert [row["text"] for row in timeline_rows] == ["嗯", "好", "对", "是", "的"]
