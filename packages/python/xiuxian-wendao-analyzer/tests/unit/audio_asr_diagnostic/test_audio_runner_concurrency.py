"""Audio diagnostic hosted request concurrency tests."""

from __future__ import annotations

import argparse
import time

from xiuxian_wendao_analyzer import (
    audio_diagnostic_backends,
    audio_diagnostic_runner_pipeline,
)

from .support import Path, _load_audio_asr_diagnostic


def test_hosted_audio_request_concurrency_preserves_result_order(
    tmp_path: Path, monkeypatch
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    source = tmp_path / "forum.MP3"
    source.write_bytes(b"mp3")
    chunks = []
    for index in range(2):
        chunk_path = tmp_path / f"chunk-{index}.wav"
        chunk_path.write_bytes(b"wav")
        chunks.append(
            diagnostic.AudioChunk(
                source=source,
                path=chunk_path,
                index=index,
                start_seconds=float(index * 10),
                duration_seconds=10.0,
                format="wav",
                shard_id=f"shard-{index}",
                cache_key=f"audio-shards-v1:shard-{index}",
                source_sha256="a" * 64,
                sample_rate_hz=16000,
                channels=1,
            )
        )

    def transcribe_with_out_of_order_completion(chunk, **_kwargs):
        if chunk.index == 0:
            time.sleep(0.02)
        return f"云端{chunk.index}"

    monkeypatch.setattr(
        audio_diagnostic_backends,
        "transcribe_openrouter",
        transcribe_with_out_of_order_completion,
    )
    args = argparse.Namespace(
        hosted_request_concurrency=2,
        openrouter_model="xiaomi/mimo-v2.5",
        openrouter_base_url=diagnostic.DEFAULT_OPENROUTER_URL,
        local_asr_model=diagnostic.DEFAULT_LOCAL_ASR_MODEL,
        local_language=diagnostic.DEFAULT_LOCAL_LANGUAGE,
        fireredasr2s_command=diagnostic.DEFAULT_FIREREDASR2S_COMMAND,
        max_tokens=128,
        temperature=0.0,
        timeout_seconds=10,
    )

    results = audio_diagnostic_runner_pipeline.run_diagnostic_backends(
        args,
        chunks=chunks,
        backends=["openrouter-chat-audio"],
        output_dir=tmp_path / "out",
        api_key="test-key",
        prompt=diagnostic.DEFAULT_PROMPT,
        result_cache_dir=None,
    )

    assert [row.chunk_index for row in results] == [0, 1]
    assert [
        Path(row.transcript_path).read_text(encoding="utf-8") for row in results
    ] == [
        "云端0",
        "云端1",
    ]
