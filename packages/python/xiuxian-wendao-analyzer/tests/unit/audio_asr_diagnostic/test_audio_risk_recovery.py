"""Audio risk recovery plan tests."""

from __future__ import annotations

import json

from .support import Path, _load_audio_asr_diagnostic


def _quality_row(
    index: int,
    *,
    repeat: float = 0.02,
    chinese: float = 0.9,
    chars_per_minute: float = 240.0,
) -> dict[str, object]:
    return {
        "source": "forum.mp3",
        "backend": "local-openai-audio",
        "model": "qwen3-asr-1.7b-mlx",
        "chunk_index": index,
        "start_seconds": float(index * 60),
        "duration_seconds": 60.0,
        "transcript_chars": int(chars_per_minute),
        "chars_per_minute": chars_per_minute,
        "chinese_ratio": chinese,
        "repeated_ngram_ratio": repeat,
    }


def test_audio_risk_recovery_selects_explainable_parent_rows() -> None:
    diagnostic = _load_audio_asr_diagnostic()
    quality_rows = [
        _quality_row(0),
        _quality_row(1),
        _quality_row(2, repeat=0.20),
        _quality_row(3, chinese=0.82),
        _quality_row(4, chars_per_minute=90.0),
        _quality_row(5),
    ]
    result_rows = {
        1: {"chunk_index": 1, "wall_seconds": 58.0},
    }

    selected = diagnostic.select_audio_risk_parent_rows(
        quality_rows,
        result_rows=result_rows,
        options=diagnostic.AudioRiskRecoveryOptions(limit_parents=10),
    )

    assert [row["parentChunkIndex"] for row in selected] == [0, 1, 2, 3, 4, 5]
    reasons = {row["parentChunkIndex"]: row["reasons"] for row in selected}
    assert reasons[0] == ["timeline-boundary"]
    assert reasons[1] == ["high-latency"]
    assert reasons[2] == ["high-repetition"]
    assert reasons[3] == ["low-chinese-ratio"]
    assert reasons[4] == ["low-text-density"]
    assert reasons[5] == ["timeline-boundary"]


def test_audio_risk_recovery_reserves_boundary_rows_under_limit() -> None:
    diagnostic = _load_audio_asr_diagnostic()
    quality_rows = [
        _quality_row(0),
        _quality_row(1, repeat=0.18),
        _quality_row(2, repeat=0.19),
        _quality_row(3, repeat=0.20),
        _quality_row(4),
    ]

    selected = diagnostic.select_audio_risk_parent_rows(
        quality_rows,
        result_rows={},
        options=diagnostic.AudioRiskRecoveryOptions(limit_parents=3),
    )

    assert [row["parentChunkIndex"] for row in selected] == [0, 3, 4]
    assert selected[0]["reasons"] == ["timeline-boundary"]
    assert selected[-1]["reasons"] == ["timeline-boundary"]


def test_audio_risk_recovery_builds_30s_explicit_windows(tmp_path: Path) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    output_json = tmp_path / "risk-plan.json"
    quality_json = tmp_path / "quality.json"
    results_json = tmp_path / "results.json"
    quality_json.write_text(
        json.dumps(
            [
                _quality_row(7, repeat=0.22),
                _quality_row(8),
            ]
        ),
        encoding="utf-8",
    )
    results_json.write_text(
        json.dumps(
            [
                {"chunk_index": 7, "wall_seconds": 21.0},
                {"chunk_index": 8, "wall_seconds": 20.0},
            ]
        ),
        encoding="utf-8",
    )

    report = diagnostic.build_risk_recovery_plan_report(
        quality_json=quality_json,
        results_json=results_json,
        output_json=output_json,
        options=diagnostic.AudioRiskRecoveryOptions(
            split_seconds=30.0,
            limit_parents=4,
            include_boundaries=False,
        ),
    )

    assert report["schema"] == "xiuxian_wendao.audio_risk_recovery_plan.v1"
    assert report["selectedParentRows"] == 1
    assert report["recoveryRows"] == 2
    assert [
        (
            row["chunkIndex"],
            row["parentChunkIndex"],
            row["startSeconds"],
            row["durationSeconds"],
        )
        for row in report["rows"]
    ] == [
        (70, 7, 420.0, 30.0),
        (71, 7, 450.0, 30.0),
    ]
    assert output_json.exists()

    source = tmp_path / "forum.mp3"
    source.write_bytes(b"mp3")
    windows = diagnostic.load_explicit_windows(output_json, source=source)
    assert [
        (row.index, row.start_seconds, row.duration_seconds) for row in windows
    ] == [
        (70, 420.0, 30.0),
        (71, 450.0, 30.0),
    ]
