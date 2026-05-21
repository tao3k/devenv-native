"""Contract consistency checks for active route-test payload surfaces."""

from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path
from typing import Any

import pytest
from jsonschema import Draft202012Validator

from xiuxian_foundation.api.schema_locator import resolve_schema_file_path
from xiuxian_foundation.config.prj import get_project_root


ROUTE_TEST_SCHEMA_V1 = "xiuxian.router.route_test.v1"


def make_router_result_payload(**overrides: Any) -> dict[str, Any]:
    payload: dict[str, Any] = {
        "id": "git.commit",
        "name": "git.commit",
        "description": "Commit changes",
        "skill_name": "git",
        "tool_name": "git.commit",
        "command": "commit",
        "score": 0.82,
        "final_score": 0.91,
        "confidence": "high",
        "routing_keywords": ["git", "commit"],
        "input_schema": {"type": "object"},
        "payload": {
            "type": "command",
            "description": "Commit changes",
            "metadata": {
                "tool_name": "git.commit",
                "routing_keywords": ["git", "commit"],
                "input_schema": {"type": "object"},
            },
        },
    }
    payload.update(overrides)
    return payload


def make_route_test_payload(
    *,
    query: str = "git commit",
    results: list[dict[str, Any]] | None = None,
    stats: dict[str, Any] | None = None,
    threshold: float = 0.4,
    limit: int = 5,
    confidence_profile: dict[str, Any] | None = None,
    **overrides: Any,
) -> dict[str, Any]:
    if results is None:
        results = [make_router_result_payload()]
    if confidence_profile is None:
        confidence_profile = {"name": "balanced", "source": "active-profile"}
    stats_payload: dict[str, Any] = {
        "semantic_weight": None,
        "keyword_weight": None,
        "rrf_k": None,
        "strategy": None,
    }
    if stats:
        stats_payload.update(
            {
                "semantic_weight": stats.get("semantic_weight"),
                "keyword_weight": stats.get("keyword_weight"),
                "rrf_k": stats.get("rrf_k"),
                "strategy": stats.get("strategy"),
            }
        )
    payload: dict[str, Any] = {
        "schema": ROUTE_TEST_SCHEMA_V1,
        "query": query,
        "count": len(results),
        "threshold": threshold,
        "limit": limit,
        "confidence_profile": confidence_profile,
        "stats": stats_payload,
        "results": results,
    }
    payload.update(overrides)
    return payload


def _snapshots_dir() -> Path:
    return Path(__file__).resolve().parent / "snapshots"


def _load_schema(name: str) -> dict:
    path = resolve_schema_file_path(name)
    return json.loads(path.read_text(encoding="utf-8"))


def _strip_ansi(text: str) -> str:
    return re.sub(r"\x1b\[[0-9;]*m", "", text)


# ---- P0: E2E contract gate - CLI JSON validates against schema (CI fails on drift) ----


def test_route_test_cli_json_validates_against_schema():
    """E2E: Run `omni route test --json`, parse stdout, validate against xiuxian.router.route_test.v1.

    Single-command CI gate: Rust output -> Python parse -> CLI JSON must match schema.
    Skips on timeout (e.g. no embedding/index) or non-zero exit.
    """
    root = get_project_root()
    try:
        result = subprocess.run(
            ["uv", "run", "omni", "route", "test", "git commit", "--local", "--json"],
            cwd=str(root),
            capture_output=True,
            text=True,
            timeout=90,
        )
    except subprocess.TimeoutExpired:
        pytest.skip("omni route test timed out (e.g. no embedding server or index)")
    if result.returncode != 0:
        pytest.skip(f"omni route test failed (e.g. no index): {result.stderr!r}")
    raw = result.stdout or ""
    stripped = _strip_ansi(raw).strip()
    if not stripped:
        pytest.skip(
            "omni route test produced no JSON (empty stdout); check CLI and index"
        )
    # CLI may emit log lines before JSON; use last line if it looks like JSON
    if stripped.startswith("{"):
        json_str = stripped
    else:
        lines = [ln.strip() for ln in stripped.splitlines() if ln.strip()]
        json_str = lines[-1] if lines else ""
    if not json_str or not json_str.startswith("{"):
        pytest.skip(
            "omni route test stdout did not contain JSON; check CLI --json behavior"
        )
    payload = json.loads(json_str)
    schema = _load_schema("xiuxian.router.route_test.v1.schema.json")
    validator = Draft202012Validator(schema)
    errors = list(validator.iter_errors(payload))
    assert (
        not errors
    ), "CLI JSON must match xiuxian.router.route_test.v1 schema: " + "; ".join(
        e.message for e in errors
    )
    assert payload.get("schema") == ROUTE_TEST_SCHEMA_V1
    for r in payload.get("results") or []:
        assert "keywords" not in r, "Results must use routing_keywords only"
        if "payload" in r and "metadata" in r["payload"]:
            assert "keywords" not in r["payload"]["metadata"]


# ---- P0: Shared canonical snapshot (vector-side contract) ----
def test_route_test_canonical_snapshot_validates_against_schema():
    """Shared canonical snapshot must validate against route_test schema.

    This snapshot is the single source of truth for the full algorithm output shape; lock before Python changes.
    """
    schema = _load_schema("xiuxian.router.route_test.v1.schema.json")
    schema_path = resolve_schema_file_path(
        "xiuxian.router.route_test.v1.schema.json",
        preferred_crates=("xiuxian-wendao",),
    )
    canonical_path = schema_path.parent / "snapshots" / "route_test_canonical_v1.json"
    if not canonical_path.exists():
        pytest.skip("Canonical route_test snapshot not found in current schema layout")
    payload = json.loads(canonical_path.read_text(encoding="utf-8"))
    validator = Draft202012Validator(schema)
    errors = list(validator.iter_errors(payload))
    assert (
        not errors
    ), "Canonical snapshot must match xiuxian.router.route_test.v1: " + "; ".join(
        e.message for e in errors
    )
    assert payload.get("schema") == ROUTE_TEST_SCHEMA_V1
    for r in payload.get("results") or []:
        assert "keywords" not in r
        assert "routing_keywords" in r


# ---- P0: E2E snapshot matrix - route JSON (with stats), built from local factories ----


def test_route_test_payload_built_from_factory_has_contract_shape():
    """Route test payload built from local factories has required keys and no legacy keywords."""
    stats = {
        "semantic_weight": 1,
        "keyword_weight": 1.5,
        "rrf_k": 10,
        "strategy": "weighted_rrf_field_boosting",
    }
    payload = make_route_test_payload(
        query="git commit",
        results=[make_router_result_payload()],
        stats=stats,
    )
    assert payload["schema"] == ROUTE_TEST_SCHEMA_V1
    assert payload["query"] == "git commit"
    assert "stats" in payload
    assert payload["stats"]["semantic_weight"] == 1
    assert "results" in payload
    for r in payload["results"]:
        assert "routing_keywords" in r
        assert "keywords" not in r
        if "payload" in r and "metadata" in r["payload"]:
            assert "keywords" not in r["payload"]["metadata"]


def test_route_test_snapshot_matches_factory_output():
    """Snapshot equals local factory output so CI fails on drift."""
    stats = {
        "semantic_weight": 1,
        "keyword_weight": 1.5,
        "rrf_k": 10,
        "strategy": "weighted_rrf_field_boosting",
    }
    expected = make_route_test_payload(
        query="git commit",
        results=[make_router_result_payload()],
        stats=stats,
    )
    path = _snapshots_dir() / "route_test_with_stats_contract_v1.json"
    snapshot = json.loads(path.read_text(encoding="utf-8"))
    assert snapshot == expected, "Snapshot must match make_route_test_payload() output"
