"""Audio diagnostic tests."""

from __future__ import annotations

import argparse
import json
import subprocess

from xiuxian_wendao_analyzer import audio_diagnostic_firered

from .support import Path, _load_audio_asr_diagnostic, _load_fireredasr2s_local_setup


def test_fireredasr2s_adapter_extracts_cli_jsonl_text(
    tmp_path: Path, monkeypatch
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    chunk = tmp_path / "chunk.wav"
    chunk.write_bytes(b"wav")
    calls: list[list[str]] = []

    def fake_run(command, *, check, capture_output, text):
        calls.append(command)
        outdir = Path(command[command.index("--outdir") + 1])
        outdir.mkdir(parents=True, exist_ok=True)
        (outdir / "result.jsonl").write_text(
            json.dumps({"text": "智能家居论坛"}, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )
        return subprocess.CompletedProcess(command, 0, "", "")

    monkeypatch.setattr(audio_diagnostic_firered.subprocess, "run", fake_run)

    text = diagnostic.transcribe_fireredasr2s(
        diagnostic.AudioChunk(
            source=tmp_path / "source.MP3",
            path=chunk,
            index=0,
            start_seconds=0.0,
            duration_seconds=30.0,
            format="wav",
        ),
        tmp_path / "out",
        command="python -m fireredasr2s_cli",
    )

    assert text == "智能家居论坛"
    assert calls[0][:3] == ["python", "-m", "fireredasr2s_cli"]
    assert "--wav_paths" in calls[0]


def test_fireredasr2s_setup_rejects_cpu_command(tmp_path: Path) -> None:
    setup = _load_fireredasr2s_local_setup()

    try:
        setup.build_firered_command(
            venv_dir=tmp_path / "venv",
            model_root=tmp_path / "models",
            resolved_device="cpu",
        )
    except ValueError as exc:
        assert "CPU fallback is intentionally disabled" in str(exc)
    else:
        raise AssertionError("FireRedASR2S CPU command should be rejected")


def test_fireredasr2s_setup_builds_cuda_command(tmp_path: Path) -> None:
    setup = _load_fireredasr2s_local_setup()

    command = setup.build_firered_command(
        venv_dir=tmp_path / "venv",
        model_root=tmp_path / "models",
        resolved_device="cuda",
    )

    parts = setup.shlex.split(command)
    assert parts[0].endswith("/venv/bin/fireredasr2s-cli")
    assert parts[parts.index("--asr_use_gpu") + 1] == "1"
    assert parts[parts.index("--vad_use_gpu") + 1] == "1"
    assert parts[parts.index("--lid_use_gpu") + 1] == "1"
    assert parts[parts.index("--punc_use_gpu") + 1] == "1"
    assert str(tmp_path / "models" / "FireRedASR2-AED") in parts
    assert str(tmp_path / "models" / "FireRedVAD" / "VAD") in parts


def test_fireredasr2s_setup_rejects_mps_device(tmp_path: Path, monkeypatch) -> None:
    setup = _load_fireredasr2s_local_setup()
    monkeypatch.setattr(
        setup,
        "probe_torch_devices",
        lambda **_kwargs: {"cuda": False, "mps": True, "mpsBuilt": True},
    )

    try:
        setup.resolve_firered_device(
            "mps",
            venv_dir=tmp_path / "venv",
            dry_run=False,
        )
    except RuntimeError as exc:
        assert ".cuda()" in str(exc)
    else:
        raise AssertionError("FireRedASR2S MPS should be rejected explicitly")


def test_fireredasr2s_auto_rejects_mps_without_cpu_fallback(
    tmp_path: Path, monkeypatch
) -> None:
    setup = _load_fireredasr2s_local_setup()
    monkeypatch.setattr(
        setup,
        "probe_torch_devices",
        lambda **_kwargs: {"cuda": False, "mps": True, "mpsBuilt": True},
    )

    try:
        setup.resolve_firered_device(
            "auto",
            venv_dir=tmp_path / "venv",
            dry_run=False,
        )
    except RuntimeError as exc:
        assert "MPS/Metal" in str(exc)
        assert "CPU fallback" in str(exc)
    else:
        raise AssertionError("FireRedASR2S auto should reject MPS-only hosts")


def test_fireredasr2s_setup_dry_run_writes_summary(tmp_path: Path) -> None:
    setup = _load_fireredasr2s_local_setup()
    summary_path = tmp_path / "summary.json"
    args = argparse.Namespace(
        repo_url="https://example.invalid/FireRedASR2S.git",
        repo_rev="rev",
        repo_dir=tmp_path / "repo",
        venv_dir=tmp_path / "venv",
        model_root=tmp_path / "models",
        python="python3.11",
        device="cuda",
        download_models=True,
        skip_repo=False,
        skip_deps=False,
        dry_run=True,
        summary_json=summary_path,
    )

    summary = setup.run_setup(args)

    assert summary["repoRev"] == "rev"
    assert summary["downloadModels"] is True
    assert summary["resolvedDevice"] == "cuda"
    assert "fireredasr2s-cli" in summary["fireRedAsr2sCommand"]
    assert json.loads(summary_path.read_text(encoding="utf-8")) == summary


def test_fireredasr2s_model_download_skips_existing_models(
    tmp_path: Path, monkeypatch
) -> None:
    setup = _load_fireredasr2s_local_setup()
    model_root = tmp_path / "models"
    (model_root / "FireRedASR2-AED").mkdir(parents=True)
    (model_root / "FireRedASR2-AED" / "config.yaml").write_text("ok", encoding="utf-8")
    calls: list[list[str]] = []

    def fake_run(command, **kwargs):
        calls.append(list(command))
        return setup.CommandResult(list(command), 0, "", "")

    monkeypatch.setattr(setup, "run_command", fake_run)

    setup.download_models(
        venv_dir=tmp_path / "venv",
        model_root=model_root,
        dry_run=False,
    )

    assert not any("FireRedTeam/FireRedASR2-AED" in call for call in calls)
    assert any("FireRedTeam/FireRedVAD" in call for call in calls)
