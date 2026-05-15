"""Qwen3-ASR MLX audio adapter protocol tests."""

from __future__ import annotations

import base64

from xiuxian_wendao_analyzer.audio_backend import (
    qwen3_asr_mlx_openai_adapter as adapter,
)


def test_complete_chat_audio_accepts_hosted_worker_input_audio_shape(monkeypatch):
    calls: list[tuple[str, str, str]] = []

    def fake_transcribe(
        audio_path,
        *,
        model_path: str,
        context: str = "",
    ) -> tuple[str, list[dict[str, object]]]:
        calls.append((audio_path.read_bytes().decode("utf-8"), model_path, context))
        return "本地中文转写", [
            {"startSeconds": 0.0, "endSeconds": 1.2, "text": "本地中文转写"}
        ]

    monkeypatch.setenv("WENDAO_AUDIO_LOCAL_MODEL_PATH", "qwen3-test-model")
    monkeypatch.setenv("WENDAO_AUDIO_LOCAL_MODEL", "wendao-qwen3-test")
    monkeypatch.setattr(adapter, "_transcribe_audio", fake_transcribe)

    response = adapter.complete_chat_audio(
        [
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": "保留中文术语"},
                    {
                        "type": "input_audio",
                        "input_audio": {
                            "data": base64.b64encode(b"audio-bytes").decode("ascii"),
                            "format": "wav",
                        },
                    },
                ],
            }
        ],
        requested_model=None,
    )

    assert calls == [("audio-bytes", "qwen3-test-model", "保留中文术语")]
    assert response["object"] == "chat.completion"
    assert response["model"] == "wendao-qwen3-test"
    assert response["choices"][0]["message"] == {
        "role": "assistant",
        "content": "本地中文转写",
    }
    assert response["segments"] == [
        {"startSeconds": 0.0, "endSeconds": 1.2, "text": "本地中文转写"}
    ]


def test_transcribe_audio_calls_mlx_qwen3_asr(monkeypatch, tmp_path):
    calls: list[dict[str, object]] = []
    audio_path = tmp_path / "chunk.wav"
    audio_path.write_bytes(b"wav")

    class FakeResult:
        def __init__(self) -> None:
            self.text = "中文转写"
            self.chunks = [{"timestamp": [0.0, 1.25], "text": "中文"}]

    class FakeQwen3Asr:
        @staticmethod
        def transcribe(_audio_path, **kwargs):
            calls.append(kwargs)
            return FakeResult()

    monkeypatch.setitem(__import__("sys").modules, "mlx_qwen3_asr", FakeQwen3Asr)
    monkeypatch.setenv("WENDAO_AUDIO_QWEN3_MAX_NEW_TOKENS", "256")

    text, segments = adapter._transcribe_audio(
        audio_path,
        model_path="Qwen/Qwen3-ASR-1.7B",
        context="居家行业论坛",
    )

    assert text == "中文转写"
    assert segments == [{"startSeconds": 0.0, "endSeconds": 1.25, "text": "中文"}]
    assert calls[0]["model"] == "Qwen/Qwen3-ASR-1.7B"
    assert calls[0]["context"] == "居家行业论坛"
    assert calls[0]["language"] == "zh"
    assert calls[0]["return_timestamps"] is False
    assert calls[0]["return_chunks"] is False
    assert calls[0]["max_new_tokens"] == 256


def test_transcribe_audio_can_request_qwen_timestamp_chunks(monkeypatch, tmp_path):
    calls: list[dict[str, object]] = []
    audio_path = tmp_path / "chunk.wav"
    audio_path.write_bytes(b"wav")

    class FakeResult:
        def __init__(self) -> None:
            self.text = "中文转写"
            self.chunks = [{"start": 0, "end": 1.5, "text": "中文转写"}]

    class FakeQwen3Asr:
        @staticmethod
        def transcribe(_audio_path, **kwargs):
            calls.append(kwargs)
            return FakeResult()

    monkeypatch.setitem(__import__("sys").modules, "mlx_qwen3_asr", FakeQwen3Asr)
    monkeypatch.setenv("WENDAO_AUDIO_QWEN3_RETURN_TIMESTAMPS", "1")
    monkeypatch.setenv("WENDAO_AUDIO_QWEN3_RETURN_CHUNKS", "1")

    text, segments = adapter._transcribe_audio(
        audio_path,
        model_path="Qwen/Qwen3-ASR-1.7B",
    )

    assert text == "中文转写"
    assert segments == [{"startSeconds": 0.0, "endSeconds": 1.5, "text": "中文转写"}]
    assert calls[0]["return_timestamps"] is True
    assert calls[0]["return_chunks"] is True


def test_complete_chat_audio_rejects_missing_audio() -> None:
    try:
        adapter.complete_chat_audio([{"role": "user", "content": []}])
    except adapter.AudioAdapterRequestError as exc:
        assert exc.detail == "missing input_audio content"
    else:
        raise AssertionError("missing audio should fail")
