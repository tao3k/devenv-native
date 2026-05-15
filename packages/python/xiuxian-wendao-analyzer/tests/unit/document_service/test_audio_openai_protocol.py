"""OpenAI-compatible audio protocol helper tests."""

from __future__ import annotations

import base64

from xiuxian_wendao_analyzer.audio_openai_protocol import (
    AUDIO_OPENAI_DEFAULT_PROMPT,
    build_chat_audio_payload,
    extract_input_audio,
    extract_openai_message_content,
    extract_text_prompt,
)


def test_build_chat_audio_payload_round_trips_input_audio(tmp_path) -> None:
    audio_path = tmp_path / "chunk.wav"
    audio_path.write_bytes(b"abc")

    payload = build_chat_audio_payload(
        model="audio-model",
        audio_path=audio_path,
        audio_format="WAV",
    )

    assert payload["model"] == "audio-model"
    assert payload["stream"] is False
    content = payload["messages"][0]["content"]
    assert content[0] == {"type": "text", "text": AUDIO_OPENAI_DEFAULT_PROMPT}
    assert content[1]["type"] == "input_audio"
    assert content[1]["input_audio"]["format"] == "wav"
    assert content[1]["input_audio"]["data"] == base64.b64encode(b"abc").decode("ascii")

    decoded = extract_input_audio(payload["messages"])
    assert decoded.data == b"abc"
    assert decoded.format == "wav"


def test_extract_openai_message_content_accepts_text_parts() -> None:
    assert (
        extract_openai_message_content(
            {
                "choices": [
                    {
                        "message": {
                            "content": [
                                {"type": "text", "text": "hello"},
                                {"type": "text", "text": " world"},
                            ]
                        }
                    }
                ]
            }
        )
        == "hello world"
    )


def test_extract_text_prompt_collects_text_parts() -> None:
    prompt = extract_text_prompt(
        [
            {"role": "system", "content": "ignore"},
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": "保留术语"},
                    {
                        "type": "input_audio",
                        "input_audio": {"data": "abcd", "format": "wav"},
                    },
                    {"type": "text", "text": "不要总结"},
                ],
            },
        ]
    )

    assert prompt == "ignore\n保留术语\n不要总结"


def test_extract_input_audio_rejects_invalid_base64() -> None:
    try:
        extract_input_audio(
            [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "input_audio",
                            "input_audio": {"data": "%%%", "format": "wav"},
                        }
                    ],
                }
            ]
        )
    except ValueError as exc:
        assert str(exc) == "invalid input_audio data"
    else:
        raise AssertionError("invalid audio data should fail")
