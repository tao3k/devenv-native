from __future__ import annotations

from pathlib import Path


PROJECT_ROOT = Path(__file__).resolve().parents[6]


def test_local_vector_service_modules_are_absent() -> None:
    assert not (
        PROJECT_ROOT
        / "packages/python/foundation/src/xiuxian_foundation/services/vector/__init__.py"
    ).exists()
    assert not (
        PROJECT_ROOT / "packages/python/foundation/src/xiuxian_foundation/services/vector/search.py"
    ).exists()
    assert not (
        PROJECT_ROOT
        / "packages/python/foundation/src/xiuxian_foundation/services/vector/constants.py"
    ).exists()
    assert not (
        PROJECT_ROOT / "packages/python/foundation/src/xiuxian_foundation/services/vector/models.py"
    ).exists()
    assert not (
        PROJECT_ROOT / "packages/python/foundation/src/xiuxian_foundation/services/vector_schema.py"
    ).exists()
    assert not (
        PROJECT_ROOT / "packages/python/foundation/tests/unit/services/test_vector_schema.py"
    ).exists()
    assert not (
        PROJECT_ROOT
        / "packages/python/foundation/tests/unit/services/test_vector_search_helpers.py"
    ).exists()


def test_tool_search_python_contract_surface_is_absent() -> None:
    assert not (
        PROJECT_ROOT
        / "packages/python/foundation/tests/unit/services/snapshots/tool_router_result_contract_v1.json"
    ).exists()


def test_rag_package_surface_is_absent() -> None:
    assert not (PROJECT_ROOT / "packages/python/foundation/src/xiuxian_rag").exists()
    assert not (PROJECT_ROOT / "packages/python/foundation/tests/unit/rag").exists()
    assert not (PROJECT_ROOT / "packages/python/foundation/tests/test_fusion.py").exists()
    assert not (
        PROJECT_ROOT / "packages/python/foundation/tests/test_link_graph_enhancer.py"
    ).exists()


def test_vector_contract_helper_module_is_absent() -> None:
    assert not (
        PROJECT_ROOT / "packages/python/foundation/tests/unit/services/_vector_payloads.py"
    ).exists()


def test_tracer_package_surface_is_absent() -> None:
    assert not (PROJECT_ROOT / "packages/python/foundation/src/xiuxian_tracer").exists()
    assert not (
        PROJECT_ROOT / "packages/python/foundation/tests/unit/tracer"
    ).exists()


def test_graph_enhancement_doc_no_longer_mentions_deleted_python_vector_search_module() -> None:
    graph_doc = (PROJECT_ROOT / "docs/01_core/wendao/graph-enhancement.md").read_text(
        encoding="utf-8"
    )

    assert "vector/search.py" not in graph_doc
