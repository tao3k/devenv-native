"""Audio diagnostic tests."""

from __future__ import annotations

import json
import subprocess

from xiuxian_wendao_analyzer import audio_diagnostic_materialization

from .support import Path, _load_audio_asr_diagnostic


def test_load_speech_segments_accepts_seconds_and_ms_rows(tmp_path: Path) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    source = tmp_path / "forum.mp3"
    source.write_bytes(b"mp3")
    sidecar = tmp_path / "segments.jsonl"
    sidecar.write_text(
        "\n".join(
            [
                json.dumps(
                    {
                        "source": "other.mp3",
                        "startSeconds": 0.0,
                        "durationSeconds": 1.0,
                    }
                ),
                json.dumps(
                    {
                        "source": "forum.mp3",
                        "startSeconds": 10.0,
                        "endSeconds": 13.5,
                        "confidence": 0.91,
                    }
                ),
                json.dumps(
                    {
                        "sourceId": str(source),
                        "startMs": 15000,
                        "durationMs": 2500,
                        "label": "speech",
                    }
                ),
            ]
        )
        + "\n",
        encoding="utf-8",
    )

    segments = diagnostic.load_speech_segments(sidecar, source=source)

    assert [(row.start_seconds, row.duration_seconds) for row in segments] == [
        (10.0, 3.5),
        (15.0, 2.5),
    ]
    assert segments[0].confidence == 0.91
    assert segments[1].label == "speech"


def test_speech_segment_sampling_uses_variable_windows(tmp_path: Path) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    segments = [
        diagnostic.SpeechSegment(
            source="forum.mp3",
            index=0,
            start_seconds=4.0,
            duration_seconds=2.5,
        ),
        diagnostic.SpeechSegment(
            source="forum.mp3",
            index=1,
            start_seconds=9.0,
            duration_seconds=3.0,
        ),
    ]

    windows = diagnostic.chunk_windows(
        duration_seconds=30.0,
        chunk_seconds=10,
        limit_chunks=1,
        sample_strategy="speech-segments",
        start_offset_seconds=0.0,
        speech_segments=segments,
    )

    assert windows == [(4.0, 2.5)]


def test_speech_segment_sampling_packs_short_windows_without_long_context() -> None:
    diagnostic = _load_audio_asr_diagnostic()
    segments = [
        diagnostic.SpeechSegment(
            source="forum.mp3",
            index=0,
            start_seconds=0.0,
            duration_seconds=8.0,
        ),
        diagnostic.SpeechSegment(
            source="forum.mp3",
            index=1,
            start_seconds=8.5,
            duration_seconds=10.0,
        ),
        diagnostic.SpeechSegment(
            source="forum.mp3",
            index=2,
            start_seconds=19.0,
            duration_seconds=20.0,
        ),
    ]

    windows = diagnostic.chunk_windows(
        duration_seconds=60.0,
        chunk_seconds=60,
        limit_chunks=3,
        sample_strategy="speech-segments",
        start_offset_seconds=0.0,
        speech_segments=segments,
        speech_segment_merge_gap_seconds=1.0,
        speech_segment_max_window_seconds=30.0,
    )

    assert windows == [(0.0, 18.5), (19.0, 20.0)]


def test_speech_segment_sampling_splits_long_vad_rows() -> None:
    diagnostic = _load_audio_asr_diagnostic()
    segments = [
        diagnostic.SpeechSegment(
            source="forum.mp3",
            index=0,
            start_seconds=0.0,
            duration_seconds=75.0,
        )
    ]

    windows = diagnostic.chunk_windows(
        duration_seconds=90.0,
        chunk_seconds=60,
        limit_chunks=4,
        sample_strategy="speech-segments",
        start_offset_seconds=0.0,
        speech_segments=segments,
        speech_segment_merge_gap_seconds=1.0,
        speech_segment_max_window_seconds=30.0,
    )

    assert windows == [(0.0, 30.0), (30.0, 30.0), (60.0, 15.0)]


def test_speech_segment_sampling_packs_tiny_isolated_windows() -> None:
    diagnostic = _load_audio_asr_diagnostic()
    segments = [
        diagnostic.SpeechSegment(
            source="forum.mp3",
            index=0,
            start_seconds=0.0,
            duration_seconds=4.0,
        ),
        diagnostic.SpeechSegment(
            source="forum.mp3",
            index=1,
            start_seconds=9.0,
            duration_seconds=3.0,
        ),
        diagnostic.SpeechSegment(
            source="forum.mp3",
            index=2,
            start_seconds=25.0,
            duration_seconds=3.0,
        ),
    ]

    windows = diagnostic.chunk_windows(
        duration_seconds=60.0,
        chunk_seconds=60,
        limit_chunks=4,
        sample_strategy="speech-segments",
        start_offset_seconds=0.0,
        speech_segments=segments,
        speech_segment_merge_gap_seconds=1.0,
        speech_segment_min_window_seconds=8.0,
        speech_segment_max_window_seconds=30.0,
    )

    assert windows == [(0.0, 12.0), (25.0, 3.0)]


def test_speech_segment_short_merge_gap_limits_silence_expansion() -> None:
    diagnostic = _load_audio_asr_diagnostic()
    segments = [
        diagnostic.SpeechSegment(
            source="forum.mp3",
            index=0,
            start_seconds=0.0,
            duration_seconds=4.0,
        ),
        diagnostic.SpeechSegment(
            source="forum.mp3",
            index=1,
            start_seconds=9.0,
            duration_seconds=3.0,
        ),
        diagnostic.SpeechSegment(
            source="forum.mp3",
            index=2,
            start_seconds=14.0,
            duration_seconds=3.0,
        ),
    ]

    windows = diagnostic.chunk_windows(
        duration_seconds=60.0,
        chunk_seconds=60,
        limit_chunks=4,
        sample_strategy="speech-segments",
        start_offset_seconds=0.0,
        speech_segments=segments,
        speech_segment_merge_gap_seconds=1.0,
        speech_segment_min_window_seconds=8.0,
        speech_segment_short_merge_gap_seconds=3.0,
        speech_segment_max_window_seconds=30.0,
    )

    assert windows == [(0.0, 4.0), (9.0, 8.0)]


def test_speech_window_plan_report_compares_minimum_candidates() -> None:
    diagnostic = _load_audio_asr_diagnostic()
    segments = [
        diagnostic.SpeechSegment(
            source="forum.mp3",
            index=0,
            start_seconds=0.0,
            duration_seconds=4.0,
        ),
        diagnostic.SpeechSegment(
            source="forum.mp3",
            index=1,
            start_seconds=9.0,
            duration_seconds=3.0,
        ),
        diagnostic.SpeechSegment(
            source="forum.mp3",
            index=2,
            start_seconds=25.0,
            duration_seconds=10.0,
        ),
    ]

    report = diagnostic.build_speech_window_plan_report(
        speech_segments=segments,
        duration_seconds=60.0,
        chunk_seconds=60,
        limit_chunks=10,
        merge_gap_seconds=1.0,
        max_window_seconds=30.0,
        min_window_candidates=diagnostic.parse_window_min_candidates("0,8"),
    )

    assert report["schema"] == "xiuxian_wendao.audio_speech_window_plan.v1"
    assert report["rawSpeechSegmentCount"] == 3
    assert report["rawSpeechDurationSeconds"] == 17.0
    assert report["shortMergeGapSeconds"] is None
    assert report["candidates"][0]["minWindowSeconds"] == 0.0
    assert report["candidates"][0]["chunks"] == 3
    assert report["candidates"][1]["minWindowSeconds"] == 8.0
    assert report["candidates"][1]["chunks"] == 2
    assert report["candidates"][1]["coverageExpansionSeconds"] == 5.0


def test_speech_window_plan_report_accepts_short_merge_gap() -> None:
    diagnostic = _load_audio_asr_diagnostic()
    segments = [
        diagnostic.SpeechSegment(
            source="forum.mp3",
            index=0,
            start_seconds=0.0,
            duration_seconds=4.0,
        ),
        diagnostic.SpeechSegment(
            source="forum.mp3",
            index=1,
            start_seconds=9.0,
            duration_seconds=3.0,
        ),
    ]

    report = diagnostic.build_speech_window_plan_report(
        speech_segments=segments,
        duration_seconds=60.0,
        chunk_seconds=60,
        limit_chunks=10,
        merge_gap_seconds=1.0,
        max_window_seconds=30.0,
        min_window_candidates=[8.0],
        short_merge_gap_seconds=3.0,
    )

    assert report["shortMergeGapSeconds"] == 3.0
    assert report["candidates"][0]["chunks"] == 2
    assert report["candidates"][0]["coverageExpansionSeconds"] == 0.0


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

    monkeypatch.setattr(audio_diagnostic_materialization.subprocess, "run", fake_run)

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

    monkeypatch.setattr(audio_diagnostic_materialization.subprocess, "run", fake_run)

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


def test_materialize_audio_chunks_can_preserve_native_sample_rate(
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

    monkeypatch.setattr(audio_diagnostic_materialization.subprocess, "run", fake_run)

    chunks = diagnostic.materialize_audio_chunks(
        source,
        chunk_dir=tmp_path / "chunks",
        chunk_seconds=30,
        limit_chunks=1,
        sample_rate=16000,
        audio_format="wav",
        audio_materialization_mode=diagnostic.AUDIO_MATERIALIZATION_NATIVE_RATE_WAV,
        source_sample_rate_hz=44100,
        ffmpeg_path="/fake/ffmpeg",
    )

    assert calls[0][calls[0].index("-ar") + 1] == "44100"
    assert chunks[0].sample_rate_hz == 44100
    assert chunks[0].format == "wav"


def test_materialize_audio_chunks_source_direct_uses_original_media(
    tmp_path: Path,
) -> None:
    diagnostic = _load_audio_asr_diagnostic()
    source = tmp_path / "forum.MP3"
    source.write_bytes(b"mp3")

    chunks = diagnostic.materialize_audio_chunks(
        source,
        chunk_dir=tmp_path / "chunks",
        chunk_seconds=30,
        limit_chunks=1,
        sample_rate=16000,
        audio_format="wav",
        audio_materialization_mode=diagnostic.AUDIO_MATERIALIZATION_SOURCE_DIRECT,
        source_duration_seconds=123.0,
        source_sample_rate_hz=44100,
        source_channels=2,
        ffmpeg_path="/fake/ffmpeg",
    )

    assert chunks[0].path == source
    assert chunks[0].format == "mp3"
    assert chunks[0].duration_seconds == 123.0
    assert chunks[0].sample_rate_hz == 44100
    assert chunks[0].channels == 2


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
        audio_materialization_mode=diagnostic.AUDIO_MATERIALIZATION_NATIVE_RATE_WAV,
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
    assert manifest["audioMaterializationMode"] == "native-rate-wav"
    assert manifest["items"][0]["startMs"] == 1500
    assert manifest["items"][0]["mediaDurationMs"] == 31000
    assert manifest["items"][0]["contextAfterMs"] == 1000
    assert manifest["items"][0]["cacheKey"].startswith("audio-shards-v1:")
