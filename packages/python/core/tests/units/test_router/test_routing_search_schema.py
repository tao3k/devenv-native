"""Tests for routing search algorithm schema and canonical alignment.

Ensures that the canonical instance (routing_search_canonical_v1.json) stays
the single source of truth and that implementations (Tantivy boosts, rerank
fields) are validated against it. When the algorithm is tuned, update the
canonical and this test; sync Rust keyword/index.rs boosts to match.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from xiuxian_foundation.api.schema_locator import resolve_schema_file_path


def _path_to_canonical() -> Path:
    """Path to routing search canonical snapshot, if present."""
    schema_path = resolve_schema_file_path(
        "xiuxian.router.routing_search.v1.schema.json",
        preferred_crates=("xiuxian-wendao",),
    )
    candidates = [schema_path.parent / "snapshots" / "routing_search_canonical_v1.json"]
    for candidate in candidates:
        if candidate.exists():
            return candidate
    return candidates[0]


@pytest.fixture(scope="module")
def canonical():
    """Load canonical routing search algorithm v1."""
    path = _path_to_canonical()
    if not path.exists():
        pytest.skip(f"Canonical not found: {path}")
    with open(path) as f:
        return json.load(f)


def test_canonical_has_schema_id(canonical):
    assert canonical.get("schema") == "xiuxian.router.routing_search.v1"


def test_canonical_keyword_boosts_match_rust_contract(canonical):
    """Keyword field boosts must match Rust keyword/index.rs (QueryParser set_field_boost).

    Rust uses: tool_name 5.0, intents 4.0, keywords 3.0, description 1.0;
    category is stored but not in the query parser.
    """
    fields = {f["name"]: f for f in canonical["keyword"]["fields"]}
    assert fields["tool_name"]["boost"] == 5
    assert fields["tool_name"]["in_query_parser"] is True
    assert fields["intents"]["boost"] == 4
    assert fields["intents"]["in_query_parser"] is True
    assert fields["keywords"]["boost"] == 3
    assert fields["keywords"]["in_query_parser"] is True
    assert fields["description"]["boost"] == 1
    assert fields["description"]["in_query_parser"] is True
    assert fields["category"]["in_query_parser"] is False


def test_canonical_intent_strategies_match_agentic(canonical):
    """Intent strategies must match Rust agentic.rs (exact→keyword_only, etc.)."""
    strategies = canonical["intent"]["strategies"]
    assert strategies["exact"] == "keyword_only"
    assert strategies["semantic"] == "vector_only"
    assert strategies["hybrid"] == "hybrid"


def test_canonical_rerank_fields_match_fusion(canonical):
    """Rerank fields must match Rust keyword/fusion.rs metadata_alignment_boost and file_discovery_boost."""
    rerank = set(canonical["intent"]["rerank_fields"])
    assert rerank >= {"routing_keywords", "intents", "description", "category"}


def test_canonical_semantic_template_has_required_placeholders(canonical):
    """Embedding template must include tool_name, description, intents for indexer alignment."""
    template = canonical["semantic"]["embedding_source"]["template"]
    assert "{tool_name}" in template
    assert "{description}" in template
    assert "{intents}" in template
