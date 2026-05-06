"""OpenAI-compatible HTTP transport for OCR2 requests."""

from __future__ import annotations

import json
import time
import urllib.error
import urllib.request
from collections.abc import Mapping
from typing import Any

_DEEPSEEK_OCR2_TRANSIENT_HTTP_STATUS = {408, 409, 425, 429, 500, 502, 503, 504}
_DEEPSEEK_OCR2_MAX_TRANSIENT_RETRIES = 3
_DEEPSEEK_OCR2_RETRY_BASE_SECONDS = 0.25
_DEEPSEEK_OCR2_RATE_LIMIT_RETRY_BASE_SECONDS = 2.0
_DEEPSEEK_OCR2_MAX_RETRY_DELAY_SECONDS = 8.0


def send_completion_request(
    *,
    completion_url: str,
    headers: Mapping[str, str],
    timeout_seconds: float,
    payload: Mapping[str, Any],
) -> tuple[int | None, Any]:
    request_data = json.dumps(payload).encode("utf-8")
    for attempt in range(_DEEPSEEK_OCR2_MAX_TRANSIENT_RETRIES + 1):
        request = urllib.request.Request(
            completion_url,
            data=request_data,
            headers=dict(headers),
            method="POST",
        )
        try:
            with urllib.request.urlopen(request, timeout=timeout_seconds) as response:
                http_status = response_http_status(response)
                response_payload = json.loads(response.read().decode("utf-8"))
            return http_status, response_payload
        except urllib.error.HTTPError as exc:
            if not should_retry_ocr2_http_error(exc, attempt):
                raise
            sleep(ocr2_retry_delay_seconds(attempt, exc))
        except (OSError, urllib.error.URLError):
            if attempt >= _DEEPSEEK_OCR2_MAX_TRANSIENT_RETRIES:
                raise
            sleep(ocr2_retry_delay_seconds(attempt, None))
    raise RuntimeError("unreachable OCR2 retry loop")


def sleep(seconds: float) -> None:
    time.sleep(seconds)


def chat_completion_url(base_url: str) -> str:
    normalized = base_url.rstrip("/")
    if normalized.endswith("/chat/completions"):
        return normalized
    return f"{normalized}/chat/completions"


def response_http_status(response: object) -> int | None:
    status = getattr(response, "status", None)
    if isinstance(status, int):
        return status
    code = getattr(response, "code", None)
    if isinstance(code, int):
        return code
    return None


def should_retry_ocr2_http_error(error: urllib.error.HTTPError, attempt: int) -> bool:
    return (
        error.code in _DEEPSEEK_OCR2_TRANSIENT_HTTP_STATUS
        and attempt < _DEEPSEEK_OCR2_MAX_TRANSIENT_RETRIES
    )


def is_transient_ocr2_failure(result: Mapping[str, Any]) -> bool:
    if result.get("status") != "failed":
        return False
    error_message = str(result.get("errorMessage") or "")
    return any(
        f"HTTP Error {status}" in error_message
        for status in _DEEPSEEK_OCR2_TRANSIENT_HTTP_STATUS
    )


def ocr2_retry_delay_seconds(
    attempt: int,
    error: urllib.error.HTTPError | None,
) -> float:
    retry_after = ocr2_retry_after_seconds(error)
    if retry_after is not None:
        return retry_after
    base_seconds = (
        _DEEPSEEK_OCR2_RATE_LIMIT_RETRY_BASE_SECONDS
        if error is not None and error.code == 429
        else _DEEPSEEK_OCR2_RETRY_BASE_SECONDS
    )
    return min(
        base_seconds * (2**attempt),
        _DEEPSEEK_OCR2_MAX_RETRY_DELAY_SECONDS,
    )


def ocr2_retry_after_seconds(error: urllib.error.HTTPError | None) -> float | None:
    if error is None:
        return None
    headers = getattr(error, "headers", None)
    if headers is None:
        return None
    value = headers.get("Retry-After")
    if value is None:
        return None
    try:
        seconds = float(value)
    except ValueError:
        return None
    return min(max(seconds, 0.0), _DEEPSEEK_OCR2_MAX_RETRY_DELAY_SECONDS)


def short_error_message(error: BaseException | None) -> str | None:
    if error is None:
        return None
    message = str(error)
    if len(message) <= 240:
        return message
    return f"{message[:237]}..."


def extract_openai_message_content(payload: Mapping[str, Any]) -> str:
    choices = payload.get("choices")
    if not isinstance(choices, list) or not choices:
        raise ValueError("OpenAI-compatible response does not contain choices")
    first_choice = choices[0]
    if not isinstance(first_choice, Mapping):
        raise ValueError("OpenAI-compatible response choice is not an object")
    message = first_choice.get("message")
    if not isinstance(message, Mapping):
        raise ValueError("OpenAI-compatible response choice does not contain message")
    content = message.get("content")
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        parts = []
        for part in content:
            if isinstance(part, Mapping) and isinstance(part.get("text"), str):
                parts.append(part["text"])
        if parts:
            return "".join(parts)
    raise ValueError("OpenAI-compatible response message content is not text")
