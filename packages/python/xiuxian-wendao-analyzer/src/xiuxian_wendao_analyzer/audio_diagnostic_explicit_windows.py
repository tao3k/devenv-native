"""Explicit audio window plan loading for targeted ASR reruns."""

from __future__ import annotations

import json
from collections.abc import Mapping, Sequence
from pathlib import Path

from xiuxian_wendao_analyzer.audio_diagnostic_identity import SpeechSegment


def load_explicit_windows(path: Path | None, *, source: Path) -> list[SpeechSegment]:
    """Load explicit ASR windows from a JSON risk or review plan."""

    if path is None:
        return []
    payload = json.loads(path.read_text(encoding="utf-8"))
    rows = _payload_rows(payload)
    windows: list[SpeechSegment] = []
    for row_index, row in enumerate(rows):
        row_source = _json_string(row, "source", "sourceId", "sourcePath")
        if not _source_matches(row_source, source):
            continue
        start_seconds = _json_seconds(row, "startSeconds", "start_seconds", "startMs")
        duration_seconds = _json_duration_seconds(row)
        if start_seconds is None:
            raise ValueError(f"explicit window row {row_index} is missing start")
        if duration_seconds is None or duration_seconds <= 0:
            raise ValueError(
                f"explicit window row {row_index} has non-positive duration"
            )
        windows.append(
            SpeechSegment(
                source=row_source,
                index=int(
                    _json_number(row, "chunkIndex", "index", "windowIndex") or row_index
                ),
                start_seconds=start_seconds,
                duration_seconds=duration_seconds,
                confidence=None,
                label=_json_label(row),
            )
        )
    return sorted(windows, key=lambda window: (window.start_seconds, window.index))


def _payload_rows(payload: object) -> Sequence[Mapping[str, object]]:
    if isinstance(payload, list):
        rows = payload
    elif isinstance(payload, Mapping):
        rows = payload.get("rows", [])
    else:
        raise ValueError("explicit windows JSON must be an object or array")
    if not isinstance(rows, Sequence) or isinstance(rows, str | bytes):
        raise ValueError("explicit windows JSON rows must be an array")
    typed_rows: list[Mapping[str, object]] = []
    for index, row in enumerate(rows):
        if not isinstance(row, Mapping):
            raise ValueError(f"explicit window row {index} must be an object")
        typed_rows.append(row)
    return typed_rows


def _json_number(row: Mapping[str, object], *names: str) -> float | None:
    for name in names:
        value = row.get(name)
        if isinstance(value, int | float) and not isinstance(value, bool):
            return float(value)
    return None


def _json_seconds(row: Mapping[str, object], *names: str) -> float | None:
    for name in names:
        value = _json_number(row, name)
        if value is None:
            continue
        return value / 1000.0 if name.endswith("Ms") else value
    return None


def _json_duration_seconds(row: Mapping[str, object]) -> float | None:
    duration_seconds = _json_seconds(
        row,
        "durationSeconds",
        "duration_seconds",
        "durationMs",
    )
    if duration_seconds is not None:
        return duration_seconds
    start_seconds = _json_seconds(row, "startSeconds", "start_seconds", "startMs")
    end_seconds = _json_seconds(row, "endSeconds", "end_seconds", "endMs")
    if start_seconds is None or end_seconds is None:
        return None
    return end_seconds - start_seconds


def _json_string(row: Mapping[str, object], *names: str) -> str:
    for name in names:
        value = row.get(name)
        if isinstance(value, str):
            return value
    return ""


def _json_label(row: Mapping[str, object]) -> str:
    reasons = row.get("reasons")
    if isinstance(reasons, list):
        return ",".join(str(item) for item in reasons if str(item))
    return _json_string(row, "label", "reason", "kind")


def _source_matches(row_source: str, source: Path) -> bool:
    if not row_source:
        return True
    return row_source == str(source) or Path(row_source).name == source.name
