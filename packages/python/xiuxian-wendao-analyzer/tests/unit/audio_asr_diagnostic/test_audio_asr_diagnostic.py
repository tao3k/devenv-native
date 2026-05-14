"""Tests for the package-managed audio ASR diagnostic script."""

from __future__ import annotations

import argparse
import json
import os
import subprocess

from .support import (
    Path,
    _load_audio_asr_diagnostic,
    _load_fireredasr2s_local_setup,
)


def test_discover_audio_sources_is_case_insensitive_and_bounded(
    tmp_path: Path,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    (tmp_path / "b.MP3").write_bytes(b"b")
    (tmp_path / "a.mp3").write_bytes(b"a")
    (tmp_path / "notes.txt").write_text("skip", encoding="utf-8")

    sources = diagnostic.discover_audio_sources(tmp_path, limit_files=1)

    assert [path.name for path in sources] == ["a.mp3"]


def test_resolve_openrouter_key_uses_standard_name_only(tmp_path: Path) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    env_file = tmp_path / ".env"
    env_file.write_text(
        "OPENROUTE_API_KEY=wrong\nOPENROUTER_API_KEY=right\n",
        encoding="utf-8",
    )

    assert diagnostic.resolve_openrouter_api_key({}, env_file=env_file) == "right"
    assert (
        diagnostic.resolve_openrouter_api_key(
            {"OPENROUTER_API_KEY": '"env-value"'}, env_file=env_file
        )
        == "env-value"
    )
    assert (
        diagnostic.resolve_openrouter_api_key(
            {"OPENROUTE_API_KEY": "wrong"}, env_file=None
        )
        is None
    )


def test_build_openrouter_payload_uses_audio_input_shape() -> None:
    diagnostic = _load_audio_asr_diagnostic()

    payload = diagnostic.build_openrouter_payload(
        model="xiaomi/mimo-v2.5",
        prompt="transcribe",
        audio_bytes=b"audio",
        audio_format="wav",
        max_tokens=256,
        temperature=0.0,
    )

    assert payload["model"] == "xiaomi/mimo-v2.5"
    content = payload["messages"][0]["content"]
    assert content[0] == {"type": "text", "text": "transcribe"}
    assert content[1]["type"] == "input_audio"
    assert content[1]["input_audio"]["format"] == "wav"
    assert content[1]["input_audio"]["data"]


def test_normalize_whisper_model_name_accepts_docling_style_names() -> None:
    diagnostic = _load_audio_asr_diagnostic()

    assert diagnostic.normalize_whisper_model_name("WHISPER_TINY") == "tiny"
    assert diagnostic.normalize_whisper_model_name("WHISPER_BASE_NATIVE") == "base"
    assert diagnostic.normalize_whisper_model_name("turbo") == "turbo"


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

    monkeypatch.setattr(diagnostic.subprocess, "run", fake_run)

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


def test_fireredasr2s_setup_builds_cpu_command(tmp_path: Path) -> None:
    setup = _load_fireredasr2s_local_setup()

    command = setup.build_firered_command(
        venv_dir=tmp_path / "venv",
        model_root=tmp_path / "models",
    )

    parts = setup.shlex.split(command)
    assert parts[0].endswith("/venv/bin/fireredasr2s-cli")
    assert parts[parts.index("--asr_use_gpu") + 1] == "0"
    assert parts[parts.index("--vad_use_gpu") + 1] == "0"
    assert parts[parts.index("--lid_use_gpu") + 1] == "0"
    assert parts[parts.index("--punc_use_gpu") + 1] == "0"
    assert str(tmp_path / "models" / "FireRedASR2-AED") in parts
    assert str(tmp_path / "models" / "FireRedVAD" / "VAD") in parts


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
        download_models=True,
        skip_repo=False,
        skip_deps=False,
        dry_run=True,
        summary_json=summary_path,
    )

    summary = setup.run_setup(args)

    assert summary["repoRev"] == "rev"
    assert summary["downloadModels"] is True
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


def test_materialize_audio_chunks_invokes_ffmpeg_with_offsets(
    tmp_path: Path, monkeypatch
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    source = tmp_path / "forum.MP3"
    source.write_bytes(b"mp3")
    calls: list[list[str]] = []

    def fake_run(command, *, check, capture_output, text):
        calls.append(command)
        Path(command[-1]).write_bytes(b"wav")
        return subprocess.CompletedProcess(command, 0, "", "")

    monkeypatch.setattr(diagnostic.subprocess, "run", fake_run)

    chunks = diagnostic.materialize_audio_chunks(
        source,
        chunk_dir=tmp_path / "chunks",
        chunk_seconds=60,
        limit_chunks=2,
        sample_rate=16000,
        audio_format="wav",
        ffmpeg_path="/fake/ffmpeg",
    )

    assert [chunk.index for chunk in chunks] == [0, 1]
    assert calls[0][calls[0].index("-ss") + 1] == "0.000"
    assert calls[1][calls[1].index("-ss") + 1] == "60.000"
    assert calls[0][calls[0].index("-ar") + 1] == "16000"
    assert chunks[0].shard_id
    assert chunks[0].cache_key.startswith("audio-shards-v1:")
    assert chunks[0].sample_rate_hz == 16000


def test_materialize_audio_chunks_can_include_context_window(
    tmp_path: Path, monkeypatch
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    source = tmp_path / "forum.MP3"
    source.write_bytes(b"mp3")
    calls: list[list[str]] = []

    def fake_run(command, *, check, capture_output, text):
        calls.append(command)
        Path(command[-1]).write_bytes(b"wav")
        return subprocess.CompletedProcess(command, 0, "", "")

    monkeypatch.setattr(diagnostic.subprocess, "run", fake_run)

    chunks = diagnostic.materialize_audio_chunks(
        source,
        chunk_dir=tmp_path / "chunks",
        chunk_seconds=30,
        limit_chunks=1,
        sample_rate=16000,
        audio_format="wav",
        chunk_context_seconds=2.0,
        start_offset_seconds=10.0,
        source_duration_seconds=90.0,
        ffmpeg_path="/fake/ffmpeg",
    )

    assert calls[0][calls[0].index("-ss") + 1] == "8.000"
    assert calls[0][calls[0].index("-t") + 1] == "34.000"
    assert chunks[0].start_seconds == 10.0
    assert chunks[0].media_start_seconds == 8.0
    assert chunks[0].context_before_seconds == 2.0
    assert chunks[0].context_after_seconds == 2.0


def test_audio_shard_manifest_is_model_agnostic(tmp_path: Path) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    source = tmp_path / "forum.mp3"
    source.write_bytes(b"mp3")
    item = diagnostic.build_audio_shard_manifest_item(
        source,
        profile=diagnostic.DEFAULT_AUDIO_SHARD_PROFILE,
        source_sha256="a" * 64,
        chunk_index=0,
        start_seconds=1.5,
        duration_seconds=30.0,
        media_start_seconds=0.0,
        media_duration_seconds=31.0,
        sample_rate_hz=16000,
        channels=1,
        audio_format="WAV",
    )

    manifest = diagnostic.audio_shard_manifest(
        profile=diagnostic.DEFAULT_AUDIO_SHARD_PROFILE,
        sample_strategy="uniform",
        chunks=[
            diagnostic.AudioChunk(
                source=source,
                path=tmp_path / "chunk.wav",
                index=0,
                start_seconds=1.5,
                duration_seconds=30.0,
                format="wav",
                shard_id=item.shardId,
                cache_key=item.cacheKey,
                source_sha256=item.sourceSha256,
                sample_rate_hz=16000,
                channels=1,
                media_start_seconds=0.0,
                media_duration_seconds=31.0,
                context_before_seconds=0.0,
                context_after_seconds=1.0,
            )
        ],
    )

    assert manifest["schema"] == "xiuxian_wendao.audio_shards.v1"
    assert manifest["profile"] == "audio-shards-v1"
    assert manifest["items"][0]["startMs"] == 1500
    assert manifest["items"][0]["mediaDurationMs"] == 31000
    assert manifest["items"][0]["contextAfterMs"] == 1000
    assert manifest["items"][0]["cacheKey"].startswith("audio-shards-v1:")


def test_audio_result_cache_reuses_successful_backend_result(
    tmp_path: Path, monkeypatch
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    chunk = diagnostic.AudioChunk(
        source=tmp_path / "forum.mp3",
        path=tmp_path / "chunk.wav",
        index=0,
        start_seconds=0.0,
        duration_seconds=30.0,
        format="wav",
        shard_id="shard",
        cache_key="audio-shards-v1:shard",
    )
    cache_key = diagnostic.audio_result_cache_key(
        shard_cache_key=chunk.cache_key,
        task_profile="transcription",
        backend_id="openrouter-chat-audio",
        backend_config_hash=diagnostic.backend_config_hash(
            "openrouter-chat-audio",
            openrouter_model="xiaomi/mimo-v2.5",
            openrouter_base_url=diagnostic.DEFAULT_OPENROUTER_URL,
            local_asr_model=diagnostic.DEFAULT_LOCAL_ASR_MODEL,
            local_language=diagnostic.DEFAULT_LOCAL_LANGUAGE,
            fireredasr2s_command=diagnostic.DEFAULT_FIREREDASR2S_COMMAND,
            prompt=diagnostic.DEFAULT_PROMPT,
            max_tokens=128,
            temperature=0.0,
            audio_format="wav",
        ),
    )
    diagnostic.write_result_cache(
        tmp_path / "cache",
        result_cache_key=cache_key,
        backend="openrouter-chat-audio",
        model="xiaomi/mimo-v2.5",
        transcript="缓存文本",
    )

    def fail_transcribe(*_args, **_kwargs):
        raise AssertionError("backend should not be called on cache hit")

    monkeypatch.setattr(diagnostic, "transcribe_openrouter", fail_transcribe)

    result = diagnostic.run_backend(
        "openrouter-chat-audio",
        chunk,
        output_dir=tmp_path / "out",
        openrouter_api_key="key",
        openrouter_model="xiaomi/mimo-v2.5",
        openrouter_base_url=diagnostic.DEFAULT_OPENROUTER_URL,
        local_asr_model=diagnostic.DEFAULT_LOCAL_ASR_MODEL,
        local_language=diagnostic.DEFAULT_LOCAL_LANGUAGE,
        fireredasr2s_command=diagnostic.DEFAULT_FIREREDASR2S_COMMAND,
        prompt=diagnostic.DEFAULT_PROMPT,
        max_tokens=128,
        temperature=0.0,
        timeout_seconds=10,
        result_cache_dir=tmp_path / "cache",
    )

    assert result.status == "ok"
    assert result.transcript_chars == 4
    assert result.result_cache_key == cache_key
    assert Path(result.transcript_path).read_text(encoding="utf-8") == "缓存文本"


def test_uniform_chunk_offsets_cover_document_surface() -> None:
    diagnostic = _load_audio_asr_diagnostic()

    offsets = diagnostic.chunk_start_offsets(
        duration_seconds=300.0,
        chunk_seconds=30,
        limit_chunks=3,
        strategy="uniform",
        start_offset_seconds=10.0,
    )

    assert offsets == [10.0, 140.0, 270.0]


def test_ensure_ffmpeg_on_path_links_imageio_binary(
    tmp_path: Path, monkeypatch
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    fake_ffmpeg = tmp_path / "imageio-ffmpeg"
    fake_ffmpeg.write_bytes(b"bin")
    fake_ffmpeg.chmod(0o755)
    monkeypatch.setattr(diagnostic.shutil, "which", lambda _name: None)
    monkeypatch.setattr(
        diagnostic, "resolve_ffmpeg_executable", lambda: str(fake_ffmpeg)
    )
    monkeypatch.setenv("PATH", "/usr/bin")

    diagnostic.ensure_ffmpeg_on_path(tmp_path / "bin")

    assert (tmp_path / "bin" / "ffmpeg").exists()
    assert os.environ["PATH"].startswith(str(tmp_path / "bin"))


def test_quality_rows_classify_reference_and_proxy_statuses(tmp_path: Path) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    good_transcript = tmp_path / "good.txt"
    good_transcript.write_text(
        "这是一个中文转写，包含智能家居论坛内容。", encoding="utf-8"
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
            backend="local-whisper",
            source="/tmp/forum.MP3",
            chunk="/tmp/chunk1.wav",
            chunk_index=1,
            start_seconds=30.0,
            duration_seconds=30.0,
            model="openai-whisper:base:zh",
            status="ok",
            wall_seconds=3.0,
            transcript_chars=noisy_transcript.stat().st_size,
            transcript_path=str(noisy_transcript),
            error="",
        ),
    ]

    rows = diagnostic.build_quality_rows(
        results,
        references={("forum.MP3", 0): "这是一个中文转写，包含智能家居论坛内容。"},
        min_chars_per_minute=20.0,
        min_chinese_ratio=0.35,
        max_inaudible_per_minute=1.0,
    )

    assert rows[0].review_status == "reference-pass"
    assert rows[0].reference_cer == 0
    assert rows[1].review_status == "weak-language-ratio"
    summary = diagnostic.summarize_quality(rows)
    assert summary["qualityByBackend"]["openrouter-chat-audio"]["referencePass"] == 1
    assert summary["qualityByBackend"]["local-whisper"]["weakRows"] == 1


def test_quality_review_tsv_contains_chunk_and_status(tmp_path: Path) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    row = diagnostic.QualityRow(
        backend="openrouter-chat-audio",
        source="/tmp/forum.MP3",
        chunk_index=2,
        start_seconds=60.0,
        status="ok",
        review_status="review-needed",
        model="xiaomi/mimo-v2.5",
        transcript_chars=80,
        chinese_ratio=0.8,
        inaudible_count=1,
        inaudible_per_minute=2.0,
        chars_per_minute=160.0,
        reference_cer=None,
        transcript_path="/tmp/transcript.txt",
        error="",
    )

    diagnostic.write_quality_tsv(tmp_path / "review.tsv", [row])

    content = (tmp_path / "review.tsv").read_text(encoding="utf-8")
    assert "reviewStatus" in content
    assert "review-needed" in content
    assert "\t2\t60.000\t" in content


def test_run_diagnostic_writes_summary_and_results(tmp_path: Path, monkeypatch) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    source_root = tmp_path / "audio"
    source_root.mkdir()
    source = source_root / "forum.MP3"
    source.write_bytes(b"mp3")
    chunk = tmp_path / "chunk.wav"
    chunk.write_bytes(b"wav")

    monkeypatch.setattr(
        diagnostic,
        "materialize_audio_chunks",
        lambda *_args, **_kwargs: [
            diagnostic.AudioChunk(
                source=source,
                path=chunk,
                index=0,
                start_seconds=0.0,
                duration_seconds=60.0,
                format="wav",
                shard_id="shard",
                cache_key="audio-shards-v1:shard",
                source_sha256="a" * 64,
                sample_rate_hz=16000,
                channels=1,
            )
        ],
    )
    monkeypatch.setattr(
        diagnostic, "transcribe_local_docling", lambda *_args, **_kwargs: "本地"
    )
    monkeypatch.setattr(
        diagnostic, "transcribe_local_whisper", lambda *_args, **_kwargs: "本地"
    )
    monkeypatch.setattr(
        diagnostic, "transcribe_openrouter", lambda *_args, **_kwargs: "云端"
    )

    args = argparse.Namespace(
        source_root=str(source_root),
        backend="both",
        output_dir=tmp_path / "out",
        env_file=None,
        chunk_seconds=60,
        limit_files=1,
        limit_chunks=1,
        sample_strategy="head",
        start_offset_seconds=0.0,
        chunk_context_seconds=0.0,
        sample_rate=16000,
        audio_format="wav",
        openrouter_model="xiaomi/mimo-v2.5",
        openrouter_base_url=diagnostic.DEFAULT_OPENROUTER_URL,
        local_asr_model=diagnostic.DEFAULT_LOCAL_ASR_MODEL,
        local_language=diagnostic.DEFAULT_LOCAL_LANGUAGE,
        fireredasr2s_command=diagnostic.DEFAULT_FIREREDASR2S_COMMAND,
        prompt=diagnostic.DEFAULT_PROMPT,
        max_tokens=128,
        temperature=0.0,
        timeout_seconds=10,
        result_cache_dir=None,
        no_result_cache=False,
        reference_jsonl=None,
        min_chars_per_minute=40.0,
        min_chinese_ratio=0.35,
        max_inaudible_per_minute=30.0,
        force=False,
    )
    monkeypatch.setenv("OPENROUTER_API_KEY", "test-key")

    report = diagnostic.run_diagnostic(args)

    assert report["resultCount"] == 2
    assert report["errorCount"] == 0
    assert "qualityByBackend" in report
    assert (tmp_path / "out" / "summary.json").exists()
    assert (tmp_path / "out" / "results.json").exists()
    assert (tmp_path / "out" / "audio_shards.json").exists()
    assert (tmp_path / "out" / "quality.json").exists()
    assert (tmp_path / "out" / "review.tsv").exists()
    manifest = json.loads((tmp_path / "out" / "audio_shards.json").read_text())
    assert manifest["schema"] == "xiuxian_wendao.audio_shards.v1"
    assert report["audioShardProfile"] == "audio-shards-v1"
