"""Audio diagnostic tests."""

from __future__ import annotations

import json
import os

from xiuxian_wendao_analyzer import (
    audio_diagnostic_backends,
    audio_diagnostic_materialization,
    audio_diagnostic_media_probe,
)

from .support import Path, _load_audio_asr_diagnostic


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

    monkeypatch.setattr(
        audio_diagnostic_backends, "transcribe_openrouter", fail_transcribe
    )

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


def test_openai_compatible_backend_writes_segment_timeline(
    tmp_path: Path, monkeypatch
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    chunk = diagnostic.AudioChunk(
        source=tmp_path / "forum.mp3",
        path=tmp_path / "chunk.wav",
        index=0,
        start_seconds=10.0,
        duration_seconds=30.0,
        format="wav",
        shard_id="shard",
        cache_key="audio-shards-v1:shard",
        media_start_seconds=8.0,
        media_duration_seconds=34.0,
    )
    chunk.path.write_bytes(b"wav")

    monkeypatch.setattr(
        audio_diagnostic_backends,
        "transcribe_openrouter",
        lambda *_args, **_kwargs: (
            "first segment second segment",
            [
                {"startSeconds": 0.5, "endSeconds": 1.2, "text": "first segment"},
                {"startSeconds": 1.2, "endSeconds": 2.0, "text": "second segment"},
            ],
        ),
    )

    result = diagnostic.run_backend(
        "local-openai-audio",
        chunk,
        output_dir=tmp_path / "out",
        openrouter_api_key=None,
        openrouter_model="wendao-local-audio",
        openrouter_base_url="http://127.0.0.1:8012/v1/chat/completions",
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
    assert result.segment_count == 2
    segments = [
        json.loads(line)
        for line in Path(result.segments_path).read_text(encoding="utf-8").splitlines()
    ]
    assert segments[0]["startSeconds"] == 8.5
    assert segments[1]["endSeconds"] == 10.0
    quality_rows = diagnostic.build_quality_rows(
        [result],
        references={},
        max_reference_cer=0.15,
        required_terms=[],
        min_required_term_recall=1.0,
        min_chars_per_minute=20.0,
        min_chinese_ratio=0.35,
        max_inaudible_per_minute=1.0,
        max_repeated_ngram_ratio=0.35,
    )
    diagnostic.write_transcript_timeline_vtt(tmp_path / "timeline.vtt", quality_rows)
    timeline = (tmp_path / "timeline.vtt").read_text(encoding="utf-8")
    assert "00:00:08.500 --> 00:00:09.200" in timeline
    assert "first segment" in timeline


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
    monkeypatch.setattr(
        audio_diagnostic_media_probe.shutil, "which", lambda _name: None
    )
    monkeypatch.setattr(
        audio_diagnostic_materialization,
        "resolve_ffmpeg_executable",
        lambda: str(fake_ffmpeg),
    )
    monkeypatch.setenv("PATH", "/usr/bin")

    diagnostic.ensure_ffmpeg_on_path(tmp_path / "bin")

    assert (tmp_path / "bin" / "ffmpeg").exists()
    assert os.environ["PATH"].startswith(str(tmp_path / "bin"))
