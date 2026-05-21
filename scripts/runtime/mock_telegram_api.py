#!/usr/bin/env python3
"""
Minimal Telegram Bot API mock server for local/CI webhook black-box tests.

Supported endpoints:
  - GET  /health
  - POST /bot<TOKEN>/sendMessage
  - POST /bot<TOKEN>/sendChatAction
  - POST /bot<TOKEN>/sendPhoto
  - POST /bot<TOKEN>/sendDocument
  - POST /bot<TOKEN>/sendVideo
  - POST /bot<TOKEN>/sendAudio
  - POST /bot<TOKEN>/sendVoice
  - POST /bot<TOKEN>/sendMediaGroup
"""

from __future__ import annotations

from http.server import ThreadingHTTPServer

from mock_telegram_api_config import parse_args
from mock_telegram_api_handler import (
    MESSAGE_ID_COUNTER,
    SUPPORTED_METHODS,
    TelegramMockHandler,
)


def main() -> int:
    try:
        config = parse_args()
    except ValueError as error:
        print(f"Error: {error}")
        return 2

    server = ThreadingHTTPServer((config.host, config.port), TelegramMockHandler)
    print(f"Telegram mock API listening on http://{config.host}:{config.port}")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
    return 0


__all__ = [
    "MESSAGE_ID_COUNTER",
    "SUPPORTED_METHODS",
    "TelegramMockHandler",
    "main",
    "parse_args",
]


if __name__ == "__main__":
    raise SystemExit(main())
