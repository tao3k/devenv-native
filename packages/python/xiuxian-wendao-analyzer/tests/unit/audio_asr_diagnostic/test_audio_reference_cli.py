"""Audio diagnostic tests."""

from __future__ import annotations

import json

from .support import Path, _load_audio_asr_diagnostic


def test_curate_reference_draft_cli_mode_writes_curated_jsonl(
    tmp_path: Path,
    capsys,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    draft_path = tmp_path / "reference_draft.jsonl"
    output_path = tmp_path / "reference_curated.jsonl"
    diagnostic.write_jsonl(
        draft_path,
        [{"source": "forum.MP3", "chunkIndex": 2, "text": "curated transcript"}],
    )

    exit_code = diagnostic.main(
        [
            "--curate-reference-draft",
            str(draft_path),
            "--curated-reference-jsonl",
            str(output_path),
        ]
    )

    assert exit_code == 0
    assert diagnostic.load_reference_transcripts(output_path) == {
        ("forum.MP3", 2): "curated transcript"
    }
    assert diagnostic.reference_candidate_draft_row_count(output_path) == 0
    assert json.loads(capsys.readouterr().out)["rows"] == 1


def test_curate_reference_tsv_cli_mode_writes_curated_jsonl(
    tmp_path: Path,
    capsys,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    draft_path = tmp_path / "reference_draft.tsv"
    output_path = tmp_path / "reference_curated.jsonl"
    draft_path.write_text(
        "\t".join(
            [
                "source",
                "chunkIndex",
                "startSeconds",
                "durationSeconds",
                "referenceStatus",
                "text",
            ]
        )
        + "\n"
        + "\t".join(
            ["forum.MP3", "2", "60.0", "30.0", "candidate-draft", "curated text"]
        )
        + "\n",
        encoding="utf-8",
    )

    exit_code = diagnostic.main(
        [
            "--curate-reference-tsv",
            str(draft_path),
            "--curated-reference-jsonl",
            str(output_path),
        ]
    )

    assert exit_code == 0
    assert diagnostic.load_reference_transcripts(output_path) == {
        ("forum.MP3", 2): "curated text"
    }
    assert diagnostic.reference_candidate_draft_row_count(output_path) == 0
    assert json.loads(capsys.readouterr().out)["rows"] == 1


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
