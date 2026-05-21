"""
Runtime server info schema API.

Load, validate, and build payloads for runtime server-info responses.
"""

from __future__ import annotations

from functools import lru_cache
from typing import Any

from jsonschema import Draft202012Validator

from .schema_provider import get_schema

# SSOT: xiuxian.runtime.server_info.v1
SCHEMA_ID = "xiuxian.runtime.server_info.v1"
SCHEMA_NAME = "xiuxian.runtime.server_info.v1.schema.json"
NAME_KEY = "name"
VERSION_KEY = "version"
PROTOCOL_VERSION_KEY = "protocolVersion"
MESSAGE_KEY = "message"


@lru_cache(maxsize=1)
def get_validator() -> Draft202012Validator:
    """Cached validator for the agent server info schema."""
    return Draft202012Validator(get_schema(SCHEMA_ID))


def validate(payload: dict[str, Any]) -> None:
    """Raise ValueError if payload does not conform to the shared schema."""
    errs = sorted(get_validator().iter_errors(payload), key=lambda e: list(e.path))
    if not errs:
        return
    first = errs[0]
    loc = ".".join(str(p) for p in first.path) or "<root>"
    raise ValueError(f"Agent server info schema violation at {loc}: {first.message}")


def build_server_info(
    name: str = "xiuxian-wendao",
    version: str = "2.0.0",
    protocol_version: str = "2024-11-05",
    message: str | None = None,
) -> dict[str, Any]:
    """Build a payload that conforms to the shared schema."""
    out: dict[str, Any] = {
        NAME_KEY: name,
        VERSION_KEY: version,
        PROTOCOL_VERSION_KEY: protocol_version,
    }
    if message is not None:
        out[MESSAGE_KEY] = message
    validate(out)
    return out


__all__ = [
    "MESSAGE_KEY",
    "NAME_KEY",
    "PROTOCOL_VERSION_KEY",
    "SCHEMA_NAME",
    "VERSION_KEY",
    "build_server_info",
    "get_validator",
    "validate",
]
