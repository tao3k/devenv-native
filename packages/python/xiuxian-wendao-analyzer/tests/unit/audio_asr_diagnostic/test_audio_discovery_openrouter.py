"""Audio diagnostic tests."""

from __future__ import annotations

import json

from .support import Path, _load_audio_asr_diagnostic


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
        diagnostic.resolve_openrouter_api_key({"OPENROUTE_API_KEY": "wrong"}, env_file=None) is None
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
    assert payload["reasoning"] == {"effort": "none"}


def test_build_openrouter_transcription_payload_uses_stt_shape() -> None:
    diagnostic = _load_audio_asr_diagnostic()

    payload = diagnostic.build_openrouter_transcription_payload(
        model="qwen/qwen3-asr-flash-2026-02-10",
        audio_bytes=b"audio",
        audio_format="mp3",
    )

    assert payload == {
        "model": "qwen/qwen3-asr-flash-2026-02-10",
        "input_audio": {
            "data": "YXVkaW8=",
            "format": "mp3",
        },
    }
    assert diagnostic.is_openrouter_transcription_url(
        "https://openrouter.ai/api/v1/audio/transcriptions"
    )
    assert not diagnostic.is_openrouter_transcription_url(
        "https://openrouter.ai/api/v1/chat/completions"
    )


def test_extract_openrouter_segments_from_structured_responses() -> None:
    diagnostic = _load_audio_asr_diagnostic()

    direct = {
        "text": "第一段 第二段",
        "segments": [
            {"start": 0.0, "end": 1.2, "text": "第一段"},
            {"startMs": 1200, "durationMs": 800, "text": "第二段"},
        ],
    }
    chat_json = {
        "choices": [
            {
                "message": {
                    "content": json.dumps(
                        {
                            "transcript": "第三段",
                            "segments": [
                                {
                                    "startSeconds": "2.0",
                                    "endSeconds": "2.7",
                                    "text": "第三段",
                                }
                            ],
                        },
                    )
                }
            }
        ]
    }

    assert diagnostic.extract_openrouter_transcript(direct) == "第一段 第二段"
    assert diagnostic.extract_openrouter_segments(direct) == [
        {"startSeconds": 0.0, "endSeconds": 1.2, "text": "第一段"},
        {"startSeconds": 1.2, "endSeconds": 2.0, "text": "第二段"},
    ]
    assert diagnostic.extract_openrouter_transcript(chat_json) == "第三段"
    assert diagnostic.extract_openrouter_segments(chat_json) == [
        {"startSeconds": 2.0, "endSeconds": 2.7, "text": "第三段"}
    ]
