#!/usr/bin/env python3
"""Scalar parsing helpers for channel config resolution."""

from __future__ import annotations


def strip_inline_comment(value: str) -> str:
    """Strip trailing inline config/env comments while honoring quote context."""
    in_single = False
    in_double = False
    out: list[str] = []
    for char in value:
        if char == "'" and not in_double:
            in_single = not in_single
            out.append(char)
            continue
        if char == '"' and not in_single:
            in_double = not in_double
            out.append(char)
            continue
        if char == "#" and not in_single and not in_double:
            break
        out.append(char)
    return "".join(out).strip()


def unquote(value: str) -> str:
    """Unquote single or double quoted scalar string."""
    payload = value.strip()
    if len(payload) >= 2 and (
        (payload[0] == "'" and payload[-1] == "'")
        or (payload[0] == '"' and payload[-1] == '"')
    ):
        return payload[1:-1].strip()
    return payload


def split_csv_entries(raw: str) -> list[str]:
    """Split comma-separated values and drop empty entries."""
    entries: list[str] = []
    for item in raw.split(","):
        token = item.strip()
        if token:
            entries.append(token)
    return entries


def parse_scalar_list(raw: str) -> list[str]:
    """Parse inline list-like scalar (`[a,b]`) or CSV scalar list."""
    payload = strip_inline_comment(raw).strip()
    if payload in {"", "null", "None", "~"}:
        return []
    if payload.startswith("[") and payload.endswith("]"):
        inner = payload[1:-1].strip()
        if not inner:
            return []
        return [unquote(item.strip()) for item in inner.split(",") if item.strip()]
    return split_csv_entries(unquote(payload))
