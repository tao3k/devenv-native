"""Audio diagnostic tests."""

from __future__ import annotations

import argparse
import json

from xiuxian_wendao_analyzer import (
    audio_diagnostic_backends,
    audio_diagnostic_runner_pipeline,
)

from .support import Path, _load_audio_asr_diagnostic


def test_private_audio_diagnostic_requires_cache_output(tmp_path: Path) -> None:
    diagnostic = _load_audio_asr_diagnostic()

    try:
        diagnostic.validate_private_output_dir(
            tmp_path / "package-fixtures",
            start=tmp_path,
            input_privacy=diagnostic.PRIVATE_INPUT_PRIVACY,
            allow_private_output_outside_cache=False,
        )
    except ValueError as exc:
        assert ".cache/agent/evidence" in str(exc)
    else:
        raise AssertionError("private output outside cache should be rejected")

    diagnostic.validate_private_output_dir(
        tmp_path / ".cache" / "agent" / "evidence" / "audio",
        start=tmp_path,
        input_privacy=diagnostic.PRIVATE_INPUT_PRIVACY,
        allow_private_output_outside_cache=False,
    )


def test_run_diagnostic_writes_summary_and_results(tmp_path: Path, monkeypatch) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    source_root = tmp_path / "audio"
    source_root.mkdir()
    source = source_root / "forum.MP3"
    source.write_bytes(b"mp3")
    chunk = tmp_path / "chunk.wav"
    chunk.write_bytes(b"wav")

    monkeypatch.setattr(
        audio_diagnostic_runner_pipeline,
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
        audio_diagnostic_backends,
        "transcribe_local_docling",
        lambda *_args, **_kwargs: "本地",
    )
    monkeypatch.setattr(
        audio_diagnostic_backends,
        "transcribe_openrouter",
        lambda *_args, **_kwargs: "云端",
    )

    args = argparse.Namespace(
        source_root=str(source_root),
        backend="both",
        output_dir=tmp_path / "out",
        input_privacy=diagnostic.SHAREABLE_INPUT_PRIVACY,
        allow_private_output_outside_cache=False,
        env_file=None,
        chunk_seconds=60,
        limit_files=1,
        limit_chunks=1,
        sample_strategy="head",
        start_offset_seconds=0.0,
        chunk_context_seconds=0.0,
        audio_materialization_mode=diagnostic.AUDIO_MATERIALIZATION_NORMALIZED_16K_WAV,
        sample_rate=16000,
        audio_format="wav",
        openrouter_model="xiaomi/mimo-v2.5",
        openrouter_base_url=diagnostic.DEFAULT_OPENROUTER_URL,
        local_asr_model=diagnostic.DEFAULT_LOCAL_ASR_MODEL,
        local_language=diagnostic.DEFAULT_LOCAL_LANGUAGE,
        fireredasr2s_command=diagnostic.DEFAULT_FIREREDASR2S_COMMAND,
        prompt=diagnostic.DEFAULT_PROMPT,
        domain_terms_file=None,
        required_terms_file=None,
        max_tokens=128,
        temperature=0.0,
        timeout_seconds=10,
        result_cache_dir=None,
        no_result_cache=False,
        reference_jsonl=None,
        truth_template_jsonl=None,
        max_reference_cer=0.15,
        min_required_term_recall=1.0,
        min_chars_per_minute=40.0,
        min_chinese_ratio=0.35,
        max_inaudible_per_minute=30.0,
        max_repeated_ngram_ratio=0.35,
        force=False,
    )
    monkeypatch.setenv("OPENROUTER_API_KEY", "test-key")

    report = diagnostic.run_diagnostic(args)

    assert report["resultCount"] == 2
    assert report["errorCount"] == 0
    assert report["diagnosticWallSeconds"] > 0
    assert "qualityByBackend" in report
    assert (tmp_path / "out" / "summary.json").exists()
    assert (tmp_path / "out" / "results.json").exists()
    assert (tmp_path / "out" / "audio_shards.json").exists()
    assert (tmp_path / "out" / "quality.json").exists()
    assert (tmp_path / "out" / "review.tsv").exists()
    assert (tmp_path / "out" / "transcript_review.tsv").exists()
    assert (tmp_path / "out" / "transcript_timeline.jsonl").exists()
    assert (tmp_path / "out" / "transcript_timeline.vtt").exists()
    assert (tmp_path / "out" / "transcript_timeline.srt").exists()
    assert (tmp_path / "out" / "reference_draft.jsonl").exists()
    assert (tmp_path / "out" / "reference_draft.tsv").exists()
    assert (tmp_path / "out" / "truth_template.jsonl").exists()
    manifest = json.loads((tmp_path / "out" / "audio_shards.json").read_text())
    assert manifest["schema"] == "xiuxian_wendao.audio_shards.v1"
    assert manifest["audioMaterializationMode"] == "normalized-16k-wav"
    assert report["audioShardProfile"] == "audio-shards-v1"
    assert report["audioMaterializationMode"] == "normalized-16k-wav"
    assert report["inputPrivacy"] == "shareable"
    assert report["requestedBackends"] == ["local-docling", "openrouter-chat-audio"]
    assert report["openAiCompatibleAudioEnabled"] is True
    assert report["hostedAudioEnabled"] is True
    assert report["hostedAudioApiKeyConfigured"] is True
    assert report["truthTemplatePath"] == str(tmp_path / "out" / "truth_template.jsonl")
    assert report["referenceDraftPath"] == str(
        tmp_path / "out" / "reference_draft.jsonl"
    )
    assert report["precisionGatePassed"] is False
    assert report["precisionGateReason"] == "reference-not-configured"


def test_run_diagnostic_local_openai_audio_is_not_hosted(
    tmp_path: Path, monkeypatch
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    source_root = tmp_path / "audio"
    source_root.mkdir()
    source = source_root / "forum.MP3"
    source.write_bytes(b"mp3")
    chunk = tmp_path / "chunk.wav"
    chunk.write_bytes(b"wav")

    monkeypatch.setattr(
        audio_diagnostic_runner_pipeline,
        "materialize_audio_chunks",
        lambda *_args, **_kwargs: [
            diagnostic.AudioChunk(
                source=source,
                path=chunk,
                index=0,
                start_seconds=0.0,
                duration_seconds=15.0,
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
        audio_diagnostic_backends,
        "transcribe_openrouter",
        lambda *_args, **_kwargs: "本地",
    )
    monkeypatch.delenv("OPENROUTER_API_KEY", raising=False)

    args = argparse.Namespace(
        source_root=str(source_root),
        backend="local-openai-audio",
        output_dir=tmp_path / "out",
        input_privacy=diagnostic.SHAREABLE_INPUT_PRIVACY,
        allow_private_output_outside_cache=False,
        env_file=None,
        chunk_seconds=15,
        limit_files=1,
        limit_chunks=1,
        sample_strategy="head",
        start_offset_seconds=0.0,
        chunk_context_seconds=0.0,
        audio_materialization_mode=diagnostic.AUDIO_MATERIALIZATION_NORMALIZED_16K_WAV,
        sample_rate=16000,
        audio_format="wav",
        openrouter_model="wendao-local-audio",
        openrouter_base_url="http://127.0.0.1:8012/v1/chat/completions",
        local_asr_model=diagnostic.DEFAULT_LOCAL_ASR_MODEL,
        local_language=diagnostic.DEFAULT_LOCAL_LANGUAGE,
        fireredasr2s_command=diagnostic.DEFAULT_FIREREDASR2S_COMMAND,
        prompt=diagnostic.DEFAULT_PROMPT,
        domain_terms_file=None,
        required_terms_file=None,
        max_tokens=128,
        temperature=0.0,
        timeout_seconds=10,
        result_cache_dir=None,
        no_result_cache=False,
        reference_jsonl=None,
        truth_template_jsonl=None,
        max_reference_cer=0.15,
        min_required_term_recall=1.0,
        min_chars_per_minute=40.0,
        min_chinese_ratio=0.35,
        max_inaudible_per_minute=30.0,
        max_repeated_ngram_ratio=0.35,
        force=False,
    )

    report = diagnostic.run_diagnostic(args)

    assert report["requestedBackends"] == ["local-openai-audio"]
    assert report["openAiCompatibleAudioEnabled"] is True
    assert report["hostedAudioEnabled"] is False
    assert report["hostedAudioApiKeyConfigured"] is False
    assert report["openRouterApiKeyConfigured"] is False
