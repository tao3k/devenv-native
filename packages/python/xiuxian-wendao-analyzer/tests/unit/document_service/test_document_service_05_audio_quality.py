"""document_service audio quality gate tests."""

from __future__ import annotations

from xiuxian_wendao_analyzer.audio_shard_quality import AudioTranscriptQualityOptions

from .support import (
    HostedAudioConfig,
    HostedAudioShardWorker,
    _sample_audio_shard_input_table,
    build_audio_shard_result_table,
)


def _hosted_config(*, max_attempts: int = 1) -> HostedAudioConfig:
    return HostedAudioConfig(
        provider="openai-compatible",
        base_url="https://example.test/v1",
        model="audio-model",
        api_key="key",
        timeout_seconds=5.0,
        request_concurrency=1,
        primary_language="zh",
        max_attempts=max_attempts,
    )


def test_hosted_audio_worker_rejects_repetition_hallucination(tmp_path) -> None:
    shard = tmp_path / "chunk.wav"
    shard.write_bytes(b"abc")
    input_table = _sample_audio_shard_input_table(str(shard))
    row = input_table.to_pylist()[0]
    row["durationMs"] = 60_000
    input_table = input_table.from_pylist([row], schema=input_table.schema)
    repeated_text = "瑞士那个-" * 220

    worker = HostedAudioShardWorker(
        config=_hosted_config(),
        request_sender=lambda _config, _payload: {
            "choices": [{"message": {"content": repeated_text}}]
        },
    )

    result = build_audio_shard_result_table(input_table, worker=worker)

    output = result.to_pylist()[0]
    assert output["status"] == "failed"
    assert "audio transcript quality gate failed" in output["errorMessage"]
    assert "chars_per_minute" in output["errorMessage"]
    assert "repeated_ngram_ratio" in output["errorMessage"]


def test_hosted_audio_worker_rejects_refusal_text_for_chinese_audio(tmp_path) -> None:
    shard = tmp_path / "chunk.wav"
    shard.write_bytes(b"abc")
    input_table = _sample_audio_shard_input_table(str(shard))

    worker = HostedAudioShardWorker(
        config=_hosted_config(),
        request_sender=lambda _config, _payload: {
            "choices": [
                {
                    "message": {
                        "content": (
                            "I appreciate your request, but I don't see an audio file attached."
                        )
                    }
                }
            ]
        },
    )

    result = build_audio_shard_result_table(input_table, worker=worker)

    output = result.to_pylist()[0]
    assert output["status"] == "failed"
    assert "hosted_refusal_text" in output["errorMessage"]
    assert "latin_ratio_for_chinese" in output["errorMessage"]


def test_hosted_audio_worker_retries_quality_failure(tmp_path) -> None:
    shard = tmp_path / "chunk.wav"
    shard.write_bytes(b"abc")
    input_table = _sample_audio_shard_input_table(str(shard))
    calls = 0

    def request_sender(_config, _payload):
        nonlocal calls
        calls += 1
        if calls == 1:
            return {"choices": [{"message": {"content": "瑞士那个-" * 220}}]}
        return {"choices": [{"message": {"content": "今天讨论家居行业供应链"}}]}

    worker = HostedAudioShardWorker(
        config=_hosted_config(max_attempts=2),
        request_sender=request_sender,
    )

    result = build_audio_shard_result_table(input_table, worker=worker)

    output = result.to_pylist()[0]
    assert output["status"] == "succeeded"
    assert output["text"] == "今天讨论家居行业供应链"
    assert calls == 2


def test_hosted_audio_worker_rejects_no_transcribable_speech_text(tmp_path) -> None:
    shard = tmp_path / "chunk.wav"
    shard.write_bytes(b"abc")
    input_table = _sample_audio_shard_input_table(str(shard))

    worker = HostedAudioShardWorker(
        config=_hosted_config(),
        request_sender=lambda _config, _payload: {
            "choices": [{"message": {"content": "该音频中没有可转录的语音内容。"}}]
        },
    )

    result = build_audio_shard_result_table(input_table, worker=worker)

    output = result.to_pylist()[0]
    assert output["status"] == "failed"
    assert "no_transcribable_speech_text" in output["errorMessage"]


def test_hosted_audio_quality_gate_can_be_disabled(tmp_path) -> None:
    shard = tmp_path / "chunk.wav"
    shard.write_bytes(b"abc")
    input_table = _sample_audio_shard_input_table(str(shard))
    repeated_text = "瑞士那个-" * 220

    worker = HostedAudioShardWorker(
        config=HostedAudioConfig(
            provider="openai-compatible",
            base_url="https://example.test/v1",
            model="audio-model",
            api_key="key",
            timeout_seconds=5.0,
            request_concurrency=1,
            primary_language="zh",
            quality_options=AudioTranscriptQualityOptions(enabled=False),
        ),
        request_sender=lambda _config, _payload: {
            "choices": [{"message": {"content": repeated_text}}]
        },
    )

    result = build_audio_shard_result_table(input_table, worker=worker)

    output = result.to_pylist()[0]
    assert output["status"] == "succeeded"
    assert output["text"] == repeated_text
