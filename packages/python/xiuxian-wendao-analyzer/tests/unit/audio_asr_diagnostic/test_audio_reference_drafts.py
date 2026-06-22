"""Audio diagnostic tests."""

from __future__ import annotations

import hashlib

import pytest

from .support import Path, _load_audio_asr_diagnostic


def _materialize_review_table(
    diagnostic,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    rows: list[dict[str, object]],
) -> Path:
    source_path = tmp_path / "forum.mp3"
    source_path.write_bytes(b"audio")
    selection_path = tmp_path / "selected.jsonl"
    clip_dir = tmp_path / "clips"

    def fake_run(command, **_kwargs):
        Path(command[-1]).write_bytes(b"clip")
        return type("Result", (), {"returncode": 0, "stderr": ""})()

    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.audio_diagnostic_reference_pack.subprocess.run",
        fake_run,
    )
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.audio_diagnostic_reference_pack.audio_duration_seconds",
        lambda _path: float(rows[0].get("durationSeconds", 8.0)) if rows else 8.0,
    )
    diagnostic.write_jsonl(
        selection_path,
        [
            {
                "source": row.get("source", source_path.name),
                "sourceId": str(source_path),
                "chunkIndex": row.get("chunkIndex", 2),
                "startSeconds": row.get("startSeconds", 12.5),
                "durationSeconds": row.get("durationSeconds", 8.0),
                "reviewStatus": row.get("reviewStatus", "review-needed"),
                "selectionReason": row.get("selectionReason", "timeline-spread"),
                "referenceStatus": row.get(
                    "referenceStatus",
                    diagnostic.REFERENCE_STATUS_CANDIDATE_DRAFT,
                ),
                "text": row.get("text", "draft text"),
            }
            for row in rows
        ],
    )
    pack = diagnostic.materialize_reference_selection_pack(
        selection_jsonl=selection_path,
        clip_dir=clip_dir,
        ffmpeg_path="ffmpeg",
    )
    return Path(pack["reviewTable"])


def test_truth_template_rows_do_not_include_transcript_text(tmp_path: Path) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    source = tmp_path / "forum.MP3"
    chunk = diagnostic.AudioChunk(
        source=source,
        path=tmp_path / "chunk.wav",
        index=2,
        start_seconds=60.0,
        duration_seconds=30.0,
        format="wav",
        shard_id="shard-2",
        cache_key="audio-shards-v1:shard-2",
        source_sha256="b" * 64,
        sample_rate_hz=16000,
        channels=1,
        media_start_seconds=60.0,
        media_duration_seconds=30.0,
    )

    rows = diagnostic.truth_template_rows([chunk])

    assert rows == [
        {
            "source": "forum.MP3",
            "sourceId": str(source),
            "sourceSha256": "b" * 64,
            "chunkIndex": 2,
            "shardId": "shard-2",
            "cacheKey": "audio-shards-v1:shard-2",
            "startSeconds": 60.0,
            "durationSeconds": 30.0,
            "mediaStartSeconds": 60.0,
            "mediaDurationSeconds": 30.0,
            "audioFormat": "wav",
            "text": "",
        }
    ]


def test_reference_draft_rows_prefill_transcript_text(tmp_path: Path) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    transcript = tmp_path / "chunk.txt"
    transcript.write_text("candidate transcript text", encoding="utf-8")
    row = diagnostic.QualityRow(
        backend="local-openai-audio",
        source="/private/forum.MP3",
        chunk_index=3,
        start_seconds=90.0,
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
    )

    assert diagnostic.reference_draft_rows([row]) == [
        {
            "source": "forum.MP3",
            "sourceId": "/private/forum.MP3",
            "chunkIndex": 3,
            "startSeconds": 90.0,
            "durationSeconds": 30.0,
            "backend": "local-openai-audio",
            "model": "qwen3-asr-1.7b-mlx",
            "reviewStatus": "review-needed",
            "referenceStatus": "candidate-draft",
            "text": "candidate transcript text",
        }
    ]


def test_reference_draft_jsonl_can_feed_reference_loader(tmp_path: Path) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    transcript = tmp_path / "chunk.txt"
    transcript.write_text("corrected reference transcript", encoding="utf-8")
    row = diagnostic.QualityRow(
        backend="local-openai-audio",
        source="/private/forum.MP3",
        chunk_index=4,
        start_seconds=120.0,
        duration_seconds=30.0,
        status="ok",
        review_status="review-needed",
        model="qwen3-asr-1.7b-mlx",
        transcript_chars=28,
        chinese_ratio=1.0,
        inaudible_count=0,
        inaudible_per_minute=0.0,
        chars_per_minute=56.0,
        repeated_ngram_ratio=0.0,
        reference_cer=None,
        required_terms_count=0,
        missing_required_terms="",
        required_term_recall=None,
        transcript_path=str(transcript),
        error="",
    )
    reference_path = tmp_path / "reference_draft.jsonl"

    diagnostic.write_reference_draft_jsonl(reference_path, [row])

    assert diagnostic.load_reference_transcripts(reference_path) == {
        ("forum.MP3", 4): "corrected reference transcript"
    }
    assert diagnostic.reference_candidate_draft_row_count(reference_path) == 1


def test_curated_reference_metadata_is_safe_for_promotion(tmp_path: Path) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    reference_path = tmp_path / "reference.jsonl"
    diagnostic.write_jsonl(
        reference_path,
        [
            {
                "source": "forum.MP3",
                "chunkIndex": 4,
                "referenceStatus": "curated",
                "backend": "local-openai-audio",
                "model": "qwen3-asr-1.7b-mlx",
                "text": "corrected reference transcript",
            }
        ],
    )

    assert diagnostic.load_reference_transcripts(reference_path) == {
        ("forum.MP3", 4): "corrected reference transcript"
    }
    assert diagnostic.reference_candidate_draft_row_count(reference_path) == 0


def test_curated_reference_rows_from_org_strip_diagnostic_metadata(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    review_table = _materialize_review_table(
        diagnostic,
        tmp_path,
        monkeypatch,
        [
            {
                "source": "forum.MP3",
                "chunkIndex": 2,
                "startSeconds": 60.0,
                "durationSeconds": 30.0,
                "reviewStatus": "review-needed",
                "referenceStatus": "candidate-draft",
                "text": "model draft text",
            }
        ],
    )
    output_org = tmp_path / "reference_selection_review.org"
    diagnostic.write_reference_selection_review_org(
        review_table=review_table,
        output_org=output_org,
    )
    org_text = (
        output_org.read_text(encoding="utf-8")
        .replace("** TODO Row 01", "** DONE Row 01")
        .replace(":REFERENCE_STATUS: candidate-draft", ":REFERENCE_STATUS: curated")
        .replace(
            "#+begin_src text :name reference_text\n#+end_src",
            "#+begin_src text :name reference_text\n corrected reference transcript \n#+end_src",
        )
    )
    output_org.write_text(org_text, encoding="utf-8")

    assert diagnostic.curated_reference_rows_from_org(output_org) == [
        {
            "source": "forum.MP3",
            "sourceId": str(tmp_path / "forum.mp3"),
            "chunkIndex": 2,
            "startSeconds": 60.0,
            "durationSeconds": 30.0,
            "referenceStatus": "curated",
            "text": "corrected reference transcript",
        }
    ]


def test_curated_reference_rows_from_org_rejects_candidate_draft(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    review_table = _materialize_review_table(diagnostic, tmp_path, monkeypatch, [{}])
    output_org = tmp_path / "reference_selection_review.org"
    diagnostic.write_reference_selection_review_org(
        review_table=review_table,
        output_org=output_org,
    )
    output_org.write_text(
        output_org.read_text(encoding="utf-8")
        .replace("** TODO Row 01", "** DONE Row 01")
        .replace(
            "#+begin_src text :name reference_text\n#+end_src",
            "#+begin_src text :name reference_text\nhuman text\n#+end_src",
        ),
        encoding="utf-8",
    )

    with pytest.raises(ValueError, match="not curated"):
        diagnostic.curated_reference_rows_from_org(output_org)


def test_curated_reference_rows_from_org_rejects_empty_text(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    review_table = _materialize_review_table(diagnostic, tmp_path, monkeypatch, [{}])
    output_org = tmp_path / "reference_selection_review.org"
    diagnostic.write_reference_selection_review_org(
        review_table=review_table,
        output_org=output_org,
    )
    output_org.write_text(
        output_org.read_text(encoding="utf-8")
        .replace("** TODO Row 01", "** DONE Row 01")
        .replace(":REFERENCE_STATUS: candidate-draft", ":REFERENCE_STATUS: curated"),
        encoding="utf-8",
    )

    with pytest.raises(ValueError, match="empty reference text"):
        diagnostic.curated_reference_rows_from_org(output_org)


def test_curated_reference_rows_from_org_rejects_unfinished_row(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    review_table = _materialize_review_table(
        diagnostic,
        tmp_path,
        monkeypatch,
        [{}],
    )
    output_org = tmp_path / "reference_selection_review.org"
    diagnostic.write_reference_selection_review_org(
        review_table=review_table,
        output_org=output_org,
    )

    with pytest.raises(ValueError, match="not marked DONE"):
        diagnostic.curated_reference_rows_from_org(output_org)


def test_select_reference_rows_prioritizes_risky_and_spreads_timeline() -> None:
    diagnostic = _load_audio_asr_diagnostic()
    rows = [
        {
            "chunkIndex": index,
            "startSeconds": float(index * 10),
            "durationSeconds": 10.0,
            "reviewStatus": "review-needed",
            "referenceStatus": diagnostic.REFERENCE_STATUS_CANDIDATE_DRAFT,
            "text": f"row {index}",
        }
        for index in range(8)
    ]
    rows[3]["reviewStatus"] = "short-utterance-review"
    rows[6]["reviewStatus"] = "weak-inaudible-heavy"

    selected = diagnostic.select_reference_rows(rows, limit=4)

    assert [row["chunkIndex"] for row in selected] == [0, 3, 6, 7]
    assert selected[1]["selectionReason"] == "short-utterance"
    assert selected[2]["selectionReason"] == "weak-quality"


def test_select_reference_rows_fills_limit_after_priority_overlap() -> None:
    diagnostic = _load_audio_asr_diagnostic()
    rows = [
        {
            "chunkIndex": index,
            "startSeconds": float(index * 30),
            "durationSeconds": 30.0,
            "reviewStatus": (
                "short-utterance-review" if index in {2, 12, 13, 14, 15} else "review-needed"
            ),
            "referenceStatus": diagnostic.REFERENCE_STATUS_CANDIDATE_DRAFT,
            "text": f"row {index}",
        }
        for index in range(20)
    ]

    selected = diagnostic.select_reference_rows(rows, limit=20)

    assert len(selected) == 20
    assert [row["chunkIndex"] for row in selected] == list(range(20))
    assert selected[2]["selectionReason"] == "short-utterance"
    assert selected[0]["selectionReason"] == "timeline-spread"


def test_select_reference_rows_never_exceeds_limit_for_many_priority_rows() -> None:
    diagnostic = _load_audio_asr_diagnostic()
    rows = [
        {
            "chunkIndex": index,
            "startSeconds": float(index * 30),
            "durationSeconds": 30.0,
            "reviewStatus": "short-utterance-review",
            "referenceStatus": diagnostic.REFERENCE_STATUS_CANDIDATE_DRAFT,
            "text": f"row {index}",
        }
        for index in range(8)
    ]

    selected = diagnostic.select_reference_rows(rows, limit=3)

    assert [row["chunkIndex"] for row in selected] == [0, 1, 2]
    assert all(row["selectionReason"] == "short-utterance" for row in selected)


def test_select_reference_draft_report_writes_jsonl(tmp_path: Path) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    draft_path = tmp_path / "reference_draft.jsonl"
    selected_jsonl = tmp_path / "selected.jsonl"
    diagnostic.write_jsonl(
        draft_path,
        [
            {
                "source": "forum.mp3",
                "chunkIndex": 0,
                "startSeconds": 0.0,
                "durationSeconds": 10.0,
                "reviewStatus": "short-utterance-review",
                "referenceStatus": diagnostic.REFERENCE_STATUS_CANDIDATE_DRAFT,
                "text": "啊来",
            }
        ],
    )

    report = diagnostic.select_reference_draft_report(
        draft_jsonl=draft_path,
        limit=4,
        selected_jsonl=selected_jsonl,
    )

    assert report["schema"] == "xiuxian_wendao.audio_reference_selection.v1"
    assert report["selectedRows"] == 1
    assert selected_jsonl.exists()
    assert report["selectedJsonl"] == str(selected_jsonl)


def test_select_reference_draft_report_can_use_rechecked_quality(
    tmp_path: Path,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    draft_path = tmp_path / "reference_draft.jsonl"
    quality_path = tmp_path / "quality_recheck.json"
    diagnostic.write_jsonl(
        draft_path,
        [
            {
                "source": "forum.mp3",
                "chunkIndex": 0,
                "startSeconds": 0.0,
                "durationSeconds": 10.0,
                "reviewStatus": "weak-too-short",
                "referenceStatus": diagnostic.REFERENCE_STATUS_CANDIDATE_DRAFT,
                "text": "啊来",
            }
        ],
    )
    diagnostic.write_json(
        quality_path,
        {
            "qualityRows": [
                {
                    "source": "/tmp/forum.mp3",
                    "chunk_index": 0,
                    "review_status": "short-utterance-review",
                }
            ]
        },
    )

    report = diagnostic.select_reference_draft_report(
        draft_jsonl=draft_path,
        limit=4,
        quality_json=quality_path,
    )

    assert report["qualityJson"] == str(quality_path)
    assert report["selected"][0]["reviewStatus"] == "short-utterance-review"
    assert "short-utterance" in report["selected"][0]["selectionReason"].split("|")


def test_materialize_reference_selection_pack_writes_review_clips(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    source_path = tmp_path / "forum.mp3"
    source_path.write_bytes(b"audio")
    selection_path = tmp_path / "selected.jsonl"
    clip_dir = tmp_path / "clips"
    commands: list[list[str]] = []

    def fake_run(command, **_kwargs):
        commands.append(command)
        Path(command[-1]).write_bytes(b"clip")
        return type("Result", (), {"returncode": 0, "stderr": ""})()

    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.audio_diagnostic_reference_pack.subprocess.run",
        fake_run,
    )
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.audio_diagnostic_reference_pack.audio_duration_seconds",
        lambda _path: 8.0,
    )
    diagnostic.write_jsonl(
        selection_path,
        [
            {
                "source": "forum.mp3",
                "sourceId": str(source_path),
                "chunkIndex": 2,
                "startSeconds": 12.5,
                "durationSeconds": 8.0,
                "reviewStatus": "review-needed",
                "selectionReason": "timeline-spread",
                "referenceStatus": diagnostic.REFERENCE_STATUS_CANDIDATE_DRAFT,
                "text": "draft text",
            }
        ],
    )

    report = diagnostic.materialize_reference_selection_pack(
        selection_jsonl=selection_path,
        clip_dir=clip_dir,
        ffmpeg_path="ffmpeg",
    )

    assert report["schema"] == "xiuxian_wendao.audio_reference_selection_pack.v1"
    assert report["rows"] == 1
    assert commands[0][0] == "ffmpeg"
    assert commands[0][commands[0].index("-ss") + 1] == "12.500"
    assert commands[0][commands[0].index("-t") + 1] == "8.000"
    assert Path(report["clips"][0]["clipPath"]).exists()
    review_table = clip_dir / "reference_selection_review.parquet"
    assert report["reviewTable"] == str(review_table)
    assert review_table.exists()
    validation = diagnostic.validate_reference_selection_review_table(review_table=review_table)
    assert validation["packReady"] is True
    assert validation["candidateDraftRows"] == 1


def test_validate_reference_selection_review_table_reports_pack_and_curated_status(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    review_table = _materialize_review_table(diagnostic, tmp_path, monkeypatch, [{}])
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.audio_diagnostic_reference_pack.audio_duration_seconds",
        lambda _path: 8.1,
    )

    report = diagnostic.validate_reference_selection_review_table(review_table=review_table)

    assert report["schema"] == ("xiuxian_wendao.audio_reference_selection_pack_validation.v1")
    assert report["packReady"] is True
    assert report["curatedReady"] is False
    assert report["candidateDraftRows"] == 1
    assert report["pendingRows"] == [
        {
            "row": 1,
            "clipPath": str(tmp_path / "clips" / "forum__chunk_0002.wav"),
            "source": "forum.mp3",
            "sourceId": str(tmp_path / "forum.mp3"),
            "chunkIndex": 2,
            "startSeconds": 12.5,
            "durationSeconds": 8.0,
            "reviewStatus": "review-needed",
            "selectionReason": "timeline-spread",
            "referenceStatus": "candidate-draft",
            "textCharCount": len("draft text"),
            "textSha256": hashlib.sha256(b"draft text").hexdigest(),
        }
    ]
    assert report["curatedRowSummaries"] == []
    assert report["issueRows"] == 0


def test_reference_selection_review_org_includes_candidate_text(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    review_table = _materialize_review_table(
        diagnostic,
        tmp_path,
        monkeypatch,
        [{"text": "private draft text"}],
    )
    output_org = tmp_path / "reference_selection_review.org"
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.audio_diagnostic_reference_pack.audio_duration_seconds",
        lambda _path: 8.0,
    )

    report = diagnostic.write_reference_selection_review_org(
        review_table=review_table,
        output_org=output_org,
    )

    assert report["schema"] == "xiuxian_wendao.audio_reference_selection_review_org.v1"
    assert report["rows"] == 1
    org_text = output_org.read_text(encoding="utf-8")
    assert "private draft text" in org_text
    assert "#+begin_src text :name candidate_text\nprivate draft text\n#+end_src" in org_text
    assert "#+begin_src text :name reference_text\n#+end_src" in org_text
    assert f":CLIP_PATH: {tmp_path / 'clips' / 'forum__chunk_0002.wav'}" in org_text
    assert ":REFERENCE_STATUS: candidate-draft" in org_text
    assert ":TEXT_CHAR_COUNT: 18" in org_text


def test_reference_selection_review_org_accepts_review_table(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    source_path = tmp_path / "forum.mp3"
    source_path.write_bytes(b"audio")
    selection_path = tmp_path / "selected.jsonl"
    clip_dir = tmp_path / "clips"
    output_org = tmp_path / "reference_selection_review.org"

    def fake_run(command, **_kwargs):
        Path(command[-1]).write_bytes(b"clip")
        return type("Result", (), {"returncode": 0, "stderr": ""})()

    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.audio_diagnostic_reference_pack.subprocess.run",
        fake_run,
    )
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.audio_diagnostic_reference_pack.audio_duration_seconds",
        lambda _path: 8.0,
    )
    diagnostic.write_jsonl(
        selection_path,
        [
            {
                "source": "forum.mp3",
                "sourceId": str(source_path),
                "chunkIndex": 2,
                "startSeconds": 12.5,
                "durationSeconds": 8.0,
                "reviewStatus": "review-needed",
                "selectionReason": "timeline-spread",
                "referenceStatus": diagnostic.REFERENCE_STATUS_CANDIDATE_DRAFT,
                "text": "private draft text",
            }
        ],
    )
    pack = diagnostic.materialize_reference_selection_pack(
        selection_jsonl=selection_path,
        clip_dir=clip_dir,
        ffmpeg_path="ffmpeg",
    )

    report = diagnostic.write_reference_selection_review_org(
        review_table=Path(pack["reviewTable"]),
        output_org=output_org,
    )

    assert report["reviewTable"] == pack["reviewTable"]
    org_text = output_org.read_text(encoding="utf-8")
    assert "private draft text" in org_text
    assert ":REVIEW_TABLE: " in org_text


def test_model_review_reference_selection_pack_is_redacted(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    review_table = _materialize_review_table(
        diagnostic,
        tmp_path,
        monkeypatch,
        [{"text": "candidate text"}],
    )
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.audio_diagnostic_reference_pack.audio_duration_seconds",
        lambda _path: 8.0,
    )

    report = diagnostic.model_review_reference_selection_pack(
        review_table=review_table,
        api_key="test-key",
        model="qwen/qwen3-asr-flash-2026-02-10",
        base_url="https://openrouter.ai/api/v1/audio/transcriptions",
        prompt="transcribe",
        max_tokens=1024,
        temperature=0.0,
        timeout_seconds=30,
        request_sender=lambda *_args: {"text": "candidate text"},
    )

    assert report["schema"] == ("xiuxian_wendao.audio_reference_selection_model_review.v1")
    assert report["succeededRows"] == 1
    assert report["failedRows"] == 0
    assert report["modelConsistentRows"] == 1
    assert report["promotionSafety"] == {
        "createsCuratedReferences": False,
        "requiresHumanCuratedReferenceText": True,
    }
    row = report["rowsReviewed"][0]
    assert row["modelReviewStatus"] == "model-consistent"
    assert row["textSha256"] == hashlib.sha256(b"candidate text").hexdigest()
    assert row["modelTextSha256"] == hashlib.sha256(b"candidate text").hexdigest()
    assert "candidate text" not in str(report)


def test_validate_reference_selection_review_table_flags_bad_duration(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    review_table = _materialize_review_table(
        diagnostic,
        tmp_path,
        monkeypatch,
        [{"referenceStatus": "curated", "text": "curated text"}],
    )
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.audio_diagnostic_reference_pack.audio_duration_seconds",
        lambda _path: 10.0,
    )

    report = diagnostic.validate_reference_selection_review_table(review_table=review_table)

    assert report["packReady"] is False
    assert report["curatedReady"] is False
    assert report["issues"][0]["issues"] == ["clip-duration-mismatch"]
