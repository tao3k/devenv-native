"""Audio diagnostic tests."""

from __future__ import annotations

import json

from .support import Path, _load_audio_asr_diagnostic


def test_validate_reference_jsonl_rejects_invalid_timeline_authority(
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
                        "startMs": 10_000,
                        "durationMs": 30_000,
                        "mediaStartMs": 15_000,
                        "mediaDurationMs": 5_000,
                        "readingOrderKey": "000000.000000010000",
                    }
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
    assert report["timelineAuthorityPassed"] is False
    assert report["timelineAuthorityIssueRows"] == 1
    assert report["timelineAuthorityIssues"][0]["issues"] == [
        "media-window-does-not-cover-logical-chunk"
    ]
    assert report["issues"] == ["audio-shard-timeline-invalid"]


def test_validate_reference_jsonl_cli_mode_prints_report(
    tmp_path: Path,
    capsys,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    reference_path = tmp_path / "reference.jsonl"
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

    exit_code = diagnostic.main(["--validate-reference-jsonl", str(reference_path)])

    assert exit_code == 0
    assert json.loads(capsys.readouterr().out)["ready"] is True


def test_validate_reference_jsonl_cli_mode_writes_report(
    tmp_path: Path,
    capsys,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    reference_path = tmp_path / "reference.jsonl"
    report_path = tmp_path / "reference_validation_report.json"
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

    exit_code = diagnostic.main(
        [
            "--validate-reference-jsonl",
            str(reference_path),
            "--reference-validation-report-json",
            str(report_path),
        ]
    )

    assert exit_code == 0
    assert json.loads(capsys.readouterr().out)["ready"] is True
    assert json.loads(report_path.read_text(encoding="utf-8"))["ready"] is True
