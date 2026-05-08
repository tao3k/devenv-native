"""Receipt loading and normalization for the knowledge retrieval benchmark."""

from __future__ import annotations

import json
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path


EXPECTED_SOURCE_SCHEMA = "xiuxian_wendao.real_repo_search_precision.v1"


def load_receipt(path: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    validate_receipt(payload)
    return payload


def validate_receipt(payload: dict[str, Any]) -> None:
    schema = payload.get("schema")
    if schema != EXPECTED_SOURCE_SCHEMA:
        raise ValueError(
            f"unsupported source receipt schema `{schema}`, expected `{EXPECTED_SOURCE_SCHEMA}`"
        )
    repositories = payload.get("repositories")
    if not isinstance(repositories, list):
        raise ValueError("source receipt must contain a repositories list")


def query_receipts_by_id(repository: dict[str, Any]) -> dict[str, dict[str, Any]]:
    return {
        str(query.get("query_id")): query
        for query in repository.get("query_receipts", [])
        if isinstance(query, dict) and query.get("query_id")
    }
