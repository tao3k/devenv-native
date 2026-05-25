"""Audio diagnostic tests."""

from __future__ import annotations

import json

from .support import Path, _load_audio_asr_diagnostic


def test_curate_reference_org_cli_mode_writes_curated_jsonl(
    tmp_path: Path,
    monkeypatch,
    capsys,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    source_path = tmp_path / "forum.mp3"
    source_path.write_bytes(b"audio")
    selection_path = tmp_path / "selected.jsonl"
    clip_dir = tmp_path / "clips"
    review_org = tmp_path / "reference_selection_review.org"
    output_path = tmp_path / "reference_curated.jsonl"

    def fake_run(command, **_kwargs):
        Path(command[-1]).write_bytes(b"clip")
        return type("Result", (), {"returncode": 0, "stderr": ""})()

    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.audio_diagnostic_reference_pack.subprocess.run",
        fake_run,
    )
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.audio_diagnostic_reference_pack.audio_duration_seconds",
        lambda _path: 30.0,
    )
    diagnostic.write_jsonl(
        selection_path,
        [
            {
                "source": "forum.MP3",
                "sourceId": str(source_path),
                "chunkIndex": 2,
                "startSeconds": 60.0,
                "durationSeconds": 30.0,
                "reviewStatus": "review-needed",
                "selectionReason": "timeline-spread",
                "referenceStatus": "curated",
                "text": "curated transcript",
            }
        ],
    )
    pack = diagnostic.materialize_reference_selection_pack(
        selection_jsonl=selection_path,
        clip_dir=clip_dir,
        ffmpeg_path="ffmpeg",
    )
    diagnostic.write_reference_selection_review_org(
        review_table=Path(pack["reviewTable"]),
        output_org=review_org,
    )
    review_org.write_text(
        review_org.read_text(encoding="utf-8")
        .replace("** TODO Row 01", "** DONE Row 01")
        .replace(":REFERENCE_STATUS: curated", ":REFERENCE_STATUS: curated")
        .replace(
            "#+begin_src text :name reference_text\n#+end_src",
            "#+begin_src text :name reference_text\ncurated transcript\n#+end_src",
        ),
        encoding="utf-8",
    )

    exit_code = diagnostic.main(
        [
            "--curate-reference-org",
            str(review_org),
            "--curated-reference-jsonl",
            str(output_path),
        ]
    )

    assert exit_code == 0
    assert diagnostic.load_reference_transcripts(output_path) == {
        ("forum.MP3", 2): "curated transcript"
    }
    assert json.loads(capsys.readouterr().out)["rows"] == 1


def test_curate_reference_org_cli_rejects_candidate_draft(
    tmp_path: Path,
    monkeypatch,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    source_path = tmp_path / "forum.mp3"
    source_path.write_bytes(b"audio")
    selection_path = tmp_path / "selected.jsonl"
    clip_dir = tmp_path / "clips"
    review_org = tmp_path / "reference_selection_review.org"
    output_path = tmp_path / "reference_curated.jsonl"

    def fake_run(command, **_kwargs):
        Path(command[-1]).write_bytes(b"clip")
        return type("Result", (), {"returncode": 0, "stderr": ""})()

    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.audio_diagnostic_reference_pack.subprocess.run",
        fake_run,
    )
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.audio_diagnostic_reference_pack.audio_duration_seconds",
        lambda _path: 30.0,
    )
    diagnostic.write_jsonl(
        selection_path,
        [
            {
                "source": "forum.MP3",
                "sourceId": str(source_path),
                "chunkIndex": 2,
                "startSeconds": 60.0,
                "durationSeconds": 30.0,
                "reviewStatus": "review-needed",
                "selectionReason": "timeline-spread",
                "referenceStatus": diagnostic.REFERENCE_STATUS_CANDIDATE_DRAFT,
                "text": "model draft",
            }
        ],
    )
    pack = diagnostic.materialize_reference_selection_pack(
        selection_jsonl=selection_path,
        clip_dir=clip_dir,
        ffmpeg_path="ffmpeg",
    )
    diagnostic.write_reference_selection_review_org(
        review_table=Path(pack["reviewTable"]),
        output_org=review_org,
    )
    review_org.write_text(
        review_org.read_text(encoding="utf-8")
        .replace("** TODO Row 01", "** DONE Row 01")
        .replace(
            "#+begin_src text :name reference_text\n#+end_src",
            "#+begin_src text :name reference_text\nhuman text\n#+end_src",
        ),
        encoding="utf-8",
    )

    exit_code = diagnostic.main(
        [
            "--curate-reference-org",
            str(review_org),
            "--curated-reference-jsonl",
            str(output_path),
        ]
    )

    assert exit_code == 1
    assert not output_path.exists()


def test_validate_reference_selection_review_table_cli_writes_review_org(
    tmp_path: Path,
    monkeypatch,
    capsys,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    source_path = tmp_path / "forum.mp3"
    source_path.write_bytes(b"audio")
    selection_path = tmp_path / "selected.jsonl"
    clip_dir = tmp_path / "clips"
    review_org = tmp_path / "reference_selection_review.org"
    validation_json = tmp_path / "validation.json"

    def fake_run(command, **_kwargs):
        Path(command[-1]).write_bytes(b"clip")
        return type("Result", (), {"returncode": 0, "stderr": ""})()

    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.audio_diagnostic_reference_pack.subprocess.run",
        fake_run,
    )
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.audio_diagnostic_reference_pack.audio_duration_seconds",
        lambda _path: 30.0,
    )
    diagnostic.write_jsonl(
        selection_path,
        [
            {
                "source": "forum.mp3",
                "sourceId": str(source_path),
                "chunkIndex": 0,
                "startSeconds": 0.0,
                "durationSeconds": 30.0,
                "reviewStatus": "review-needed",
                "selectionReason": "timeline-spread",
                "referenceStatus": diagnostic.REFERENCE_STATUS_CANDIDATE_DRAFT,
                "text": "private candidate text",
            }
        ],
    )
    pack = diagnostic.materialize_reference_selection_pack(
        selection_jsonl=selection_path,
        clip_dir=clip_dir,
        ffmpeg_path="ffmpeg",
    )

    exit_code = diagnostic.main(
        [
            "--validate-reference-selection-review-table",
            pack["reviewTable"],
            "--reference-selection-review-org",
            str(review_org),
            "--reference-selection-validation-report-json",
            str(validation_json),
        ]
    )

    assert exit_code == 0
    report = json.loads(capsys.readouterr().out)
    assert report["reviewOrg"]["schema"] == "xiuxian_wendao.audio_reference_selection_review_org.v1"
    assert report["candidateDraftRows"] == 1
    org_text = review_org.read_text(encoding="utf-8")
    assert "private candidate text" in org_text
    assert "#+begin_src text :name candidate_text\nprivate candidate text\n#+end_src" in org_text
    assert "#+begin_src text :name reference_text\n#+end_src" in org_text
    assert json.loads(validation_json.read_text(encoding="utf-8"))["reviewOrg"]["rows"] == 1


def test_validate_reference_jsonl_reports_ready_against_manifest(
    tmp_path: Path,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    reference_path = tmp_path / "reference.jsonl"
    manifest_path = tmp_path / "audio_shards.json"
    diagnostic.write_jsonl(
        reference_path,
        [
            {
                "source": "forum.MP3",
                "chunkIndex": 0,
                "referenceStatus": "curated",
                "text": "curated transcript",
            }
        ],
    )
    manifest_path.write_text(
        json.dumps(
            {
                "items": [
                    {
                        "shardId": "shard-0",
                        "sourceId": "/private/forum.MP3",
                        "chunkIndex": 0,
                        "startMs": 0,
                        "durationMs": 30_000,
                        "mediaStartMs": 0,
                        "mediaDurationMs": 30_000,
                        "readingOrderKey": "000000.000000000000",
                    }
                ]
            }
        ),
        encoding="utf-8",
    )

    assert diagnostic.validate_reference_jsonl(
        reference_path,
        audio_shards_path=manifest_path,
    ) == {
        "ready": True,
        "referenceRows": 1,
        "candidateDraftRows": 0,
        "emptyTextRows": 0,
        "duplicateKeys": 0,
        "expectedShardRows": 1,
        "missingShardRows": 0,
        "extraReferenceRows": 0,
        "timelineAuthorityConfigured": True,
        "timelineAuthorityPassed": True,
        "timelineAuthorityIssueRows": 0,
        "timelineAuthorityIssues": [],
        "issues": [],
    }


def test_validate_reference_jsonl_reports_draft_and_coverage_issues(
    tmp_path: Path,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    reference_path = tmp_path / "reference.jsonl"
    manifest_path = tmp_path / "audio_shards.json"
    diagnostic.write_jsonl(
        reference_path,
        [
            {
                "source": "forum.MP3",
                "chunkIndex": 0,
                "referenceStatus": "candidate-draft",
                "text": "",
            },
            {
                "source": "forum.MP3",
                "chunkIndex": 0,
                "text": "duplicate transcript",
            },
        ],
    )
    manifest_path.write_text(
        json.dumps(
            {
                "items": [
                    {
                        "shardId": "shard-0",
                        "sourceId": "/private/forum.MP3",
                        "chunkIndex": 0,
                        "startMs": 0,
                        "durationMs": 30_000,
                        "mediaStartMs": 0,
                        "mediaDurationMs": 30_000,
                        "readingOrderKey": "000000.000000000000",
                    },
                    {
                        "shardId": "shard-1",
                        "sourceId": "/private/forum.MP3",
                        "chunkIndex": 1,
                        "startMs": 30_000,
                        "durationMs": 30_000,
                        "mediaStartMs": 30_000,
                        "mediaDurationMs": 30_000,
                        "readingOrderKey": "000001.000000030000",
                    },
                ]
            }
        ),
        encoding="utf-8",
    )

    report = diagnostic.validate_reference_jsonl(
        reference_path,
        audio_shards_path=manifest_path,
    )

    assert report["ready"] is False
    assert report["candidateDraftRows"] == 1
    assert report["emptyTextRows"] == 1
    assert report["duplicateKeys"] == 1
    assert report["missingShardRows"] == 1
    assert report["issues"] == [
        "empty-reference-text",
        "duplicate-reference-key",
        "candidate-draft-reference",
        "reference-coverage-missing",
    ]
