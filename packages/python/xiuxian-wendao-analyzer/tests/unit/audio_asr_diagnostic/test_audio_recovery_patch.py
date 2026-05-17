"""Audio recovery patch gate tests."""

from __future__ import annotations

import json

from .support import Path, _load_audio_asr_diagnostic


def _quality_row(
    index: int,
    *,
    repeat: float,
    chinese: float = 0.9,
    chars: int = 100,
) -> dict[str, object]:
    return {
        "source": "forum.mp3",
        "backend": "local-openai-audio",
        "model": "qwen3-asr-1.7b-mlx",
        "chunk_index": index,
        "start_seconds": float(index * 30),
        "duration_seconds": 30.0,
        "transcript_chars": chars,
        "chars_per_minute": float(chars * 2),
        "chinese_ratio": chinese,
        "repeated_ngram_ratio": repeat,
    }


def _write_json(path: Path, payload: object) -> None:
    path.write_text(json.dumps(payload), encoding="utf-8")


def test_audio_recovery_patch_gate_accepts_precise_short_windows(
    tmp_path: Path,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    base_quality = tmp_path / "base-quality.json"
    base_results = tmp_path / "base-results.json"
    recovery_quality = tmp_path / "recovery-quality.json"
    recovery_results = tmp_path / "recovery-results.json"
    recovery_plan = tmp_path / "plan.json"
    output_json = tmp_path / "patch-gate.json"
    _write_json(base_quality, [_quality_row(7, repeat=0.20, chinese=0.84, chars=100)])
    _write_json(base_results, [{"chunk_index": 7, "wall_seconds": 44.0}])
    _write_json(
        recovery_quality,
        [
            _quality_row(70, repeat=0.05, chinese=0.84, chars=50),
            _quality_row(71, repeat=0.10, chinese=0.85, chars=55),
        ],
    )
    _write_json(
        recovery_results,
        [
            {"chunk_index": 70, "wall_seconds": 19.0},
            {"chunk_index": 71, "wall_seconds": 20.0},
        ],
    )
    _write_json(
        recovery_plan,
        {
            "rows": [
                {"chunkIndex": 70, "parentChunkIndex": 7},
                {"chunkIndex": 71, "parentChunkIndex": 7},
            ]
        },
    )

    report = diagnostic.build_recovery_patch_gate_report(
        base_quality_json=base_quality,
        base_results_json=base_results,
        recovery_quality_json=recovery_quality,
        recovery_results_json=recovery_results,
        recovery_plan_json=recovery_plan,
        output_json=output_json,
    )

    assert report["schema"] == "xiuxian_wendao.audio_recovery_patch_gate.v1"
    assert report["parentRows"] == 1
    assert report["acceptedPatches"] == 1
    assert report["rejectedPatches"] == 0
    row = report["rows"][0]
    assert row["decision"] == "accept-patch"
    assert row["rejectionReasons"] == []
    assert row["recovery"]["chunkIndexes"] == [70, 71]
    assert row["recovery"]["charRatio"] == 1.05
    assert output_json.exists()


def test_audio_recovery_patch_gate_rejects_precision_risks(tmp_path: Path) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    base_quality = tmp_path / "base-quality.json"
    recovery_quality = tmp_path / "recovery-quality.json"
    recovery_plan = tmp_path / "plan.json"
    _write_json(
        base_quality,
        [
            _quality_row(1, repeat=0.10, chinese=0.90, chars=100),
            _quality_row(2, repeat=0.20, chinese=0.90, chars=100),
            _quality_row(3, repeat=0.20, chinese=0.90, chars=200),
        ],
    )
    _write_json(
        recovery_quality,
        [
            _quality_row(10, repeat=0.12, chinese=0.90, chars=100),
            _quality_row(20, repeat=0.05, chinese=0.80, chars=100),
            _quality_row(30, repeat=0.05, chinese=0.90, chars=80),
        ],
    )
    _write_json(
        recovery_plan,
        {
            "rows": [
                {"chunkIndex": 10, "parentChunkIndex": 1},
                {"chunkIndex": 20, "parentChunkIndex": 2},
                {"chunkIndex": 30, "parentChunkIndex": 3},
            ]
        },
    )

    report = diagnostic.build_recovery_patch_gate_report(
        base_quality_json=base_quality,
        base_results_json=None,
        recovery_quality_json=recovery_quality,
        recovery_results_json=None,
        recovery_plan_json=recovery_plan,
        output_json=None,
    )

    decisions = {
        row["parentChunkIndex"]: row["rejectionReasons"] for row in report["rows"]
    }
    assert report["acceptedPatches"] == 0
    assert report["rejectedPatches"] == 3
    assert decisions[1] == ["repeat-not-improved"]
    assert decisions[2] == ["chinese-ratio-drop"]
    assert decisions[3] == ["char-collapse"]


def test_audio_recovery_patch_gate_cli_writes_report(tmp_path: Path) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    base_quality = tmp_path / "base-quality.json"
    recovery_quality = tmp_path / "recovery-quality.json"
    recovery_plan = tmp_path / "plan.json"
    output_json = tmp_path / "patch-gate.json"
    _write_json(base_quality, [_quality_row(1, repeat=0.20, chars=100)])
    _write_json(recovery_quality, [_quality_row(10, repeat=0.02, chars=100)])
    _write_json(
        recovery_plan,
        {"rows": [{"chunkIndex": 10, "parentChunkIndex": 1}]},
    )

    exit_code = diagnostic.main(
        [
            "--build-risk-recovery-patch-gate-base-quality-json",
            str(base_quality),
            "--risk-recovery-patch-recovery-quality-json",
            str(recovery_quality),
            "--risk-recovery-patch-plan-json",
            str(recovery_plan),
            "--risk-recovery-patch-output-json",
            str(output_json),
        ]
    )

    assert exit_code == 0
    payload = json.loads(output_json.read_text(encoding="utf-8"))
    assert payload["acceptedPatches"] == 1
