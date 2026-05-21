from __future__ import annotations

import importlib.util
import sys
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import pytest

_MODULE_PATH = Path(__file__).resolve().with_name("realtime_probe_api.py")
_SPEC = importlib.util.spec_from_file_location("realtime_probe_api", _MODULE_PATH)
assert _SPEC and _SPEC.loader
_MODULE = importlib.util.module_from_spec(_SPEC)
sys.modules[_SPEC.name] = _MODULE
_SPEC.loader.exec_module(_MODULE)


def test_resolve_telegram_identity_from_env(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("OMNI_TEST_CHAT_ID", "-5101776367")
    monkeypatch.setenv("OMNI_TEST_USER_ID", "1304799691")

    assert _MODULE.resolve_telegram_probe_chat_id(None) == -5101776367
    assert _MODULE.resolve_telegram_probe_user_id(None) == 1304799691


def test_run_telegram_realtime_probe_success(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    log_file = tmp_path / "telegram.log"
    log_file.write_text("", encoding="utf-8")
    fixed_time = 1234.567
    update_id = int(fixed_time * 1_000_000)
    monkeypatch.setattr(_MODULE.time, "time", lambda: fixed_time)

    chunks = [
        [
            f"Webhook received Telegram update update_id=Some({update_id})",
            "2026-03-01T00:00:01Z INFO → Bot: probe reply",
        ]
    ]

    def _read_new_lines(_path: Path, cursor: int) -> tuple[int, list[str]]:
        if chunks:
            return cursor + 1, chunks.pop(0)
        return cursor, []

    cfg = _MODULE.TelegramRealtimeProbeConfig(
        webhook_url="http://127.0.0.1:18081/telegram/webhook",
        log_file=log_file,
        chat_id=-5101776367,
        user_id=1304799691,
        prompt="probe",
        max_wait_secs=2.0,
        max_idle_secs=0.2,
    )

    result = _MODULE.run_telegram_realtime_probe(
        cfg,
        count_lines_fn=lambda _path: 0,
        read_new_lines_fn=_read_new_lines,
        post_webhook_fn=lambda _url, _payload, _secret: (200, '{"ok":true}'),
        sleep_fn=lambda _secs: None,
        monotonic_fn=lambda: 0.0,
    )

    assert result.ok
    assert result.inbound_seen
    assert result.bot_seen
    assert result.post_status == 200


def test_run_discord_realtime_probe_success(tmp_path: Path) -> None:
    log_file = tmp_path / "discord.log"
    log_file.write_text("", encoding="utf-8")
    chunks = [
        [
            "discord ingress parsed message",
            'event="discord.command.control_admin_required.replied" recipient="discord:1:2:3"',
        ]
    ]

    def _read_new_lines(_path: Path, cursor: int) -> tuple[int, list[str]]:
        if chunks:
            return cursor + 1, chunks.pop(0)
        return cursor, []

    cfg = _MODULE.DiscordRealtimeProbeConfig(
        ingress_url="http://127.0.0.1:18082/discord/ingress",
        log_file=log_file,
        channel_id="2001",
        user_id="1001",
        prompt="probe",
        expected_events=("discord.command.control_admin_required.replied",),
        max_wait_secs=2.0,
        max_idle_secs=0.2,
    )

    result = _MODULE.run_discord_realtime_probe(
        cfg,
        count_lines_fn=lambda _path: 0,
        read_new_lines_fn=_read_new_lines,
        post_ingress_fn=lambda _url, _payload, _secret, _timeout: (
            200,
            '{"ok":true}',
            3.2,
        ),
        sleep_fn=lambda _secs: None,
        monotonic_fn=lambda: 0.0,
    )

    assert result.ok
    assert result.inbound_seen
    assert result.reply_seen
    assert result.missing_expected_events == ()
